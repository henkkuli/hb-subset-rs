use std::env;
use std::path::PathBuf;

use bindgen::callbacks::ParseCallbacks;

fn main() {
    // First ensure that appropriate version of HarfBuzz exists
    let include_paths = if cfg!(feature = "bundled") {
        build_harfbuzz()
    } else {
        pkg_config::probe_library("harfbuzz-subset")
            .unwrap()
            .include_paths
    };
    // Then build the sys bindings
    build_bindings(include_paths);
}

fn build_harfbuzz() -> Vec<PathBuf> {
    cc::Build::new()
        .cpp(true)
        .flag("-std=c++11")
        .warnings(false)
        .file("harfbuzz/src/harfbuzz-subset.cc")
        .compile("embedded-harfbuzz-subset");

    println!("cargo:rerun-if-changed=harfbuzz/src");

    vec!["harfbuzz/src/".into()]
}

fn build_bindings(include_paths: Vec<PathBuf>) {
    let bindings = bindgen::Builder::default()
        .clang_args(
            include_paths
                .into_iter()
                .map(|path| format!("-I{}", path.display())),
        )
        .header("wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .parse_callbacks(Box::new(NoCommentsCallback))
        .allowlist_item("hb_.*")
        .bitfield_enum("hb_subset_flags_t")
        .bitfield_enum("hb_subset_sets_t")
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

/// [`ParseCallbacks`] which make Bindgen generate no comments.
///
/// This is because Bindgen does not properly support the style of comments used in the C headers, and hence emitting
/// them to Rust code adds only unnecessary noise.
#[derive(Debug)]
struct NoCommentsCallback;
impl ParseCallbacks for NoCommentsCallback {
    fn process_comment(&self, _comment: &str) -> Option<String> {
        Some("".into())
    }

    fn enum_variant_name(
        &self,
        enum_name: Option<&str>,
        original_variant_name: &str,
        _variant_value: bindgen::callbacks::EnumVariantValue,
    ) -> Option<String> {
        if enum_name == Some("hb_subset_sets_t") {
            original_variant_name
                .strip_prefix("HB_SUBSET_SETS_")
                .map(String::from)
        } else if enum_name == Some("hb_subset_flags_t") {
            original_variant_name
                .strip_prefix("HB_SUBSET_FLAGS_")
                .map(String::from)
        } else {
            None
        }
    }
}
