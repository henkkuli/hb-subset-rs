//! This crate exposes a [HarfBuzz](https://github.com/harfbuzz/harfbuzz) API for subsetting a font.
//!
//! From HarfBuzz documentation:
//! > Subsetting reduces the codepoint coverage of font files and removes all data that is no longer needed. A subset
//! > input describes the desired subset. The input is provided along with a font to the subsetting operation. Output is
//! > a new font file containing only the data specified in the input.
//! >
//! > Currently most outline and bitmap tables are supported: glyf, CFF, CFF2, sbix, COLR, and CBDT/CBLC. This also
//! > includes fonts with variable outlines via OpenType variations. Notably EBDT/EBLC and SVG are not supported. Layout
//! > subsetting is supported only for OpenType Layout tables (GSUB, GPOS, GDEF). Notably subsetting of graphite or AAT
//! > tables is not yet supported.
//! >
//! > Fonts with graphite or AAT tables may still be subsetted but will likely need to use the retain glyph ids option
//! > and configure the subset to pass through the layout tables untouched.
//!
//! # Usage
//! The simplest way to construct a subset of a font is to use [`subset`] function:
//! ```no_run
//! # use std::fs;
//! let font = fs::read("fonts/NotoSans/full/variable/NotoSans[wdth,wght].ttf").unwrap();
//! let subset_font = hb_subset::subset(&font, "abc".chars()).unwrap();
//! fs::write("fonts/subset.ttf", subset_font).unwrap();
//! ```
//!
//! To get more control over how the font is subset and what gets included, you can use the lower level API directly:
//! ```rust
//! # use hb_subset::*;
//! // Load font directly from a file
//! let font = Blob::from_file("fonts/NotoSans/full/variable/NotoSans[wdth,wght].ttf").unwrap();
//! let font = FontFace::new(font).unwrap();
//!
//! // Construct a subset manually and include only some of the letters
//! let mut subset = SubsetInput::new().unwrap();
//! subset.unicode_set().insert('f' as u32);
//! subset.unicode_set().insert('i' as u32);
//!
//! // Subset the font using just-constructed subset input
//! let new_font = subset.subset_font(&font).unwrap();
//!
//! // Extract the raw font and write to an output file
//! std::fs::write("out.ttf", &*new_font.underlying_blob()).unwrap();
//! ```

#![warn(missing_docs)]

use std::{
    ffi::{c_char, CString},
    hash::Hash,
    marker::PhantomData,
    ops::{Deref, RangeBounds},
    os::unix::prelude::OsStringExt,
    path::Path,
    ptr::null_mut,
    slice,
};

use sys::hb_subset_or_fail;
use thiserror::Error;

pub mod sys;

/// An enumeration over possible errors.
#[derive(Debug, Error)]
pub enum Error {
    /// An error returned when an allocation fails.
    #[error("Failed to allocate object")]
    AllocationError,
    #[error("Failed to subset font face")]
    /// An error returned when font face could not be subset.
    SubsetError,
    /// An error returned when a font face could not be extracted from blob.
    #[error("Failed to extract font face from blob")]
    FontFaceExtractionError,
}

/// Blobs wrap a chunk of binary data.
///
/// Blob handles lifecycle management of data while it is passed between client and HarfBuzz. Blobs are primarily used
/// to create font faces, but also to access font face tables, as well as pass around other binary data.
pub struct Blob<'a>(*mut sys::hb_blob_t, PhantomData<&'a [u8]>);

impl Blob<'static> {
    /// Creates a new blob containing the data from the specified binary font file.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, Error> {
        let path = path.as_ref();
        // TODO: Try to make more succinct
        let path = CString::new(path.as_os_str().to_os_string().into_vec()).unwrap();

        let blob = unsafe { sys::hb_blob_create_from_file_or_fail(path.as_ptr()) };
        if blob.is_null() {
            return Err(Error::AllocationError);
        }
        Ok(Self(blob, PhantomData))
    }
}

impl<'a> Blob<'a> {
    /// Creates a new blob object by wrapping a slice.
    pub fn from_bytes(buffer: &'a [u8]) -> Result<Self, Error> {
        let blob = unsafe {
            sys::hb_blob_create_or_fail(
                buffer.as_ptr() as *const c_char,
                buffer
                    .len()
                    .try_into()
                    .map_err(|_| Error::AllocationError)?,
                sys::hb_memory_mode_t_HB_MEMORY_MODE_READONLY,
                null_mut(),
                None,
            )
        };
        if blob.is_null() {
            return Err(Error::AllocationError);
        }
        Ok(Self(blob, PhantomData))
    }

