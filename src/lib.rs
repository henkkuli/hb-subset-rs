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
//! let font = fs::read("tests/fonts/NotoSans.ttf").unwrap();
//! let subset_font = hb_subset::subset(&font, "abc".chars()).unwrap();
//! fs::write("fonts/subset.ttf", subset_font).unwrap();
//! ```
//!
//! To get more control over how the font is subset and what gets included, you can use the lower level API directly:
//! ```rust
//! # use hb_subset::bindings::*;
//! // Load font directly from a file
//! let font = Blob::from_file("tests/fonts/NotoSans.ttf").unwrap();
//! let font = FontFace::new(font).unwrap();
//!
//! // Construct a subset manually and include only some of the letters
//! let mut subset = SubsetInput::new().unwrap();
//! subset.unicode_set().insert('f');
//! subset.unicode_set().insert('i');
//!
//! // Subset the font using just-constructed subset input
//! let new_font = subset.subset_font(&font).unwrap();
//!
//! // Extract the raw font and write to an output file
//! std::fs::write("out.ttf", &*new_font.underlying_blob()).unwrap();
//! ```

#![warn(missing_docs)]

use bindings::{Blob, FontFace, SubsetInput};
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

pub mod bindings;

/// A convenient method to create a subset of a font over given characters.
///
/// The returned font can be used everywhere where the original font was used, as long as the string contains only
/// characters from the given set. In particular, the font includes all relevant ligatures.
pub fn subset(font: &[u8], characters: impl IntoIterator<Item = char>) -> Result<Vec<u8>, Error> {
    // Add all characters to subset, and nothing more.
    let mut subset = SubsetInput::new()?;
    let mut unicode_set = subset.unicode_set();
    for char in characters {
        unicode_set.insert(char);
    }

    // Load the original font, and then construct a subset from it
    let font = FontFace::new(Blob::from_bytes(font)?)?;
    let new_font = subset.subset_font(&font)?;
    let new_font = new_font.underlying_blob().to_vec();
    Ok(new_font)
}
