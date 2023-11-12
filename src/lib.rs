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
//! The simplest way to construct a subset of a font is to use [`subset()`] function:
//! ```
//! # use std::fs;
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let font = fs::read("tests/fonts/NotoSans.ttf")?;
//! let subset_font = hb_subset::subset(&font, "abc".chars())?;
//! fs::write("fonts/subset.ttf", subset_font)?;
//! # Ok(())
//! # }
//! ```
//!
//! To get more control over how the font is subset and what gets included, you can use the lower level API directly:
//! ```
//! # use hb_subset::*;
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Load font directly from a file
//! let font = Blob::from_file("tests/fonts/NotoSans.ttf")?;
//! let font = FontFace::new(font)?;
//!
//! // Construct a subset manually and include only some of the letters
//! let mut subset = SubsetInput::new()?;
//! subset.unicode_set().insert('f');
//! subset.unicode_set().insert('i');
//!
//! // Subset the font using just-constructed subset input
//! let new_font = subset.subset_font(&font)?;
//!
//! // Extract the raw font and write to an output file
//! std::fs::write("out.ttf", &*new_font.underlying_blob())?;
//! # Ok(())
//! # }
//! ```

#![warn(missing_docs)]

mod blob;
mod common;
mod error;
mod font_face;
mod map;
mod set;
mod subset;

pub mod sys;

pub use blob::*;
pub use common::*;
pub use error::*;
pub use font_face::*;
pub use map::*;
pub use set::*;
pub use subset::*;

/// A convenient method to create a subset of a font over given characters.
///
/// The returned font can be used everywhere where the original font was used, as long as the string contains only
/// characters from the given set. In particular, the font includes all relevant ligatures.
pub fn subset(
    font: &[u8],
    characters: impl IntoIterator<Item = char>,
) -> Result<Vec<u8>, SubsettingError> {
    // Add all characters to subset, and nothing more.
    let mut subset = SubsetInput::new().map_err(|_| SubsettingError)?;
    let mut unicode_set = subset.unicode_set();
    for char in characters {
        unicode_set.insert(char);
    }

    // Load the original font, and then construct a subset from it
    let font = FontFace::new(Blob::from_bytes(font).map_err(|_| SubsettingError)?)
        .map_err(|_| SubsettingError)?;
    let new_font = subset.subset_font(&font)?;
    let new_font = new_font.underlying_blob().to_vec();
    Ok(new_font)
}

#[cfg(test)]
mod tests {
    /// Path for Noto Sans font.
    pub(crate) const NOTO_SANS: &str = "tests/fonts/NotoSans.ttf";
    /// Path for variable version of Noto Sans font.
    pub(crate) const NOTO_SANS_VARIABLE: &str = "tests/fonts/NotoSans-Variable.ttf";
}