    /// Tests whether the blob is empty, i.e. its length is 0.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the number of bytes in the blob.
    pub fn len(&self) -> usize {
        (unsafe { sys::hb_blob_get_length(self.0) }) as usize
    }

    /// Converts the blob into raw [`sys::hb_blob_t`] object.
    ///
    /// This method transfers the ownership of the blob to the caller. It is up to the caller to call
    /// [`sys::hb_blob_destroy`] to free the object, or call [`Self::from_raw`] to convert it back into [`Blob`].
    pub fn into_raw(self) -> *mut sys::hb_blob_t {
        let ptr = self.0;
        std::mem::forget(self);
        ptr
    }

    /// Exposes the raw inner pointer without transferring the ownership.
    ///
    /// Unlike [`Self::into_raw`], this method does not transfer the ownership of the pointer to the caller.
    pub fn as_raw(&self) -> *mut sys::hb_blob_t {
        self.0
    }

    /// Constructs a blob from raw [`sys::hb_blob_t`] object.
    ///
    /// # Safety
    /// The given `blob` pointer must either be constructed by some Harfbuzz function, or be returned from
    /// [`Self::into_raw`].
    pub unsafe fn from_raw(blob: *mut sys::hb_blob_t) -> Self {
        Self(blob, PhantomData)
    }
}

impl Deref for Blob<'_> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        let mut len = 0u32;
        let data = unsafe { sys::hb_blob_get_data(self.0, &mut len as *mut u32) } as *const u8;
        if data.is_null() {
            // TODO: Consider returning an error instead
            return &[];
        }
        unsafe { slice::from_raw_parts(data, len as usize) }
    }
}

impl<'a> Drop for Blob<'a> {
    fn drop(&mut self) {
        unsafe { sys::hb_blob_destroy(self.0) }
    }
}

impl<'a> Clone for Blob<'a> {
    fn clone(&self) -> Self {
        Self(unsafe { sys::hb_blob_reference(self.0) }, PhantomData)
    }
}

/// A font face is an object that represents a single face from within a font family.
///
/// More precisely, a font face represents a single face in a binary font file. Font faces are typically built from a
/// binary blob and a face index. Font faces are used to create fonts.
pub struct FontFace<'a>(*mut sys::hb_face_t, PhantomData<Blob<'a>>);

impl<'a> FontFace<'a> {
    /// Constructs a new face object from the specified blob.
    ///
    /// This defaults to taking the first face in the blob. If you need to specify which font face to load, you can use
    /// [`new_with_index`] instead.
    ///
    /// [`new_with_index`]: Self::new_with_index
    pub fn new(blob: Blob<'a>) -> Result<Self, Error> {
        Self::new_with_index(blob, 0)
    }

    /// Constructs a new face object from the specified blob and a face index into that blob.
    ///
    /// The face index is used for blobs of file formats such as TTC and DFont that can contain more than one face. Face
    /// indices within such collections are zero-based.
    pub fn new_with_index(blob: Blob<'a>, index: u32) -> Result<Self, Error> {
        let face = unsafe { sys::hb_face_create(blob.0, index) };
        if face.is_null() {
            return Err(Error::FontFaceExtractionError);
        }
        Ok(Self(face, PhantomData))
    }

    /// Gets the blob underlying this font face.
    ///
    /// Useful when you want to output the font face to a file.
    ///
    /// Returns an empty blob if referencing face data is not possible.
    pub fn underlying_blob(&self) -> Blob<'_> {
        Blob(
            unsafe { sys::hb_face_reference_blob(self.as_raw()) },
            PhantomData,
        )
    }

    /// Collects all of the Unicode characters covered by the font face.
    pub fn collect_unicodes(&self) -> Result<Set, Error> {
        let set = Set::new()?;
        unsafe { sys::hb_face_collect_unicodes(self.as_raw(), set.as_raw()) };
        Ok(set)
    }

    /// Converts the font face into raw [`sys::hb_face_t`] object.
    ///
    /// This method transfers the ownership of the font face to the caller. It is up to the caller to call
    /// [`sys::hb_face_destroy`] to free the object, or call [`Self::from_raw`] to convert it back into [`FontFace`].
    pub fn into_raw(self) -> *mut sys::hb_face_t {
        let ptr = self.0;
        std::mem::forget(self);
        ptr
    }

    /// Exposes the raw inner pointer without transferring the ownership.
    ///
    /// Unlike [`Self::into_raw`], this method does not transfer the ownership of the pointer to the caller.
    pub fn as_raw(&self) -> *mut sys::hb_face_t {
        self.0
    }

    /// Constructs a font face from raw [`sys::hb_face_t`] object.
    ///
    /// # Safety
    /// The given `font_face` pointer must either be constructed by some Harfbuzz function, or be returned from
    /// [`Self::into_raw`].
    pub unsafe fn from_raw(font_face: *mut sys::hb_face_t) -> Self {
        Self(font_face, PhantomData)
    }
}

