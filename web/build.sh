#!/bin/sh
# Builds the interpreter for WASI and puts it next to the page, and refreshes
# the example programs from samples/ (see build-examples.mjs).
#
# Needs: rustup target add wasm32-wasip1
set -eu
cd "$(dirname "$0")/.."

# the page only interprets, so the compiler (and the runtime it embeds) is left out
cargo build --release --target wasm32-wasip1 --no-default-features
cp target/wasm32-wasip1/release/chuk-basic.wasm web/chuk-basic.wasm

node web/build-examples.mjs

echo "web/chuk-basic.wasm: $(wc -c < web/chuk-basic.wasm | tr -d ' ') bytes"
