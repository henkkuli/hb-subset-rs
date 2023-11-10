use std::{
    ffi::{c_char, CString},
    marker::PhantomData,
    ops::Deref,
    os::unix::prelude::OsStrExt,
    path::Path,
    ptr::null_mut,
    slice,
};

use crate::{sys, Error};

/// Blobs wrap a chunk of binary data.
///
/// Blob handles lifecycle management of data while it is passed between client and HarfBuzz. Blobs are primarily used
/// to create font faces, but also to access font face tables, as well as pass around other binary data.
pub struct Blob<'a>(*mut sys::hb_blob_t, PhantomData<&'a [u8]>);

impl Blob<'static> {
    /// Creates a new blob containing the data from the specified binary font file.
    #[doc(alias = "hb_blob_create_from_file")]
    #[doc(alias = "hb_blob_create_from_file_or_fail")]
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, Error> {
        let path = CString::new(path.as_ref().as_os_str().as_bytes()).unwrap();

        let blob = unsafe { sys::hb_blob_create_from_file_or_fail(path.as_ptr()) };
        if blob.is_null() {
            return Err(Error::AllocationError);
        }
        Ok(Self(blob, PhantomData))
    }
}

impl<'a> Blob<'a> {
    /// Creates a new blob object by wrapping a slice.
    #[doc(alias = "hb_blob_create")]
    #[doc(alias = "hb_blob_create_or_fail")]
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
    #[doc(alias = "hb_blob_get_length")]
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

    #[doc(alias = "hb_blob_get_data")]
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
    #[doc(alias = "hb_blob_destroy")]
    fn drop(&mut self) {
        unsafe { sys::hb_blob_destroy(self.0) }
    }
}

impl<'a> Clone for Blob<'a> {
    #[doc(alias = "hb_blob_reference")]
    fn clone(&self) -> Self {
        Self(unsafe { sys::hb_blob_reference(self.0) }, PhantomData)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bindings::tests::NOTO_SANS;

    #[test]
    fn empty_is_empty() {
        assert!(Blob::from_bytes(&[]).unwrap().is_empty());
    }

    #[test]
    fn non_empty_is_not_empty() {
        assert!(!Blob::from_bytes(&[1, 2, 3]).unwrap().is_empty());
    }

    #[test]
    fn len_works() {
        assert_eq!(Blob::from_bytes(&[]).unwrap().len(), 0);
        assert_eq!(Blob::from_bytes(&[1, 2, 3]).unwrap().len(), 3);
    }

    #[test]
    fn content_is_correct() {
        assert_eq!(&*Blob::from_bytes(&[1, 2, 3]).unwrap(), &[1, 2, 3]);
    }

    #[test]
    fn from_file_loads_file() {
        let correct = std::fs::read(NOTO_SANS).unwrap();
        let blob = Blob::from_file(NOTO_SANS).unwrap();
        assert_eq!(correct, &*blob);
    }

    #[test]
    fn clone_refers_to_same_object() {
        let b1 = Blob::from_bytes(&[1, 2, 3]).unwrap();
        let b2 = b1.clone();
        assert_eq!(&*b1, &[1, 2, 3]);
        assert_eq!(&*b2, &[1, 2, 3]);
        drop(b1);
        assert_eq!(&*b2, &[1, 2, 3]);
    }

    #[test]
    fn convert_into_raw_and_back() {
        let blob = Blob::from_bytes(&[1, 2, 3]).unwrap();
        let blob_ptr = blob.into_raw();
        let blob = unsafe { Blob::from_raw(blob_ptr) };
        drop(blob);
    }
}
