use std::{borrow::Borrow, iter::FusedIterator};

use crate::{cdc::change::ChangeEvent, core::pair::Pair};

use super::set::BTreeSet;

#[derive(Debug)]
pub struct BTreeMap<K, V>
where
    K: Send + Ord + Clone + 'static,
    V: Send + Clone + 'static,
{
    pub(crate) set: BTreeSet<Pair<K, V>>,
}

impl<K: Send + Ord + Clone, V: Send + Clone + 'static> Default for BTreeMap<K, V> {
    fn default() -> Self {
        Self {
            set: Default::default(),
        }
    }
}

pub struct Iter<'a, K, V>
where
    K: Send + Ord + Clone + 'static,
    V: Send + Clone + 'static,
{
    inner: super::set::Iter<'a, Pair<K, V>>,
}

impl<'a, K, V> Iterator for Iter<'a, K, V>
where
    K: Send + Ord + Clone + 'static,
    V: Send + Clone + 'static,
{
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(entry) = self.inner.next() {
            return Some((&entry.key, &entry.value));
        }

        None
    }
}

impl<'a, K, V> DoubleEndedIterator for Iter<'a, K, V>
where
    K: Send + Ord + Clone + 'static,
    V: Send + Clone + 'static,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        if let Some(entry) = self.inner.next_back() {
            return Some((&entry.key, &entry.value));
        }

        None
    }
}

impl<'a, K, V> FusedIterator for Iter<'a, K, V>
where
    K: Send + Ord + Clone + 'static,
    V: Send + Clone + 'static,
{
}

pub struct Range<'a, K, V>
where
    K: Send + Ord + Clone + 'static,
    V: Send + Clone + 'static,
{
    inner: super::set::Range<'a, Pair<K, V>>,
}

impl<'a, K, V> Iterator for Range<'a, K, V>
where
    K: Send + Ord + Clone + 'static,
    V: Send + Clone + 'static,
{
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(entry) = self.inner.next() {
            return Some((&entry.key, &entry.value));
        }

        None
    }
}

impl<'a, K, V> DoubleEndedIterator for Range<'a, K, V>
where
    K: Send + Ord + Clone + 'static,
    V: Send + Clone + 'static,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        if let Some(entry) = self.inner.next_back() {
            return Some((&entry.key, &entry.value));
        }

        None
    }
}

impl<'a, K, V> FusedIterator for Range<'a, K, V>
where
    K: Send + Ord + Clone + 'static,
    V: Send + Clone + 'static,
{
}

