#!/usr/bin/env bash
# Refresh the vendored KMC sources from a checkout of the KMC fork.
#
#   ./sync_from_fork.sh /path/to/KMC
#
# Only the pieces needed to build the k-mer counter (and the standalone `kmc` / `kmc_dump`
# CLIs for debugging) are copied: kmc_core, kmc_api, kmc_CLI, kmc_dump. Everything else in
# upstream KMC (kmc_tools, py_kmc_api, tests, Visual Studio projects, the bundled zlib) is
# deliberately left out. Files that belong to myloasm (this script, ffi/, Makefile, README.md)
# are never touched.
set -euo pipefail

src=${1:?usage: $0 /path/to/KMC-checkout}
dst=$(cd "$(dirname "$0")" && pwd)

for d in kmc_core kmc_api kmc_CLI kmc_dump; do
    rm -rf "$dst/$d"
    mkdir -p "$dst/$d"
    find "$src/$d" -maxdepth 1 -type f \( -name '*.cpp' -o -name '*.h' -o -name '*.hpp' \) \
        -exec cp {} "$dst/$d/" \;
done
# ntHash is header-only and pulled in by kmc_core/params.h
mkdir -p "$dst/kmc_core/libs/ntHash"
cp "$src"/kmc_core/libs/ntHash/*.h "$src"/kmc_core/libs/ntHash/*.hpp "$dst/kmc_core/libs/ntHash/"

cp "$src/README.md" "$dst/README.upstream.md"
git -C "$src" describe --always --dirty --abbrev=12 > "$dst/UPSTREAM_COMMIT"
git -C "$src" branch --show-current >> "$dst/UPSTREAM_COMMIT"

echo "synced from $src ($(git -C "$src" rev-parse --short HEAD)) into $dst"
