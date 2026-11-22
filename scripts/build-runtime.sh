#!/bin/sh
# Rebuilds runtime/chuk_basic_rt.wasm, the runtime module that
# `chuk-basic --compile` links every compiled program against.
#
# Run this after changing anything in runtime/ or src/rt/, then rebuild
# chuk-basic itself (which embeds the file).
#
# Needs: rustup target add wasm32-wasip1
set -eu
cd "$(dirname "$0")/.."

cargo build -p chuk-basic-rt --release --target wasm32-wasip1
cp target/wasm32-wasip1/release/chuk_basic_rt.wasm runtime/chuk_basic_rt.wasm
echo "runtime/chuk_basic_rt.wasm: $(wc -c < runtime/chuk_basic_rt.wasm | tr -d ' ') bytes"
