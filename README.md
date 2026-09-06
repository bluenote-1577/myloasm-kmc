# myloasm-kmc

The on-disk k-mer counter for [myloasm](https://github.com/bluenote-1577/myloasm). It is a
small Rust wrapper around a fork of [KMC](https://github.com/refresh-bio/KMC) that adds
additional functionality that myloasm needs (details in
[`kmc/README.md`](kmc/README.md)). 

`myloasm --kmc` runs this as a subprocess so that
huge read sets can be counted with a fixed RAM budget instead of in memory.

This is a separate program because KMC is GPL-3 (myloasm is MIT/Apache) and needs a C++14
toolchain that myloasm itself does not.

## Installing

Requirements: a C++14 compiler (GCC 5+ or Clang), `zlib.h`, and Rust.

```sh
git clone https://github.com/bluenote-1577/myloasm-kmc.git
cd myloasm-kmc
cargo install --path . # installs myloasm-kmc-v1 into ~/.cargo/bin
```

`myloasm --kmc` requires this program to be installed. It invokes an executable named
`myloasm-kmc-v1`, looking first next to the `myloasm` executable and then on `PATH`. The command
above installs it to `~/.cargo/bin`, which must be on `PATH` (as it normally is after installing
Rust with rustup).

Set `ZLIB_DIR` if `zlib.h` is not on the default include path. x86_64 and aarch64 are supported
(Linux and macOS).

## Interface version

myloasm invokes the binary by name, and the `v1` suffix is the interface version it expects.
Version 1 consists of:

* the command line: `myloasm-kmc-v1 --output <db> --tmp-dir <dir> --kmer-size <k> --threads <n>
  --ram-gb <gb> --min-count <c> --min-count-per-strand <s> --min-mid-quality <q> -- <reads>...`
* the output: a KMC database (`<db>.kmc_pre`, `<db>.kmc_suf`) with per-strand counters
  (format versions `0x100`/`0x300`), 4-byte counters, canonical k-mers with total count
  `>= c`, each strand `>= s`, and k-mers whose middle base has Phred quality `< q` excluded.

Additive changes (new optional flags) keep the version; anything that would make an older
myloasm misread the output or fail to invoke the binary bumps it to `v2`.

## Running by hand

```
myloasm-kmc-v1 -o counts --tmp-dir /scratch -k 21 -t 16 --ram-gb 32 reads.fq.gz
myloasm reads.fq.gz -o out --kmc-stranded-db counts       # reuse the database
```

`myloasm-kmc-v1 --help` lists the options. Input files must all be FASTA or all be FASTQ
(gzip allowed, bzip2 not).

```

## License

GPL-3.0 (see `LICENSE`), because the vendored KMC sources are GPL-3.
