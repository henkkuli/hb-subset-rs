# A Rust wrapper for HarfBuzz subsetting API

Subsetting is the process of taking a font and constructing a new, smaller font, by including parts of the font that are necessary for the situation. For example, many modern fonts, like [Noto](https://fonts.google.com/noto) include thousands of glyps, out of which only a couple of dozen or hundred are needed at any given time.

This crate offers an API to subset a font to include only the needed glyphs to be able to prsent any string containing given unicode code points. This is especially useful for static page generators which know at compile time which characters are needed and which not.

## License
Copyright 2023 Henrik Lievonen

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the “Software”), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
