//! Safe mid-level bindings for HarfBuzz objects.

mod blob;
mod font_face;
mod set;
mod subset;

pub use blob::Blob;
pub use font_face::FontFace;
pub use set::{CharSet, Set, SetIter, Tag, TagSet, U32Set};
pub use subset::{Flags, SubsetInput};

#[cfg(test)]
mod tests {
    /// Path for Noto Sans font.
    pub(crate) const NOTO_SANS: &'static str = "tests/fonts/NotoSans.ttf";
    /// Path for variable version of Noto Sans font.
    pub(crate) const NOTO_SANS_VARIABLE: &'static str = "tests/fonts/NotoSans-Variable.ttf";
}
