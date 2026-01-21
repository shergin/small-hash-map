# SmallHashMap

A hash map optimized for small collections with automatic stack-to-heap transition.

## Motivation

In many programs, hash maps are created with only a few entries. The standard `HashMap` always allocates on the heap, which involves:
1. Memory allocation syscalls
2. Hash table bucket management
3. Poor cache locality for small datasets

For maps that typically hold 4-16 entries, linear search through a contiguous array is often faster than hash-based lookup due to cache efficiency.

`SmallHashMap` solves this by:
1. Starting with a stack-allocated `InlineMap` for small collections
2. Automatically transitioning to a heap-allocated `HeapMap` when capacity is exceeded
3. Supporting any hasher via the generic `S` parameter (defaults to `RandomState`, same as `std::HashMap`)

This provides optimal performance across the full range of collection sizes.

## Use Cases

- **Game engines**: Per-entity components, particle attributes, input bindings — millions of small maps without heap thrashing
- **Trading systems**: Order parameters, market data snapshots — zero allocation in hot paths
- **Request handlers**: HTTP headers, session attributes — created and destroyed per request
- **Compilers/interpreters**: AST node metadata, symbol tables per scope
- **Embedded structs**: When you have millions of objects each containing a small map

## API Overview

```rust
use small_hash_map::SmallHashMap;

// Create a map with inline capacity of 8
let mut map: SmallHashMap<&str, i32, 8> = SmallHashMap::new();

// Standard map operations
map.insert("one", 1);
map.insert("two", 2);
map.insert("three", 3);

assert_eq!(map.get(&"two"), Some(&2));
assert_eq!(map.len(), 3);
assert!(map.contains_key(&"one"));

// Iteration
for (key, value) in map.iter() {
    println!("{}: {}", key, value);
}

// Remove entries
map.remove(&"two");
assert_eq!(map.get(&"two"), None);

// Automatic transition to HeapMap when exceeding capacity
for i in 0..20 {
    map.insert(Box::leak(format!("key{}", i).into_boxed_str()), i);
}
// Now using HeapMap internally, but API remains the same
```

### Pre-sizing for Large Collections

```rust
use small_hash_map::SmallHashMap;

// If you know you'll exceed inline capacity, start with HeapMap
let map: SmallHashMap<String, i32, 8> = SmallHashMap::with_capacity(100);
// Starts directly with HeapMap, avoiding transition overhead
```

### Custom Hashers

Like `std::collections::HashMap`, `SmallHashMap` supports custom hashers:

```rust
use small_hash_map::SmallHashMap;
use std::collections::hash_map::RandomState;

// Default hasher (RandomState) - same as std::HashMap
let map1: SmallHashMap<String, i32, 8> = SmallHashMap::new();

// Explicit hasher
let map2: SmallHashMap<String, i32, 8, RandomState> =
    SmallHashMap::with_hasher(RandomState::new());

// With capacity and custom hasher
let map3: SmallHashMap<String, i32, 8, RandomState> =
    SmallHashMap::with_capacity_and_hasher(100, RandomState::new());

// Access the hasher
let _hasher: &RandomState = map2.hasher();
```

For performance-critical applications, you can use faster hashers like `fxhash`:

```rust,ignore
use fxhash::FxBuildHasher;
use small_hash_map::SmallHashMap;

let mut map: SmallHashMap<String, i32, 8, FxBuildHasher> =
    SmallHashMap::with_hasher(FxBuildHasher::default());
```

## Limitations

### One-Way Transition

Once a `SmallHashMap` transitions from `InlineMap` to `HeapMap`, it never transitions back. Even if you remove elements, it remains heap-allocated.

```rust,ignore
let mut map: SmallHashMap<i32, i32, 4> = SmallHashMap::new();

// Fill and exceed capacity
for i in 0..10 {
    map.insert(i, i);
}
// Now using HeapMap

map.clear();
// Still HeapMap, even though empty
```

**Mitigation**: If you need to reclaim stack allocation, create a new `SmallHashMap`.

### Linear Scan for InlineMap

`InlineMap` uses O(n) linear search, not hash-based lookup. This is intentional and typically faster for small n due to cache locality, but becomes slower as n approaches the capacity limit.

### Trait Bounds

