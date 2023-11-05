//! Raw FFI bindings to HarfBuzz.
//!
//! See <https://harfbuzz.github.io/reference-manual.html> for documentation on these methods.

#![allow(missing_docs)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub const HB_SET_VALUE_INVALID: u32 = u32::MAX;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(
            unsafe { hb_version_atleast(7, 0, 0) } != 0,
            "The minimum supported version of HarfBuzz is 7.0.0"
        );
    }
}
