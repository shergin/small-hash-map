use std::collections::hash_map;
use std::collections::hash_map::RandomState;
use std::fmt;
use std::hash::{BuildHasher, Hash};
use std::slice;

use super::heap_map::HeapMap;
use super::inline_map::InlineMap;
use super::map::MapKind;

/// An adaptive map that starts with an `InlineMap` and transitions to
/// `HeapMap` when it grows beyond a threshold.
///
/// This provides the best of both worlds: stack allocation for small
/// collections and heap allocation for larger ones, with automatic transition
/// based on size.
///
/// # Type Parameters
///
/// - `K`: The key type
/// - `V`: The value type
/// - `N`: The inline capacity (stack-allocated storage size)
/// - `S`: The hasher type, defaults to `RandomState` (same as `std::collections::HashMap`)
///
/// # Transition Threshold
/// The map transitions from `InlineMap` to `HeapMap` when it exceeds the `N`
/// capacity of the `InlineMap`. This ensures that we never exceed the
/// stack-allocated capacity and always have room for growth.
///
/// # Custom Hashers
///
/// You can use a custom hasher by specifying the `S` type parameter:
///
/// ```rust
/// use small_hash_map::SmallHashMap;
/// use std::collections::hash_map::RandomState;
///
/// // Using the default hasher (RandomState)
/// let map1: SmallHashMap<String, i32, 8> = SmallHashMap::new();
///
/// // Using a custom hasher
/// let map2: SmallHashMap<String, i32, 8, RandomState> =
///     SmallHashMap::with_hasher(RandomState::new());
/// ```
pub struct SmallHashMap<K, V, const N: usize, S = RandomState> {
    inner: MapKind<K, V, N, S>,
    transition_threshold: usize,
    hash_builder: S,
}

impl<K: Clone, V: Clone, const N: usize, S: Clone> Clone for SmallHashMap<K, V, N, S> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            transition_threshold: self.transition_threshold,
            hash_builder: self.hash_builder.clone(),
        }
    }
}

impl<K: fmt::Debug, V: fmt::Debug, const N: usize, S> fmt::Debug for SmallHashMap<K, V, N, S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SmallHashMap")
            .field("inner", &self.inner)
            .field("transition_threshold", &self.transition_threshold)
            .finish()
    }
}

impl<K, V, const N: usize, S> SmallHashMap<K, V, N, S>
where
    K: Hash + Eq,
    S: BuildHasher + Default,
{
    /// Creates a new `SmallHashMap` that starts with an `InlineMap`.
    ///
    /// The transition threshold is set to `N` (the capacity of the
    /// `InlineMap`).
    ///
    /// # Example
    ///
    /// ```
    /// use small_hash_map::SmallHashMap;
    ///
    /// let mut map: SmallHashMap<String, i32, 8> = SmallHashMap::new();
    /// assert!(map.is_empty());
    /// assert_eq!(map.capacity(), 8);
    /// ```
    pub fn new() -> Self {
        Self::with_hasher(S::default())
    }
}

impl<K, V, const N: usize, S> SmallHashMap<K, V, N, S>
where
    K: Hash + Eq,
    S: BuildHasher + Default + Clone,
{
    /// Creates a new `SmallHashMap` with the specified capacity hint.
    ///
    /// If the capacity is greater than the transition threshold, it starts with
    /// a `HeapMap`. Otherwise, it starts with an `InlineMap`.
    pub fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity_and_hasher(capacity, S::default())
    }
}

