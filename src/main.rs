//! `myloasm-kmc-v1`: myloasm's on-disk k-mer counter, a thin command-line wrapper around the
//! vendored KMC fork in kmc/. It counts canonical k-mers with a separate counter per strand and
//! writes a KMC database that myloasm reads back: `myloasm --kmc` looks for this
//! binary on PATH and runs it, and `myloasm --kmc-stranded-db` accepts a database made by hand.
//!
//! The `v1` in the binary name is the interface version myloasm depends on: the command-line
//! flags below and the database format (stranded KMC database, 4-byte counters). Bump it when
//! either changes incompatibly.

mod ffi;
mod input;

use clap::Parser;
use input::{detect_shared_format, ReadFormat};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use std::path::PathBuf;
use std::process::ExitCode;

/// Each per-strand counter takes 4 bytes, so counts match myloasm's in-memory `u32` counts exactly.
const COUNTER_MAX: u64 = u32::MAX as u64;

// Defaults matching myloasm's in-memory counter. myloasm passes all three explicitly, so these
// only matter when the binary is run by hand.
const DEFAULT_MIN_COUNT_TOTAL: u32 = 3;
const DEFAULT_MIN_COUNT_PER_STRAND: u32 = 1;
const DEFAULT_MIN_MIDDLE_BASE_QUALITY: u32 = 10;

#[derive(Parser, Debug)]
#[command(
    name = "myloasm-kmc-v1",
    version,
    about = "On-disk k-mer counter bundled with myloasm (KMC with per-strand counters). \
             Writes <OUTPUT>.kmc_pre and <OUTPUT>.kmc_suf."
)]
struct Args {
    /// Read files, all FASTA or all FASTQ, optionally gzip-compressed
    #[arg(required = true, value_name = "READS")]
    input_files: Vec<String>,

    /// Database base path to write
    #[arg(short, long, value_name = "PATH")]
    output: PathBuf,

    /// Existing directory for temporary files (needs free space comparable to the input size)
    #[arg(long, value_name = "DIR")]
    tmp_dir: PathBuf,

    /// K-mer size
    #[arg(short, long, default_value_t = 21)]
    kmer_size: u32,

    /// Number of threads
    #[arg(short, long, default_value_t = 8)]
    threads: u32,

    /// RAM budget in GB
    #[arg(long, default_value_t = 12)]
    ram_gb: u32,

    /// Keep k-mers seen at least this many times in total
    #[arg(long, default_value_t = DEFAULT_MIN_COUNT_TOTAL)]
    min_count: u32,

    /// Keep k-mers seen at least this many times on each strand
    #[arg(long, default_value_t = DEFAULT_MIN_COUNT_PER_STRAND)]
    min_count_per_strand: u32,

    /// Drop k-mers whose middle base has Phred quality below this (FASTQ only; 0 keeps all)
    #[arg(long, default_value_t = DEFAULT_MIN_MIDDLE_BASE_QUALITY)]
    min_mid_quality: u32,
}

fn main() -> ExitCode {
    let args = Args::parse();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("myloasm-kmc-v1: error: {message}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: &Args) -> Result<(), String> {
    let format = detect_shared_format(&args.input_files)?;
    if !args.tmp_dir.is_dir() {
        return Err(format!(
            "temporary directory {} does not exist",
            args.tmp_dir.display()
        ));
    }

    let inputs = args
        .input_files
        .iter()
        .map(|f| c_string(f))
        .collect::<Result<Vec<_>, _>>()?;
    let input_ptrs: Vec<*const c_char> = inputs.iter().map(|s| s.as_ptr()).collect();
    let output = c_string(&args.output.to_string_lossy())?;
    let tmp_dir = c_string(&args.tmp_dir.to_string_lossy())?;

    let config = ffi::KmcStrandedConfig {
        input_files: input_ptrs.as_ptr(),
        n_input_files: input_ptrs.len(),
        input_is_fasta: (format == ReadFormat::Fasta) as c_int,
        output_db: output.as_ptr(),
        tmp_dir: tmp_dir.as_ptr(),
        kmer_len: args.kmer_size,
        n_threads: args.threads.max(1),
        max_ram_gb: args.ram_gb.max(1),
        min_count_total: args.min_count as u64,
        min_count_per_strand: args.min_count_per_strand as u64,
        counter_max: COUNTER_MAX,
        min_middle_base_quality: args.min_mid_quality,
        log: Some(print_to_stderr),
        log_ctx: std::ptr::null_mut(),
    };
    let mut stats = ffi::KmcStrandedStats::default();
    let mut error = vec![0 as c_char; 4096];

    eprintln!(
        "Counting {}-mers in {} {:?} file(s) with {} thread(s), {} GB of RAM: total count >= {}, \
         each strand >= {}, middle-base quality >= {}",
        args.kmer_size,
        args.input_files.len(),
        format,
        config.n_threads,
        config.max_ram_gb,
        args.min_count,
        args.min_count_per_strand,
        args.min_mid_quality
    );

    // SAFETY: every pointer in `config` points into a CString or Vec that outlives this call,
    // `error` is writable for `error.len()` bytes, and the C side neither throws nor keeps pointers.
    let status =
        unsafe { ffi::kmc_count_stranded(&config, &mut stats, error.as_mut_ptr(), error.len()) };
    if status != 0 {
        // SAFETY: the C side wrote a NUL-terminated string into `error` on failure.
        let message = unsafe { CStr::from_ptr(error.as_ptr()) }.to_string_lossy();
        return Err(format!("KMC failed: {message}"));
    }

    eprintln!(
        "Done: {} sequences, {} k-mer occurrences, {} distinct k-mers, {} written to {} \
         ({} below the thresholds); stage 1 {:.1}s, stage 2 {:.1}s, {:.2} GB of temporary files",
        stats.n_sequences,
        stats.n_total_kmers,
        stats.n_unique_kmers,
        stats.n_written,
        args.output.display(),
        stats.n_below_cutoff,
        stats.stage1_seconds,
        stats.stage2_seconds,
        stats.tmp_bytes as f64 / 1e9
    );
    Ok(())
}

fn c_string(s: &str) -> Result<CString, String> {
    CString::new(s).map_err(|_| format!("path contains a NUL byte: {s:?}"))
}

unsafe extern "C" fn print_to_stderr(_ctx: *mut c_void, message: *const c_char) {
    if !message.is_null() {
        // SAFETY: the C side passes a NUL-terminated string that lives for the duration of the call.
        let message = unsafe { CStr::from_ptr(message) };
        eprintln!("{}", message.to_string_lossy());
    }
}
