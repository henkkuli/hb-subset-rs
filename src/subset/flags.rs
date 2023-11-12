use crate::{sys, SubsetInput};
use std::ops::Deref;
use std::ops::DerefMut;

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
pub struct Flags(pub sys::hb_subset_flags_t);

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

/// Helper which sets the flags on associated [`SubsetInput`] on drop.
///
/// See [`SubsetInput::flags`].
pub struct FlagRef<'s>(pub(crate) &'s mut SubsetInput, pub(crate) Flags);

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
        unsafe { sys::hb_subset_input_set_flags(self.0.as_raw(), self.1 .0 .0) }
    }
}
