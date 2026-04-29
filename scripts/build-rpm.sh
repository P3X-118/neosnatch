#!/bin/bash
# Build an RPM for neosnatch via `cargo-generate-rpm`.
# Works from any host (no rpmbuild required).
#
# Native build:
#   bash scripts/build-rpm.sh
#
# Cross build:
#   TARGET=aarch64-unknown-linux-gnu RPM_ARCH=aarch64 bash scripts/build-rpm.sh

set -euo pipefail
cd "$(dirname "$0")/.."

TARGET=${TARGET:-}
RPM_ARCH=${RPM_ARCH:-$(uname -m)}

if ! command -v cargo-generate-rpm >/dev/null 2>&1; then
    echo "==> installing cargo-generate-rpm"
    cargo install --locked cargo-generate-rpm
fi

echo "==> building release binary (target=${TARGET:-host})"
if [ -n "${TARGET}" ]; then
    cargo build --release --target "${TARGET}"
else
    cargo build --release
fi

echo "==> generating RPM"
ARGS=(--payload-compress zstd)
if [ -n "${TARGET}" ]; then
    ARGS+=(--target "${TARGET}")
fi
ARGS+=(--arch "${RPM_ARCH}")

cargo generate-rpm "${ARGS[@]}"
echo
ls -lh target/generate-rpm/*.rpm