impl<K: Send + Ord + Clone + 'static, V: Send + Clone + 'static> BTreeMap<K, V> {
    /// Makes a new, empty, persistent `BTreeMap`.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use indexset::concurrent::map::BTreeMap;
    ///
    /// let mut map = BTreeMap::new();
    ///
    /// // entries can now be inserted into the empty map
    /// map.insert(1, "a");
    /// ```
    pub fn new() -> Self {
        Self {
            set: Default::default(),
        }
    }
    /// Returns `true` if the map contains a value for the specified key.
    ///
    /// The key may be any borrowed form of the map's key type, but the ordering
    /// on the borrowed form *must* match the ordering on the key type.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use indexset::concurrent::map::BTreeMap;
    ///
    /// let mut map = BTreeMap::new();
    /// map.insert(1, "a");
    /// assert_eq!(map.contains_key(&1), true);
    /// assert_eq!(map.contains_key(&2), false);
    /// ```
    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        Pair<K, V>: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        self.set.contains(key)
    }
    /// Returns a reference to a pair whose key corresponds to the input.
    ///
    /// The key may be any borrowed form of the map's key type, but the ordering
    /// on the borrowed form *must* match the ordering on the key type.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use indexset::concurrent::map::BTreeMap;
    ///
    /// let mut map = BTreeMap::new();
    /// map.insert(1, "a");
    /// assert_eq!(map.get(&1).and_then(|e| Some(e.get().value)), Some("a"));
    /// assert_eq!(map.get(&2).and_then(|e| Some(e.get().value)), None);
    /// ```
    pub fn get<Q>(&self, key: &Q) -> Option<super::set::Ref<Pair<K, V>>>
    where
        Pair<K, V>: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        self.set.get(key)
    }
    /// Inserts a key-value pair into the map.
    ///
    /// If the map did not have this key present, it will be inserted.
    ///
    /// Otherwise, the value is updated.
    ///
    /// [module-level documentation]: index.html#insert-and-complex-keys
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use indexset::concurrent::map::BTreeMap;
    ///
    /// let mut map = BTreeMap::new();
    /// assert_eq!(map.insert(37, "a"), None);
    /// assert_eq!(map.len() == 0, false);
    ///
    /// map.insert(37, "b");
    /// assert_eq!(map.insert(37, "c"), Some("b"));
    /// assert_eq!(map.get(&37).and_then(|e| Some(e.get().value)), Some("c"));
    /// ```
    pub fn insert(&self, key: K, value: V) -> Option<V> {
        let new_entry = Pair { key, value };

        self.set
            .put_cdc(new_entry)
            .0
            .and_then(|pair| Some(pair.value))
    }
    pub fn insert_cdc(&self, key: K, value: V) -> (Option<V>, Vec<ChangeEvent<Pair<K, V>>>) {
        let new_entry = Pair { key, value };

        let (old_value, cdc) = self.set.put_cdc(new_entry);

        (old_value.and_then(|pair| Some(pair.value)), cdc)
    }
    /// Removes a key from the map, returning the key and the value if the key
    /// was previously in the map.
    ///
    /// The key may be any borrowed form of the map's key type, but the ordering
    /// on the borrowed form *must* match the ordering on the key type.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use indexset::concurrent::map::BTreeMap;
    ///
    /// let mut map = BTreeMap::new();
    /// map.insert(1, "a");
    /// assert_eq!(map.remove(&1), Some((1, "a")));
    /// assert_eq!(map.remove(&1), None);
    /// ```
    pub fn remove<Q>(&mut self, key: &Q) -> Option<(K, V)>
    where
        Pair<K, V>: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        return self
            .set
            .remove(key)
            .and_then(|pair| Some((pair.key, pair.value)));
    }
    pub fn remove_cdc<Q>(&self, key: &Q) -> (Option<(K, V)>, Vec<ChangeEvent<Pair<K, V>>>)
    where
        Pair<K, V>: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        let (old_value, cdc) = self.set.remove_cdc(key);

        (old_value.and_then(|pair| Some((pair.key, pair.value))), cdc)
    }
    /// Returns the number of elements in the map.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use indexset::concurrent::map::BTreeMap;
    ///
    /// let mut a = BTreeMap::new();
    /// assert_eq!(a.len(), 0);
    /// a.insert(1, "a");
    /// assert_eq!(a.len(), 1);
    /// ```
    pub fn len(&self) -> usize {
        self.set.len()
    }
    /// Gets an iterator over the entries of the map, sorted by key.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use indexset::concurrent::map::BTreeMap;
    ///
    /// let mut map = BTreeMap::new();
    /// map.insert(3, "c");
    /// map.insert(2, "b");
    /// map.insert(1, "a");
    ///
    /// for (key, value) in map.iter() {
    ///     println!("{key}: {value}");
    /// }
    ///
    /// let (first_key, first_value) = map.iter().next().unwrap();
    /// assert_eq!((*first_key, *first_value), (1, "a"));
    /// ```
    pub fn iter(&self) -> Iter<K, V> {
        Iter {
            inner: self.set.iter(),
        }
    }
}

#[cfg(test)]
mod cdc_tests {
    use super::*;

    #[derive(Debug, Default)]
    struct PersistedBTreeMap<K, V>
    where
        K: Ord + Clone,
        V: Clone + PartialEq,
    {
        nodes: std::collections::BTreeMap<K, Vec<Pair<K, V>>>,
    }

