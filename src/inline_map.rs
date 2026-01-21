use std::fmt;
use std::hash::Hash;
use std::mem::MaybeUninit;

/// A minimal map implementation optimized for small collections.
///
/// Uses static arrays for both keys and values with no heap allocation.
/// Linear scan for all operations - optimal for small N due to cache locality.
///
/// Keys do not need to implement Default, using MaybeUninit for uninitialized
/// storage.
pub struct InlineMap<K, V, const N: usize> {
    keys: [MaybeUninit<K>; N],
    values: [MaybeUninit<V>; N],
    len: usize,
}

impl<K: Clone, V: Clone, const N: usize> Clone for InlineMap<K, V, N> {
    fn clone(&self) -> Self {
        let mut keys = [(); N].map(|_| MaybeUninit::uninit());
        let mut values = [(); N].map(|_| MaybeUninit::uninit());

        for i in 0..self.len {
            keys[i] = MaybeUninit::new(unsafe { self.keys[i].assume_init_ref() }.clone());
            values[i] = MaybeUninit::new(unsafe { self.values[i].assume_init_ref() }.clone());
        }

        Self {
            keys,
            values,
            len: self.len,
        }
    }
}

impl<K: fmt::Debug, V: fmt::Debug, const N: usize> fmt::Debug for InlineMap<K, V, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut map = f.debug_map();
        for i in 0..self.len {
            let key = unsafe { self.keys[i].assume_init_ref() };
            let value = unsafe { self.values[i].assume_init_ref() };
            map.entry(key, value);
        }
        map.finish()
    }
}

impl<K, V, const N: usize> InlineMap<K, V, N> {
    /// Returns the maximum number of elements the map can hold.
    pub const fn capacity(&self) -> usize {
        N
    }

    /// Creates an empty map at compile time.
    ///
    /// This const constructor allows for static initialization.
    /// Uses unsafe initialization since we can't use array::map in const
    /// context yet.
    pub const fn const_new() -> Self {
        // SAFETY: We're creating an empty map with uninitialized memory.
        // The len is 0, so no elements are accessible until they're properly
        // initialized.
        unsafe {
            Self {
                keys: std::mem::MaybeUninit::uninit().assume_init(),
                values: std::mem::MaybeUninit::uninit().assume_init(),
                len: 0,
            }
        }
    }

    /// Creates a new empty map.
    pub fn new() -> Self {
        Self {
            keys: [(); N].map(|_| MaybeUninit::uninit()),
            values: [(); N].map(|_| MaybeUninit::uninit()),
            len: 0,
        }
    }

    /// Creates a new map with the specified capacity hint.
    ///
    /// Note: Capacity is ignored since InlineMap uses fixed-size arrays.
    pub fn with_capacity(_capacity: usize) -> Self {
        Self::new()
    }

    /// Returns the number of elements in the map.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the map contains no elements.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl<K, V, const N: usize> Drop for InlineMap<K, V, N> {
    fn drop(&mut self) {
        // Drop all initialized elements
        for i in 0..self.len {
            unsafe {
                std::ptr::drop_in_place(self.keys[i].as_mut_ptr());
                std::ptr::drop_in_place(self.values[i].as_mut_ptr());
            }
        }
    }
}

impl<K: Default, V: Default, const N: usize> Default for InlineMap<K, V, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Hash + Eq, V, const N: usize> IntoIterator for InlineMap<K, V, N> {
    type Item = (K, V);
    type IntoIter = std::vec::IntoIter<(K, V)>;

    /// Consumes the map and returns an iterator over owned key-value pairs.
    fn into_iter(mut self) -> Self::IntoIter {
        self.drain().into_iter()
    }
}

impl<K: Hash + Eq, V, const N: usize> Extend<(K, V)> for InlineMap<K, V, N> {
    /// Extends the map with the contents of an iterator.
    ///
    /// # Panics
    ///
    /// Panics if the map would exceed its capacity.
    fn extend<I: IntoIterator<Item = (K, V)>>(&mut self, iter: I) {
        for (key, value) in iter {
            self.insert(key, value);
        }
    }
}

impl<K: Hash + Eq, V, const N: usize> InlineMap<K, V, N> {
    /// Clears the map, removing all key-value pairs.
    pub fn clear(&mut self) {
        // Drop all initialized elements
        for i in 0..self.len {
            unsafe {
                std::ptr::drop_in_place(self.keys[i].as_mut_ptr());
                std::ptr::drop_in_place(self.values[i].as_mut_ptr());
            }
        }
        self.len = 0;
    }

    /// Returns a reference to the value corresponding to the key.
    pub fn get(&self, key: &K) -> Option<&V> {
        for i in 0..self.len {
            // SAFETY: Index i < self.len, so this slot is initialized.
            if unsafe { self.keys[i].assume_init_ref() } == key {
                return Some(unsafe { self.values[i].assume_init_ref() });
            }
        }
        None
    }

