// Builds what the extension carries besides its own files:  node build.js
//
//   syntaxes/basic.tmLanguage.json   the grammar (scripts/build-grammar.js)
//   server/chuk-basic.wasm           the interpreter and language server,
//                                    compiled for WASI
//
// Needs Rust with the WebAssembly target: rustup target add wasm32-wasip1
"use strict";

const { execFileSync } = require("child_process");
const fs = require("fs");
const path = require("path");

const here = __dirname;
const root = path.join(here, "..", "..");
const built = path.join(root, "target", "wasm32-wasip1", "release", "chuk-basic.wasm");
const bundled = path.join(here, "server", "chuk-basic.wasm");

execFileSync(process.execPath, [path.join(here, "scripts", "build-grammar.js")], { stdio: "inherit" });
// the editor tools, without the compiler to WebAssembly and the runtime it embeds
execFileSync(
  "cargo",
  ["build", "--release", "--target", "wasm32-wasip1", "--no-default-features", "--features", "editor"],
  { cwd: root, stdio: "inherit" },
);
fs.copyFileSync(built, bundled);
console.log(`server/chuk-basic.wasm: ${fs.statSync(bundled).size} bytes`);
