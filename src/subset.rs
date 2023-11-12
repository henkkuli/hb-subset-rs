use std::marker::PhantomData;

use crate::{sys, AllocationError, CharSet, FontFace, Map, Set, SubsettingError, TagSet, U32Set};

mod flags;

pub use flags::*;

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
    pub fn new() -> Result<Self, AllocationError> {
        let input = unsafe { sys::hb_subset_input_create_or_fail() };
        if input.is_null() {
            return Err(AllocationError);
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
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut subset = SubsetInput::new()?;
    /// subset.flags().retain_glyph_names();
    /// assert_eq!(*subset.flags(), *Flags::default().retain_glyph_names());
    ///
    /// *subset.flags() = Flags::default();
    /// assert_eq!(*subset.flags(), Flags::default());
    /// # Ok(())
    /// # }
    /// ```
    #[doc(alias = "hb_subset_input_set_flags")]
    #[doc(alias = "hb_subset_input_get_flags")]
    pub fn flags(&mut self) -> FlagRef<'_> {
        FlagRef(
            self,
            Flags(unsafe { sys::hb_subset_input_get_flags(self.as_raw()) }),
        )
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

    /// Returns a map which can be used to provide an explicit mapping from old to new glyph id's in the produced
    /// subset. The caller should populate the map as desired. If this map is left empty then glyph ids will be
    /// automatically mapped to new values by the subsetter. If populated, the mapping must be unique. That is no two
    /// original glyph ids can be mapped to the same new id. Additionally, if a mapping is provided then the retain gids
    /// option cannot be enabled.
    ///
    /// Any glyphs that are retained in the subset which are not specified in this mapping will be assigned glyph ids
    /// after the highest glyph id in the mapping.
    ///
    /// Note: this will accept and apply non-monotonic mappings, however this may result in unsorted Coverage tables.
    /// Such fonts may not work for all use cases (for example ots will reject unsorted coverage tables). So it's
    /// recommended, if possible, to supply a monotonic mapping.
    #[doc(alias = "hb_subset_input_old_to_new_glyph_mapping")]
    pub fn old_to_new_glyph_mapping(&mut self) -> Map<'_, u32, u32> {
        unsafe {
            Map::from_raw(sys::hb_map_reference(
                sys::hb_subset_input_old_to_new_glyph_mapping(self.as_raw()),
            ))
        }
    }

    /// Subsets a font according to provided input.
    #[doc(alias = "hb_subset_or_fail")]
    pub fn subset_font(&self, font: &FontFace<'_>) -> Result<FontFace<'static>, SubsettingError> {
        let face = unsafe { sys::hb_subset_or_fail(font.as_raw(), self.as_raw()) };
        if face.is_null() {
            return Err(SubsettingError);
        }
        Ok(unsafe { FontFace::from_raw(face) })
    }

    /// Computes a plan for subsetting the supplied face according to a provided input.
    ///
    /// The plan describes which tables and glyphs should be retained.
    #[doc(alias = "hb_subset_plan_create_or_fail")]
    pub fn plan<'f>(&self, font: &'f FontFace<'_>) -> Result<SubsetPlan<'f, '_>, SubsettingError> {
        let plan = unsafe { sys::hb_subset_plan_create_or_fail(font.as_raw(), self.as_raw()) };
        if plan.is_null() {
            return Err(SubsettingError);
        }
        Ok(unsafe { SubsetPlan::from_raw(plan) })
    }
}

impl SubsetInput {
    /// Converts the subset input into raw [`sys::hb_subset_input_t`] pointer.
    ///
    /// This method transfers the ownership of the subset input to the caller. It is up to the caller to call
    /// [`sys::hb_subset_input_destroy`] to free the pointer, or call [`Self::from_raw`] to convert it back into
    /// [`SubsetInput`].
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

    /// Constructs a subset input from raw [`sys::hb_subset_input_t`] pointer.
    ///
    /// # Safety
    /// The given `subset` pointer must either be constructed by some Harfbuzz function, or be returned from
    /// [`Self::into_raw`].
    pub unsafe fn from_raw(subset: *mut sys::hb_subset_input_t) -> Self {
        Self(subset)
    }
}

impl Drop for SubsetInput {
    #[doc(alias = "hb_subset_input_destroy")]
    fn drop(&mut self) {
        unsafe { sys::hb_subset_input_destroy(self.0) }
    }
}

/// Information about how a subsetting operation will be executed.
///
/// This includes e.g. how glyph ids are mapped from the original font to the subset.
pub struct SubsetPlan<'f, 'b> {
    plan: *mut sys::hb_subset_plan_t,
    // The lifetime here is actually referring to the lifetime of SubsetPlan
    unicode_to_old_glyph_mapping: Map<'static, char, u32>,
    new_to_old_glyph_mapping: Map<'static, u32, u32>,
    old_to_new_glyph_mapping: Map<'static, u32, u32>,
    _font: PhantomData<&'f FontFace<'b>>,
}

