use std::{
    hash::Hash,
    marker::PhantomData,
    ops::{Bound, RangeBounds},
};

use crate::{sys, Error};

/// Set objects represent a mathematical set of integer values.
///
/// Sets are used in non-shaping APIs to query certain sets of characters or glyphs, or other integer values.
pub struct Set<'a, T>(InnerSet, PhantomData<(&'a (), T)>);

impl<T> Set<'static, T> {
    /// Creates a new, initially empty set.
    pub(crate) fn new() -> Result<Self, Error> {
        let set = unsafe { sys::hb_set_create() };
        if set.is_null() {
            return Err(Error::AllocationError);
        }
        Ok(Self(InnerSet(set), PhantomData))
    }
}

impl<'a, T> Set<'a, T> {
    /// Tests whether a set is empty (contains no elements)
    pub fn is_empty(&self) -> bool {
        (unsafe { sys::hb_set_is_empty(self.as_raw()) }) != 0
    }

    /// Returns the number of elements in the set.
    pub fn len(&self) -> usize {
        (unsafe { sys::hb_set_get_population(self.as_raw()) }) as usize
    }

    /// Clears out the contents of a set.
    pub fn clear(&mut self) {
        unsafe { sys::hb_set_clear(self.as_raw()) }
    }
}
impl<'a, T> Set<'a, T>
where
    T: Into<u32> + Copy,
{
    /// Tests whether a value belongs to set.
    pub fn contains(&self, value: T) -> bool {
        (unsafe { sys::hb_set_has(self.as_raw(), value.into()) }) != 0
    }

    /// Inserts a value to set.
    pub fn insert(&mut self, value: T) {
        unsafe { sys::hb_set_add(self.as_raw(), value.into()) }
    }

    /// Removes a value from set.
    pub fn remove(&mut self, value: T) {
        unsafe { sys::hb_set_del(self.as_raw(), value.into()) }
    }

    /// Converts a range to inclusive bounds.
    fn range_to_bounds(range: impl RangeBounds<T>) -> Option<(u32, u32)> {
        fn bound_to_u32<T: Into<u32> + Copy>(bound: Bound<&T>) -> Bound<u32> {
            match bound {
                Bound::Included(&b) => Bound::Included(b.into()),
                Bound::Excluded(&b) => Bound::Excluded(b.into()),
                Bound::Unbounded => Bound::Unbounded,
            }
        }
        let lower = match bound_to_u32(range.start_bound()) {
            Bound::Included(lower) => lower,
            Bound::Excluded(lower) => {
                if lower == u32::MAX {
                    return None;
                } else {
                    lower + 1
                }
            }
            Bound::Unbounded => 0,
        };
        let upper = match bound_to_u32(range.end_bound()) {
            Bound::Included(upper) => upper,
            Bound::Excluded(upper) => {
                if upper == 0 {
                    return None;
                } else {
                    upper - 1
                }
            }
            Bound::Unbounded => u32::MAX,
        };
        if upper < lower {
            return None;
        }
        Some((lower, upper))
    }

    /// Inserts a range of values to set.
    pub fn insert_range(&mut self, range: impl RangeBounds<T>) {
        let Some((lower, upper)) = Self::range_to_bounds(range) else {
            return;
        };
        unsafe { sys::hb_set_add_range(self.as_raw(), lower, upper) }
    }

    /// Removes a range of values from set.
    pub fn remove_range(&mut self, range: impl RangeBounds<T>) {
        // TODO: Assert that sys::HB_SET_VALUE_INVALID is u32::MAX like it should be
        // const _: () = assert!(u32::MAX <= sys::HB_SET_VALUE_INVALID);
        let Some((lower, upper)) = Self::range_to_bounds(range) else {
            return;
        };
        unsafe { sys::hb_set_del_range(self.as_raw(), lower, upper) }
    }
}

impl<'a, T> Set<'a, T> {
    /// Converts the set into raw [`sys::hb_set_t`] object.
    ///
    /// This method transfers the ownership of the set to the caller. It is up to the caller to call
    /// [`sys::hb_set_destroy`] to free the object, or call [`Self::from_raw`] to convert it back into [`Set`].
    pub fn into_raw(self) -> *mut sys::hb_set_t {
        let ptr = self.0 .0;
        std::mem::forget(self);
        ptr
    }

