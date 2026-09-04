.PHONY: fuzz fuzz-loop fuzz-minimize

# The fuzz budget per target in seconds, the jobs to run it on and the
# sanitizer to build with. The sanitizer stays off by default, its runtime
# hangs before main on macOS.
FUZZ_TIME ?= 60
FUZZ_JOBS ?= $(shell nproc 2>/dev/null || sysctl -n hw.ncpu)
FUZZ_SANITIZER ?= none

# fuzz runs every fuzz target for the budget each on all cores, from its
# corpus, stopping at the first finding.
fuzz:
	for target in $$(cargo +nightly fuzz list); do \
		cargo +nightly fuzz run -s $(FUZZ_SANITIZER) -j $(FUZZ_JOBS) $$target -- -max_total_time=$(FUZZ_TIME) || exit 1; \
	done

# fuzz-minimize drops the inputs of every fuzz corpus that add no coverage,
# keeping a corpus grown by fuzz runs small before it is committed.
fuzz-minimize:
	for target in $$(cargo +nightly fuzz list); do \
		cargo +nightly fuzz cmin -s $(FUZZ_SANITIZER) $$target || exit 1; \
	done

# fuzz-loop runs the fuzz targets round robin until a finding stops it or the
# loop is interrupted, the corpus growing across rounds.
fuzz-loop:
	while true; do $(MAKE) --no-print-directory fuzz || exit 1; done
