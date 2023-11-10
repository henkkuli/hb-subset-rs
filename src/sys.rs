//! Raw FFI bindings to HarfBuzz.
//!
//! See <https://harfbuzz.github.io/reference-manual.html> for documentation on these methods.

#![allow(missing_docs)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub const HB_SET_VALUE_INVALID: u32 = u32::MAX;
pub const HB_LANGUAGE_INVALID: *const hb_language_impl_t = std::ptr::null();

impl From<hb_ot_name_id_predefined_t> for hb_ot_name_id_t {
    fn from(value: hb_ot_name_id_predefined_t) -> Self {
        Self(value.0)
    }
}

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