    /// Exposes the raw inner pointer without transferring the ownership.
    ///
    /// Unlike [`Self::into_raw`], this method does not transfer the ownership of the pointer to the caller.
    pub fn as_raw(&self) -> *mut sys::hb_set_t {
        self.0 .0
    }

    /// Constructs a set from raw [`sys::hb_set_t`] object.
    ///
    /// # Safety
    /// The given `set` pointer must either be constructed by some Harfbuzz function, or be returned from
    /// [`Self::into_raw`].
    pub unsafe fn from_raw(set: *mut sys::hb_set_t) -> Self {
        Self(InnerSet(set), PhantomData)
    }
}

impl<'a, T> Hash for Set<'a, T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        unsafe { sys::hb_set_hash(self.as_raw()) }.hash(state);
    }
}

/// Implementation detail of Set to hide source reference from drop check.
///
/// If the pointer was directly contained in [`Set`] with `Drop` implemented, the following code would not compile:
/// ```rust
/// # use hb_subset::bindings::*;
/// let mut subset = SubsetInput::new().unwrap();
/// let mut unicode_set = subset.unicode_set();
/// // drop(unicode_set);                               // This needs to be called to delete unicode_set,
/// # let font = FontFace::new(Blob::from_bytes(&[]).unwrap()).unwrap();
/// let new_font = subset.subset_font(&font).unwrap();  // otherwise this line would not compile as unicode_set is already
///                                                     // holding a mutable reference to subset.
/// ```
pub(crate) struct InnerSet(*mut sys::hb_set_t);

impl Drop for InnerSet {
    fn drop(&mut self) {
        unsafe { sys::hb_set_destroy(self.0) }
    }
}

/// Set over unicodecode points.
pub type CharSet<'a> = Set<'a, char>;

/// Set over [`u32`]s.
///
/// [`U32Set`] is commonly used to represent sets of glyph IDs.
pub type U32Set<'a> = Set<'a, u32>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_empty_works() {
        let mut set = U32Set::new().unwrap();
        assert!(set.is_empty());
        assert!(set.is_empty());
        set.insert(10);
        assert!(!set.is_empty());
        set.insert(20);
        assert!(!set.is_empty());
        set.remove(10);
        assert!(!set.is_empty());
        set.remove(20);
        assert!(set.is_empty());
    }

    #[test]
    fn len_works() {
        let mut set = U32Set::new().unwrap();
        assert_eq!(set.len(), 0);
        set.insert(10);
        assert_eq!(set.len(), 1);
        set.insert_range(5..15);
        assert_eq!(set.len(), 10);
        set.remove(13);
        assert_eq!(set.len(), 9);
    }

    #[test]
    fn clear_empties_set() {
        let mut set = U32Set::new().unwrap();
        set.insert_range(123..456);
        assert!(!set.is_empty());
        assert_eq!(set.len(), 333);
        set.clear();
        assert!(set.is_empty());
        assert_eq!(set.len(), 0);
    }

    #[test]
    fn set_contains_inserted_values() {
        let mut set = U32Set::new().unwrap();
        set.insert(1);
        assert!(!set.contains(3));
        set.insert(1);
        assert!(!set.contains(3));
        set.insert(3);
        assert!(set.contains(3));
        set.remove(1);
        assert!(set.contains(3));
        set.remove(3);
        assert!(!set.contains(3));
    }

    #[test]
    fn range_insertions_and_deletions_work() {
        let mut set = U32Set::new().unwrap();
        set.insert_range(0..100);
        assert_eq!(set.len(), 100);
        set.remove_range(21..=30);
        assert_eq!(set.len(), 90);
        set.remove_range(90..200);
        assert_eq!(set.len(), 80);
    }

    #[test]
    fn convert_into_raw_and_back() {
        let set = U32Set::new().unwrap();
        let set_ptr = set.into_raw();
        let set = unsafe { U32Set::from_raw(set_ptr) };
        drop(set);
    }
}