impl<'a> Drop for FontFace<'a> {
    fn drop(&mut self) {
        unsafe { sys::hb_face_destroy(self.0) }
    }
}

/// A description of how a font should be subset.
///
/// Subsetting reduces the codepoint coverage of font files and removes all data that is no longer needed. A subset
/// input describes the desired subset. The input is provided along with a font to the subsetting operation. Output is a
/// new font file containing only the data specified in the input.
///
/// Currently most outline and bitmap tables are supported: glyf, CFF, CFF2, sbix, COLR, and CBDT/CBLC. This also
/// includes fonts with variable outlines via OpenType variations. Notably EBDT/EBLC and SVG are not supported. Layout
/// subsetting is supported only for OpenType Layout tables (GSUB, GPOS, GDEF). Notably subsetting of graphite or AAT
/// tables is not yet supported.
///
/// Fonts with graphite or AAT tables may still be subsetted but will likely need to use the retain glyph ids option and
/// configure the subset to pass through the layout tables untouched.
pub struct SubsetInput(*mut sys::hb_subset_input_t);

impl SubsetInput {
    /// Creates a new subset input object.
    pub fn new() -> Result<Self, Error> {
        let input = unsafe { sys::hb_subset_input_create_or_fail() };
        if input.is_null() {
            return Err(Error::AllocationError);
        }
        Ok(Self(input))
    }

    /// Configure input object to keep everything in the font face. That is, all Unicodes, glyphs, names, layout items,
    /// glyph names, etc.
    ///
    /// The input can be tailored afterwards by the caller.
    pub fn keep_everything(&mut self) {
        unsafe { sys::hb_subset_input_keep_everything(self.0) }
    }

    /// Gets the set of Unicode code points to retain, the caller should modify the set as needed.
    pub fn unicode_set(&mut self) -> Set<'_> {
        Set(
            InnerSet(unsafe { sys::hb_set_reference(sys::hb_subset_input_unicode_set(self.0)) }),
            PhantomData,
        )
    }

    /// Gets the set of glyph IDs to retain, the caller should modify the set as needed.
    pub fn glyph_set(&mut self) -> Set<'_> {
        Set(
            InnerSet(unsafe { sys::hb_set_reference(sys::hb_subset_input_glyph_set(self.0)) }),
            PhantomData,
        )
    }

    /// Subsets a font according to provided input.
    pub fn subset_font(&self, font: &FontFace<'_>) -> Result<FontFace<'static>, Error> {
        let face = unsafe { hb_subset_or_fail(font.0, self.0) };
        if face.is_null() {
            return Err(Error::SubsetError);
        }
        Ok(FontFace(face, PhantomData))
    }

    /// Converts the subset input into raw [`sys::hb_subset_input_t`] object.
    ///
    /// This method transfers the ownership of the subset input to the caller. It is up to the caller to call
    /// [`sys::hb_blob_destroy`] to free the object, or call [`Self::from_raw`] to convert it back into [`SubsetInput`].
    pub fn into_raw(self) -> *mut sys::hb_subset_input_t {
        let ptr = self.0;
        std::mem::forget(self);
        ptr
    }

    /// Exposes the raw inner pointer without transferring the ownership.
    ///
    /// Unlike [`Self::into_raw`], this method does not transfer the ownership of the pointer to the caller.
    pub fn as_raw(&self) -> *mut sys::hb_subset_input_t {
        self.0
    }

    /// Constructs a subset input from raw [`sys::hb_subset_input_t`] object.
    ///
    /// # Safety
    /// The given `subset` pointer must either be constructed by some Harfbuzz function, or be returned from
    /// [`Self::into_raw`].
    pub unsafe fn from_raw(subset: *mut sys::hb_subset_input_t) -> Self {
        Self(subset)
    }
}

impl Clone for SubsetInput {
    fn clone(&self) -> Self {
        Self(unsafe { sys::hb_subset_input_reference(self.0) })
    }
}

impl Drop for SubsetInput {
    fn drop(&mut self) {
        unsafe { sys::hb_subset_input_destroy(self.0) }
    }
}

