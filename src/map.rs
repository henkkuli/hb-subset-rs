//! Map represents an integer-to-integer mapping.
//!
//! While map in this module can be used as a general-purpose map in Rust code, it is recommended that you instead use
//! [`std::collections::BTreeMap`] or [`std::collections::HashMap`] as those are implemented directly in Rust and do not
//! rely on FFI to work.

use std::{
    fmt,
    hash::Hash,
    iter::{FilterMap, FusedIterator},
    marker::PhantomData,
};

use crate::{set::Set, sys, AllocationError};

/// Map objects are integer-to-integer hash-maps.
///
/// The map can be specialized to work over other integer-like types, like [`char`]s.
pub struct Map<'a, K = u32, V = u32>(InnerMap, PhantomData<(&'a (), (K, V))>);

impl<K, V> Map<'static, K, V> {
    /// Creates a new, initially empty map.
    #[doc(alias = "hb_map_create")]
    pub fn new() -> Result<Self, AllocationError> {
        let map = unsafe { sys::hb_map_create() };
        if map.is_null() {
            return Err(AllocationError);
        }
        Ok(Self(InnerMap(map), PhantomData))
    }
}

impl<'a, K, V> Map<'a, K, V> {
    /// Tests whether map is empty i.e. contains no elements.
    #[doc(alias = "hb_map_is_empty")]
    pub fn is_empty(&self) -> bool {
        (unsafe { sys::hb_map_is_empty(self.as_raw()) }) != 0
    }

    /// Returns the number of key-value pairs in the set.
    #[doc(alias = "hb_map_get_population")]
    pub fn len(&self) -> usize {
        (unsafe { sys::hb_map_get_population(self.as_raw()) }) as usize
    }

    /// Clears out the contents of a map.
    #[doc(alias = "hb_map_clear")]
    pub fn clear(&mut self) {
        unsafe { sys::hb_map_clear(self.as_raw()) }
    }

    /// Constructs a copy of the map with `'static` lifetime.
    #[doc(alias = "hb_map_copy")]
    pub fn clone_static(&self) -> Map<'static, K, V> {
        Map(
            InnerMap(unsafe { sys::hb_map_copy(self.as_raw()) }),
            PhantomData,
        )
    }

    /// Adds the contents of `other` to map.
    pub fn update(&mut self, other: &Self) {
        unsafe { sys::hb_map_update(self.as_raw(), other.as_raw()) }
    }

    /// Gets the set of keys that have been defined.
    #[doc(alias = "hb_map_keys")]
    pub fn keys(&self) -> Result<Set<'static, K>, AllocationError> {
        let set = Set::new()?;
        unsafe { sys::hb_map_keys(self.as_raw(), set.as_raw()) };
        Ok(set)
    }

    /// Gets the set of values that are associated with a key.
    #[doc(alias = "hb_map_values")]
    pub fn values(&self) -> Result<Set<'static, V>, AllocationError> {
        let set = Set::new()?;
        unsafe { sys::hb_map_values(self.as_raw(), set.as_raw()) };
        Ok(set)
    }
}

impl<'a, K, V> Map<'a, K, V>
where
    K: Into<u32>,
    V: Into<u32>,
{
    /// Tests whether the map contains a key.
    #[doc(alias = "hb_map_has")]
    pub fn contains(&self, key: K) -> bool {
        (unsafe { sys::hb_map_has(self.as_raw(), key.into()) }) != 0
    }

    /// Removes key and its associated value from map.
    #[doc(alias = "hb_map_del")]
    pub fn remove(&mut self, key: K) {
        unsafe { sys::hb_map_del(self.as_raw(), key.into()) }
    }

    /// Inserts a key-value pair to map.
    #[doc(alias = "hb_map_set")]
    pub fn insert(&mut self, key: K, value: V) {
        let key = key.into();
        let value = value.into();
        unsafe { sys::hb_map_set(self.as_raw(), key, value) }
    }
}

impl<'a, K, V> Map<'a, K, V>
where
    K: Into<u32>,
    V: TryFrom<u32>,
{
    /// Gets a value based on a key.
    #[doc(alias = "hb_map_get")]
    pub fn get(&self, key: K) -> Option<V> {
        let key = key.into();
        if (unsafe { sys::hb_map_has(self.as_raw(), key) }) != 0 {
            V::try_from(unsafe { sys::hb_map_get(self.as_raw(), key) }).ok()
        } else {
            None
        }
    }
}

