#!/usr/bin/env bash
set -euo pipefail

cargo fetch --locked
cargo clippy --all-targets --frozen -- -D warnings
