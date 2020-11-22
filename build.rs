use std::env;
use std::path::PathBuf;
use std::fs;
use semver::Version;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;
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

// qreal is a double, unless QT_COORD_TYPE says otherwise:
// https://doc.qt.io/qt-5/qtglobal.html#qreal-typedef
fn detect_qreal_size(qt_include_path: &str) {
    let path = Path::new(qt_include_path).join("QtCore").join("qconfig.h");
    let f = std::fs::File::open(&path).unwrap_or_else(|_| panic!("Cannot open `{:?}`", path));
    let b = BufReader::new(f);

    // Find declaration of QT_COORD_TYPE
    for line in b.lines() {
        let line = line.expect("qconfig.h is valid UTF-8");
        if line.contains("QT_COORD_TYPE") {
            if line.contains("float") {
                println!("cargo:rustc-cfg=qreal_is_float");
                return;
            } else {
                panic!("QT_COORD_TYPE with unknown declaration {}", line);
            }
        }
    }
}

fn main() {
    let qt_include_path = qmake_query("QT_INSTALL_HEADERS");
    let qt_library_path = qmake_query("QT_INSTALL_LIBS");
    let qt_version =
        qmake_query("QT_VERSION").parse::<Version>().expect("Parsing Qt version failed");
    let mut config = cpp_build::Config::new();

    if cfg!(target_os = "macos") {
        config.flag("-F");
        config.flag(qt_library_path.trim());
    }

    for minor in 7..=15 {
        if qt_version >= Version::new(5, minor, 0) {
            println!("cargo:rustc-cfg=qt_5_{}", minor);
        }
    }

    detect_qreal_size(&qt_include_path.trim());

    config.include(qt_include_path.trim()).build("src/dialog.rs");

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
