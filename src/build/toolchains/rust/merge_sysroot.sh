#!/bin/sh
set -eu

rustc_dist=$1
std_dist=$2
output=$3
target_triple=$4

mkdir -p "$output"
cp -R "$rustc_dist/rustc/." "$output/"
mkdir -p "$output/lib/rustlib/$target_triple"
cp -R \
    "$std_dist/rust-std-$target_triple/lib/rustlib/$target_triple/lib" \
    "$output/lib/rustlib/$target_triple/lib"
