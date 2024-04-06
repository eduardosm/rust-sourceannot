#!/usr/bin/env bash
set -euo pipefail

export RUSTDOCFLAGS="-D warnings"

cargo fetch --locked

cargo build --all-targets --frozen
cargo test --frozen
cargo doc --frozen
