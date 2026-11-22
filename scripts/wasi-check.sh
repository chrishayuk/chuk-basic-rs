#!/bin/sh
# Builds the release binary natively and for WASI, runs every sample program
# through both, and checks that they print exactly the same thing.
#
# Needs: rustup target add wasm32-wasip1, and wasmtime on the PATH.
set -eu
cd "$(dirname "$0")/.."

cargo build --release --quiet
cargo build --release --quiet --target wasm32-wasip1
native=target/release/chuk-basic
wasm=target/wasm32-wasip1/release/chuk-basic.wasm

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

# keyboard input for programs that have no script of their own
i=0
while [ $i -lt 60 ]; do
    printf '1\n2\nYES\n5\n3,4\nN\n10\n0\n'
    i=$((i + 1))
done > "$work/keys"

total=0
failed=0
for program in samples/*.bas samples/*/*.bas; do
    input="${program%.bas}.in"
    [ -f "$input" ] || input="$work/keys"
    set -- --seed 42 --echo --max-steps 2000000 "$program"
    "$native" "$@" < "$input" > "$work/native.out" 2> "$work/native.err" || true
    wasmtime run --dir=. "$wasm" "$@" < "$input" > "$work/wasm.out" 2> "$work/wasm.err" || true
    total=$((total + 1))
    if ! cmp -s "$work/native.out" "$work/wasm.out" || ! cmp -s "$work/native.err" "$work/wasm.err"; then
        failed=$((failed + 1))
        echo "DIFFERS: $program"
    fi
done

echo "$total programs compared, $failed differ between native and WASI"
[ "$failed" -eq 0 ]
