use std::env;
use std::path::PathBuf;
use std::fs;

fn main() {
    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed=wrapper.h");

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
        fs::copy(&out_path, &format!("{}/obs-controller-bindings.rs", env::var("CARGO_TARGET_DIR").unwrap())).expect("Could not copy bindings!");
    } else {
        println!("cargo:warning=Could not find obs headers - using pre-compiled.");
        println!("cargo:warning=This could result in a library that doesn't work.");
        fs::copy(&format!("{}/obs-controller-bindings.rs", env::var("CARGO_TARGET_DIR").unwrap()), out_path).expect("Could not copy bindings!");
    }
}