use std::{
    any::TypeId,
    borrow::Borrow,
    fmt,
    hash::Hash,
    iter::{FilterMap, FusedIterator},
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
    #[doc(alias = "hb_set_create")]
    pub fn new() -> Result<Self, Error> {
        let set = unsafe { sys::hb_set_create() };
        if set.is_null() {
            return Err(Error::AllocationError);
        }
        Ok(Self(InnerSet(set), PhantomData))
    }
}

impl<'a, T> Set<'a, T> {
    /// Tests whether a set is empty (contains no elements)
    #[doc(alias = "hb_set_is_empty")]
    pub fn is_empty(&self) -> bool {
        (unsafe { sys::hb_set_is_empty(self.as_raw()) }) != 0
    }

    /// Returns the number of elements in the set.
    ///
    /// Note that this returns the number of elements in the underlying raw set over [`u32`], *not* the number of
    /// elements that can be represented as `T`. This is especially evident when the set is over [`char`]s and invalid
    /// code points have been added with [`Self::insert_range`].
    /// ```rust
    /// # use hb_subset::bindings::CharSet;
    /// let mut set = CharSet::new().unwrap();
    /// set.insert_range('\u{D7FF}'..'\u{E000}'); // Add all surrogate pairs (and \u{D7FF} for technical reasons)
    /// assert_eq!(set.len(), 2049);
    /// ```
    #[doc(alias = "hb_set_get_population")]
    pub fn len(&self) -> usize {
        (unsafe { sys::hb_set_get_population(self.as_raw()) }) as usize
    }

    /// Clears out the contents of a set.
    #[doc(alias = "hb_set_clear")]
    pub fn clear(&mut self) {
        unsafe { sys::hb_set_clear(self.as_raw()) }
    }

    /// Makes the contents of `self` equal to the contents of `other`.
    #[doc(alias = "hb_set_set")]
    pub fn copy_from(&mut self, other: &Self) {
        unsafe { sys::hb_set_set(self.as_raw(), other.as_raw()) }
    }

    /// Tests whether `self` contains `other` set.
    #[doc(alias = "hb_set_is_subset")]
    pub fn contains_set(&self, other: &Self) -> bool {
        (unsafe { sys::hb_set_is_subset(other.as_raw(), self.as_raw()) }) != 0
    }

    /// Constructs a copy of the set with `'static` lifetime.
    #[doc(alias = "hb_set_copy")]
    pub fn clone_static(&self) -> Set<'static, T> {
        Set(
            InnerSet(unsafe { sys::hb_set_copy(self.as_raw()) }),
            PhantomData,
        )
    }
}

impl<'a, T> Set<'a, T>
where
    T: Into<u32> + Copy + 'static,
{
    /// Tests whether a value belongs to set.
    #[doc(alias = "hb_set_has")]
    pub fn contains(&self, value: T) -> bool {
        (unsafe { sys::hb_set_has(self.as_raw(), value.into()) }) != 0
    }

    /// Inserts a value to set.
    ///
    /// # Panics
    ///
    /// Will panic if `value` is [`sys::HB_SET_VALUE_INVALID`].
    #[doc(alias = "hb_set_add")]
    pub fn insert(&mut self, value: T) {
        let value = value.into();
        assert_ne!(value, sys::HB_SET_VALUE_INVALID);
        unsafe { sys::hb_set_add(self.as_raw(), value) }
    }

    /// Removes a value from set.
    #[doc(alias = "hb_set_del")]
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
            Bound::Included(upper) => {
                assert_ne!(upper, sys::HB_SET_VALUE_INVALID);
                upper
            }
            Bound::Excluded(upper) => {
                if upper == 0 {
                    return None;
                } else {
                    upper - 1
                }
            }
            Bound::Unbounded => {
                // Optimization to allow half-open intervals with character sets
                if TypeId::of::<T>() == TypeId::of::<char>() {
                    char::MAX as u32
                } else {
                    u32::MAX - 1
                }
            }
        };
        if upper < lower {
            return None;
        }
        Some((lower, upper))
    }

    /// Inserts a range of values to set.
    ///
    /// # Panics
    ///
    /// Will panic if `range` explicitly contains [`sys::HB_SET_VALUE_INVALID`]:
    /// ```should_panic
    /// # use hb_subset::bindings::U32Set;
    /// U32Set::new().unwrap().insert_range(u32::MAX-10..=u32::MAX);
    /// ```
    /// These still work:
    /// ```rust
    /// # use hb_subset::bindings::U32Set;
    /// U32Set::new().unwrap().insert_range(u32::MAX-10..);
    /// U32Set::new().unwrap().insert_range(u32::MAX-10..u32::MAX);
    /// ```
    #[doc(alias = "hb_set_add_range")]
    pub fn insert_range(&mut self, range: impl RangeBounds<T>) {
        let Some((lower, upper)) = Self::range_to_bounds(range) else {
            return;
        };
        unsafe { sys::hb_set_add_range(self.as_raw(), lower, upper) }
    }

    /// Removes a range of values from set.
    #[doc(alias = "hb_set_del_range")]
    pub fn remove_range(&mut self, range: impl RangeBounds<T>) {
        // TODO: Assert that sys::HB_SET_VALUE_INVALID is u32::MAX like it should be
        #[allow(clippy::assertions_on_constants, clippy::absurd_extreme_comparisons)]
        const _: () = assert!(u32::MAX <= sys::HB_SET_VALUE_INVALID);
        let Some((lower, upper)) = Self::range_to_bounds(range) else {
            return;
        };
        unsafe { sys::hb_set_del_range(self.as_raw(), lower, upper) }
    }
}

