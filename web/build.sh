#!/bin/sh
# Builds the interpreter for WASI and puts it next to the page, and refreshes
# the example programs from samples/ (see build-examples.mjs).
#
# Needs: rustup target add wasm32-wasip1
set -eu
cd "$(dirname "$0")/.."

# The page interprets and, for its editor, asks the language server; it does
# not compile. So the compiler (and the runtime it embeds) is left out, and
# the editor tools (`--lsp`) are kept: one file serves both.
cargo build --release --target wasm32-wasip1 --no-default-features --features editor
cp target/wasm32-wasip1/release/chuk-basic.wasm web/chuk-basic.wasm

node web/build-examples.mjs

echo "web/chuk-basic.wasm: $(wc -c < web/chuk-basic.wasm | tr -d ' ') bytes"
