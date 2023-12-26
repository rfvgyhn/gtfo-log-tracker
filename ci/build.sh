#!/bin/bash
set -e

target=${1:?"First arg should be target (linux|windows)"}
file_name_version=${2:?"Second arg should be target file name version"}
version_suffix=${3}
root="$(dirname "$(readlink -f "$0")")/.."

sed -i "s/^\(version = \".*\)\"$/\1${version_suffix}\"/g" "${root}/Cargo.toml"
cargo build --release

indir="${root}/target/release"
outdir="${root}/artifacts"
release_name="gtfo-log-tracker_${file_name_version}_${target}"
staging="$outdir/$release_name"
mkdir -p "$staging"
cp "${indir}"/build/steamworks-sys-*/out/*steam_api* "$staging/"
rm "${staging}/*.lib"
[ ! -f "${indir}/gtfo-log-tracker" ] || mv "${indir}/gtfo-log-tracker" "${staging}"
[ ! -f "${indir}/gtfo-log-tracker.exe" ] || mv "${indir}/gtfo-log-tracker.exe" "${staging}"