/// Set objects represent a mathematical set of integer values.
///
/// Sets are used in non-shaping APIs to query certain sets of characters or glyphs, or other integer values.
pub struct Set<'a>(InnerSet, PhantomData<&'a ()>);

impl Set<'static> {
    /// Creates a new, initially empty set.
    fn new() -> Result<Self, Error> {
        let set = unsafe { sys::hb_set_create() };
        if set.is_null() {
            return Err(Error::AllocationError);
        }
        Ok(Self(InnerSet(set), PhantomData))
    }
}

impl<'a> Set<'a> {
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

    /// Tests whether a value belongs to set.
    pub fn contains(&self, value: u32) -> bool {
        (unsafe { sys::hb_set_has(self.as_raw(), value) }) != 0
    }

    /// Inserts a value to set.
    pub fn insert(&mut self, value: u32) {
        unsafe { sys::hb_set_add(self.as_raw(), value) }
    }

    /// Removes a value from set.
    pub fn remove(&mut self, value: u32) {
        unsafe { sys::hb_set_del(self.as_raw(), value) }
    }

    /// Converts a range to inclusive bounds.
    fn range_to_bounds(range: impl RangeBounds<u32>) -> Option<(u32, u32)> {
        let lower = match range.start_bound() {
            std::ops::Bound::Included(&lower) => lower,
            std::ops::Bound::Excluded(&lower) => {
                if lower == u32::MAX {
                    return None;
                } else {
                    lower + 1
                }
            }
            std::ops::Bound::Unbounded => 0,
        };
        let upper = match range.end_bound() {
            std::ops::Bound::Included(&upper) => upper,
            std::ops::Bound::Excluded(&upper) => {
                if upper == 0 {
                    return None;
                } else {
                    upper - 1
                }
            }
            std::ops::Bound::Unbounded => u32::MAX,
        };
        if upper < lower {
            return None;
        }
        Some((lower, upper))
    }

    /// Inserts a range of values to set.
    pub fn insert_range(&mut self, range: impl RangeBounds<u32>) {
        let Some((lower, upper)) = Self::range_to_bounds(range) else {
            return;
        };
        unsafe { sys::hb_set_add_range(self.as_raw(), lower, upper) }
    }

    /// Removes a range of values from set.
    pub fn remove_range(&mut self, range: impl RangeBounds<u32>) {
        // TODO: Assert that sys::HB_SET_VALUE_INVALID is u32::MAX like it should be
        // const _: () = assert!(u32::MAX <= sys::HB_SET_VALUE_INVALID);
        let Some((lower, upper)) = Self::range_to_bounds(range) else {
            return;
        };
        unsafe { sys::hb_set_add_range(self.as_raw(), lower, upper) }
    }

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

impl<'a> Hash for Set<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        unsafe { sys::hb_set_hash(self.as_raw()) }.hash(state);
    }
}

/// Implementation detail of Set to hide source reference from drop check.
///
/// If the pointer was directly contained in [`Set`] with `Drop` implemented, the following code would not compile:
/// ```rust
/// # use hb_subset::*;
/// let mut subset = SubsetInput::new().unwrap();
/// let mut unicode_set = subset.unicode_set();
/// // drop(unicode_set);                               // This needs to be called to delete unicode_set,
/// # let font = FontFace::new(Blob::from_bytes(&[]).unwrap()).unwrap();
/// let new_font = subset.subset_font(&font).unwrap();  // otherwise this line would not compile as unicode_set is already
///                                                     // holding a mutable reference to subset.
/// ```
struct InnerSet(*mut sys::hb_set_t);

impl Drop for InnerSet {
    fn drop(&mut self) {
        unsafe { sys::hb_set_destroy(self.0) }
    }
}

/// A convenient method to create a subset of a font over given characters.
///
/// The returned font can be used everywhere where the original font was used, as long as the string contains only
/// characters from the given set. In particular, the font includes all relevant ligatures.
pub fn subset(font: &[u8], characters: impl IntoIterator<Item = char>) -> Result<Vec<u8>, Error> {
    // Add all characters to subset, and nothing more.
    let mut subset = SubsetInput::new()?;
    let mut unicode_set = subset.unicode_set();
    for char in characters {
        unicode_set.insert(char as u32);
    }

    // Load the original font, and then construct a subset from it
    let font = FontFace::new(Blob::from_bytes(font)?)?;
    let new_font = subset.subset_font(&font)?;
    let new_font = new_font.underlying_blob().to_vec();
    Ok(new_font)
}
