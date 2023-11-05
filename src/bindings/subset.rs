use crate::{
    bindings::{CharSet, FontFace, Set, U32Set},
    sys, Error,
};

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

    /// Gets the set of Unicode codepoints to retain, the caller should modify the set as needed.
    pub fn unicode_set(&mut self) -> CharSet<'_> {
        unsafe {
            Set::from_raw(sys::hb_set_reference(sys::hb_subset_input_unicode_set(
                self.0,
            )))
        }
    }

    /// Gets the set of glyph IDs to retain, the caller should modify the set as needed.
    pub fn glyph_set(&mut self) -> U32Set<'_> {
        unsafe {
            Set::from_raw(sys::hb_set_reference(sys::hb_subset_input_glyph_set(
                self.0,
            )))
        }
    }

    /// Subsets a font according to provided input.
    pub fn subset_font(&self, font: &FontFace<'_>) -> Result<FontFace<'static>, Error> {
        let face = unsafe { sys::hb_subset_or_fail(font.as_raw(), self.0) };
        if face.is_null() {
            return Err(Error::SubsetError);
        }
        Ok(unsafe { FontFace::from_raw(face) })
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bindings::{tests::NOTO_SANS, Blob};

    #[test]
    fn keep_everything_should_keep_all_codepoints_and_glyphs() {
        let mut subset = SubsetInput::new().unwrap();
        subset.keep_everything();
        assert_eq!(subset.unicode_set().len(), u32::MAX as usize);
        assert_eq!(subset.glyph_set().len(), u32::MAX as usize);
        let orig = FontFace::new(Blob::from_file(NOTO_SANS).unwrap()).unwrap();
        let new = subset.subset_font(&orig).unwrap();
        assert_eq!(
            orig.collect_unicodes().unwrap().len(),
            new.collect_unicodes().unwrap().len()
        );
        assert_eq!(orig.get_glyph_count(), new.get_glyph_count());
    }

    #[test]
    fn keeping_codepoints_should_keep_ligatures() {
        let mut subset = SubsetInput::new().unwrap();
        subset.unicode_set().insert('f');
        subset.unicode_set().insert('i');
        let font = subset
            .subset_font(&FontFace::new(Blob::from_file(NOTO_SANS).unwrap()).unwrap())
            .unwrap();
        assert_eq!(font.collect_unicodes().unwrap().len(), 2);
        assert_eq!(font.get_glyph_count(), 6); // TODO: Actually check *which* glyphs are included
                                               // Currently just assuming [empty], f, i, ﬁ, ﬃ, and ﬀ
    }

    #[test]
    fn convert_into_raw_and_back() {
        let subset = SubsetInput::new().unwrap();
        let subset_ptr = subset.into_raw();
        let subset = unsafe { SubsetInput::from_raw(subset_ptr) };
        drop(subset);
    }
}
