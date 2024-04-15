#!/usr/bin/env bash
set -euo pipefail

mkdir checkout
find . -mindepth 1 -maxdepth 1 -not -name checkout -print0 | xargs -0 mv -t checkout
cd checkout

pkgs_dir="$(pwd)/../packages"
out_dir="../output"

echo "::group::Fetch dependencies"
cargo fetch --locked
echo "::endgroup::"

echo "::group::Vendor dependencies"
mkdir ../.cargo
cargo vendor --frozen "$pkgs_dir" > ../.cargo/config.toml
echo "::endgroup::"

mkdir "$out_dir"

crate=sourceannot
version="$(awk '/^version = ".+"$/ { sub("^version = \"", ""); sub("\"$", ""); print }' Cargo.toml)"

echo "::group::Package $crate"
cargo package -p "$crate" --frozen
tar -xf "target/package/$crate-$version.crate" -C "$pkgs_dir"
pkg_checksum="$(sha256sum "target/package/$crate-$version.crate" | awk '{print $1}')"
echo "{\"files\":{},\"package\":\"$pkg_checksum\"}" > "$pkgs_dir/$crate-$version/.cargo-checksum.json"
cp -t "$out_dir" "target/package/$crate-$version.crate"
echo "::endgroup::"