impl<K, V, const N: usize, S> SmallHashMap<K, V, N, S>
where
    K: Hash + Eq,
    S: BuildHasher,
{
    /// Creates a new `SmallHashMap` with the specified hasher.
    ///
    /// The map starts with an `InlineMap` and will transition to `HeapMap`
    /// when it exceeds the inline capacity.
    ///
    /// # Example
    ///
    /// ```
    /// use small_hash_map::SmallHashMap;
    /// use std::collections::hash_map::RandomState;
    ///
    /// let map: SmallHashMap<String, i32, 8, RandomState> =
    ///     SmallHashMap::with_hasher(RandomState::new());
    /// ```
    pub fn with_hasher(hash_builder: S) -> Self {
        Self {
            inner: MapKind::InlineMap(InlineMap::new()),
            transition_threshold: N,
            hash_builder,
        }
    }

    /// Creates a new `SmallHashMap` with the specified capacity and hasher.
    ///
    /// If the capacity is greater than the transition threshold, it starts with
    /// a `HeapMap`. Otherwise, it starts with an `InlineMap`.
    ///
    /// # Example
    ///
    /// ```
    /// use small_hash_map::SmallHashMap;
    /// use std::collections::hash_map::RandomState;
    ///
    /// // Start directly with heap storage
    /// let map: SmallHashMap<String, i32, 4, RandomState> =
    ///     SmallHashMap::with_capacity_and_hasher(100, RandomState::new());
    /// assert!(!map.is_inline());
    /// ```
    pub fn with_capacity_and_hasher(capacity: usize, hash_builder: S) -> Self
    where
        S: Clone,
    {
        if capacity > N {
            Self {
                inner: MapKind::HeapMap(HeapMap::with_capacity_and_hasher(
                    capacity,
                    hash_builder.clone(),
                )),
                transition_threshold: N,
                hash_builder,
            }
        } else {
            Self {
                inner: MapKind::InlineMap(InlineMap::with_capacity(capacity)),
                transition_threshold: N,
                hash_builder,
            }
        }
    }

    /// Returns a reference to the map's hasher.
    ///
    /// # Example
    ///
    /// ```
    /// use small_hash_map::SmallHashMap;
    /// use std::collections::hash_map::RandomState;
    ///
    /// let map: SmallHashMap<String, i32, 8, RandomState> =
    ///     SmallHashMap::with_hasher(RandomState::new());
    /// let _hasher: &RandomState = map.hasher();
    /// ```
    pub fn hasher(&self) -> &S {
        &self.hash_builder
    }

    /// Returns the number of elements in the map.
    pub fn len(&self) -> usize {
        match &self.inner {
            MapKind::InlineMap(map) => map.len(),
            MapKind::HeapMap(map) => map.len(),
        }
    }

    /// Returns `true` if the map contains no elements.
    pub fn is_empty(&self) -> bool {
        match &self.inner {
            MapKind::InlineMap(map) => map.is_empty(),
            MapKind::HeapMap(map) => map.is_empty(),
        }
    }

    /// Returns the number of elements the map can hold without reallocating or transitioning.
    ///
    /// For `InlineMap`, this returns `N`. For `HeapMap`, this delegates to the underlying
    /// HashMap's capacity.
    pub fn capacity(&self) -> usize {
        match &self.inner {
            MapKind::InlineMap(map) => map.capacity(),
            MapKind::HeapMap(map) => map.capacity(),
        }
    }

    /// Returns `true` if the map is currently using inline (stack) storage.
    ///
    /// This is primarily useful for debugging, testing, and performance analysis.
    /// Application logic should generally not depend on storage mode.
    ///
    /// # Example
    ///
    /// ```
    /// use small_hash_map::SmallHashMap;
    ///
    /// let mut map: SmallHashMap<i32, i32, 2> = SmallHashMap::new();
    /// assert!(map.is_inline());
    ///
    /// // Fill to capacity
    /// map.insert(1, 10);
    /// map.insert(2, 20);
    /// assert!(map.is_inline());
    ///
    /// // Exceed capacity - transitions to heap
    /// map.insert(3, 30);
    /// assert!(!map.is_inline());
    /// ```
    pub fn is_inline(&self) -> bool {
        matches!(&self.inner, MapKind::InlineMap(_))
    }

    /// Clears the map, removing all key-value pairs.
    pub fn clear(&mut self) {
        match &mut self.inner {
            MapKind::InlineMap(map) => map.clear(),
            MapKind::HeapMap(map) => map.clear(),
        }
    }

    /// Returns a reference to the value corresponding to the key.
    ///
    /// # Example
    ///
    /// ```
    /// use small_hash_map::SmallHashMap;
    ///
    /// let mut map: SmallHashMap<i32, &str, 8> = SmallHashMap::new();
    /// map.insert(1, "one");
    ///
    /// assert_eq!(map.get(&1), Some(&"one"));
    /// assert_eq!(map.get(&2), None);
    /// ```
    pub fn get(&self, key: &K) -> Option<&V> {
        match &self.inner {
            MapKind::InlineMap(map) => map.get(key),
            MapKind::HeapMap(map) => map.get(key),
        }
    }

    /// Returns a mutable reference to the value corresponding to the key.
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        match &mut self.inner {
            MapKind::InlineMap(map) => map.get_mut(key),
            MapKind::HeapMap(map) => map.get_mut(key),
        }
    }

    /// Returns references to both the key and value corresponding to the key.
    ///
    /// This is useful when you need access to the stored key, particularly when
    /// keys contain additional data beyond what's used for equality comparison.
    ///
    /// # Example
    ///
    /// ```
    /// use small_hash_map::SmallHashMap;
    ///
    /// let mut map: SmallHashMap<String, i32, 8> = SmallHashMap::new();
    /// map.insert("hello".to_string(), 42);
    ///
    /// let (key, value) = map.get_key_value(&"hello".to_string()).unwrap();
    /// assert_eq!(key, "hello");
    /// assert_eq!(*value, 42);
    /// ```
    pub fn get_key_value(&self, key: &K) -> Option<(&K, &V)> {
        match &self.inner {
            MapKind::InlineMap(map) => map.get_key_value(key),
            MapKind::HeapMap(map) => map.get_key_value(key),
        }
    }

    /// Returns `true` if the map contains a value for the specified key.
    pub fn contains_key(&self, key: &K) -> bool {
        match &self.inner {
            MapKind::InlineMap(map) => map.contains_key(key),
            MapKind::HeapMap(map) => map.contains_key(key),
        }
    }

    /// Removes a key from the map, returning the value at the key if the key
    /// was previously in the map.
    pub fn remove(&mut self, key: &K) -> Option<V> {
        match &mut self.inner {
            MapKind::InlineMap(map) => map.remove(key),
            MapKind::HeapMap(map) => map.remove(key),
        }
    }

    /// Returns an iterator visiting all key-value pairs.
    ///
    /// For `InlineMap`, the order is insertion order; for `HeapMap`, it's
    /// arbitrary.
    pub fn iter(&self) -> SmallHashMapIter<'_, K, V, N> {
        match &self.inner {
            MapKind::InlineMap(map) => SmallHashMapIter::InlineMap(map.iter()),
            MapKind::HeapMap(map) => SmallHashMapIter::HeapMap(map.iter()),
        }
    }

    /// Returns an iterator visiting all keys.
    ///
    /// For `InlineMap`, the order is insertion order; for `HeapMap`, it's
    /// arbitrary.
    pub fn keys(&self) -> SmallHashMapKeys<'_, K, V, N> {
        match &self.inner {
            MapKind::InlineMap(map) => SmallHashMapKeys::InlineMap(map.keys()),
            MapKind::HeapMap(map) => SmallHashMapKeys::HeapMap(map.keys()),
        }
    }

    /// Returns an iterator visiting all values.
    ///
    /// For `InlineMap`, the order is insertion order; for `HeapMap`, it's
    /// arbitrary.
    pub fn values(&self) -> SmallHashMapValues<'_, K, V, N> {
        match &self.inner {
            MapKind::InlineMap(map) => SmallHashMapValues::InlineMap(map.values()),
            MapKind::HeapMap(map) => SmallHashMapValues::HeapMap(map.values()),
        }
    }

    /// Returns a mutable iterator visiting all key-value pairs.
    ///
    /// Keys are immutable; only values can be modified.
    /// For `InlineMap`, the order is insertion order; for `HeapMap`, it's
    /// arbitrary.
    pub fn iter_mut(&mut self) -> SmallHashMapIterMut<'_, K, V, N> {
        match &mut self.inner {
            MapKind::InlineMap(map) => SmallHashMapIterMut::InlineMap(map.iter_mut()),
            MapKind::HeapMap(map) => SmallHashMapIterMut::HeapMap(map.iter_mut()),
        }
    }

    /// Returns a mutable iterator visiting all values.
    ///
    /// For `InlineMap`, the order is insertion order; for `HeapMap`, it's
    /// arbitrary.
    pub fn values_mut(&mut self) -> SmallHashMapValuesMut<'_, K, V, N> {
        match &mut self.inner {
            MapKind::InlineMap(map) => SmallHashMapValuesMut::InlineMap(map.values_mut()),
            MapKind::HeapMap(map) => SmallHashMapValuesMut::HeapMap(map.values_mut()),
        }
    }

    /// Retains only the elements specified by the predicate.
    ///
    /// In other words, remove all pairs `(k, v)` for which `f(&k, &mut v)` returns `false`.
    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&K, &mut V) -> bool,
    {
        match &mut self.inner {
            MapKind::InlineMap(map) => map.retain(f),
            MapKind::HeapMap(map) => map.retain(f),
        }
    }
}