impl<'a, T> Set<'a, T>
where
    T: TryFrom<u32>,
{
    /// Constructs an iterator over the set.
    #[doc(alias = "hb_set_next")]
    #[doc(alias = "hb_set_previous")]
    pub fn iter(&self) -> SetIter<'_, 'a, T> {
        SetIter(InnerSetIter::new(self).filter_map(|v| v.try_into().ok()))
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
    #[doc(alias = "hb_set_hash")]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        unsafe { sys::hb_set_hash(self.as_raw()) }.hash(state);
    }
}

impl<'a, T> PartialEq for Set<'a, T> {
    #[doc(alias = "hb_set_is_equal")]
    fn eq(&self, other: &Self) -> bool {
        (unsafe { sys::hb_set_is_equal(self.as_raw(), other.as_raw()) }) != 0
    }
}

impl<'a, T> Eq for Set<'a, T> {}

impl<'a, T> Clone for Set<'a, T> {
    fn clone(&self) -> Self {
        self.clone_static()
    }
}

impl<'a, T> fmt::Debug for Set<'a, T>
where
    T: TryFrom<u32> + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set().entries(self).finish()
    }
}

impl<'s, 'a, T> IntoIterator for &'s Set<'a, T>
where
    T: TryFrom<u32>,
{
    type Item = T;

    type IntoIter = SetIter<'s, 'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Iterator over [`Set`].
///
/// Use [`Set::iter`] to construct a [`SetIter`].
pub struct SetIter<'s, 'a, T>(SetIterFilter<'s, 'a, T>);
type SetIterFilter<'s, 'a, T> = FilterMap<InnerSetIter<'s, 'a, T>, fn(u32) -> Option<T>>;

impl<'s, 'a, T> Iterator for SetIter<'s, 'a, T>
where
    T: TryFrom<u32>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<'s, 'a, T> DoubleEndedIterator for SetIter<'s, 'a, T>
where
    T: TryFrom<u32>,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back()
    }
}

impl<'s, 'a, T> FusedIterator for SetIter<'s, 'a, T> where T: TryFrom<u32> {}

pub struct InnerSetIter<'s, 'a, T>(&'s Set<'a, T>, u32, u32);

impl<'s, 'a, T> InnerSetIter<'s, 'a, T> {
    const LAST_VALUE: u32 = sys::HB_SET_VALUE_INVALID - 1;
    fn new(set: &'s Set<'a, T>) -> Self {
        #[allow(clippy::assertions_on_constants, clippy::absurd_extreme_comparisons)]
        const _: () = assert!(u32::MAX == sys::HB_SET_VALUE_INVALID);
        Self(set, sys::HB_SET_VALUE_INVALID, sys::HB_SET_VALUE_INVALID)
    }