impl<'a, K, V> Map<'a, K, V>
where
    K: TryFrom<u32>,
    V: TryFrom<u32>,
{
    /// Gets an iterator over key-value pairs stored in the map.
    #[doc(alias = "hb_map_next")]
    pub fn iter(&self) -> Iter<'_, 'a, K, V> {
        Iter(
            IterImpl::new(self).filter_map(|(k, v)| Some((k.try_into().ok()?, v.try_into().ok()?))),
        )
    }
}

impl<'a, K, V> Map<'a, K, V> {
    /// Converts the map into raw [`sys::hb_map_t`] pointer.
    ///
    /// This method transfers the ownership of the map to the caller. It is up to the caller to call
    /// [`sys::hb_map_destroy`] to free the object, or call [`Self::from_raw`] to convert it back into [`Map`].
    pub fn into_raw(self) -> *mut sys::hb_map_t {
        let ptr = self.0 .0;
        std::mem::forget(self);
        ptr
    }

    /// Exposes the raw inner pointer without transferring the ownership.
    ///
    /// Unlike [`Self::into_raw`], this method does not transfer the ownership of the pointer to the caller.
    pub fn as_raw(&self) -> *mut sys::hb_map_t {
        self.0 .0
    }

    /// Constructs a map from raw [`sys::hb_map_t`] object.
    ///
    /// # Safety
    /// The given `map` pointer must either be constructed by some Harfbuzz function, or be returned from
    /// [`Self::into_raw`].
    pub unsafe fn from_raw(map: *mut sys::hb_map_t) -> Self {
        Self(InnerMap(map), PhantomData)
    }
}

impl<'a, K, V> Hash for Map<'a, K, V> {
    #[doc(alias = "hb_map_hash")]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        unsafe { sys::hb_map_hash(self.as_raw()) }.hash(state);
    }
}

impl<'a, K, V> PartialEq for Map<'a, K, V> {
    #[doc(alias = "hb_map_is_equal")]
    fn eq(&self, other: &Self) -> bool {
        (unsafe { sys::hb_map_is_equal(self.as_raw(), other.as_raw()) }) != 0
    }
}

impl<'a, K, V> Eq for Map<'a, K, V>
where
    K: Eq,
    V: Eq,
{
}

impl<'a, K, V> Clone for Map<'a, K, V> {
    fn clone(&self) -> Self {
        self.clone_static()
    }
}

impl<'a, K, V> fmt::Debug for Map<'a, K, V>
where
    K: TryFrom<u32> + fmt::Debug,
    V: TryFrom<u32> + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self).finish()
    }
}

impl<'a, K, V> FromIterator<(K, V)> for Map<'a, K, V>
where
    K: Into<u32>,
    V: Into<u32>,
{
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        let mut map = Map::new().unwrap();
        for (key, value) in iter {
            map.insert(key, value);
        }
        map
    }
}

