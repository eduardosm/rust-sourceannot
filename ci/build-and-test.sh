#!/usr/bin/env bash
set -euo pipefail

export RUSTDOCFLAGS="-D warnings"

cargo build --all-targets
cargo test
cargo doc

git diff --exit-code Cargo.lock
