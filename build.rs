use std::env;
use std::path::PathBuf;
use std::fs;
use std::process::Command;

// https://github.com/woboq/qmetaobject-rs/blob/master/qmetaobject/build.rs
fn qmake_query(var: &str) -> String {
    let qmake = std::env::var("QMAKE").unwrap_or_else(|_| "qmake".to_string());
    String::from_utf8(
        Command::new(qmake)
            .env("QT_SELECT", "qt5")
            .args(&["-query", var])
            .output()
            .expect("Failed to execute qmake. Make sure 'qmake' is in your path")
            .stdout,
    )
        .expect("UTF-8 conversion failed")
}

fn main() {
    let qt_includes = env::var("QT_INCLUDE_DIR").ok();
    let qt_libs = env::var("QT_LIB_DIR").ok();
    let qt_include_path = match qt_includes {
        Some(env) => env,
        None => qmake_query("QT_INSTALL_HEADERS")
    };
    let qt_library_path = match qt_libs {
        Some(env) => env,
        None => qmake_query("QT_INSTALL_LIBS")
    };
    let mut config = cpp_build::Config::new();
    if cfg!(target_os = "macos") {
        config.flag("-F");
        config.flag(qt_library_path.trim());
    }
    config
        .include(qt_include_path.trim())
        .build("src/dialog.rs");

    let macos_lib_search = if cfg!(target_os = "macos") { "=framework" } else { "" };
    let macos_lib_framework = if cfg!(target_os = "macos") { "" } else { "5" };

    println!("cargo:rustc-link-search{}={}", macos_lib_search, qt_library_path.trim());
    println!("cargo:rustc-link-lib{}=Qt{}Widgets", macos_lib_search, macos_lib_framework);
    println!("cargo:rustc-link-lib{}=Qt{}Core", macos_lib_search, macos_lib_framework);

    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed=wrapper.h");
    // Add linker paths for cross-compilation
    println!("cargo:rustc-link-search=native=link-libs/bin/64bit");
    println!("cargo:rustc-link-search=native=link-libs/bin/64bit/libobs.0.dylib"); // Mac
    // The macOS OBS only ships with libobs.0.dylib (not libobs.dylib)
    #[cfg(target_os = "macos")]
    println!("cargo:rustc-link-lib=dylib=obs.0");
    #[cfg(not(target_os = "macos"))]
    println!("cargo:rustc-link-lib=dylib=obs");
    // Link against the frontend API (for the recording status)
    println!("cargo:rustc-link-lib=dylib=obs-frontend-api");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("bindings.rs");

    let include_env = env::var("LIBOBS_INCLUDE_DIR").ok().map(|s| format!("-I{}", s));
    let lib_env = env::var("LIBOBS_LIB").ok();
    let frontend_env = env::var("OBS_FRONTEND_LIB").ok();

    let mut builder = bindgen::Builder::default()
        .header("wrapper.h")
        .blacklist_type("_bindgen_ty_2")
        .clang_arg("-v")
        .clang_arg("-I/usr/include/obs")
        .clang_arg("-I/usr/local/include/obs")
        .clang_arg("-IC:\\Libs\\obs")
        .clang_arg("-IC:\\Libs")
        .derive_default(true)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks));

    if let Some(include) = include_env {
        let obs = format!("{}/obs", include);
        builder = builder.clang_arg(include).clang_arg(obs);
    }
    if let Some(lib) = lib_env {
        println!("cargo:rustc-link-search=native={}", lib);
    }
    if let Some(frontend) = frontend_env {
        println!("cargo:rustc-link-search=native={}", frontend);
    }

    if let Ok(bindings) = builder.generate() {
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
