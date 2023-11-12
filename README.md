[![crates.io](https://img.shields.io/crates/v/hb-subset)](https://crates.io/crates/hb-subset)
[![docs.rs](https://img.shields.io/docsrs/hb-subset)](https://docs.rs/hb-subset/)

# hb-subset
## A Rust wrapper for HarfBuzz subsetting API

This crate exposes [HarfBuzz](https://github.com/harfbuzz/harfbuzz) API for subsetting fonts.

## What is subsetting?
From HarfBuzz documentation:
> Subsetting reduces the codepoint coverage of font files and removes all data that is no longer needed. A subset
> input describes the desired subset. The input is provided along with a font to the subsetting operation. Output is
> a new font file containing only the data specified in the input.
>
> Currently most outline and bitmap tables are supported: glyf, CFF, CFF2, sbix, COLR, and CBDT/CBLC. This also
> includes fonts with variable outlines via OpenType variations. Notably EBDT/EBLC and SVG are not supported. Layout
> subsetting is supported only for OpenType Layout tables (GSUB, GPOS, GDEF). Notably subsetting of graphite or AAT
> tables is not yet supported.
>
> Fonts with graphite or AAT tables may still be subsetted but will likely need to use the retain glyph ids option
> and configure the subset to pass through the layout tables untouched.

In other words, subsetting allows you to take a large font and construct a new, smaller font which has only those
characters that you need. Be sure to check the license of the font though, as not all fonts can be legally
subsetted.

## Why?
Many modern fonts can contain hundreds or even thousands of glyphs, of which only a couple dozen or maybe hundred is
needed in any single document. This also means that modern fonts can be very bulky compared to what is actually
needed. The solution to this is font subsetting: We can construct a font that includes only those glyphs and
features that are needed for the document.

## Usage
The simplest way to construct a subset of a font is to use [`subset()`] function. In the following example, we keep
only glyphs that are needed show any combination of characters 'a', 'b' and 'c', e.g. "abc" and "cabba" can be
rendered, but "foobar" cannot:
```rust
let font = fs::read("tests/fonts/NotoSans.ttf")?;
let subset_font = hb_subset::subset(&font, "abc".chars())?;
fs::write("tests/fonts/subset.ttf", subset_font)?;
```

To get more control over how the font is subset and what gets included, you can use the lower level API directly:
```rust
// Load font directly from a file
let font = Blob::from_file("tests/fonts/NotoSans.ttf")?;
let font = FontFace::new(font)?;

// Construct a subset manually and include only some of the letters
let mut subset = SubsetInput::new()?;
subset.unicode_set().insert('f');
subset.unicode_set().insert('i');

// Subset the font using just-constructed subset input
let new_font = subset.subset_font(&font)?;

// Extract the raw font and write to an output file
std::fs::write("tests/fonts/subset.ttf", &*new_font.underlying_blob())?;
```

## Using bundled version of HarfBuzz
By default, this crate uses the system HarfBuzz installation. If it is not available, or it is too old, this crate
can also used a bundled copy of HarfBuzz by using feature `bundled`:
```bash
cargo add hb-subset --features bundled
```

## License
This crate is licenced under MIT license ([LICENSE.md](./LICENSE.md) or https://opensource.org/licenses/MIT).

This repository includes HarfBuzz as a submodule and links against it. It is licensed with ["Old MIT" license](https://github.com/harfbuzz/harfbuzz/blob/894a1f72ee93a1fd8dc1d9218cb3fd8f048be29a/COPYING).

This repository also contains a copy of [NotoSans](https://notofonts.github.io/) font for testing purposes, contained in `tests/fonts`. It is licensed with[SIL Open Font License, Version 1.1](tests/fonts/OFL.txt).