impl<'f, 'b> SubsetPlan<'f, 'b> {
    /// Executes the subsetting plan.
    #[doc(alias = "hb_subset_plan_execute_or_fail")]
    pub fn subset(&self) -> Result<FontFace<'b>, SubsettingError> {
        let font = unsafe { sys::hb_subset_plan_execute_or_fail(self.as_raw()) };
        if font.is_null() {
            return Err(SubsettingError);
        }
        Ok(unsafe { FontFace::from_raw(font) })
    }

    /// Returns the mapping between codepoints in the original font and the associated glyph id in the original font.
    #[doc(alias = "hb_subset_plan_unicode_to_old_glyph_mapping")]
    pub fn unicode_to_old_glyph_mapping(&self) -> &'_ Map<'_, char, u32> {
        &self.unicode_to_old_glyph_mapping
    }

    /// Returns the mapping between glyphs in the subset that will be produced by plan and the glyph in the original font.
    #[doc(alias = "hb_subset_plan_new_to_old_glyph_mapping")]
    pub fn new_to_old_glyph_mapping(&self) -> &'_ Map<'_, u32, u32> {
        &self.new_to_old_glyph_mapping
    }

    /// Returns the mapping between glyphs in the original font to glyphs in the subset that will be produced by plan.
    #[doc(alias = "hb_subset_plan_old_to_new_glyph_mapping")]
    pub fn old_to_new_glyph_mapping(&self) -> &'_ Map<'_, u32, u32> {
        &self.old_to_new_glyph_mapping
    }
}

impl<'f, 'b> SubsetPlan<'f, 'b> {
    /// Converts the subset plan into raw [`sys::hb_subset_plan_t`] pointer.
    ///
    /// This method transfers the ownership of the subset plan to the caller. It is up to the caller to call
    /// [`sys::hb_subset_plan_destroy`] to free the pointer, or call [`Self::from_raw`] to convert it back into
    /// [`SubsetPlan`].
    pub fn into_raw(self) -> *mut sys::hb_subset_plan_t {
        let ptr = self.plan;
        std::mem::forget(self);
        ptr
    }

    /// Exposes the raw inner pointer without transferring the ownership.
    ///
    /// Unlike [`Self::into_raw`], this method does not transfer the ownership of the pointer to the caller.
    pub fn as_raw(&self) -> *mut sys::hb_subset_plan_t {
        self.plan
    }

    /// Constructs a subset plan from raw [`sys::hb_subset_plan_t`] pointer.
    ///
    /// # Safety
    /// The given `plan` pointer must either be constructed by some Harfbuzz function, or be returned from
    /// [`Self::into_raw`].
    pub unsafe fn from_raw(plan: *mut sys::hb_subset_plan_t) -> Self {
        let unicode_to_old_glyph_mapping = unsafe {
            Map::from_raw(sys::hb_map_reference(
                sys::hb_subset_plan_unicode_to_old_glyph_mapping(plan),
            ))
        };
        let new_to_old_glyph_mapping = unsafe {
            Map::from_raw(sys::hb_map_reference(
                sys::hb_subset_plan_new_to_old_glyph_mapping(plan),
            ))
        };
        let old_to_new_glyph_mapping = unsafe {
            Map::from_raw(sys::hb_map_reference(
                sys::hb_subset_plan_old_to_new_glyph_mapping(plan),
            ))
        };

        Self {
            plan,
            unicode_to_old_glyph_mapping,
            new_to_old_glyph_mapping,
            old_to_new_glyph_mapping,
            _font: PhantomData,
        }
    }
}

impl<'f, 'b> Drop for SubsetPlan<'f, 'b> {
    #[doc(alias = "hb_subset_plan_destroy")]
    fn drop(&mut self) {
        unsafe { sys::hb_subset_plan_destroy(self.plan) }
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
        assert_eq!(orig.glyph_count(), new.glyph_count());
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
        assert_eq!(font.glyph_count(), 6); // TODO: Actually check *which* glyphs are included
                                           // Currently just assuming [empty], f, i, ﬁ, ﬃ, and ﬀ
    }

    #[test]
    #[ignore]
    fn old_to_new_glyph_mapping() {
        todo!()
    }

    #[test]
    fn convert_into_raw_and_back() {
        let subset = SubsetInput::new().unwrap();
        let subset_ptr = subset.into_raw();
        let subset = unsafe { SubsetInput::from_raw(subset_ptr) };
        drop(subset);
    }
}
