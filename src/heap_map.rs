use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::fmt;
use std::hash::{BuildHasher, Hash};

/// A HashMap wrapper that can use any hasher implementing `BuildHasher`.
///
/// This is used internally by SmallHashMap after transitioning from
/// stack-allocated storage. By default, it uses `RandomState` (the same
/// default hasher as `std::collections::HashMap`).
pub struct HeapMap<K, V, S = RandomState> {
    map: HashMap<K, V, S>,
}

impl<K: Clone, V: Clone, S: Clone> Clone for HeapMap<K, V, S> {
    fn clone(&self) -> Self {
        Self {
            map: self.map.clone(),
        }
    }
}

impl<K: fmt::Debug, V: fmt::Debug, S> fmt::Debug for HeapMap<K, V, S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HeapMap")
            .field("len", &self.map.len())
            .field("map", &self.map)
            .finish()
    }
}

impl<K, V, S: Default> Default for HeapMap<K, V, S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Hash + Eq, V, S> IntoIterator for HeapMap<K, V, S> {
    type Item = (K, V);
    type IntoIter = std::collections::hash_map::IntoIter<K, V>;

    /// Consumes the map and returns an iterator over owned key-value pairs.
    fn into_iter(self) -> Self::IntoIter {
        self.map.into_iter()
    }
}

impl<K: Hash + Eq, V, S: BuildHasher> Extend<(K, V)> for HeapMap<K, V, S> {
    /// Extends the map with the contents of an iterator.
    fn extend<I: IntoIterator<Item = (K, V)>>(&mut self, iter: I) {
        self.map.extend(iter);
    }
}

impl<K, V, S: Default> HeapMap<K, V, S> {
    /// Creates a new empty HeapMap with the default hasher.
    pub fn new() -> Self {
        Self {
            map: HashMap::with_hasher(S::default()),
        }
    }

    /// Creates a new HeapMap with the specified capacity and default hasher.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            map: HashMap::with_capacity_and_hasher(capacity, S::default()),
        }
    }
}

impl<K, V, S> HeapMap<K, V, S> {
    /// Creates a new empty HeapMap with the specified hasher.
    pub fn with_hasher(hash_builder: S) -> Self
    where
        S: BuildHasher,
    {
        Self {
            map: HashMap::with_hasher(hash_builder),
        }
    }

    /// Creates a new HeapMap with the specified capacity and hasher.
    pub fn with_capacity_and_hasher(capacity: usize, hash_builder: S) -> Self
    where
        S: BuildHasher,
    {
        Self {
            map: HashMap::with_capacity_and_hasher(capacity, hash_builder),
        }
    }

    /// Returns a reference to the map's hasher.
    pub fn hasher(&self) -> &S {
        self.map.hasher()
    }

    /// Returns the number of elements in the map.
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Returns `true` if the map contains no elements.
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Returns the number of elements the map can hold without reallocating.
    pub fn capacity(&self) -> usize {
        self.map.capacity()
    }
}

impl<K: Hash + Eq, V, S: BuildHasher> HeapMap<K, V, S> {
    /// Clears the map, removing all key-value pairs.
    pub fn clear(&mut self) {
        self.map.clear();
    }

    /// Returns a reference to the value corresponding to the key.
    pub fn get(&self, key: &K) -> Option<&V> {
        self.map.get(key)
    }

    /// Returns a mutable reference to the value corresponding to the key.
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.map.get_mut(key)
    }

    /// Returns references to both the key and value corresponding to the key.
    pub fn get_key_value(&self, key: &K) -> Option<(&K, &V)> {
        self.map.get_key_value(key)
    }

    /// Inserts a key-value pair into the map.
    ///
    /// If the map did not have this key present, `None` is returned.
    /// If the map did have this key present, the value is updated, and the old
    /// value is returned.
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.map.insert(key, value)
    }

    /// Removes a key from the map, returning the value at the key if the key
    /// was previously in the map.
    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.map.remove(key)
    }

    /// Returns `true` if the map contains a value for the specified key.
    pub fn contains_key(&self, key: &K) -> bool {
        self.map.contains_key(key)
    }

    /// Returns an iterator visiting all key-value pairs in arbitrary order.
    pub fn iter(&self) -> std::collections::hash_map::Iter<'_, K, V> {
        self.map.iter()
    }

    /// Returns an iterator visiting all keys in arbitrary order.
    pub fn keys(&self) -> std::collections::hash_map::Keys<'_, K, V> {
        self.map.keys()
    }

    /// Returns an iterator visiting all values in arbitrary order.
    pub fn values(&self) -> std::collections::hash_map::Values<'_, K, V> {
        self.map.values()
    }

    /// Returns a mutable iterator visiting all key-value pairs in arbitrary order.
    ///
    /// Keys are immutable; only values can be modified.
    pub fn iter_mut(&mut self) -> std::collections::hash_map::IterMut<'_, K, V> {
        self.map.iter_mut()
    }

    /// Returns a mutable iterator visiting all values in arbitrary order.
    pub fn values_mut(&mut self) -> std::collections::hash_map::ValuesMut<'_, K, V> {
        self.map.values_mut()
    }

    /// Retains only the elements specified by the predicate.
    ///
    /// In other words, remove all pairs `(k, v)` for which `f(&k, &mut v)` returns `false`.
    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&K, &mut V) -> bool,
    {
        self.map.retain(f);
    }
}
