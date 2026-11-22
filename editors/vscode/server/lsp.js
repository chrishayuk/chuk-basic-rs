// Starts the BASIC language server: chuk-basic compiled for WASI, run as
// `chuk-basic --lsp`, talking the Language Server Protocol on standard input
// and output. Any arguments (an editor's `--stdio`) are ignored.
"use strict";

const path = require("path");
const { run } = require("./wasi.js");

process.exitCode = run(path.join(__dirname, "chuk-basic.wasm"), { args: ["--lsp"] });