    impl<K: Ord + Clone, V: Clone + PartialEq> PersistedBTreeMap<K, V> {
        fn persist(&mut self, event: &ChangeEvent<Pair<K, V>>) {
            match event {
                ChangeEvent::InsertNode(max_key, node) => {
                    self.nodes
                        .insert(max_key.key.clone(), node.lock_arc().clone());
                }
                ChangeEvent::RemoveNode(max_key) => {
                    self.nodes.remove(&max_key.key);
                }
                ChangeEvent::InsertAt(node_max, pair) => {
                    if let Some(node) = self.nodes.get_mut(&node_max.key) {
                        let pos = node.binary_search(pair).unwrap_or_else(|p| p);
                        node.insert(pos, pair.clone());
                    }
                }
                ChangeEvent::RemoveAt(node_max, pair) => {
                    if let Some(node) = self.nodes.get_mut(&node_max.key) {
                        if let Ok(pos) = node.binary_search(pair) {
                            node.remove(pos);
                        }
                    }
                }
            }
        }

        fn contains_pair(&self, key: &K, value: &V) -> bool {
            for node in self.nodes.values() {
                if let Ok(pos) = node.binary_search(&Pair {
                    key: key.clone(),
                    value: value.clone(),
                }) {
                    if node[pos].value == *value {
                        return true;
                    }
                }
            }
            false
        }
    }

    #[test]
    fn test_cdc_single_insert() {
        let map = BTreeMap::new();
        let mut mock_state = PersistedBTreeMap::default();

        let (_, events) = map.insert_cdc(1, "a");

        for event in events {
            mock_state.persist(&event);
        }

        assert!(mock_state.contains_pair(&1, &"a"));
        assert!(map.contains_key(&1));
        assert_eq!(map.get(&1).unwrap().get().value, "a");

        let expected_state = map
            .set
            .index
            .iter()
            .map(|e| (e.key().clone().key, e.value().lock_arc().clone()))
            .collect::<_>();
        assert_eq!(mock_state.nodes, expected_state);
    }

    #[test]
    fn test_cdc_multiple_inserts() {
        let map = BTreeMap::new();
        let mut mock_state = PersistedBTreeMap::default();

        for i in 0..1024 {
            let (_, events) = map.insert_cdc(i, format!("val{}", i));

            for event in events {
                mock_state.persist(&event);
            }
        }

        for i in 0..1024 {
            assert!(mock_state.contains_pair(&i, &format!("val{}", i)));
            assert!(map.contains_key(&i));
            assert_eq!(map.get(&i).unwrap().get().value, format!("val{}", i));
        }

        let expected_state = map
            .set
            .index
            .iter()
            .map(|e| (e.key().clone().key, e.value().lock_arc().clone()))
            .collect::<_>();
        assert_eq!(mock_state.nodes, expected_state);
    }

    #[test]
    fn test_cdc_updates() {
        let map = BTreeMap::new();
        let mut mock_state = PersistedBTreeMap::default();

        let (_, events) = map.insert_cdc(1, "a");
        for event in events {
            mock_state.persist(&event);
        }

        let (_, events) = map.insert_cdc(1, "b");
        for event in events {
            mock_state.persist(&event);
        }

        assert!(mock_state.contains_pair(&1, &"b"));
        assert!(!mock_state.contains_pair(&1, &"a"));
        assert!(map.contains_key(&1));
        assert_eq!(map.get(&1).unwrap().get().value, "b");

        let expected_state = map
            .set
            .index
            .iter()
            .map(|e| (e.key().clone().key, e.value().lock_arc().clone()))
            .collect::<_>();
        assert_eq!(mock_state.nodes, expected_state);
    }

    #[test]
    fn test_cdc_node_splits() {
        let map = BTreeMap::new();
        let mut mock_state = PersistedBTreeMap::default();

        let n = crate::core::constants::DEFAULT_INNER_SIZE + 10;

        for i in 0..n {
            let (_, events) = map.insert_cdc(i, format!("val{}", i));
            for event in events {
                mock_state.persist(&event);
            }
        }

        for i in 0..n {
            assert!(mock_state.contains_pair(&i, &format!("val{}", i)));
            assert!(map.contains_key(&i));
            assert_eq!(map.get(&i).unwrap().get().value, format!("val{}", i));
        }

        assert!(mock_state.nodes.len() > 1);

        let expected_state = map
            .set
            .index
            .iter()
            .map(|e| (e.key().clone().key, e.value().lock_arc().clone()))
            .collect::<_>();
        assert_eq!(mock_state.nodes, expected_state);
    }
}
