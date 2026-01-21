use crate::SmallHashMap;
use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hasher};

#[test]
fn test_small_hash_map_starts_with_inline_map() {
    let mut map: SmallHashMap<i32, String, 4> = SmallHashMap::new();

    // Insert elements up to the threshold
    map.insert(1, "one".to_string());
    map.insert(2, "two".to_string());
    map.insert(3, "three".to_string());
    map.insert(4, "four".to_string());

    // Should still be using InlineMap (4 elements, threshold is 4)
    assert_eq!(map.len(), 4);
    assert_eq!(map.get(&1), Some(&"one".to_string()));
    assert_eq!(map.get(&2), Some(&"two".to_string()));
    assert_eq!(map.get(&3), Some(&"three".to_string()));
    assert_eq!(map.get(&4), Some(&"four".to_string()));
}

#[test]
fn test_small_hash_map_transitions_to_heap_map() {
    let mut map: SmallHashMap<i32, String, 4> = SmallHashMap::new();

    // Insert elements up to the threshold
    map.insert(1, "one".to_string());
    map.insert(2, "two".to_string());
    map.insert(3, "three".to_string());
    map.insert(4, "four".to_string());

    // Insert one more element to trigger transition
    map.insert(5, "five".to_string());

    // Should now be using HeapMap (5 elements, exceeded threshold of 4)
    assert_eq!(map.len(), 5);
    assert_eq!(map.get(&1), Some(&"one".to_string()));
    assert_eq!(map.get(&2), Some(&"two".to_string()));
    assert_eq!(map.get(&3), Some(&"three".to_string()));
    assert_eq!(map.get(&4), Some(&"four".to_string()));
    assert_eq!(map.get(&5), Some(&"five".to_string()));
}

#[test]
fn test_small_hash_map_with_capacity_starts_with_heap_map() {
    // Create with capacity greater than threshold
    let mut map: SmallHashMap<i32, String, 4> = SmallHashMap::with_capacity(10);

    // Should start with HeapMap since capacity > threshold
    map.insert(1, "one".to_string());
    map.insert(2, "two".to_string());
    map.insert(3, "three".to_string());
    map.insert(4, "four".to_string());
    map.insert(5, "five".to_string());

    assert_eq!(map.len(), 5);
    assert_eq!(map.get(&1), Some(&"one".to_string()));
    assert_eq!(map.get(&5), Some(&"five".to_string()));
}

#[test]
fn test_small_hash_map_iteration() {
    let mut map: SmallHashMap<i32, String, 4> = SmallHashMap::new();

    map.insert(1, "one".to_string());
    map.insert(2, "two".to_string());
    map.insert(3, "three".to_string());

    let mut items: Vec<_> = map.iter().collect();
    items.sort_by_key(|(k, _)| *k);

    assert_eq!(items.len(), 3);
    assert_eq!(items[0], (&1, &"one".to_string()));
    assert_eq!(items[1], (&2, &"two".to_string()));
    assert_eq!(items[2], (&3, &"three".to_string()));
}

#[test]
fn test_small_hash_map_remove() {
    let mut map: SmallHashMap<i32, String, 4> = SmallHashMap::new();

    map.insert(1, "one".to_string());
    map.insert(2, "two".to_string());

    assert_eq!(map.remove(&1), Some("one".to_string()));
    assert_eq!(map.get(&1), None);
    assert_eq!(map.get(&2), Some(&"two".to_string()));
    assert_eq!(map.len(), 1);
}

#[test]
fn test_small_hash_map_clear() {
    let mut map: SmallHashMap<i32, String, 4> = SmallHashMap::new();

    map.insert(1, "one".to_string());
    map.insert(2, "two".to_string());
    map.insert(3, "three".to_string());
    map.insert(4, "four".to_string());
    map.insert(5, "five".to_string()); // This triggers transition to HeapMap

    assert_eq!(map.len(), 5);
    map.clear();
    assert_eq!(map.len(), 0);
    assert!(map.is_empty());
}

#[test]
fn test_get_mut() {
    let mut map: SmallHashMap<i32, String, 4> = SmallHashMap::new();
    map.insert(1, "one".to_string());

    // Modify value via get_mut
    if let Some(value) = map.get_mut(&1) {
        value.push_str("_modified");
    }

    assert_eq!(map.get(&1), Some(&"one_modified".to_string()));
    assert_eq!(map.get_mut(&999), None);
}