Keys, values, and hashers require trait bounds depending on the operation:
- `new`, `default`: `K: Hash + Eq`, `S: BuildHasher + Default`
- `with_hasher`: `K: Hash + Eq`, `S: BuildHasher`
- `with_capacity`, `with_capacity_and_hasher`: `K: Hash + Eq`, `S: BuildHasher + Default + Clone`
- `insert`, `extend`: `K: Hash + Eq`, `S: BuildHasher + Clone`
- `get`, `remove`, etc.: `K: Hash + Eq`, `S: BuildHasher`
- `clone`: `K: Clone`, `V: Clone`, `S: Clone`
- `Debug`: `K: Debug`, `V: Debug`

### No Entry API

Unlike `std::collections::HashMap`, there's no entry API for in-place mutation. The Entry API is problematic because it holds a mutable borrow of the map's internals, but inserting via a `VacantEntry` could trigger a transition from `InlineMap` to `HeapMap`, invalidating that reference. Use `get`/`insert`/`remove` instead.

## Implementation Details

### InlineMap

Stack-allocated storage using fixed-size arrays with `MaybeUninit`:

```rust,ignore
pub struct InlineMap<K, V, const N: usize> {
    keys: [MaybeUninit<K>; N],
    values: [MaybeUninit<V>; N],
    len: usize,
}
```

- Keys and values stored in separate arrays for better cache utilization during key lookup
- Uses `MaybeUninit` to avoid requiring `Default` for uninitialized slots
- Linear scan for all operations (get, insert, remove)
- Maintains insertion order

### HeapMap

Thin wrapper around `HashMap` with configurable hasher:

```rust,ignore
pub struct HeapMap<K, V, S = RandomState> {
    map: HashMap<K, V, S>,
}
```

- Supports any hasher implementing `BuildHasher`
- Defaults to `RandomState` (same as `std::HashMap`)
- Standard hash table performance characteristics
- No ordering guarantees

### SmallHashMap

Enum-based dispatch between the two implementations:

```rust,ignore
pub struct SmallHashMap<K, V, const N: usize, S = RandomState> {
    inner: MapKind<K, V, N, S>,
    transition_threshold: usize,
    hash_builder: S,
}

pub enum MapKind<K, V, const N: usize, S = RandomState> {
    InlineMap(InlineMap<K, V, N>),
    HeapMap(HeapMap<K, V, S>),
}
```

The hasher `S` is stored and used when transitioning to `HeapMap`. Transition occurs when inserting a new key would exceed capacity N.

### Safety

`InlineMap` uses `unsafe` for:
- Reading from `MaybeUninit` slots (safe because we track `len`)
- Creating slices from raw pointers for iteration

All unsafe code maintains the invariant that only indices `0..len` contain initialized values.

## Performance Characteristics

| Operation | InlineMap | HeapMap |
|-----------|-----------|---------|
| `get()` | O(n) | O(1) average |
| `insert()` | O(n) | O(1) average |
| `remove()` | O(n) | O(1) average |
| `contains_key()` | O(n) | O(1) average |
| `iter()` | O(n) | O(n) |
| `clone()` | O(n) | O(n) |
| Memory | Stack, N×(K+V) | Heap, hash table overhead |

**Crossover point**: For typical key/value sizes, `InlineMap` outperforms `HeapMap` up to roughly 16-32 entries due to:
- No hash computation
- No bucket lookup
- Cache-friendly contiguous memory
- No allocation overhead

Beyond this, hash-based O(1) lookup wins.

## Related Work

### Similar Crates