    /// Returns a mutable reference to the value corresponding to the key.
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        for i in 0..self.len {
            // SAFETY: Index i < self.len, so this slot is initialized.
            if unsafe { self.keys[i].assume_init_ref() } == key {
                return Some(unsafe { self.values[i].assume_init_mut() });
            }
        }
        None
    }

    /// Returns references to both the key and value corresponding to the key.
    pub fn get_key_value(&self, key: &K) -> Option<(&K, &V)> {
        for i in 0..self.len {
            // SAFETY: Index i < self.len, so this slot is initialized.
            let k = unsafe { self.keys[i].assume_init_ref() };
            if k == key {
                let v = unsafe { self.values[i].assume_init_ref() };
                return Some((k, v));
            }
        }
        None
    }

    /// Inserts a key-value pair into the map.
    ///
    /// If the map did not have this key present, `None` is returned.
    /// If the map did have this key present, the value is updated, and the old
    /// value is returned.
    ///
    /// # Panics
    ///
    /// Panics if the map is full and the key doesn't already exist.
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        // Check if key already exists
        for i in 0..self.len {
            if unsafe { self.keys[i].assume_init_ref() } == &key {
                let old_value = unsafe { std::ptr::read(self.values[i].as_ptr()) };
                self.values[i] = MaybeUninit::new(value);
                return Some(old_value);
            }
        }

        // Key doesn't exist, add at the end
        if self.len >= N {
            panic!("InlineMap is full, cannot insert more than {} elements", N);
        }

        self.keys[self.len] = MaybeUninit::new(key);
        self.values[self.len] = MaybeUninit::new(value);
        self.len += 1;

        None
    }

    /// Removes a key from the map, returning the value at the key if the key
    /// was previously in the map.
    pub fn remove(&mut self, key: &K) -> Option<V> {
        for i in 0..self.len {
            if unsafe { self.keys[i].assume_init_ref() } == key {
                // Read the value to return
                let removed_value = unsafe { std::ptr::read(self.values[i].as_ptr()) };
                // Drop the key
                unsafe { std::ptr::drop_in_place(self.keys[i].as_mut_ptr()) };

                // Shift remaining elements left
                for j in i..self.len - 1 {
                    self.keys[j] =
                        MaybeUninit::new(unsafe { std::ptr::read(self.keys[j + 1].as_ptr()) });
                    self.values[j] =
                        MaybeUninit::new(unsafe { std::ptr::read(self.values[j + 1].as_ptr()) });
                }

                self.len -= 1;
                return Some(removed_value);
            }
        }
        None
    }

    /// Returns `true` if the map contains a value for the specified key.
    pub fn contains_key(&self, key: &K) -> bool {
        self.find_key_index(key).is_some()
    }

    /// Returns the index of a key if it exists in the map.
    ///
    /// This is used internally to avoid duplicate key scans when checking
    /// for key existence and then inserting.
    pub fn find_key_index(&self, key: &K) -> Option<usize> {
        for i in 0..self.len {
            if unsafe { self.keys[i].assume_init_ref() } == key {
                return Some(i);
            }
        }
        None
    }

    /// Inserts a key-value pair using a pre-computed key index hint.
    ///
    /// If `existing_index` is `Some(i)`, updates the value at index `i`.
    /// If `existing_index` is `None`, inserts the key-value pair at the end.
    ///
    /// This method avoids a second key scan when the caller has already
    /// searched for the key using `find_key_index`.
    ///
    /// # Panics
    ///
    /// Panics if the map is full and `existing_index` is `None`.
    pub fn insert_with_hint(
        &mut self,
        key: K,
        value: V,
        existing_index: Option<usize>,
    ) -> Option<V> {
        if let Some(i) = existing_index {
            // Key exists at index i, update the value
            let old_value = unsafe { std::ptr::read(self.values[i].as_ptr()) };
            self.values[i] = MaybeUninit::new(value);
            // Drop the old key and replace with new one (in case K has interior data)
            unsafe { std::ptr::drop_in_place(self.keys[i].as_mut_ptr()) };
            self.keys[i] = MaybeUninit::new(key);
            Some(old_value)
        } else {
            // Key doesn't exist, add at the end
            if self.len >= N {
                panic!("InlineMap is full, cannot insert more than {} elements", N);
            }
            self.keys[self.len] = MaybeUninit::new(key);
            self.values[self.len] = MaybeUninit::new(value);
            self.len += 1;
            None
        }
    }

    /// Returns an iterator visiting all key-value pairs in insertion order.
    pub fn iter(&self) -> std::iter::Zip<std::slice::Iter<'_, K>, std::slice::Iter<'_, V>> {
        // SAFETY: We create slices from the initialized portion of our arrays.
        // - self.keys[0..self.len] and self.values[0..self.len] are guaranteed
        //   to be initialized (maintained by insert/remove/clear).
        // - The slices borrow self, preventing mutation during iteration.
        let key_slice =
            unsafe { std::slice::from_raw_parts(self.keys.as_ptr() as *const K, self.len) };
        let value_slice =
            unsafe { std::slice::from_raw_parts(self.values.as_ptr() as *const V, self.len) };
        key_slice.iter().zip(value_slice.iter())
    }

    /// Returns an iterator visiting all keys in insertion order.
    pub fn keys(&self) -> std::slice::Iter<'_, K> {
        // SAFETY: self.keys[0..self.len] is guaranteed to be initialized.
        let key_slice =
            unsafe { std::slice::from_raw_parts(self.keys.as_ptr() as *const K, self.len) };
        key_slice.iter()
    }

    /// Returns an iterator visiting all values in insertion order.
    pub fn values(&self) -> std::slice::Iter<'_, V> {
        // SAFETY: self.values[0..self.len] is guaranteed to be initialized.
        let value_slice =
            unsafe { std::slice::from_raw_parts(self.values.as_ptr() as *const V, self.len) };
        value_slice.iter()
    }

    /// Returns a mutable iterator visiting all key-value pairs in insertion order.
    ///
    /// Keys are immutable; only values can be modified.
    pub fn iter_mut(
        &mut self,
    ) -> std::iter::Zip<std::slice::Iter<'_, K>, std::slice::IterMut<'_, V>> {
        // SAFETY: We create slices from the initialized portion of our arrays.
        // - self.keys[0..self.len] and self.values[0..self.len] are guaranteed
        //   to be initialized (maintained by insert/remove/clear).
        // - Keys are borrowed immutably, values mutably.
        let key_slice =
            unsafe { std::slice::from_raw_parts(self.keys.as_ptr() as *const K, self.len) };
        let value_slice =
            unsafe { std::slice::from_raw_parts_mut(self.values.as_mut_ptr() as *mut V, self.len) };
        key_slice.iter().zip(value_slice.iter_mut())
    }

    /// Returns a mutable iterator visiting all values in insertion order.
    pub fn values_mut(&mut self) -> std::slice::IterMut<'_, V> {
        // SAFETY: self.values[0..self.len] is guaranteed to be initialized.
        let value_slice =
            unsafe { std::slice::from_raw_parts_mut(self.values.as_mut_ptr() as *mut V, self.len) };
        value_slice.iter_mut()
    }

    /// Retains only the elements specified by the predicate.
    ///
    /// In other words, remove all pairs `(k, v)` for which `f(&k, &mut v)` returns `false`.
    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&K, &mut V) -> bool,
    {
        let mut i = 0;
        while i < self.len {
            // SAFETY: i < self.len, so these slots are initialized.
            let key = unsafe { self.keys[i].assume_init_ref() };
            let value = unsafe { self.values[i].assume_init_mut() };

            if f(key, value) {
                i += 1;
            } else {
                // Remove this element by shifting remaining elements left
                // SAFETY: Drop the key and value at index i.
                unsafe {
                    std::ptr::drop_in_place(self.keys[i].as_mut_ptr());
                    std::ptr::drop_in_place(self.values[i].as_mut_ptr());
                }

                // Shift remaining elements left
                for j in i..self.len - 1 {
                    // SAFETY: j+1 < self.len, so slot j+1 is initialized.
                    // We move the value from j+1 to j.
                    self.keys[j] =
                        MaybeUninit::new(unsafe { std::ptr::read(self.keys[j + 1].as_ptr()) });
                    self.values[j] =
                        MaybeUninit::new(unsafe { std::ptr::read(self.values[j + 1].as_ptr()) });
                }

                self.len -= 1;
                // Don't increment i; we need to check the element that was shifted into position i
            }
        }
    }

    /// Removes all elements from the map and returns them as owned values.
    ///
    /// After calling this method, the map will be empty. This is useful for
    /// moving elements to another container without cloning.
    ///
    /// # Safety
    ///
    /// This method uses `ptr::read` to move values out of the `MaybeUninit`
    /// slots. Safety is ensured by:
    /// - Only reading from indices `0..len`, which are guaranteed to be initialized
    /// - Setting `len = 0` before reading, so `Drop` won't double-free if we panic
    /// - Each slot is read exactly once, transferring ownership to the returned Vec
    pub fn drain(&mut self) -> Vec<(K, V)> {
        let len = self.len;

        // SAFETY: Set len to 0 first. This ensures that if we panic during the
        // loop below, Drop will not attempt to free the already-read elements.
        // Any unread elements will be leaked rather than double-freed.
        self.len = 0;

        let mut result = Vec::with_capacity(len);
        for i in 0..len {
            // SAFETY: Index i < original len, so this slot was initialized.
            // We've set self.len = 0, so Drop won't touch this slot.
            // ptr::read moves the value out; we take ownership.
            let key = unsafe { std::ptr::read(self.keys[i].as_ptr()) };
            let value = unsafe { std::ptr::read(self.values[i].as_ptr()) };
            result.push((key, value));
        }
        result
    }
}