impl<K, V, const N: usize, S> SmallHashMap<K, V, N, S>
where
    K: Hash + Eq,
    S: BuildHasher + Clone,
{
    /// Inserts a key-value pair into the map.
    ///
    /// If the map did not have this key present, `None` is returned.
    /// If the map did have this key present, the value is updated, and the old
    /// value is returned.
    ///
    /// If inserting would exceed the inline capacity, the map automatically
    /// transitions to heap storage.
    ///
    /// # Example
    ///
    /// ```
    /// use small_hash_map::SmallHashMap;
    ///
    /// let mut map: SmallHashMap<i32, &str, 4> = SmallHashMap::new();
    ///
    /// // Insert new key returns None
    /// assert_eq!(map.insert(1, "one"), None);
    ///
    /// // Update existing key returns old value
    /// assert_eq!(map.insert(1, "ONE"), Some("one"));
    /// assert_eq!(map.get(&1), Some(&"ONE"));
    /// ```
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        // Check if we need to transition BEFORE inserting.
        // We use find_key_index to avoid scanning twice (once for existence check,
        // once for the actual insert).
        let (should_transition, existing_index) = match &self.inner {
            MapKind::InlineMap(inline_map) => {
                let idx = inline_map.find_key_index(&key);
                // Transition if key doesn't exist and we're at capacity
                let should_transition =
                    idx.is_none() && inline_map.len() >= self.transition_threshold;
                (should_transition, idx)
            }
            MapKind::HeapMap(_) => (false, None),
        };

        if should_transition {
            // We know it's InlineMap here, so this match is just to satisfy the borrow checker
            if let MapKind::InlineMap(inline_map) = &mut self.inner {
                // Move all elements from InlineMap to HeapMap (no cloning needed)
                let mut heap_map = HeapMap::with_capacity_and_hasher(
                    inline_map.len() * 2,
                    self.hash_builder.clone(),
                );
                for (existing_key, existing_value) in inline_map.drain() {
                    heap_map.insert(existing_key, existing_value);
                }
                self.inner = MapKind::HeapMap(heap_map);
            }
        }

        // Now safely insert into either map
        match &mut self.inner {
            MapKind::InlineMap(map) => map.insert_with_hint(key, value, existing_index),
            MapKind::HeapMap(map) => map.insert(key, value),
        }
    }
}

