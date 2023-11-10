use std::ops::{Deref, DerefMut};

use crate::{sys, CharSet, Error, FontFace, Set, TagSet, U32Set};

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
    #[doc(alias = "hb_subset_input_create_or_fail")]
    pub fn new() -> Result<Self, Error> {
        let input = unsafe { sys::hb_subset_input_create_or_fail() };
        if input.is_null() {
            return Err(Error::AllocationError);
        }
        Ok(Self(input))
    }

    /// Configures input object to keep everything in the font face. That is, all Unicodes, glyphs, names, layout items,
    /// glyph names, etc.
    ///
    /// The input can be tailored afterwards by the caller.
    #[doc(alias = "hb_subset_input_keep_everything")]
    pub fn keep_everything(&mut self) {
        unsafe { sys::hb_subset_input_keep_everything(self.as_raw()) }
    }

    /// Gets a proxy for modifying flags.
    ///
    /// # Example
    /// ```
    /// # use hb_subset::*;
    /// let mut subset = SubsetInput::new().unwrap();
    /// subset.flags().retain_glyph_names();
    /// assert_eq!(subset.get_flags(), *Flags::default().retain_glyph_names());
    /// ```
    #[doc(alias = "hb_subset_input_set_flags")]
    #[doc(alias = "hb_subset_input_get_flags")]
    pub fn flags(&mut self) -> FlagRef<'_> {
        FlagRef(self, self.get_flags())
    }

    /// Sets all of the flags in the input object to the values specified by `flags`.
    #[doc(alias = "hb_subset_input_set_flags")]
    pub fn set_flags(&mut self, flags: Flags) {
        unsafe { sys::hb_subset_input_set_flags(self.as_raw(), flags.0 .0) }
    }

    /// Gets all of the subsetting flags in the input object.
    #[doc(alias = "hb_subset_input_get_flags")]
    pub fn get_flags(&self) -> Flags {
        Flags(unsafe { sys::hb_subset_input_get_flags(self.as_raw()) })
    }

    /// Gets the set of glyph IDs to retain.
    ///
    /// The caller should modify the set as needed.
    #[doc(alias = "hb_subset_input_glyph_set")]
    #[doc(alias = "hb_subset_input_set")]
    #[doc(alias = "HB_SUBSET_SETS_GLYPH_INDEX")]
    pub fn glyph_set(&mut self) -> U32Set<'_> {
        unsafe {
            Set::from_raw(sys::hb_set_reference(sys::hb_subset_input_glyph_set(
                self.as_raw(),
            )))
        }
    }

    /// Gets the set of Unicode codepoints to retain.
    ///
    /// The caller should modify the set as needed.
    #[doc(alias = "hb_subset_input_unicode_set")]
    #[doc(alias = "hb_subset_input_set")]
    #[doc(alias = "HB_SUBSET_SETS_UNICODE")]
    pub fn unicode_set(&mut self) -> CharSet<'_> {
        unsafe {
            Set::from_raw(sys::hb_set_reference(sys::hb_subset_input_unicode_set(
                self.as_raw(),
            )))
        }
    }

    /// Gets the set of table tags which specifies tables that should not be subsetted.
    ///
    /// The caller should modify the set as needed.
    #[doc(alias = "hb_subset_input_set")]
    #[doc(alias = "HB_SUBSET_SETS_NO_SUBSET_TABLE_TAG")]
    pub fn no_subset_table_tag_set(&mut self) -> TagSet<'_> {
        unsafe {
            Set::from_raw(sys::hb_set_reference(sys::hb_subset_input_set(
                self.as_raw(),
                sys::hb_subset_sets_t::NO_SUBSET_TABLE_TAG,
            )))
        }
    }

    /// Gets the set of table tags which specifies tables which will be dropped in the subset.
    ///
    /// The caller should modify the set as needed.
    #[doc(alias = "hb_subset_input_set")]
    #[doc(alias = "HB_SUBSET_SETS_DROP_TABLE_TAG")]
    pub fn drop_table_tag_set(&mut self) -> TagSet<'_> {
        unsafe {
            Set::from_raw(sys::hb_set_reference(sys::hb_subset_input_set(
                self.as_raw(),
                sys::hb_subset_sets_t::DROP_TABLE_TAG,
            )))
        }
    }

    /// Gets the set of name ids that will be retained.
    ///
    /// The caller should modify the set as needed.
    #[doc(alias = "hb_subset_input_set")]
    #[doc(alias = "HB_SUBSET_SETS_NAME_ID")]
    pub fn name_id_set(&mut self) -> U32Set<'_> {
        unsafe {
            Set::from_raw(sys::hb_set_reference(sys::hb_subset_input_set(
                self.as_raw(),
                sys::hb_subset_sets_t::NAME_ID,
            )))
        }
    }

    /// Gets the set of name lang ids that will be retained.
    ///
    /// The caller should modify the set as needed.
    #[doc(alias = "hb_subset_input_set")]
    #[doc(alias = "HB_SUBSET_SETS_NAME_LANG_ID")]
    pub fn name_lang_id_set(&mut self) -> U32Set<'_> {
        unsafe {
            Set::from_raw(sys::hb_set_reference(sys::hb_subset_input_set(
                self.as_raw(),
                sys::hb_subset_sets_t::NAME_LANG_ID,
            )))
        }
    }

    /// Gets the set of layout feature tags that will be retained in the subset.
    ///
    /// The caller should modify the set as needed.
    #[doc(alias = "hb_subset_input_set")]
    #[doc(alias = "HB_SUBSET_SETS_LAYOUT_FEATURE_TAG")]
    pub fn layout_feature_tag_set(&mut self) -> TagSet<'_> {
        unsafe {
            Set::from_raw(sys::hb_set_reference(sys::hb_subset_input_set(
                self.as_raw(),
                sys::hb_subset_sets_t::LAYOUT_FEATURE_TAG,
            )))
        }
    }

    /// Gets the set of layout script tags that will be retained in the subset.
    ///
    /// Defaults to all tags. The caller should modify the set as needed.
    #[doc(alias = "hb_subset_input_set")]
    #[doc(alias = "HB_SUBSET_SETS_LAYOUT_SCRIPT_TAG")]
    pub fn layout_script_tag_set(&mut self) -> TagSet<'_> {
        unsafe {
            Set::from_raw(sys::hb_set_reference(sys::hb_subset_input_set(
                self.as_raw(),
                sys::hb_subset_sets_t::LAYOUT_SCRIPT_TAG,
            )))
        }
    }

    /// Subsets a font according to provided input.
    #[doc(alias = "hb_subset_or_fail")]
    pub fn subset_font(&self, font: &FontFace<'_>) -> Result<FontFace<'static>, Error> {
        let face = unsafe { sys::hb_subset_or_fail(font.as_raw(), self.as_raw()) };
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
    #[doc(alias = "hb_subset_input_reference")]
    fn clone(&self) -> Self {
        Self(unsafe { sys::hb_subset_input_reference(self.0) })
    }
}

impl Drop for SubsetInput {
    #[doc(alias = "hb_subset_input_destroy")]
    fn drop(&mut self) {
        unsafe { sys::hb_subset_input_destroy(self.0) }
    }
}

/// Flags for [`SubsetInput`].
///
/// These flags can be used to instruct which tables the subsetter should touch, and how.
///
/// # Default flags
/// ```
/// # use hb_subset::Flags;
/// assert_eq!(
///     *Flags::default()
///         .retain_hinting()
///         .remap_glyph_indices()
///         .retain_subroutines()
///         .retain_subroutines()
///         .remove_legacy_names()
///         .remove_overlap_simple_flag()
///         .remove_unrecognized_tables()
///         .remove_notdef_outline()
///         .remove_glyph_names()
///         .recompute_unicode_ranges()
///         .retain_layout_closure(),
///     Flags::default()
/// );
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Flags(sys::hb_subset_flags_t);

#[test]
fn test_flags_default() {}

impl Flags {
    fn add_flag(&mut self, flag: sys::hb_subset_flags_t) -> &mut Self {
        self.0 |= flag;
        self
    }

    fn remove_flag(&mut self, flag: sys::hb_subset_flags_t) -> &mut Self {
        self.0 .0 &= !flag.0;
        self
    }

    /// Instructs subsetter to remove hinting instructions.
    pub fn remove_hinting(&mut self) -> &mut Self {
        self.add_flag(sys::hb_subset_flags_t::NO_HINTING)
    }

    /// Instructs subsetter to retain hinting instructions.
    pub fn retain_hinting(&mut self) -> &mut Self {
        self.remove_flag(sys::hb_subset_flags_t::NO_HINTING)
    }

    /// Instructs subsetter to glyph indices.
    ///
    /// If a glyph gets dropped, its index will still be retained as an empty glyph.
    pub fn retain_glyph_indices(&mut self) -> &mut Self {
        self.add_flag(sys::hb_subset_flags_t::RETAIN_GIDS)
    }

    /// Instructs subsetter to map old glyph indices to new ones.
    pub fn remap_glyph_indices(&mut self) -> &mut Self {
        self.remove_flag(sys::hb_subset_flags_t::RETAIN_GIDS)
    }

    /// Instructs subsetter to remove subroutines from the CFF glyphs.
    ///
    /// This has only effect when subsetting a CFF font.
    pub fn remove_subroutines(&mut self) -> &mut Self {
        self.add_flag(sys::hb_subset_flags_t::DESUBROUTINIZE)
    }

    /// Instructs subsetter to retain subroutines for CFF glyphs.
    pub fn retain_subroutines(&mut self) -> &mut Self {
        self.remove_flag(sys::hb_subset_flags_t::DESUBROUTINIZE)
    }

    /// Instructs subsetter to keep non-unicode name records.
    pub fn retain_legacy_names(&mut self) -> &mut Self {
        self.add_flag(sys::hb_subset_flags_t::NAME_LEGACY)
    }

    /// Instructs subsetter to remove non-unicode name records.
    pub fn remove_legacy_names(&mut self) -> &mut Self {
        self.remove_flag(sys::hb_subset_flags_t::NAME_LEGACY)
    }

    /// Instructs subsetter to set `OVERLAP_SIMPLE` flag for simple glyphs.
    ///
    /// This is not required for OpenType, but may affect rendering in some platforms.
    pub fn set_overlap_simple_flag(&mut self) -> &mut Self {
        self.add_flag(sys::hb_subset_flags_t::SET_OVERLAPS_FLAG)
    }

    /// Instructs subsetter to not emit `OVERLAP_SIMPLE` flag.
    pub fn remove_overlap_simple_flag(&mut self) -> &mut Self {
        self.remove_flag(sys::hb_subset_flags_t::SET_OVERLAPS_FLAG)
    }

    /// Instructs subsetter to keep unrecognized tables.
    ///
    /// The subsetter wil just pass them trough without touching them.
    pub fn retain_unrecognized_tables(&mut self) -> &mut Self {
        self.add_flag(sys::hb_subset_flags_t::PASSTHROUGH_UNRECOGNIZED)
    }

    /// Instructs subsetter to remove unrecognized tables.
    pub fn remove_unrecognized_tables(&mut self) -> &mut Self {
        self.remove_flag(sys::hb_subset_flags_t::PASSTHROUGH_UNRECOGNIZED)
    }

    /// Instructs subsetter to keep glyph outline for `notdef``.
    pub fn retain_notdef_outline(&mut self) -> &mut Self {
        self.add_flag(sys::hb_subset_flags_t::NOTDEF_OUTLINE)
    }

    /// Instructs subsetter to remove glyph outline for `notdef``.
    pub fn remove_notdef_outline(&mut self) -> &mut Self {
        self.remove_flag(sys::hb_subset_flags_t::NOTDEF_OUTLINE)
    }

    /// Instructs subsetter to keep glyph name information.
    pub fn retain_glyph_names(&mut self) -> &mut Self {
        self.add_flag(sys::hb_subset_flags_t::GLYPH_NAMES)
    }

    /// Instructs subsetter to remove glyph name information.
    pub fn remove_glyph_names(&mut self) -> &mut Self {
        self.remove_flag(sys::hb_subset_flags_t::GLYPH_NAMES)
    }

    /// Instructs subsetter to recompute unicode ranges in `OS/2` table.
    pub fn recompute_unicode_ranges(&mut self) -> &mut Self {
        self.remove_flag(sys::hb_subset_flags_t::NO_PRUNE_UNICODE_RANGES)
    }

    /// Instructs subsetter to keep original unicode ranges in `OS/2` table.
    pub fn retain_unicode_ranges(&mut self) -> &mut Self {
        self.add_flag(sys::hb_subset_flags_t::NO_PRUNE_UNICODE_RANGES)
    }

    /// Instructs subsetter to keep glyphs for all possible combinations of already retained glyphs.
    ///
    /// For example, if glyphs corresponding to `f` and `i` are retained, then also glyphs corresponding to `ﬀ`, `ﬁ` and
    /// `ﬃ` are retained.
    pub fn retain_layout_closure(&mut self) -> &mut Self {
        self.remove_flag(sys::hb_subset_flags_t::NO_LAYOUT_CLOSURE)
    }

    /// Instructs subsetter to keep only minimum set of glyphs disregarding layout closure.
    pub fn no_layout_closure(&mut self) -> &mut Self {
        self.add_flag(sys::hb_subset_flags_t::NO_LAYOUT_CLOSURE)
    }
}

impl Default for Flags {
    fn default() -> Self {
        Self(sys::hb_subset_flags_t::DEFAULT)
    }
}

/// Helper for setting flags more easily.
///
/// See [`SubsetInput::flags`].
pub struct FlagRef<'s>(&'s mut SubsetInput, Flags);

impl<'s> Deref for FlagRef<'s> {
    type Target = Flags;

    fn deref(&self) -> &Self::Target {
        &self.1
    }
}
impl<'s> DerefMut for FlagRef<'s> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.1
    }
}

impl<'s> Drop for FlagRef<'s> {
    fn drop(&mut self) {
        self.0.set_flags(self.1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{tests::NOTO_SANS, Blob};

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
