#!/bin/bash

set -e

target=${1:?"First arg should be target (linux|windows)"}
version=$(sed -n 's/^version = "\(.*\)"$/\1/p' Cargo.toml | sed 's/+/-/g')
indir="./target/release"

if [ -f "${indir}/gtfo-log-tracker" ]; then
    bin="${indir}/gtfo-log-tracker"
else
    bin="${indir}/gtfo-log-tracker.exe"
fi

[ ! -f "$bin" ] && { echo "Need to build --release"; exit 1; }

outdir="artifacts"
release_name="gtfo-log-tracker_${version}_${target}"
staging="$outdir/$release_name"
mkdir -p "$staging"
cp ./{README.md,LICENSE,CHANGELOG.md} "${indir}"/build/steamworks-sys-*/out/*steam_api*  "$staging/"
rm "${staging}"/*.lib || true
cp "$bin" "$staging/"

if [ "$target" = "linux" ]; then
    tar czf "$staging.tar.gz" -C "$outdir" "$release_name"
else
    7z a "$staging.zip" "./$staging"
fi

rm -r "$staging"