impl<K, V, const N: usize, S> Default for SmallHashMap<K, V, N, S>
where
    K: Hash + Eq,
    S: BuildHasher + Default,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V, const N: usize, S> IntoIterator for SmallHashMap<K, V, N, S>
where
    K: Hash + Eq,
    S: BuildHasher,
{
    type Item = (K, V);
    type IntoIter = SmallHashMapIntoIter<K, V, N>;

    /// Consumes the map and returns an iterator over owned key-value pairs.
    fn into_iter(self) -> Self::IntoIter {
        match self.inner {
            MapKind::InlineMap(map) => SmallHashMapIntoIter::InlineMap(map.into_iter()),
            MapKind::HeapMap(map) => SmallHashMapIntoIter::HeapMap(map.into_iter()),
        }
    }
}

impl<K, V, const N: usize, S> Extend<(K, V)> for SmallHashMap<K, V, N, S>
where
    K: Hash + Eq,
    S: BuildHasher + Clone,
{
    /// Extends the map with the contents of an iterator.
    ///
    /// If the map transitions to heap storage during extension, subsequent
    /// inserts will go to the heap map.
    fn extend<I: IntoIterator<Item = (K, V)>>(&mut self, iter: I) {
        for (key, value) in iter {
            self.insert(key, value);
        }
    }
}

impl<K, V, const N: usize, S> std::iter::FromIterator<(K, V)> for SmallHashMap<K, V, N, S>
where
    K: Hash + Eq,
    S: BuildHasher + Default + Clone,
{
    /// Creates a `SmallHashMap` from an iterator of key-value pairs.
    ///
    /// If the iterator yields more than `N` elements with unique keys,
    /// the map will automatically transition to heap storage.
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        let iter = iter.into_iter();
        let (lower, upper) = iter.size_hint();

        // Use capacity hint to potentially skip inline storage
        let capacity_hint = upper.unwrap_or(lower);
        let mut map = if capacity_hint > N {
            Self::with_capacity(capacity_hint)
        } else {
            Self::new()
        };

        map.extend(iter);
        map
    }
}

impl<K, V, const N: usize, const M: usize, S, T> PartialEq<SmallHashMap<K, V, M, T>>
    for SmallHashMap<K, V, N, S>
