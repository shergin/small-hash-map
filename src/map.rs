use std::collections::hash_map::RandomState;

use super::heap_map::HeapMap;
use super::inline_map::InlineMap;

/// An enum dispatch type that can hold either an `InlineMap` or `HeapMap`.
///
/// This allows for runtime polymorphism between different map implementations
/// while maintaining zero-cost abstraction through manual match dispatch.
///
/// The hasher type `S` is passed through to `HeapMap` when using heap storage.
pub enum MapKind<K, V, const N: usize, S = RandomState> {
    InlineMap(InlineMap<K, V, N>),
    HeapMap(HeapMap<K, V, S>),
}

impl<K: Clone, V: Clone, const N: usize, S: Clone> Clone for MapKind<K, V, N, S> {
    fn clone(&self) -> Self {
        match self {
            MapKind::InlineMap(m) => MapKind::InlineMap(m.clone()),
            MapKind::HeapMap(m) => MapKind::HeapMap(m.clone()),
        }
    }
}

impl<K: std::fmt::Debug, V: std::fmt::Debug, const N: usize, S> std::fmt::Debug
    for MapKind<K, V, N, S>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MapKind::InlineMap(m) => std::fmt::Debug::fmt(m, f),
            MapKind::HeapMap(m) => std::fmt::Debug::fmt(m, f),
        }
    }
}
