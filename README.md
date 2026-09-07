# myloasm-kmc

The on-disk k-mer counter for [myloasm](https://github.com/bluenote-1577/myloasm). 
Myloasm v0.7.0 (**release date TODO**) will allow for disk-based k-mer counting by using this binary as a standalone process via the `--kmc` option. This alleviates a RAM bottleneck for complex metagenomes. 

It is a small Rust wrapper around a fork of [KMC](https://github.com/refresh-bio/KMC) that adds
additional functionality that myloasm needs (details in
[`kmc/README.md`](kmc/README.md)). This was vibe coded with Claude Fable 5.1; user beware. 

This is a separate program because KMC is GPL-3 (myloasm is MIT/Apache) and needs a C++14
toolchain that myloasm itself does not.

## Installing

Requirements: very standard unix-based toolchain (should be available by default) + the Rust language. Specifically, 

- a C++14 compiler (GCC 5+ or Clang)
- `zlib.h` - zlib library installed
- [Rust](https://rust-lang.org/) programming language with `cargo` and associated tools installed. 

```sh
git clone https://github.com/bluenote-1577/myloasm-kmc.git
cd myloasm-kmc
cargo install --path . # installs myloasm-kmc-v1 into ~/.cargo/bin
```

### Installation notes: 

The build locates zlib with `pkg-config`, falling back to the compiler and linker default paths.
For a custom installation, set `ZLIB_INCLUDE_DIR` and `ZLIB_LIB_DIR`, or set `ZLIB_DIR` when zlib
uses the usual `<prefix>/include` and `<prefix>/lib` layout. x86_64 and aarch64 are supported
(Linux and macOS).

`myloasm --kmc` requires this program to be installed. It invokes an executable named
`myloasm-kmc-v1`, looking first next to the `myloasm` executable and then on `PATH`. The command
above installs it to `~/.cargo/bin`, which must be on `PATH` (as it normally is after installing
Rust with rustup).

## Interface version

myloasm invokes the binary by name, and the `v1` suffix is the interface version it expects.
Version 1 consists of:

* the command line: `myloasm-kmc-v1 --output <path> [--text] --tmp-dir <dir> --kmer-size <k> --threads <n>
  --ram-gb <gb> --min-count <c> --min-count-per-strand <s> --min-mid-quality <q> -- <reads>...`
* the output: either a KMC database (`<db>.kmc_pre`, `<db>.kmc_suf`) with per-strand counters
  (format versions `0x100`/`0x300`) or, with `--text`, a TSV file. Both contain canonical k-mers
  with 4-byte counters, total count `>= c`, each strand `>= s`, and k-mers whose middle base has
  Phred quality `< q` excluded.

Additive changes (new optional flags) keep the version; anything that would make an older
myloasm misread the output or fail to invoke the binary bumps it to `v2`.

## Running by hand

```
myloasm-kmc-v1 -o counts --tmp-dir /scratch -k 21 -t 16 --ram-gb 32 reads.fq.gz
myloasm reads.fq.gz -o out --kmc-stranded-db counts       # reuse the database
myloasm-kmc-v1 --text -o counts.tsv --tmp-dir /scratch -k 21 reads.fq.gz
```

`myloasm-kmc-v1 --help` lists the options. Input files must all be FASTA or all be FASTQ
(gzip allowed, bzip2 not). `--text` is intended for debugging and writes one canonical k-mer per
line as `<k-mer>\t<forward-count>\t<reverse-count>`. The configured count and quality thresholds
still apply; use `--min-count 1 --min-count-per-strand 0 --min-mid-quality 0` to retain every
observed k-mer.

## License

GPL-3.0 (see `LICENSE`), because the vendored KMC sources are GPL-3.
