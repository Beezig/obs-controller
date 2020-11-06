use std::env;
use std::path::PathBuf;
use std::fs;

fn main() {
    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed=wrapper.h");
    // Add linker paths for cross-compilation
    println!("cargo:rustc-link-search=native=link-libs/bin/64bit");
    println!("cargo:rustc-link-search=native=link-libs/bin/64bit/libobs.0.dylib"); // Mac
    // The macOS OBS only ships with libobs.0.dylib (not libobs.dylib)
    #[cfg(feature = "macos")]
    println!("cargo:rustc-link-lib=dylib=obs.0");
    #[cfg(not(feature = "macos"))]
    println!("cargo:rustc-link-lib=dylib=obs");
    // Link against the frontend API (for the recording status)
    println!("cargo:rustc-link-lib=dylib=obs-frontend-api");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("bindings.rs");

    if let Ok(bindings) = bindgen::Builder::default()
        .header("wrapper.h")
        .blacklist_type("_bindgen_ty_2")
        .clang_arg("-I/usr/include/obs")
        .derive_default(true)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate() {
        bindings
            .write_to_file(&out_path)
            .expect("Couldn't write bindings!");
        fs::copy(&out_path, "bindings/generated.rs").expect("Could not copy bindings!");
    } else {
        println!("cargo:warning=Could not find obs headers - using pre-compiled.");
        println!("cargo:warning=This could result in a library that doesn't work.");
        fs::copy("bindings/generated.rs", out_path).expect("Could not copy bindings!");
    }
}