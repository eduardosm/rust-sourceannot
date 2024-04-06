#!/usr/bin/env bash
set -euo pipefail

export RUSTDOCFLAGS="-D warnings"

cargo build --all-targets --locked
cargo test --locked
cargo doc --locked
