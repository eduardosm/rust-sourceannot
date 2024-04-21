#!/usr/bin/env bash
set -euo pipefail

. ci/utils.sh

begin_group "Install Rust"
./ci/install-rust.sh stable.txt --profile minimal -c clippy
# shellcheck disable=SC1091
. "$HOME/.cargo/env"
end_group

begin_group "Fetch dependencies"
cargo fetch --locked
end_group

export CARGO_REGISTRY_TOKEN="$CRATES_IO_TOKEN"

crate="sourceannot"

begin_group "Publish $crate"
cargo publish -p "$crate" --no-verify --locked
end_group
