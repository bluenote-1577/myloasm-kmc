/*
  C interface to the vendored KMC k-mer counter, used by the `myloasm-kmc` binary.

  Counts canonical k-mers with a separate counter per strand (KMC `-sc`) and writes a KMC
  database (<output_db>.kmc_pre / <output_db>.kmc_suf). The database is read back by
  myloasm's pure-Rust reader (src/kmc/reader.rs). No C++ exception ever crosses this boundary.
*/
#ifndef MYLOASM_KMC_FFI_H
#define MYLOASM_KMC_FFI_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Receives one message per call, without a trailing newline. */
typedef void (*kmc_log_fn)(void* ctx, const char* message);

typedef struct kmc_stranded_config {
	const char* const* input_files;
	size_t n_input_files;
	int input_is_fasta;               /* 0: FASTQ, nonzero: FASTA (multi-line records allowed). All inputs share one format. */
	const char* output_db;            /* base path of the database to write */
	const char* tmp_dir;              /* existing directory for KMC's temporary bins */
	uint32_t kmer_len;
	uint32_t n_threads;
	uint32_t max_ram_gb;
	uint64_t min_count_total;         /* keep k-mers with fwd + rev >= this            (KMC -ci) */
	uint64_t min_count_per_strand;    /* keep k-mers with min(fwd, rev) >= this        (KMC -cb) */
	uint64_t counter_max;             /* saturation value of each per-strand counter   (KMC -cs) */
	uint32_t min_middle_base_quality; /* drop k-mers whose middle base has Phred quality below this; 0 keeps all (KMC -mq) */
	kmc_log_fn log;                   /* optional; receives KMC's verbose output and warnings */
	void* log_ctx;
} kmc_stranded_config;

typedef struct kmc_stranded_stats {
	uint64_t n_sequences;
	uint64_t n_total_kmers;     /* k-mer occurrences seen in the input */
	uint64_t n_unique_kmers;    /* distinct k-mers seen */
	uint64_t n_below_cutoff;    /* distinct k-mers removed by the count thresholds */
	uint64_t n_written;         /* records in the database */
	uint64_t tmp_bytes;         /* size of the temporary bins */
	double stage1_seconds;
	double stage2_seconds;
} kmc_stranded_stats;

/* Returns 0 on success. On failure returns 1 and writes a NUL-terminated message into
   `error` (at most `error_len` bytes, truncated if needed). `stats` may be NULL. */
int kmc_count_stranded(const kmc_stranded_config* config, kmc_stranded_stats* stats, char* error, size_t error_len);

#ifdef __cplusplus
}
#endif

#endif
