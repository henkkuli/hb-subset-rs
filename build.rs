use std::env;
use std::path::PathBuf;

fn main() {
    // First ensure that appropriate version of HarfBuzz exists
    if cfg!(feature = "bundled") {
        build_harfbuzz();
    } else {
        pkg_config::probe_library("harfbuzz-subset").unwrap();
    }
    // Then build the sys bindings
    build_bindings();
}

fn build_harfbuzz() {
    cc::Build::new()
        .cpp(true)
        .flag("-std=c++11")
        .warnings(false)
        .file("harfbuzz/src/harfbuzz-subset.cc")
        .compile("embedded-harfbuzz-subset");

    println!("cargo:rerun-if-changed=harfbuzz/src");
}

fn build_bindings() {
    println!("cargo:rerun-if-changed=wrapper.h");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .allowlist_item("hb_.*")
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
