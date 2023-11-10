use std::{ffi::c_char, marker::PhantomData, ptr::null_mut};

use crate::{sys, AllocationError, Blob, CharSet, FontFaceExtractionError, Language};

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
    #[doc(alias = "hb_face_create")]
    pub fn new(blob: Blob<'a>) -> Result<Self, FontFaceExtractionError> {
        Self::new_with_index(blob, 0)
    }

    /// Constructs a new face object from the specified blob and a face index into that blob.
    ///
    /// The face index is used for blobs of file formats such as TTC and DFont that can contain more than one face. Face
    /// indices within such collections are zero-based.
    #[doc(alias = "hb_face_create")]
    pub fn new_with_index(blob: Blob<'a>, index: u32) -> Result<Self, FontFaceExtractionError> {
        let face = unsafe { sys::hb_face_create(blob.as_raw(), index) };
        if face.is_null() {
            return Err(FontFaceExtractionError);
        }
        Ok(Self(face, PhantomData))
    }

    /// Gets the blob underlying this font face.
    ///
    /// Useful when you want to output the font face to a file.
    ///
    /// Returns an empty blob if referencing face data is not possible.
    #[doc(alias = "hb_face_reference_blob")]
    pub fn underlying_blob(&self) -> Blob<'_> {
        unsafe { Blob::from_raw(sys::hb_face_reference_blob(self.as_raw())) }
    }

    /// Fetches the glyph-count value of the specified face object.
    #[doc(alias = "hb_face_get_glyph_count")]
    pub fn glyph_count(&self) -> usize {
        (unsafe { sys::hb_face_get_glyph_count(self.as_raw()) }) as usize
    }

    /// Collects all of the Unicode characters covered by the font face.
    #[doc(alias = "hb_face_collect_unicodes")]
    pub fn collect_unicodes(&self) -> Result<CharSet, AllocationError> {
        let set = CharSet::new()?;
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

/// Functions for fetching name strings from OpenType fonts.
///
/// See [OpenType spec](https://learn.microsoft.com/en-us/typography/opentype/spec/name#name-ids) for more information
/// on these strings.
impl<'a> FontFace<'a> {
    /// Gets value from OpenType name table for given language.
    ///
    /// Instead of using this method directly, consider using one of the convenience methods for getting the correct
    /// string directly.
    ///
    /// If `language` is `null()`, English is assumed.
    #[doc(alias = "hb_ot_name_get_utf8")]
    #[doc(alias = "hb_ot_name_get_utf16")]
    #[doc(alias = "hb_ot_name_get_utf32")]
    pub fn ot_name(&self, name: impl Into<sys::hb_ot_name_id_t>, language: Language) -> String {
        let name = name.into();
        let mut len = unsafe {
            sys::hb_ot_name_get_utf8(
                self.as_raw(),
                name,
                language.as_raw(),
                null_mut(),
                null_mut(),
            )
        };
        len += 1; // Reserve space for NUL termination
        let mut buf = vec![0; len as usize];
        let full_len = unsafe {
            sys::hb_ot_name_get_utf8(
                self.as_raw(),
                name,
                language.as_raw(),
                &mut len as *mut u32,
                buf.as_mut_ptr() as *mut c_char,
            )
        };
        assert!(len <= full_len);
        buf.truncate(len as usize);

        String::from_utf8(buf).expect("Output is promised to be valid UTF-8")
    }

    /// Gets copyright notice.
    ///
    /// # Example
    /// ```
    /// # use hb_subset::*;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let font = FontFace::new(Blob::from_file("tests/fonts/NotoSans.ttf")?)?;
    /// assert_eq!(font.copyright(), "Copyright 2022 The Noto Project Authors (https://github.com/notofonts/latin-greek-cyrillic)");
    /// # Ok(())
    /// # }
    /// ```
    #[doc(alias = "HB_OT_NAME_ID_COPYRIGHT")]
    pub fn copyright(&self) -> String {
        self.ot_name(
            sys::hb_ot_name_id_predefined_t::COPYRIGHT,
            Language::default(),
        )
    }

    /// Gets font family name.
    ///
    /// # Example
    /// ```
    /// # use hb_subset::*;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let font = FontFace::new(Blob::from_file("tests/fonts/NotoSans.ttf")?)?;
    /// assert_eq!(font.font_family(), "Noto Sans");
    /// # Ok(())
    /// # }
    /// ```
    #[doc(alias = "HB_OT_NAME_ID_FONT_FAMILY")]
    pub fn font_family(&self) -> String {
        self.ot_name(
            sys::hb_ot_name_id_predefined_t::FONT_FAMILY,
            Language::default(),
        )
    }

    /// Gets font subfamily name.
    ///
    /// # Example
    /// ```
    /// # use hb_subset::*;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let font = FontFace::new(Blob::from_file("tests/fonts/NotoSans.ttf")?)?;
    /// assert_eq!(font.font_subfamily(), "Regular");
    /// # Ok(())
    /// # }
    /// ```
    #[doc(alias = "HB_OT_NAME_ID_FONT_SUBFAMILY")]
    pub fn font_subfamily(&self) -> String {
        self.ot_name(
            sys::hb_ot_name_id_predefined_t::FONT_SUBFAMILY,
            Language::default(),
        )
    }

    /// Gets unique font identifier.
    ///
    /// # Example
    /// ```
    /// # use hb_subset::*;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let font = FontFace::new(Blob::from_file("tests/fonts/NotoSans.ttf")?)?;
    /// assert_eq!(font.unique_id(), "2.013;GOOG;NotoSans-Regular");
    /// # Ok(())
    /// # }
    /// ```
    #[doc(alias = "HB_OT_NAME_ID_UNIQUE_ID")]
    pub fn unique_id(&self) -> String {
        self.ot_name(
            sys::hb_ot_name_id_predefined_t::UNIQUE_ID,
            Language::default(),
        )
    }

    /// Gets full font name that reflects all family and relevant subfamily descriptors.
    ///
    /// # Example
    /// ```
    /// # use hb_subset::*;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let font = FontFace::new(Blob::from_file("tests/fonts/NotoSans.ttf")?)?;
    /// assert_eq!(font.full_name(), "Noto Sans Regular");
    /// # Ok(())
    /// # }
    /// ```
    #[doc(alias = "HB_OT_NAME_ID_FULL_NAME")]
    pub fn full_name(&self) -> String {
        self.ot_name(
            sys::hb_ot_name_id_predefined_t::FULL_NAME,
            Language::default(),
        )
    }

    /// Gets version string.
    ///
    /// # Example
    /// ```
    /// # use hb_subset::*;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let font = FontFace::new(Blob::from_file("tests/fonts/NotoSans.ttf")?)?;
    /// assert_eq!(font.version_string(), "Version 2.013; ttfautohint (v1.8.4.7-5d5b)");
    /// # Ok(())
    /// # }
    /// ```
    #[doc(alias = "HB_OT_NAME_ID_VERSION_STRING")]
    pub fn version_string(&self) -> String {
        self.ot_name(
            sys::hb_ot_name_id_predefined_t::VERSION_STRING,
            Language::default(),
        )
    }

    /// Gets PostScript name for the font.
    ///
    /// # Example
    /// ```
    /// # use hb_subset::*;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let font = FontFace::new(Blob::from_file("tests/fonts/NotoSans.ttf")?)?;
    /// assert_eq!(font.postscript_name(), "NotoSans-Regular");
    /// # Ok(())
    /// # }
    /// ```
    #[doc(alias = "HB_OT_NAME_ID_POSTSCRIPT_NAME")]
    pub fn postscript_name(&self) -> String {
        self.ot_name(
            sys::hb_ot_name_id_predefined_t::POSTSCRIPT_NAME,
            Language::default(),
        )
    }

    /// Gets trademark information.
    ///
    /// # Example
    /// ```
    /// # use hb_subset::*;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let font = FontFace::new(Blob::from_file("tests/fonts/NotoSans.ttf")?)?;
    /// assert_eq!(font.trademark(), "Noto is a trademark of Google LLC.");
    /// # Ok(())
    /// # }
    /// ```
    #[doc(alias = "HB_OT_NAME_ID_TRADEMARK")]
    pub fn trademark(&self) -> String {
        self.ot_name(
            sys::hb_ot_name_id_predefined_t::TRADEMARK,
            Language::default(),
        )
    }

    /// Gets manufacturer name.
    ///
    /// # Example
    /// ```
    /// # use hb_subset::*;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let font = FontFace::new(Blob::from_file("tests/fonts/NotoSans.ttf")?)?;
    /// assert_eq!(font.manufacturer(), "Monotype Imaging Inc.");
    /// # Ok(())
    /// # }
    /// ```
    #[doc(alias = "HB_OT_NAME_ID_MANUFACTURER")]
    pub fn manufacturer(&self) -> String {
        self.ot_name(
            sys::hb_ot_name_id_predefined_t::MANUFACTURER,
            Language::default(),
        )
    }

    /// Gets designer name.
    ///
    /// # Example
    /// ```
    /// # use hb_subset::*;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let font = FontFace::new(Blob::from_file("tests/fonts/NotoSans.ttf")?)?;
    /// assert_eq!(font.designer(), "Monotype Design Team");
    /// # Ok(())
    /// # }
    /// ```
    #[doc(alias = "HB_OT_NAME_ID_DESIGNER")]
    pub fn designer(&self) -> String {
        self.ot_name(
            sys::hb_ot_name_id_predefined_t::DESIGNER,
            Language::default(),
        )
    }

    /// Gets description.
    ///
    /// # Example
    /// ```
    /// # use hb_subset::*;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let font = FontFace::new(Blob::from_file("tests/fonts/NotoSans.ttf")?)?;
    /// assert_eq!(font.description(), "Designed by Monotype design team, Irene Vlachou.");
    /// # Ok(())
    /// # }
    /// ```
    #[doc(alias = "HB_OT_NAME_ID_DESCRIPTION")]
    pub fn description(&self) -> String {
        self.ot_name(
            sys::hb_ot_name_id_predefined_t::DESCRIPTION,
            Language::default(),
        )
    }

    /// Gets URL of font vendor.
    ///
    /// # Example
    /// ```
    /// # use hb_subset::*;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let font = FontFace::new(Blob::from_file("tests/fonts/NotoSans.ttf")?)?;
    /// assert_eq!(font.vendor_url(), "http://www.google.com/get/noto/");
    /// # Ok(())
    /// # }
    /// ```
    #[doc(alias = "HB_OT_NAME_ID_VENDOR_URL")]
    pub fn vendor_url(&self) -> String {
        self.ot_name(
            sys::hb_ot_name_id_predefined_t::VENDOR_URL,
            Language::default(),
        )
    }

    /// Gets URL of typeface designer.
    ///
    /// # Example
    /// ```
    /// # use hb_subset::*;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let font = FontFace::new(Blob::from_file("tests/fonts/NotoSans.ttf")?)?;
    /// assert_eq!(font.designer_url(), "http://www.monotype.com/studio");
    /// # Ok(())
    /// # }
    /// ```
    #[doc(alias = "HB_OT_NAME_ID_DESIGNER_URL")]
    pub fn designer_url(&self) -> String {
        self.ot_name(
            sys::hb_ot_name_id_predefined_t::DESIGNER_URL,
            Language::default(),
        )
    }

    /// Gets license description.
    ///
    /// # Example
    /// ```
    /// # use hb_subset::*;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let font = FontFace::new(Blob::from_file("tests/fonts/NotoSans.ttf")?)?;
    /// assert_eq!(font.license(), "This Font Software is licensed under the SIL Open Font License, Version 1.1. This license is available with a FAQ at: https://scripts.sil.org/OFL");
    /// # Ok(())
    /// # }
    /// ```
    #[doc(alias = "HB_OT_NAME_ID_LICENSE")]
    pub fn license(&self) -> String {
        self.ot_name(
            sys::hb_ot_name_id_predefined_t::LICENSE,
            Language::default(),
        )
    }

    /// Gets URL where additional licensing information can be found.
    ///
    /// # Example
    /// ```
    /// # use hb_subset::*;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let font = FontFace::new(Blob::from_file("tests/fonts/NotoSans.ttf")?)?;
    /// assert_eq!(font.license_url(), "https://scripts.sil.org/OFL");
    /// # Ok(())
    /// # }
    /// ```
    #[doc(alias = "HB_OT_NAME_ID_LICENSE_URL")]
    pub fn license_url(&self) -> String {
        self.ot_name(
            sys::hb_ot_name_id_predefined_t::LICENSE_URL,
            Language::default(),
        )
    }

    /// Gets typographic family name.
    #[doc(alias = "HB_OT_NAME_ID_TYPOGRAPHIC_FAMILY")]
    pub fn typographic_family(&self) -> String {
        self.ot_name(
            sys::hb_ot_name_id_predefined_t::TYPOGRAPHIC_FAMILY,
            Language::default(),
        )
    }

    /// Gets typographic subfamily name.
    #[doc(alias = "HB_OT_NAME_ID_TYPOGRAPHIC_SUBFAMILY")]
    pub fn typographic_subfamily(&self) -> String {
        self.ot_name(
            sys::hb_ot_name_id_predefined_t::TYPOGRAPHIC_SUBFAMILY,
            Language::default(),
        )
    }

    /// Gets compatible full name for MacOS.
    #[doc(alias = "HB_OT_NAME_ID_MAC_FULL_NAME")]
    pub fn mac_full_name(&self) -> String {
        self.ot_name(
            sys::hb_ot_name_id_predefined_t::MAC_FULL_NAME,
            Language::default(),
        )
    }

    /// Gets sample text.
    #[doc(alias = "HB_OT_NAME_ID_SAMPLE_TEXT")]
    pub fn sample_text(&self) -> String {
        self.ot_name(
            sys::hb_ot_name_id_predefined_t::SAMPLE_TEXT,
            Language::default(),
        )
    }

    /// Gets PostScript CID findfont name.
    #[doc(alias = "HB_OT_NAME_ID_CID_FINDFONT_NAME")]
    pub fn cid_findfont_name(&self) -> String {
        self.ot_name(
            sys::hb_ot_name_id_predefined_t::CID_FINDFONT_NAME,
            Language::default(),
        )
    }

    /// Gets WWS family Name.
    #[doc(alias = "HB_OT_NAME_ID_WWS_FAMILY")]
    pub fn wws_family(&self) -> String {
        self.ot_name(
            sys::hb_ot_name_id_predefined_t::WWS_FAMILY,
            Language::default(),
        )
    }

    /// Gets WWS subfamily Name.
    #[doc(alias = "HB_OT_NAME_ID_WWS_SUBFAMILY")]
    pub fn wws_subfamily(&self) -> String {
        self.ot_name(
            sys::hb_ot_name_id_predefined_t::WWS_SUBFAMILY,
            Language::default(),
        )
    }

    /// Gets light background palette.
    #[doc(alias = "HB_OT_NAME_ID_LIGHT_BACKGROUND")]
    pub fn light_background(&self) -> String {
        self.ot_name(
            sys::hb_ot_name_id_predefined_t::LIGHT_BACKGROUND,
            Language::default(),
        )
    }

    /// Gets dark background palette.
    #[doc(alias = "HB_OT_NAME_ID_DARK_BACKGROUND")]
    pub fn dark_background(&self) -> String {
        self.ot_name(
            sys::hb_ot_name_id_predefined_t::DARK_BACKGROUND,
            Language::default(),
        )
    }

    /// Gets variations PostScript name prefix.
    #[doc(alias = "HB_OT_NAME_ID_VARIATIONS_PS_PREFIX")]
    pub fn variations_ps_prefix(&self) -> String {
        self.ot_name(
            sys::hb_ot_name_id_predefined_t::VARIATIONS_PS_PREFIX,
            Language::default(),
        )
    }
}

impl<'a> Drop for FontFace<'a> {
    #[doc(alias = "hb_face_destroy")]
    fn drop(&mut self) {
        unsafe { sys::hb_face_destroy(self.0) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::NOTO_SANS;

    #[test]
    fn loaded_font_contains_correct_number_of_codepoints_and_glyphs() {
        let font_face = FontFace::new(Blob::from_file(NOTO_SANS).unwrap()).unwrap();
        assert_eq!(font_face.collect_unicodes().unwrap().len(), 3094);
        assert_eq!(font_face.glyph_count(), 4671);
    }

    #[test]
    fn underlying_blob_works() {
        let blob = Blob::from_file(NOTO_SANS).unwrap();
        let font_face = FontFace::new(blob.clone()).unwrap();
        assert_eq!(&*font_face.underlying_blob(), &*blob);
    }

    #[test]
    fn convert_into_raw_and_back() {
        let font_face = FontFace::new(Blob::from_file(NOTO_SANS).unwrap()).unwrap();
        let font_face_ptr = font_face.into_raw();
        let font_face = unsafe { FontFace::from_raw(font_face_ptr) };
        drop(font_face);
    }
}
