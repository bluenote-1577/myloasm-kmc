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

    let kmc = PathBuf::from(KMC_DIR);
    let mut core = kmc_build(&kmc);
    core.files(cpp_files(&kmc.join("kmc_core"), |name| {
        !name.starts_with("raduls_")
    }));
    core.files(cpp_files(&kmc.join("kmc_api"), |_| true));
    core.file(kmc.join("ffi").join("kmc_ffi.cpp"));

    // The radix sort comes in one variant per instruction set (KMC picks one at run time), each
    // needing its own compiler flag, so they are compiled separately and added as objects.
    for (file, flag) in radix_sort_variants() {
        let mut variant = kmc_build(&kmc);
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

/// Compiler settings shared by every KMC translation unit (mirrors the upstream Makefile).
fn kmc_build(kmc: &Path) -> cc::Build {
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
        .cargo_metadata(false);
    if let Some(zlib_dir) = env::var_os("ZLIB_DIR") {
        build.include(PathBuf::from(zlib_dir).join("include"));
    }
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
