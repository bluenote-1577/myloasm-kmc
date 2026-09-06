# Standalone build of the vendored KMC counter, for debugging the counter on its own and for
# checking the vendored copy independently of cargo:
#
#   make -j            -> bin/kmc bin/kmc_dump
#
# myloasm does not use this Makefile: build.rs compiles the same sources with the `cc` crate
# into the `myloasm-kmc` binary. Object files carry header dependencies (-MMD), so editing a
# header rebuilds what depends on it.

CXX      ?= g++
CXXFLAGS ?= -O3 -std=c++14 -Wall -fsigned-char -pthread -DKMC_SYSTEM_ZLIB
LDFLAGS  ?= -pthread
LDLIBS   ?= -lz

ARCH := $(shell uname -m)

CORE_SRC := $(filter-out kmc_core/raduls_%.cpp,$(wildcard kmc_core/*.cpp))
API_SRC  := $(wildcard kmc_api/*.cpp)

# Radix sort variants for the instruction sets KMC dispatches on at run time
ifeq ($(ARCH),x86_64)
RADULS_OBJ := kmc_core/raduls_sse2.o kmc_core/raduls_sse41.o kmc_core/raduls_avx.o kmc_core/raduls_avx2.o
else
RADULS_OBJ := kmc_core/raduls_neon.o
endif
kmc_core/raduls_sse2.o:  CXXFLAGS += -msse2
kmc_core/raduls_sse41.o: CXXFLAGS += -msse4.1
kmc_core/raduls_avx.o:   CXXFLAGS += -mavx
kmc_core/raduls_avx2.o:  CXXFLAGS += -mavx2

CORE_OBJ := $(CORE_SRC:.cpp=.o) $(API_SRC:.cpp=.o) $(RADULS_OBJ)
DUMP_OBJ := kmc_dump/kmc_dump.o kmc_dump/nc_utils.o $(API_SRC:.cpp=.o)

all: bin/kmc bin/kmc_dump

bin/kmc: kmc_CLI/kmc.o $(CORE_OBJ)
	@mkdir -p bin
	$(CXX) $(LDFLAGS) -o $@ $^ $(LDLIBS)

bin/kmc_dump: $(DUMP_OBJ)
	@mkdir -p bin
	$(CXX) $(LDFLAGS) -o $@ $^

%.o: %.cpp
	$(CXX) $(CXXFLAGS) -MMD -MP -c $< -o $@

-include $(CORE_OBJ:.o=.d) $(DUMP_OBJ:.o=.d) kmc_CLI/kmc.d

clean:
	rm -f kmc_core/*.o kmc_core/*.d kmc_api/*.o kmc_api/*.d kmc_CLI/*.o kmc_CLI/*.d kmc_dump/*.o kmc_dump/*.d ffi/*.o ffi/*.d
	rm -rf bin

.PHONY: all clean