#[test]
fn test_keys_and_values() {
    let mut map: SmallHashMap<i32, String, 4> = SmallHashMap::new();
    map.insert(1, "one".to_string());
    map.insert(2, "two".to_string());
    map.insert(3, "three".to_string());

    let mut keys: Vec<_> = map.keys().cloned().collect();
    keys.sort();
    assert_eq!(keys, vec![1, 2, 3]);

    let mut values: Vec<_> = map.values().cloned().collect();
    values.sort();
    assert_eq!(values, vec!["one", "three", "two"]);
}

#[test]
fn test_iter_mut() {
    let mut map: SmallHashMap<i32, i32, 4> = SmallHashMap::new();
    map.insert(1, 10);
    map.insert(2, 20);
    map.insert(3, 30);

    // Double all values
    for (_, value) in map.iter_mut() {
        *value *= 2;
    }

    assert_eq!(map.get(&1), Some(&20));
    assert_eq!(map.get(&2), Some(&40));
    assert_eq!(map.get(&3), Some(&60));
}

#[test]
fn test_values_mut() {
    let mut map: SmallHashMap<i32, i32, 4> = SmallHashMap::new();
    map.insert(1, 10);
    map.insert(2, 20);

    // Add 5 to all values
    for value in map.values_mut() {
        *value += 5;
    }

    assert_eq!(map.get(&1), Some(&15));
    assert_eq!(map.get(&2), Some(&25));
}

#[test]
fn test_retain() {
    let mut map: SmallHashMap<i32, i32, 8> = SmallHashMap::new();
    map.insert(1, 10);
    map.insert(2, 20);
    map.insert(3, 30);
    map.insert(4, 40);

    // Keep only entries where key is even
    map.retain(|k, _| k % 2 == 0);

    assert_eq!(map.len(), 2);
    assert_eq!(map.get(&1), None);
    assert_eq!(map.get(&2), Some(&20));
    assert_eq!(map.get(&3), None);
    assert_eq!(map.get(&4), Some(&40));
}

#[test]
fn test_capacity() {
    let map: SmallHashMap<i32, i32, 8> = SmallHashMap::new();
    assert_eq!(map.capacity(), 8);

    let map_with_capacity: SmallHashMap<i32, i32, 4> = SmallHashMap::with_capacity(100);
    assert!(map_with_capacity.capacity() >= 100);
}

#[test]
fn test_into_iter() {
    let mut map: SmallHashMap<i32, String, 4> = SmallHashMap::new();
    map.insert(1, "one".to_string());
    map.insert(2, "two".to_string());

    let mut items: Vec<_> = map.into_iter().collect();
    items.sort_by_key(|(k, _)| *k);

    assert_eq!(items.len(), 2);
    assert_eq!(items[0], (1, "one".to_string()));
    assert_eq!(items[1], (2, "two".to_string()));
}

#[test]
fn test_extend() {
    let mut map: SmallHashMap<i32, String, 8> = SmallHashMap::new();
    map.insert(1, "one".to_string());

    let additional = vec![(2, "two".to_string()), (3, "three".to_string())];
    map.extend(additional);

    assert_eq!(map.len(), 3);
    assert_eq!(map.get(&1), Some(&"one".to_string()));
    assert_eq!(map.get(&2), Some(&"two".to_string()));
    assert_eq!(map.get(&3), Some(&"three".to_string()));
}

#[test]
fn test_extend_with_transition() {
    let mut map: SmallHashMap<i32, i32, 2> = SmallHashMap::new();
    map.insert(1, 10);

    // This should trigger transition to HeapMap
    let additional = vec![(2, 20), (3, 30), (4, 40)];
    map.extend(additional);

    assert_eq!(map.len(), 4);
    assert_eq!(map.get(&1), Some(&10));
    assert_eq!(map.get(&4), Some(&40));
}

// ==================== Custom Hasher Tests ====================

/// A simple deterministic hasher for testing custom hasher support.
#[derive(Clone, Default)]
struct SimpleHasher(u64);

impl Hasher for SimpleHasher {
    fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.0 = self.0.wrapping_mul(31).wrapping_add(b as u64);
        }
    }

    fn finish(&self) -> u64 {
        self.0
    }
}