    fn mark_ended(&mut self) {
        self.1 = Self::LAST_VALUE;
        self.2 = 0;
    }
}

impl<'s, 'a, T> Iterator for InnerSetIter<'s, 'a, T> {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        match self.1 {
            Self::LAST_VALUE => {
                // Previously last possible value was returned, so the iterator must have been exhausted
                None
            }
            _ => {
                let has_value =
                    (unsafe { sys::hb_set_next(self.0.as_raw(), &mut self.1 as *mut u32) }) != 0;
                if has_value {
                    if self.1 >= self.2 {
                        self.mark_ended();
                        None
                    } else {
                        Some(self.1)
                    }
                } else {
                    self.mark_ended();
                    None
                }
            }
        }
    }
}

impl<'s, 'a, T> DoubleEndedIterator for InnerSetIter<'s, 'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        match self.2 {
            0 => {
                // 0 has been returned, so nothing can be returned from this iterator anymore
                None
            }
            _ => {
                let has_value =
                    (unsafe { sys::hb_set_previous(self.0.as_raw(), &mut self.2 as *mut u32) })
                        != 0;
                if has_value {
                    if self.1 != sys::HB_SET_VALUE_INVALID && self.1 >= self.2 {
                        self.mark_ended();
                        None
                    } else {
                        Some(self.2)
                    }
                } else {
                    self.mark_ended();
                    None
                }
            }
        }
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
struct InnerSet(*mut sys::hb_set_t);

impl Drop for InnerSet {
    #[doc(alias = "hb_set_destroy")]
    fn drop(&mut self) {
        unsafe { sys::hb_set_destroy(self.0) }
    }
}

/// Set over unicodecode points.
pub type CharSet<'a> = Set<'a, char>;

/// Set over [`u32`]s, except [`u32::MAX`].
///
/// Trying to insert [`u32::MAX`] will cause a panic. [`U32Set`] is commonly used to represent sets of glyph IDs.
pub type U32Set<'a> = Set<'a, u32>;

/// Set over [`Tag`]s.
pub type TagSet<'a> = Set<'a, Tag>;

/// Four byte integers, each byte representing a character.
///
/// Tags are used to identify tables, design-variation axes, scripts, languages, font features, and baselines with
/// human-readable names.
#[derive(Clone, Copy)]
pub struct Tag(u32);

impl Tag {
    /// Constructs a new tag from bytes.
    ///
    /// # Example
    /// ```
    /// # use hb_subset::bindings::*;
    /// let mut subset = SubsetInput::new().unwrap();
    /// // Remove character-to-glyph mapping data. This can be useful in PDF files where
    /// // the mapping and positioning has already been done.
    /// subset.drop_table_tag_set().insert(Tag::new(b"cmap"));
    /// ```
    pub fn new(tag: impl Borrow<[u8; 4]>) -> Self {
        Self(u32::from_be_bytes(*tag.borrow()))
    }
}

impl From<Tag> for u32 {
    fn from(tag: Tag) -> Self {
        tag.0
    }
}

impl From<u32> for Tag {
    fn from(tag: u32) -> Self {
        Self(tag)
    }
}

impl From<Tag> for [u8; 4] {
    fn from(tag: Tag) -> Self {
        tag.0.to_be_bytes()
    }
}

