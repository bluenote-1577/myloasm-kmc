//! Declarations matching kmc/ffi/kmc_ffi.h, and the link directives for the static library that
//! build.rs produces from the vendored KMC sources.

use std::os::raw::{c_char, c_int, c_void};

pub type LogFn = Option<unsafe extern "C" fn(ctx: *mut c_void, message: *const c_char)>;

#[repr(C)]
pub struct KmcStrandedConfig {
    pub input_files: *const *const c_char,
    pub n_input_files: usize,
    pub input_is_fasta: c_int,
    pub output_db: *const c_char,
    pub tmp_dir: *const c_char,
    pub kmer_len: u32,
    pub n_threads: u32,
    pub max_ram_gb: u32,
    pub min_count_total: u64,
    pub min_count_per_strand: u64,
    pub counter_max: u64,
    pub min_middle_base_quality: u32,
    pub log: LogFn,
    pub log_ctx: *mut c_void,
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct KmcStrandedStats {
    pub n_sequences: u64,
    pub n_total_kmers: u64,
    pub n_unique_kmers: u64,
    pub n_below_cutoff: u64,
    pub n_written: u64,
    pub tmp_bytes: u64,
    pub stage1_seconds: f64,
    pub stage2_seconds: f64,
}

#[link(name = "kmc_core", kind = "static")]
#[cfg_attr(target_os = "macos", link(name = "c++"))]
#[cfg_attr(not(target_os = "macos"), link(name = "stdc++"))]
extern "C" {
    /// Returns 0 on success; otherwise 1 with a NUL-terminated message written into `error`.
    pub fn kmc_count_stranded(
        config: *const KmcStrandedConfig,
        stats: *mut KmcStrandedStats,
        error: *mut c_char,
        error_len: usize,
    ) -> c_int;
}
