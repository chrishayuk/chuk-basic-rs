// Runs a BASIC program with the interpreter the extension carries:
//
//   node run.js [--dialect dartmouth] [--seed N] [--check] program.bas
//
// Options are passed on to chuk-basic; the last argument is the file. The
// program's INPUT comes from standard input, so in a terminal it is typed.
"use strict";

const fs = require("fs");
const path = require("path");
const { run } = require("./wasi.js");

const options = process.argv.slice(2);
const file = options.pop();
if (!file || file.startsWith("-")) {
  console.error("usage: run.js [options] program.bas");
  process.exit(2);
}
let program;
try {
  program = fs.readFileSync(file);
} catch (error) {
  console.error(`chuk-basic: cannot read ${file}: ${error.message}`);
  process.exit(2);
}
// The interpreter sees one file, under the name it had on disk, so that
// `--check` reports it as `name.bas:line:column`.
const name = path.basename(file);
process.exitCode = run(path.join(__dirname, "chuk-basic.wasm"), {
  args: [...options, name],
  files: { [name]: program },
});