impl<'m, 'a, K, V> IntoIterator for &'m Map<'a, K, V>
where
    K: TryFrom<u32>,
    V: TryFrom<u32>,
{
    type Item = (K, V);
    type IntoIter = Iter<'m, 'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Iterator over [`Map`] key-value pairs.
///
/// Use [`Map::iter`] to construct [`Iter`].
pub struct Iter<'m, 'a, K, V>(IterFilter<'m, 'a, K, V>);
type IterFilter<'m, 'a, K, V> = FilterMap<IterImpl<'m, 'a, K, V>, fn((u32, u32)) -> Option<(K, V)>>;

impl<'m, 'a, K, V> Iterator for Iter<'m, 'a, K, V>
where
    K: TryFrom<u32>,
    V: TryFrom<u32>,
{
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<'m, 'a, K, V> FusedIterator for Iter<'m, 'a, K, V>
where
    K: TryFrom<u32>,
    V: TryFrom<u32>,
{
}

/// Iterator over raw elements over map, disregarding whether they can be represented as (K, V).
struct IterImpl<'m, 'a, K, V>(&'m Map<'a, K, V>, i32);

impl<'m, 'a, K, V> IterImpl<'m, 'a, K, V> {
    fn new(map: &'m Map<'a, K, V>) -> Self {
        Self(map, -1)
    }
}

impl<'m, 'a, K, V> Iterator for IterImpl<'m, 'a, K, V> {
    type Item = (u32, u32);

    fn next(&mut self) -> Option<Self::Item> {
        let mut key = 0;
        let mut value = 0;
        let prev_state = self.1;
        let has_next = unsafe {
            sys::hb_map_next(
                self.0.as_raw(),
                &mut self.1 as *mut i32,
                &mut key as *mut u32,
                &mut value as *mut u32,
            )
        } != 0;
        if has_next {
            Some((key, value))
        } else {
            self.1 = prev_state; // To fuse the iterator
            None
        }
    }
}

/// Implementation detail of Map to hide source reference from drop check.
///
/// See [`set::InnerSet`] for more detailed explanation.
struct InnerMap(*mut sys::hb_map_t);

impl Drop for InnerMap {
    #[doc(alias = "hb_map_destroy")]
    fn drop(&mut self) {
        unsafe { sys::hb_map_destroy(self.0) }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;

    #[test]
    fn is_empty_works() {
        let mut map = Map::<u32, u32>::new().unwrap();
        assert!(map.is_empty());
        map.insert(0, 0);
        assert!(!map.is_empty());
        map.insert(0, 10);
        assert!(!map.is_empty());
        map.remove(0);
        assert!(map.is_empty());
    }

    #[test]
    fn len_works() {
        let mut map = Map::<u32, u32>::new().unwrap();
        assert_eq!(map.len(), 0);
        map.insert(0, 0);
        assert_eq!(map.len(), 1);
        map.insert(0, 10);
        assert_eq!(map.len(), 1);
        map.insert(1, 10);
        assert_eq!(map.len(), 2);
        map.remove(0);
        assert_eq!(map.len(), 1);
        map.remove(0);
        assert_eq!(map.len(), 1);
        map.remove(1);
        assert_eq!(map.len(), 0);
    }

    #[test]
    fn clear_works() {
        let mut map = Map::<u32, u32>::from_iter([(0, 1), (0, 2), (1, 3)]);
        assert_eq!(map.len(), 2);
        map.clear();
        assert!(map.is_empty());
        assert_eq!(map.len(), 0);
    }

    #[test]
    fn clone_does_not_change_original() {
        let mut a = Map::<u32, u32>::from_iter([(0, 1), (1, 2), (10, 11)]);
        let mut b = a.clone();
        assert_eq!(a, b);
        assert_eq!(b.len(), 3);
        a.insert(20, 21);
        assert_eq!(a.len(), 4);
        assert_eq!(b.len(), 3);
        b.remove(0);
        assert_eq!(a.len(), 4);
        assert_eq!(b.len(), 2);
    }

    #[test]
    fn update_replaces_keys() {
        let mut a = Map::<u32, u32>::from_iter([(0, 0), (1, 0), (10, 0)]);
        let b = Map::<u32, u32>::from_iter([(1, 1), (5, 1), (11, 1)]);
        a.update(&b);
        assert_eq!(a.len(), 5);
        assert_eq!(a.get(0).unwrap(), 0);
        assert_eq!(a.get(1).unwrap(), 1);
        assert_eq!(a.get(5).unwrap(), 1);
        assert_eq!(a.get(10).unwrap(), 0);
        assert_eq!(a.get(11).unwrap(), 1);
    }

    #[test]
    fn keys_works() {
        assert_eq!(
            Map::<u32, u32>::from_iter([]).keys().unwrap(),
            Set::from_iter([])
        );
        assert_eq!(
            Map::<u32, u32>::from_iter([(0, 100), (1, 101), (10, 110)])
                .keys()
                .unwrap(),
            Set::from_iter([0, 1, 10])
        );
    }

    #[test]
    fn values_works() {
        assert_eq!(
            Map::<u32, u32>::from_iter([]).values().unwrap(),
            Set::from_iter([])
        );
        assert_eq!(
            Map::<u32, u32>::from_iter([(0, 100), (1, 101), (10, 110)])
                .values()
                .unwrap(),
            Set::from_iter([100, 101, 110])
        );
    }

    #[test]
    fn contains_works() {
        let mut map = Map::<u32, u32>::new().unwrap();
        assert!(!map.contains(0));
        map.insert(0, 0);
        assert!(map.contains(0));
        map.insert(0, 10);
        assert!(map.contains(0));
        assert!(!map.contains(1));
        map.insert(1, 10);
        assert!(map.contains(0));
        assert!(map.contains(1));
        map.remove(0);
        assert!(!map.contains(0));
        assert!(map.contains(1));
        map.remove(0);
        assert!(!map.contains(0));
        assert!(map.contains(1));
        map.remove(1);
        assert!(!map.contains(0));
        assert!(!map.contains(1));
    }

    #[track_caller]
    fn assert_set_is_correct<T: Ord + fmt::Debug>(
        left: impl IntoIterator<Item = T>,
        right: impl IntoIterator<Item = T>,
    ) {
        let left: BTreeSet<T> = BTreeSet::from_iter(left);
        let right: BTreeSet<T> = BTreeSet::from_iter(right);
        assert_eq!(left, right);
    }

    #[test]
    #[should_panic]
    fn assert_set_is_correct_detects_differences() {
        assert_set_is_correct([1, 2], [1, 2, 3]);
    }

    #[test]
    fn iter_works() {
        let mut map = Map::<u32, u32>::from_iter([(0, 100), (4, 104)]);
        assert_set_is_correct(map.iter(), [(0, 100), (4, 104)]);
        map.insert(u32::MAX, u32::MAX);
        assert_set_is_correct(map.iter(), [(0, 100), (4, 104), (u32::MAX, u32::MAX)]);
    }

    #[test]
    fn iter_is_fused() {
        let map = Map::<u32, u32>::from_iter([(0, 100), (4, 104)]);
        let mut iter = map.iter();
        assert!(iter.next().is_some());
        assert!(iter.next().is_some());
        assert!(iter.next().is_none());
        assert!(iter.next().is_none());
        assert!(iter.next().is_none());
        assert!(iter.next().is_none());
        assert!(iter.next().is_none());
    }

    #[test]
    fn iter_of_invalid_codepoints_works() {
        let mut map = Map::<u32, u32>::new().unwrap();
        // Close to invalid code points
        map.insert(0xD7FF, 10);
        map.insert(0xE000, 10);
        map.insert(20, 0xD7FF);
        map.insert(21, 0xE000);

        // Invalid code points in key
        map.insert(0xD800, 3);
        map.insert(0xD912, 4);
        map.insert(0xDFFF, 5);

        // Invalid code points in value
        map.insert(23, 0xD800);
        map.insert(24, 0xD912);
        map.insert(25, 0xDFFF);

        let char_to_u32_map = unsafe { Map::<char, u32>::from_raw(map.clone().into_raw()) };
        assert_set_is_correct(
            &char_to_u32_map,
            [
                ('\u{D7FF}', 10),
                ('\u{E000}', 10),
                ('\u{14}', 0xD7FF),
                ('\u{15}', 0xE000),
                ('\u{17}', 0xD800),
                ('\u{18}', 0xD912),
                ('\u{19}', 0xDFFF),
            ],
        );

        let u32_to_char_map = unsafe { Map::<u32, char>::from_raw(map.clone().into_raw()) };
        assert_set_is_correct(
            &u32_to_char_map,
            [
                (0xD7FF, '\u{0a}'),
                (0xE000, '\u{0a}'),
                (20, '\u{D7FF}'),
                (21, '\u{E000}'),
                (0xD800, '\u{3}'),
                (0xD912, '\u{4}'),
                (0xDFFF, '\u{5}'),
            ],
        );

        let char_to_char_map = unsafe { Map::<char, char>::from_raw(map.clone().into_raw()) };
        assert_set_is_correct(
            &char_to_char_map,
            [
                ('\u{D7FF}', '\u{0a}'),
                ('\u{E000}', '\u{0a}'),
                ('\u{14}', '\u{D7FF}'),
                ('\u{15}', '\u{E000}'),
            ],
        );
    }

    #[test]
    fn value_can_be_u32_max() {
        let mut map = Map::<u32, u32>::new().unwrap();
        map.insert(0, u32::MAX - 1);
        map.insert(1, u32::MAX);
        assert_eq!(map.len(), 2);
        assert_eq!(map.get(0).unwrap(), u32::MAX - 1);
        assert_eq!(map.get(1).unwrap(), u32::MAX);
    }

    #[test]
    fn key_can_be_u32_max() {
        let mut map = Map::<u32, u32>::new().unwrap();
        map.insert(u32::MAX - 1, 10);
        map.insert(u32::MAX, 20);
        assert_eq!(map.len(), 2);
        assert_eq!(map.get(u32::MAX - 1).unwrap(), 10);
        assert_eq!(map.get(u32::MAX).unwrap(), 20);
    }
}
