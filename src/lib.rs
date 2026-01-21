#![doc = include_str!("../README.md")]
//!
//! # Quick Start
//!
//! ```rust
//! use small_hash_map::SmallHashMap;
//!
//! // Create a map with inline capacity of 8
//! let mut map: SmallHashMap<&str, i32, 8> = SmallHashMap::new();
//!
//! // Insert some values
//! map.insert("one", 1);
//! map.insert("two", 2);
//! map.insert("three", 3);
//!
//! // Look up values
//! assert_eq!(map.get(&"two"), Some(&2));
//! assert_eq!(map.len(), 3);
//!
//! // The map is still using inline (stack) storage
//! assert!(map.is_inline());
//! ```
//!
//! # Collecting from Iterators
//!
//! ```rust
//! use small_hash_map::SmallHashMap;
//!
//! let pairs = vec![("a", 1), ("b", 2), ("c", 3)];
//! let map: SmallHashMap<&str, i32, 8> = pairs.into_iter().collect();
//!
//! assert_eq!(map.get(&"b"), Some(&2));
//! ```

mod heap_map;
mod inline_map;
mod map;
mod small_hash_map;

pub use heap_map::HeapMap;
pub use inline_map::InlineMap;
pub use small_hash_map::{
    SmallHashMap, SmallHashMapIntoIter, SmallHashMapIter, SmallHashMapIterMut, SmallHashMapKeys,
    SmallHashMapValues, SmallHashMapValuesMut,
};

#[cfg(test)]
#[path = "tests/small_hash_map_tests.rs"]
mod tests;
