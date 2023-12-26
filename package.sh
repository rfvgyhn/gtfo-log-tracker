#!/bin/bash

set -e

target=${1:?"First arg should be target (linux|windows)"}
version=$(sed -n 's/^version = "\(.*\)"$/\1/p' Cargo.toml | sed 's/+/-/g')
indir="./target/release"
bin="${indir}/gtfo-log-tracker"

[ ! -f "$bin" ] && { echo "Need to build --release"; exit 1; }

outdir="artifacts"
release_name="gtfo-log-tracker_${version}_${target}"
staging="$outdir/$release_name"
mkdir -p "$staging"
cp ./{README.md,LICENSE,CHANGELOG.md} "${indir}"/build/steamworks-sys-*/out/libsteam_api*  "$staging/"
cp "$bin" "$staging/"
tar czf "$staging.tar.gz" -C "$outdir" "$release_name"