where
    K: Hash + Eq,
    V: PartialEq,
    S: BuildHasher,
    T: BuildHasher,
{
    /// Two maps are equal if they contain the same key-value pairs,
    /// regardless of internal storage mode, capacity parameter, or hasher type.
    fn eq(&self, other: &SmallHashMap<K, V, M, T>) -> bool {
        if self.len() != other.len() {
            return false;
        }
        self.iter().all(|(k, v)| other.get(k) == Some(v))
    }
}

impl<K, V, const N: usize, S> Eq for SmallHashMap<K, V, N, S>
where
    K: Hash + Eq,
    V: Eq,
    S: BuildHasher,
{
}

/// Iterator type for SmallHashMap that can handle both InlineMap and HeapMap
/// iterators.
pub enum SmallHashMapIter<'a, K, V, const N: usize>
where
    K: 'a,
    V: 'a,
{
    InlineMap(std::iter::Zip<slice::Iter<'a, K>, slice::Iter<'a, V>>),
    HeapMap(hash_map::Iter<'a, K, V>),
}

impl<'a, K, V, const N: usize> Iterator for SmallHashMapIter<'a, K, V, N> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            SmallHashMapIter::InlineMap(iter) => iter.next(),
            SmallHashMapIter::HeapMap(iter) => iter.next(),
        }
    }
}

/// Iterator over keys of a SmallHashMap.
pub enum SmallHashMapKeys<'a, K, V, const N: usize>
where
    K: 'a,
    V: 'a,
{
    InlineMap(slice::Iter<'a, K>),
    HeapMap(hash_map::Keys<'a, K, V>),
}

impl<'a, K, V, const N: usize> Iterator for SmallHashMapKeys<'a, K, V, N> {
    type Item = &'a K;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            SmallHashMapKeys::InlineMap(iter) => iter.next(),
            SmallHashMapKeys::HeapMap(iter) => iter.next(),
        }
    }
}

/// Iterator over values of a SmallHashMap.
pub enum SmallHashMapValues<'a, K, V, const N: usize>
where
    K: 'a,
    V: 'a,
{
    InlineMap(slice::Iter<'a, V>),
    HeapMap(hash_map::Values<'a, K, V>),
}

impl<'a, K, V, const N: usize> Iterator for SmallHashMapValues<'a, K, V, N> {
    type Item = &'a V;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            SmallHashMapValues::InlineMap(iter) => iter.next(),
            SmallHashMapValues::HeapMap(iter) => iter.next(),
        }
    }
}

/// Mutable iterator over key-value pairs of a SmallHashMap.
pub enum SmallHashMapIterMut<'a, K, V, const N: usize>
where
    K: 'a,
    V: 'a,
{
    InlineMap(std::iter::Zip<slice::Iter<'a, K>, slice::IterMut<'a, V>>),
    HeapMap(hash_map::IterMut<'a, K, V>),
}

impl<'a, K, V, const N: usize> Iterator for SmallHashMapIterMut<'a, K, V, N> {
    type Item = (&'a K, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            SmallHashMapIterMut::InlineMap(iter) => iter.next(),
            SmallHashMapIterMut::HeapMap(iter) => iter.next(),
        }
    }
}

/// Mutable iterator over values of a SmallHashMap.
pub enum SmallHashMapValuesMut<'a, K, V, const N: usize>
where
    K: 'a,
    V: 'a,
{
    InlineMap(slice::IterMut<'a, V>),
    HeapMap(hash_map::ValuesMut<'a, K, V>),
}

impl<'a, K, V, const N: usize> Iterator for SmallHashMapValuesMut<'a, K, V, N> {
    type Item = &'a mut V;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            SmallHashMapValuesMut::InlineMap(iter) => iter.next(),
            SmallHashMapValuesMut::HeapMap(iter) => iter.next(),
        }
    }
}

/// Consuming iterator over key-value pairs of a SmallHashMap.
pub enum SmallHashMapIntoIter<K, V, const N: usize> {
    InlineMap(std::vec::IntoIter<(K, V)>),
    HeapMap(hash_map::IntoIter<K, V>),
}

impl<K, V, const N: usize> Iterator for SmallHashMapIntoIter<K, V, N> {
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            SmallHashMapIntoIter::InlineMap(iter) => iter.next(),
            SmallHashMapIntoIter::HeapMap(iter) => iter.next(),
        }
    }
}
