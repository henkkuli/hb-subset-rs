use std::marker::PhantomData;

use crate::{
    bindings::{Blob, Set},
    sys, Error,
};

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
        let face = unsafe { sys::hb_face_create(blob.as_raw(), index) };
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
        unsafe { Blob::from_raw(sys::hb_face_reference_blob(self.as_raw())) }
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
