//! Compiles the vendored KMC k-mer counter (kmc/) into a static library, `libkmc_core.a`, which
//! src/ffi.rs links into the `myloasm-kmc-v1` binary.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const KMC_DIR: &str = "kmc";

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={KMC_DIR}");
    println!("cargo:rerun-if-env-changed=ZLIB_DIR");
    println!("cargo:rerun-if-env-changed=ZLIB_INCLUDE_DIR");
    println!("cargo:rerun-if-env-changed=ZLIB_LIB_DIR");

    let kmc = PathBuf::from(KMC_DIR);
    let zlib_include_paths = find_zlib();
    let mut core = kmc_build(&kmc, &zlib_include_paths);
    core.files(cpp_files(&kmc.join("kmc_core"), |name| {
        !name.starts_with("raduls_")
    }));
    core.files(cpp_files(&kmc.join("kmc_api"), |_| true));
    core.file(kmc.join("ffi").join("kmc_ffi.cpp"));

    // The radix sort comes in one variant per instruction set (KMC picks one at run time), each
    // needing its own compiler flag, so they are compiled separately and added as objects.
    for (file, flag) in radix_sort_variants() {
        let mut variant = kmc_build(&kmc, &zlib_include_paths);
        if let Some(flag) = flag {
            variant.flag(flag);
        }
        variant.file(kmc.join("kmc_core").join(file));
        for object in variant.compile_intermediates() {
            core.object(object);
        }
    }
    core.compile("kmc_core");

    // Linking is declared by the binary that uses the library, not here.
    println!(
        "cargo:rustc-link-search=native={}",
        env::var("OUT_DIR").unwrap()
    );
}

/// Locates zlib and returns the include paths needed by the vendored C++ sources.
///
/// Explicit paths take precedence, followed by pkg-config. If pkg-config is unavailable (notably
/// with the zlib supplied by the macOS SDK), the compiler and linker default search paths are used.
fn find_zlib() -> Vec<PathBuf> {
    let prefix = env::var_os("ZLIB_DIR").map(PathBuf::from);
    let include_dir = env::var_os("ZLIB_INCLUDE_DIR")
        .map(PathBuf::from)
        .or_else(|| prefix.as_ref().map(|path| path.join("include")));
    let lib_dir = env::var_os("ZLIB_LIB_DIR")
        .map(PathBuf::from)
        .or_else(|| prefix.as_ref().map(|path| path.join("lib")));

    if include_dir.is_some() || lib_dir.is_some() {
        if let Some(path) = &include_dir {
            assert!(
                path.is_dir(),
                "configured zlib include directory does not exist: {}",
                path.display()
            );
        }
        if let Some(path) = &lib_dir {
            assert!(
                path.is_dir(),
                "configured zlib library directory does not exist: {}",
                path.display()
            );
            println!("cargo:rustc-link-search=native={}", path.display());
        }
        println!("cargo:rustc-link-lib=z");
        return include_dir.into_iter().collect();
    }

    match pkg_config::Config::new().probe("zlib") {
        Ok(library) => library.include_paths,
        Err(error) => {
            println!(
                "cargo:warning=pkg-config could not locate zlib ({error}); trying the compiler and linker default paths"
            );
            println!("cargo:rustc-link-lib=z");
            Vec::new()
        }
    }
}

/// Compiler settings shared by every KMC translation unit (mirrors the upstream Makefile).
fn kmc_build(kmc: &Path, zlib_include_paths: &[PathBuf]) -> cc::Build {
    let mut build = cc::Build::new();
    build
        .cpp(true)
        .std("c++14")
        .opt_level(3)
        .warnings(false)
        .flag("-fsigned-char") // KMC keeps symbol codes in `char` and relies on it being signed
        .flag_if_supported("-pthread")
        .define("KMC_SYSTEM_ZLIB", None) // <zlib.h> instead of the zlib bundled with upstream KMC
        .include(kmc)
        .includes(zlib_include_paths)
        .cargo_metadata(false);
    build
}

fn radix_sort_variants() -> Vec<(&'static str, Option<&'static str>)> {
    match env::var("CARGO_CFG_TARGET_ARCH").unwrap().as_str() {
        "x86_64" => vec![
            ("raduls_sse2.cpp", Some("-msse2")),
            ("raduls_sse41.cpp", Some("-msse4.1")),
            ("raduls_avx.cpp", Some("-mavx")),
            ("raduls_avx2.cpp", Some("-mavx2")),
        ],
        "aarch64" => vec![("raduls_neon.cpp", None)],
        other => panic!(
            "KMC has no radix sort for the {other} architecture (x86_64 and aarch64 are supported)"
        ),
    }
}

fn cpp_files(dir: &Path, keep: impl Fn(&str) -> bool) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()))
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "cpp"))
        .filter(|path| keep(path.file_name().unwrap().to_str().unwrap()))
        .collect();
    files.sort();
    files
}
