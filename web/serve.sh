#!/bin/sh
# Serves the playground at http://localhost:8000 (or the port given).
# The page must come from a web server: browsers will not load a wasm file
# or a module script from file://.
set -eu
cd "$(dirname "$0")"

if [ ! -f chuk-basic.wasm ]; then
    echo "chuk-basic.wasm is missing; run web/build.sh first" >&2
    exit 1
fi

port="${1:-8000}"
echo "http://localhost:$port"
exec python3 -m http.server "$port" --bind 127.0.0.1