/// BuildHasher implementation for SimpleHasher.
#[derive(Clone, Default)]
struct SimpleBuildHasher;

impl BuildHasher for SimpleBuildHasher {
    type Hasher = SimpleHasher;

    fn build_hasher(&self) -> Self::Hasher {
        SimpleHasher(0)
    }
}

#[test]
fn test_with_custom_hasher() {
    let mut map: SmallHashMap<String, i32, 4, SimpleBuildHasher> =
        SmallHashMap::with_hasher(SimpleBuildHasher);

    map.insert("one".to_string(), 1);
    map.insert("two".to_string(), 2);
    map.insert("three".to_string(), 3);

    assert_eq!(map.get(&"one".to_string()), Some(&1));
    assert_eq!(map.get(&"two".to_string()), Some(&2));
    assert_eq!(map.get(&"three".to_string()), Some(&3));
    assert_eq!(map.len(), 3);
}

#[test]
fn test_transition_with_custom_hasher() {
    let mut map: SmallHashMap<i32, String, 2, SimpleBuildHasher> =
        SmallHashMap::with_hasher(SimpleBuildHasher);

    // Fill to capacity
    map.insert(1, "one".to_string());
    map.insert(2, "two".to_string());
    assert!(map.is_inline());

    // Trigger transition to HeapMap
    map.insert(3, "three".to_string());
    assert!(!map.is_inline());

    // Verify all values are still accessible
    assert_eq!(map.get(&1), Some(&"one".to_string()));
    assert_eq!(map.get(&2), Some(&"two".to_string()));
    assert_eq!(map.get(&3), Some(&"three".to_string()));
    assert_eq!(map.len(), 3);
}

#[test]
fn test_with_capacity_and_hasher() {
    let map: SmallHashMap<i32, i32, 4, SimpleBuildHasher> =
        SmallHashMap::with_capacity_and_hasher(100, SimpleBuildHasher);

    // Should start with HeapMap since capacity > N
    assert!(!map.is_inline());
    assert!(map.capacity() >= 100);
}

#[test]
fn test_hasher_method() {
    let map: SmallHashMap<i32, i32, 4, RandomState> = SmallHashMap::with_hasher(RandomState::new());

    // Verify hasher() returns a reference to the hasher
    let _hasher: &RandomState = map.hasher();
}

#[test]
fn test_clone_with_custom_hasher() {
    let mut map: SmallHashMap<i32, String, 4, SimpleBuildHasher> =
        SmallHashMap::with_hasher(SimpleBuildHasher);

    map.insert(1, "one".to_string());
    map.insert(2, "two".to_string());

    let cloned = map.clone();

    assert_eq!(cloned.get(&1), Some(&"one".to_string()));
    assert_eq!(cloned.get(&2), Some(&"two".to_string()));
    assert_eq!(cloned.len(), 2);
}

#[test]
fn test_default_hasher_backward_compat() {
    // Verify that existing syntax without hasher type still works
    let mut map: SmallHashMap<i32, i32, 4> = SmallHashMap::new();
    map.insert(1, 10);
    map.insert(2, 20);

    assert_eq!(map.get(&1), Some(&10));
    assert_eq!(map.get(&2), Some(&20));
}

#[test]
fn test_with_random_state_hasher() {
    // Test with std's RandomState explicitly specified
    let mut map: SmallHashMap<String, i32, 4, RandomState> =
        SmallHashMap::with_hasher(RandomState::new());

    map.insert("hello".to_string(), 1);
    map.insert("world".to_string(), 2);

    assert_eq!(map.get(&"hello".to_string()), Some(&1));
    assert_eq!(map.get(&"world".to_string()), Some(&2));
}

#[test]
fn test_equality_with_different_hashers() {
    // Two maps with different hashers should be equal if they contain the same data
    let mut map1: SmallHashMap<i32, i32, 4, RandomState> =
        SmallHashMap::with_hasher(RandomState::new());
    let mut map2: SmallHashMap<i32, i32, 4, SimpleBuildHasher> =
        SmallHashMap::with_hasher(SimpleBuildHasher);

    map1.insert(1, 10);
    map1.insert(2, 20);

    map2.insert(1, 10);
    map2.insert(2, 20);

    assert_eq!(map1, map2);
}
