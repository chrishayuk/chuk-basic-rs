// Fills web/examples/ from samples/ and writes web/examples/index.json.
//
//   web/build.sh            (runs this after building the wasm)
//   node web/build-examples.mjs
//
// The programs in featured.json come first, with the titles, manual pages and
// credits written there. After them comes every other program in
// samples/tutorial, samples/classic and samples/games, so the page offers the
// whole bank that the tests run. Whether a program asks for input is found by
// running it, not written by hand.

import { copyFileSync, mkdirSync, readdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

import { runProgram } from "./session.js";

const web = dirname(fileURLToPath(import.meta.url));
const root = join(web, "..");
const out = join(web, "examples");
const module = new WebAssembly.Module(readFileSync(join(web, "chuk-basic.wasm")));

const featured = JSON.parse(readFileSync(join(web, "featured.json"), "utf8"));
const taken = new Set(featured.map((entry) => entry.from));

const words = (name) => name.replace(/[-_]/g, " ").replace(/^\w/, (c) => c.toUpperCase());
const firstRemark = (source) => {
  const match = /^\s*\d+\s+REM\s+(.+?)\s*$/im.exec(source.split("\n")[0]);
  return match ? match[1].replace(/\.$/, "") : null;
};

const groups = [
  {
    folder: "tutorial",
    group: "Tutorial",
    title: (name) => `${Number(name.slice(0, 2))}. ${words(name.slice(3))}`,
  },
  {
    folder: "classic",
    group: "Classics",
    title: (name, source) => firstRemark(source) || words(name),
  },
  {
    folder: "games",
    group: "BASIC Computer Games, 1978",
    credit: "BASIC Computer Games, 1978",
    title: (name) => words(name),
  },
];

rmSync(out, { recursive: true, force: true });
mkdirSync(out, { recursive: true });

const index = [];
function add(from, file, entry) {
  // the 1978 listings have Windows line endings
  const source = readFileSync(join(root, from), "utf8").replace(/\r\n/g, "\n");
  mkdirSync(dirname(join(out, file)), { recursive: true });
  writeFileSync(join(out, file), source);
  const result = runProgram(module, { source, seed: 42, dialect: entry.dialect });
  index.push({ file, ...entry, input: result.status === "waiting" });
  return source;
}

for (const { from, ...entry } of featured) {
  add(from, from.split("/").pop(), { group: entry.group || "Start here", ...entry });
}
for (const { folder, group, credit, title } of groups) {
  const names = readdirSync(join(root, "samples", folder))
    .filter((name) => name.endsWith(".bas"))
    .sort();
  for (const name of names) {
    const from = `samples/${folder}/${name}`;
    if (taken.has(from)) continue;
    const source = readFileSync(join(root, from), "utf8");
    const entry = { title: title(name.replace(/\.bas$/, ""), source), group };
    if (credit) entry.credit = credit;
    add(from, `${folder}/${name}`, entry);
  }
}

writeFileSync(join(out, "index.json"), JSON.stringify(index, null, 2) + "\n");
const counts = {};
for (const entry of index) counts[entry.group] = (counts[entry.group] || 0) + 1;
console.log(`web/examples: ${index.length} programs`, counts);
