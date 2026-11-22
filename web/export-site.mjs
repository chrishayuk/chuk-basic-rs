// Copies what a host page needs to run the interpreter into another folder:
// the wasm, the WASI shim, the session logic, the worker, the example
// programs gathered into one examples.json, and the manual pages that the
// examples point at.
//
//   web/build.sh && node web/export-site.mjs ../some-site/public/demos/basic
//
// Used for the interactive study on chrishayuk.com, whose own page provides
// the editor and transcript. Edit the files here and export again, rather
// than editing the copies.

import { copyFileSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const scans = join(here, "..", "docs", "scans", "dartmouth-1964");
const to = process.argv[2];
if (!to) {
  console.error("usage: node web/export-site.mjs <destination folder>");
  process.exit(2);
}
mkdirSync(join(to, "scans"), { recursive: true });

for (const file of ["chuk-basic.wasm", "wasi.js", "session.js", "worker.js"]) {
  copyFileSync(join(here, file), join(to, file));
}

const index = JSON.parse(readFileSync(join(here, "examples", "index.json"), "utf8"));
const pages = new Set(["cover.jpg", "title-page.jpg", "p04-instruction-format.jpg", "p05-formulas.jpg", "detail-sqr-absolute-value.jpg", "detail-program-2-result.jpg"]);
const examples = index.map(({ file, ...rest }) => {
  for (const page of [rest.scan, ...(rest.details || [])]) if (page) pages.add(page);
  delete rest.from;
  return {
    name: file.replace(/\.bas$/, ""),
    ...rest,
    program: readFileSync(join(here, "examples", file), "utf8"),
  };
});
writeFileSync(join(to, "examples.json"), JSON.stringify(examples));
for (const page of pages) copyFileSync(join(scans, page), join(to, "scans", page));

console.log(`${to}: wasm, wasi.js, session.js, worker.js, examples.json (${examples.length} programs), ${pages.size} manual pages`);
