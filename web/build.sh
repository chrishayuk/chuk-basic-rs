#!/bin/sh
# Builds the interpreter for WASI and puts it next to the page, and refreshes
# the example programs from samples/.
#
# Needs: rustup target add wasm32-wasip1
set -eu
cd "$(dirname "$0")/.."

cargo build --release --target wasm32-wasip1
cp target/wasm32-wasip1/release/chuk-basic.wasm web/chuk-basic.wasm

for name in hello primes mandelbrot guess; do
    cp "samples/classic/$name.bas" web/examples/
done
for name in hi-lo hammurabi lunar superstartrek; do
    cp "samples/games/$name.bas" web/examples/
done

echo "web/chuk-basic.wasm: $(wc -c < web/chuk-basic.wasm | tr -d ' ') bytes"
