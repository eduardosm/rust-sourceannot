#!/usr/bin/env bash
set -euo pipefail

. ci/utils.sh

begin_group "Install Rust"
./ci/install-rust.sh stable.txt --profile minimal -c clippy
# shellcheck disable=SC1090
. "$HOME/.cargo/env"
end_group

begin_group "Fetch dependencies"
cargo fetch --locked
end_group

export CARGO_REGISTRY_TOKEN="$CRATES_IO_TOKEN"

begin_group "Publish crate"
crate="sourceannot"
cargo publish -p "$crate" --no-verify --locked --dry-run
end_group