| Crate | Storage | Heap Spill | SIMD | Key Constraint | Notes |
|-------|---------|------------|------|----------------|-------|
| **SmallHashMap** | Stack array | Yes | No | `Hash + Eq` | Automatic transition to HashMap |
| [`small-map`](https://crates.io/crates/small-map) | Stack array | Yes | Yes | `Hash + Eq` | SIMD-accelerated (SSE2/NEON) |
| [`stackmap`](https://crates.io/crates/stackmap) | Stack array | No | No | `Hash + Eq` | Fixed capacity, panics if exceeded |
| [`smallmap`](https://crates.io/crates/smallmap) | Page array | No | No | Byte-indexable | Max 256 entries, specialized |
| [`vecmap-rs`](https://crates.io/crates/vecmap-rs) | Heap Vec | N/A | No | `Eq` only | No Hash required, `no_std` |

### Why No SIMD?

SIMD-accelerated lookup (as in `small-map`) uses hash fingerprinting to compare 16 keys in parallel. However, this requires:
1. Computing a hash for every lookup
2. Storing an extra byte per entry for the fingerprint
3. A minimum of ~16 elements before SIMD outperforms linear scan

For the typical use case of SmallHashMap (4-16 entries), plain linear scan is faster because:
- No hash computation overhead
- Better cache utilization (keys are contiguous)
- Simpler branch prediction

SIMD becomes beneficial only when N ≥ 16 AND the map is frequently near capacity.

### SmallHashMap vs small-map

[`small-map`](https://crates.io/crates/small-map) is an excellent crate with SIMD acceleration. **If you're unsure which to use, try `small-map` first** — it's well-designed and performs great across a wide range of sizes.

Choose **SmallHashMap** when:
- Your maps typically have **4-12 entries** — linear scan beats SIMD due to hash overhead
- You have **millions of small maps** — the h2 fingerprint array adds N bytes plus alignment padding per map, which might be significant
- You have **insert-heavy workloads** — no fingerprint computation needed
- You want **minimal dependencies** and simpler code to audit

Choose **small-map** when:
- Your maps typically have **16+ entries** — SIMD parallel comparison wins
- You have **lookup-heavy workloads** — hash cost amortizes over many lookups
- Your keys are **expensive to compare** — h2 filtering reduces full comparisons

| Aspect | SmallHashMap | small-map |
|--------|--------------|-----------|
| Lookup (N ≤ 8) | Faster | Slower (hash overhead) |
| Lookup (N = 8-16) | ~Equal | ~Equal |
| Lookup (N > 16) | Slower | Faster (SIMD) |
| Insert | Faster | Slower (stores h2) |
| Memory/entry | `K + V` | `K + V + 1 byte` |

### How SmallHashMap Differs

`SmallHashMap` applies the small-vector optimization pattern to hash maps:

| Aspect | SmallHashMap | std HashMap |
|--------|--------------|-------------|
| Small collection | Stack allocated | Heap allocated |
| Memory locality | Excellent for small n | Hash table scattered |
| Allocation | Zero for small n | Always allocates |
| Lookup (small n) | O(n) but cache-friendly | O(1) but cache-unfriendly |
| Lookup (large n) | O(1) after transition | O(1) |

## Quick Reference

### Construction

| Method | Description |
|--------|-------------|
| `SmallHashMap::new()` | Creates empty map with inline storage |
| `SmallHashMap::with_capacity(n)` | Pre-sizes; uses HeapMap if `n > N` |
| `SmallHashMap::with_hasher(s)` | Creates with custom hasher |
| `SmallHashMap::with_capacity_and_hasher(n, s)` | Pre-sizes with custom hasher |
| `SmallHashMap::default()` | Same as `new()` |
| `iter.collect()` | Creates from iterator |
| `map.hasher()` | Returns reference to the hasher |

### Core Operations

| Method | Returns | Description |
|--------|---------|-------------|
| `insert(k, v)` | `Option<V>` | Insert or update; returns old value |
| `get(&k)` | `Option<&V>` | Get reference to value |
| `get_mut(&k)` | `Option<&mut V>` | Get mutable reference |
| `get_key_value(&k)` | `Option<(&K, &V)>` | Get key-value pair |
| `remove(&k)` | `Option<V>` | Remove and return value |
| `contains_key(&k)` | `bool` | Check if key exists |

### Capacity & Size

| Method | Returns | Description |
|--------|---------|-------------|
| `len()` | `usize` | Number of entries |
| `is_empty()` | `bool` | True if no entries |
| `capacity()` | `usize` | Current capacity |
| `is_inline()` | `bool` | True if using stack storage |
| `clear()` | `()` | Remove all entries |

### Iteration

| Method | Yields | Description |
|--------|--------|-------------|
| `iter()` | `(&K, &V)` | Immutable iteration |
| `iter_mut()` | `(&K, &mut V)` | Mutable value iteration |
| `keys()` | `&K` | Iterate over keys |
| `values()` | `&V` | Iterate over values |
| `values_mut()` | `&mut V` | Mutable value iteration |
| `into_iter()` | `(K, V)` | Consuming iteration |
| `retain(f)` | `()` | Filter in place |

## When to Use SmallHashMap

**Good fit:**
- Maps that usually contain fewer than 16 entries
- Performance-critical code with many small maps
- Avoiding allocation pressure in hot paths
- Embedded maps in frequently-instantiated structs

**Poor fit:**
- Maps that consistently grow large (use `HashMap` directly)
- Need for entry API or other `HashMap`-specific features
- Types that don't implement required trait bounds
- `no_std` environments (depends on `std::collections::HashMap`)