impl fmt::Debug for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        struct FieldFormatter([u8; 4]);
        impl fmt::Debug for FieldFormatter {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(
                    f,
                    "{}{}{}{}",
                    self.0[0] as char, self.0[1] as char, self.0[2] as char, self.0[3] as char
                )
            }
        }
        f.debug_tuple("Tag")
            .field(&FieldFormatter((*self).into()))
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tag_is_correct() {
        assert_eq!(u32::from(Tag::new([b'D', b'S', b'I', b'G'])), 0x44534947u32);
        assert_eq!(
            format!("{:?}", Tag::new([b'D', b'S', b'I', b'G'])),
            "Tag(DSIG)"
        );
        assert_eq!(format!("{:?}", Tag::new(b"DSIG")), "Tag(DSIG)");
    }

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
    #[should_panic]
    fn cannot_insert_u32_max() {
        let mut set = U32Set::new().unwrap();
        set.insert(u32::MAX);
    }

    #[test]
    #[should_panic]
    fn cannot_insert_range_u32_max() {
        let mut set = U32Set::new().unwrap();
        set.insert_range(..=u32::MAX);
    }

    #[test]
    fn does_not_contain_u32_max() {
        let mut set = U32Set::new().unwrap();
        set.insert_range(..);
        assert!(!set.contains(u32::MAX));
    }

    #[test]
    fn can_contain_max_value() {
        let mut set = U32Set::new().unwrap();
        set.insert(u32::MAX - 1);
        assert!(set.contains(u32::MAX - 1));
        assert!(!set.is_empty());
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn copy_from_works() {
        let mut a = U32Set::new().unwrap();
        a.insert(5);
        let mut b = U32Set::new().unwrap();
        b.insert(10);
        assert_eq!(a.iter().collect::<Vec<_>>(), [5]);
        assert_eq!(b.iter().collect::<Vec<_>>(), [10]);
        a.copy_from(&b);
        assert_eq!(a.iter().collect::<Vec<_>>(), [10]);
        b.insert(1);
        assert_eq!(a.iter().collect::<Vec<_>>(), [10]);
        a.remove(10);
        assert_eq!(b.iter().collect::<Vec<_>>(), [1, 10]);
    }

    #[test]
    fn contains_its_subset() {
        let mut a = U32Set::new().unwrap();
        a.insert_range(5..=15);
        a.insert_range(55..=65);
        assert!(a.contains_set(&a));
        let mut b = U32Set::new().unwrap();
        b.insert_range(7..=14);
        b.insert(60);
        assert!(b.contains_set(&b));
        assert!(a.contains_set(&b));
        assert!(!b.contains_set(&a));
        b.insert(65);
        assert!(a.contains_set(&b));
        b.insert(66);
        assert!(!a.contains_set(&b));
        assert!(!b.contains_set(&a));
    }

    #[test]
    fn contains_inserted_values() {
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

    #[test]
    fn equal_works() {
        let mut a = U32Set::new().unwrap();
        for i in 0..10 {
            a.insert(i);
        }
        assert_eq!(a, a);
        let mut b = U32Set::new().unwrap();
        assert_ne!(a, b);
        b.insert_range(0..10);
        assert_eq!(a, b);
    }

    #[test]
    fn debug_works() {
        let mut set = U32Set::new().unwrap();
        set.insert_range(3..=5);
        set.insert(7);
        let mut str = String::new();
        use fmt::Write;
        write!(&mut str, "{set:?}").unwrap();
        assert_eq!(str, "{3, 4, 5, 7}");
    }

    #[test]
    fn cloned_set_does_not_modify_original() {
        let mut a = U32Set::new().unwrap();
        a.insert(3);
        a.insert(5);
        let mut b = a.clone();
        assert_eq!(a.len(), 2);
        assert_eq!(b.len(), 2);
        a.insert(10);
        assert_eq!(a.len(), 3);
        assert_eq!(b.len(), 2);
        b.remove(3);
        assert_eq!(a.len(), 3);
        assert_eq!(b.len(), 1);
    }

    #[test]
    fn iter_works() {
        let mut set = U32Set::new().unwrap();
        assert!(set.iter().next().is_none());
        set.insert(0);
        assert_eq!(set.iter().collect::<Vec<_>>(), [0]);
        set.insert(0);
        assert_eq!(set.iter().collect::<Vec<_>>(), [0]);
        set.insert(10);
        assert_eq!(set.iter().collect::<Vec<_>>(), [0, 10]);
        set.insert_range(6..12);
        assert_eq!(set.iter().collect::<Vec<_>>(), [0, 6, 7, 8, 9, 10, 11]);
        set.remove_range(8..=10);
        assert_eq!(set.iter().collect::<Vec<_>>(), [0, 6, 7, 11]);
    }

    #[test]
    fn iter_near_max_works() {
        let mut set = U32Set::new().unwrap();
        set.insert(u32::MAX - 3);
        set.insert(u32::MAX - 2);
        assert_eq!(set.iter().collect::<Vec<_>>(), [u32::MAX - 3, u32::MAX - 2]);
        set.insert(u32::MAX - 1);
        assert_eq!(
            set.iter().collect::<Vec<_>>(),
            [u32::MAX - 3, u32::MAX - 2, u32::MAX - 1]
        );
        set.clear();
        assert!(set.is_empty());
        set.insert_range((Bound::Excluded(u32::MAX - 3), Bound::Unbounded));
        assert_eq!(set.iter().collect::<Vec<_>>(), [u32::MAX - 2, u32::MAX - 1]);
    }

    #[test]
    fn iter_of_invalid_codepoints_works() {
        let mut set = CharSet::new().unwrap();
        set.insert_range('\u{D7FF}'..'\u{E001}'); // Add all surrogate pairs, and then some
        assert_eq!(set.iter().collect::<Vec<_>>(), ['\u{D7FF}', '\u{E000}']);

        let mut set = CharSet::new().unwrap();
        set.insert_range('\u{10FFFF}'..);
        assert_eq!(set.iter().collect::<Vec<_>>(), ['\u{10FFFF}']);
    }

    #[test]
    fn iter_is_fused() {
        fn assert_fused(mut iter: impl Iterator) {
            while let Some(_) = iter.next() {}
            for _ in 0..10 {
                assert!(iter.next().is_none());
            }
            // Believe that iterator is fused after it has returned 11 Nones
        }
        let mut set = U32Set::new().unwrap();
        assert_fused(set.iter());
        assert_fused(set.iter().rev());
        set.insert(0);
        assert_fused(set.iter());
        assert_fused(set.iter().rev());
        set.insert(1);
        assert_fused(set.iter());
        assert_fused(set.iter().rev());
        set.insert(u32::MAX - 3);
        assert_fused(set.iter());
        assert_fused(set.iter().rev());
        set.insert(u32::MAX - 2);
        assert_fused(set.iter());
        assert_fused(set.iter().rev());
        set.insert(u32::MAX - 1);
        assert_fused(set.iter());
        assert_fused(set.iter().rev());

        let mut iter = set.iter();
        assert_eq!(iter.next_back(), Some(u32::MAX - 1));
        assert_fused(iter);

        let mut iter = set.iter().rev();
        assert_eq!(iter.next_back(), Some(0));
        assert_fused(iter);
    }

    #[test]
    fn iter_next_back_works() {
        let mut set = U32Set::new().unwrap();
        assert!(set.iter().next().is_none());
        set.insert(0);
        set.insert_range(6..12);
        assert_eq!(
            set.iter().rev().collect::<Vec<_>>(),
            [11, 10, 9, 8, 7, 6, 0]
        );
        set.remove_range(8..=10);
        assert_eq!(set.iter().rev().collect::<Vec<_>>(), [11, 7, 6, 0]);

        let mut iter = set.iter();
        assert_eq!(iter.next(), Some(0));
        assert_eq!(iter.next_back(), Some(11));
        assert_eq!(iter.next_back(), Some(7));
        assert_eq!(iter.next(), Some(6));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next_back(), None);

        let mut iter = set.iter();
        assert_eq!(iter.next_back(), Some(11));
        assert_eq!(iter.next_back(), Some(7));
        assert_eq!(iter.next(), Some(0));
        assert_eq!(iter.next(), Some(6));
        assert_eq!(iter.next_back(), None);
        assert_eq!(iter.next(), None);

        let mut iter = set.iter();
        assert_eq!(iter.next_back(), Some(11));
        assert_eq!(iter.next_back(), Some(7));
        assert_eq!(iter.next(), Some(0));
        assert_eq!(iter.next_back(), Some(6));
        assert_eq!(iter.next_back(), None);
        assert_eq!(iter.next(), None);

        let mut iter = set.iter();
        assert_eq!(iter.next_back(), Some(11));
        assert_eq!(iter.next_back(), Some(7));
        assert_eq!(iter.next(), Some(0));
        assert_eq!(iter.next_back(), Some(6));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next_back(), None);
    }
}
