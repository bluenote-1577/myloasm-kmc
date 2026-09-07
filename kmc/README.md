# Vendored KMC (k-mer counter)

A trimmed copy of the [KMC](https://github.com/refresh-bio/KMC) k-mer counter (upstream version
3.2.4) taken from Jim Shaw's (bluenote-1577) fork, which adds three things upstream KMC lacks:

* **per-strand counters** (`-sc`): each canonical k-mer stores how often it was read as itself and
  how often as its reverse complement; `-cb<n>` keeps only k-mers whose weaker strand was seen at
  least `n` times. Databases with these counters use format versions `0x100` / `0x300`, which
  stock KMC readers reject instead of misparsing.
* **a middle-base quality filter** (`-mq<q>`): k-mers whose middle base (offset k/2) has Phred
  quality below `q` are dropped without affecting their neighbours. FASTQ only; a read whose
  qualities are all identical is not filtered. This is the filter myloasm's in-memory counter applies.
* `-DKMC_SYSTEM_ZLIB`: build against the system `<zlib.h>` instead of the zlib bundled upstream.

The crate's `build.rs` compiles these sources into the `myloasm-kmc-v1` executable, which
`myloasm --kmc` runs as a subprocess. The resulting database is read back by
myloasm's own Rust reader (`src/kmc/reader.rs` in the myloasm repository).

## License

KMC is distributed under the GNU GPL v3 (`../LICENSE`).

## Layout

| path                    | contents                                                                 |
|-------------------------|--------------------------------------------------------------------------|
| `kmc_core/`, `kmc_api/` | the counter and the database API, everything needed to count             |
| `kmc_CLI/`, `kmc_dump/` | upstream command-line tools, used only by the standalone `Makefile` build |
| `ffi/`                  | the C shim (`kmc_count_stranded`) called from `src/ffi.rs`                |
| `Makefile`              | standalone build of `bin/kmc` and `bin/kmc_dump` for debugging           |
| `sync_from_fork.sh`     | refreshes the copy from a checkout of the fork                           |
| `UPSTREAM_COMMIT`       | fork commit (and branch) the copy was taken from                         |

Left out from upstream: `kmc_tools`, `py_kmc_api`, tests, documentation, Visual Studio
projects and the bundled zlib.

## Updating

Make changes in the fork, so they are versioned there, then:

```
./sync_from_fork.sh /path/to/KMC     # copies kmc_core, kmc_api, kmc_CLI, kmc_dump
cd .. && cargo build --release       # rebuilds myloasm-kmc-v1
```

then run myloasm's `tests/kmer_counting_equivalence.rs` with the new binary on PATH.
`build.rs` recompiles when anything under this directory changes, headers included.

## Platforms

x86_64 (SSE2 / SSE4.1 / AVX / AVX2 radix sorts, chosen at run time) and aarch64 (NEON), on
Linux and macOS, with GCC or Clang supporting C++14. Needs `zlib.h`; set `ZLIB_DIR` if it is not
on the default include path.
