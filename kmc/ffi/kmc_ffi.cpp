#include "kmc_ffi.h"
#include "../kmc_core/kmc_runner.h"

#include <algorithm>
#include <cstring>
#include <exception>
#include <stdexcept>
#include <string>
#include <vector>

namespace {

// Routes KMC's verbose / warning messages to the caller's callback, one line per call.
class CallbackLogger : public KMC::ILogger {
	kmc_log_fn fn;
	void* ctx;
	std::string prefix;

public:
	CallbackLogger(kmc_log_fn fn, void* ctx, std::string prefix) : fn(fn), ctx(ctx), prefix(std::move(prefix)) {}

	void Log(const std::string& msg) override {
		if (!fn)
			return;
		// KMC's verbose messages are multi-line blocks ending in a newline.
		size_t start = 0;
		while (start < msg.size()) {
			size_t end = msg.find('\n', start);
			if (end == std::string::npos)
				end = msg.size();
			if (end > start) {
				std::string line = prefix + msg.substr(start, end - start);
				fn(ctx, line.c_str());
			}
			start = end + 1;
		}
	}
};

void set_error(char* error, size_t error_len, const std::string& msg) {
	if (!error || error_len == 0)
		return;
	size_t n = std::min(msg.size(), error_len - 1);
	std::memcpy(error, msg.data(), n);
	error[n] = '\0';
}

} // namespace

extern "C" int kmc_count_stranded(const kmc_stranded_config* cfg, kmc_stranded_stats* stats, char* error, size_t error_len) {
	try {
		if (!cfg || !cfg->output_db || !cfg->tmp_dir)
			throw std::runtime_error("kmc_count_stranded: config, output_db and tmp_dir are required");
		if (cfg->n_input_files == 0)
			throw std::runtime_error("kmc_count_stranded: no input files");

		std::vector<std::string> inputs(cfg->input_files, cfg->input_files + cfg->n_input_files);

		CallbackLogger verbose(cfg->log, cfg->log_ctx, "");
		CallbackLogger warnings(cfg->log, cfg->log_ctx, "warning: ");
		KMC::NullPercentProgressObserver no_percent;
		KMC::NullProgressObserver no_progress;

		KMC::Stage1Params stage1;
		stage1.SetInputFiles(inputs)
			.SetTmpPath(cfg->tmp_dir)
			.SetKmerLen(cfg->kmer_len)
			.SetNThreads(cfg->n_threads)
			.SetMaxRamGB(cfg->max_ram_gb)
			.SetInputFileType(cfg->input_is_fasta ? KMC::InputFileType::MULTILINE_FASTA : KMC::InputFileType::FASTQ)
			.SetCanonicalKmers(true)
			.SetStrandedCounters(true)
			.SetMinMiddleBaseQuality(cfg->min_middle_base_quality)
			.SetVerboseLogger(&verbose)
			.SetWarningsLogger(&warnings)
			.SetPercentProgressObserver(&no_percent)
			.SetProgressObserver(&no_progress);

		KMC::Stage2Params stage2;
		stage2.SetNThreads(cfg->n_threads)
			.SetMaxRamGB(cfg->max_ram_gb)
			.SetCutoffMin(cfg->min_count_total)
			.SetCutoffMinPerStrand(cfg->min_count_per_strand)
			.SetCounterMax(cfg->counter_max)
			.SetOutputFileName(cfg->output_db)
			.SetOutputFileType(KMC::OutputFileType::KMC);

		KMC::Runner runner;
		KMC::Stage1Results r1 = runner.RunStage1(stage1);
		KMC::Stage2Results r2 = runner.RunStage2(stage2);

		if (stats) {
			stats->n_sequences = r1.nSeqences;
			stats->n_total_kmers = r2.nTotalKmers;
			stats->n_unique_kmers = r2.nUniqueKmers;
			stats->n_below_cutoff = r2.nBelowCutoffMin;
			stats->n_written = r2.nUniqueKmers - r2.nBelowCutoffMin - r2.nAboveCutoffMax;
			stats->tmp_bytes = r1.tmpSize;
			stats->stage1_seconds = r1.time;
			stats->stage2_seconds = r2.time;
		}
		return 0;
	} catch (const std::exception& e) {
		set_error(error, error_len, e.what());
		return 1;
	} catch (...) {
		set_error(error, error_len, "unknown error inside KMC");
		return 1;
	}
}
