use std::{
    borrow::Borrow,
    ffi::{c_char, CStr},
    fmt,
    ptr::null,
    str::FromStr,
};

use crate::{sys, AllocationError};

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
    /// # use hb_subset::*;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut subset = SubsetInput::new()?;
    /// // Remove character-to-glyph mapping data. This can be useful in PDF files where
    /// // the mapping and positioning has already been done.
    /// subset.drop_table_tag_set().insert(Tag::new(b"cmap"));
    /// # Ok(())
    /// # }
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

/// Data type for languages.
///
/// Corresponds to a [BCP 47 language tag](https://en.wikipedia.org/wiki/IETF_language_tag).
#[derive(Clone, Copy)]
pub struct Language(sys::hb_language_t);

impl Language {
    /// Exposes the raw inner pointer without transferring the ownership.
    pub fn as_raw(&self) -> sys::hb_language_t {
        self.0
    }

    /// Constructs a language from raw [`sys::hb_language_t`] object.
    ///
    /// # Safety
    /// The given `set` pointer must either be constructed by some Harfbuzz function, or be returned from
    /// [`Self::as_raw`].
    pub unsafe fn from_raw(lang: sys::hb_language_t) -> Self {
        Self(lang)
    }
}

impl Default for Language {
    fn default() -> Self {
        Self(null())
    }
}

impl FromStr for Language {
    type Err = AllocationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Ok(Self(sys::HB_LANGUAGE_INVALID));
        }
        let lang =
            unsafe { sys::hb_language_from_string(s.as_ptr() as *const c_char, s.len() as i32) };
        if lang.is_null() {
            return Err(AllocationError);
        }
        Ok(Self(lang))
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let lang = unsafe { sys::hb_language_to_string(self.0) };
        if lang.is_null() {
            return write!(f, "[invalid]");
        }
        let lang = unsafe { CStr::from_ptr(lang) };
        if let Ok(lang) = lang.to_str() {
            write!(f, "{lang}")
        } else {
            write!(f, "[invalid]")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tag_debug_is_correct() {
        assert_eq!(u32::from(Tag::new([b'D', b'S', b'I', b'G'])), 0x44534947u32);
        assert_eq!(
            format!("{:?}", Tag::new([b'D', b'S', b'I', b'G'])),
            "Tag(DSIG)"
        );
        assert_eq!(format!("{:?}", Tag::new(b"DSIG")), "Tag(DSIG)");
    }

    #[test]
    fn language_works() {
        assert_eq!(Language::from_str("").unwrap().to_string(), "[invalid]");
        assert_eq!(Language::from_str("en").unwrap().to_string(), "en");
        assert_eq!(
            Language::from_str("non-existent").unwrap().to_string(),
            "non-existent"
        );
    }
}
