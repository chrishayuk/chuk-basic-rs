// Tests for the playground's WASI shim and re-run session, under node.
//
//   web/build.sh && cargo build --release && node web/test.mjs
//
// Uses the same wasi.js and session.js as the page. The native binary is the
// reference for what a program should print.

import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

import { Session, localRunner, runProgram, transcriptText } from "./session.js";

const web = dirname(fileURLToPath(import.meta.url));
const root = join(web, "..");
const wasmPath = join(web, "chuk-basic.wasm");
const native = join(root, "target", "release", "chuk-basic");

for (const [path, how] of [
  [wasmPath, "web/build.sh"],
  [native, "cargo build --release"],
]) {
  if (!existsSync(path)) {
    console.error(`Missing ${path}\nBuild it first: ${how}`);
    process.exit(2);
  }
}

const module = new WebAssembly.Module(readFileSync(wasmPath));
const runner = localRunner(module);
const sample = (name) => join(root, "samples", name);
const read = (path) => readFileSync(path, "utf8");

let failed = 0;
async function test(name, body) {
  try {
    await body();
    console.log(`ok   ${name}`);
  } catch (error) {
    failed += 1;
    console.log(`FAIL ${name}\n${String(error.stack || error).replace(/^/gm, "     ")}`);
  }
}

// 1. programs without input print exactly what the native binary prints
for (const name of ["classic/hello.bas", "classic/primes.bas", "classic/mandelbrot.bas"]) {
  await test(`${name}: same output as the native binary`, () => {
    const expected = execFileSync(native, ["--seed", "1", sample(name)], { encoding: "utf8" });
    const result = runProgram(module, { source: read(sample(name)), seed: 1 });
    assert.equal(result.status, "done");
    assert.equal(result.exitCode, 0);
    assert.equal(result.message, "");
    assert.ok(expected.length > 0);
    assert.equal(transcriptText(result), expected);
  });
}

// 2. the re-run input flow
await test("guess.bas: one line at a time, by re-running, matches native --echo", async () => {
  const lines = read(sample("classic/guess.in")).trimEnd().split("\n");
  const expected = execFileSync(native, ["--seed", "42", "--echo", sample("classic/guess.bas")], {
    input: lines.join("\n") + "\n",
    encoding: "utf8",
  });

  const session = new Session(runner, { source: read(sample("classic/guess.bas")), seed: 42 });
  let result = await session.start();
  assert.equal(result.status, "waiting");
  assert.ok(transcriptText(result).endsWith("YOUR GUESS? "), "stops at the first prompt");

  let previous = transcriptText(result);
  for (const line of lines) {
    assert.ok(session.waiting, `waiting before "${line}"`);
    result = await session.submit(line);
    const text = transcriptText(result);
    assert.ok(text.startsWith(previous + line + "\n"), "the replay repeats the session so far");
    previous = text;
  }

  assert.equal(result.status, "done");
  assert.ok(session.ended);
  assert.equal(session.runs, lines.length + 1);
  assert.match(transcriptText(result), /YOU GOT IT IN 7 TRIES!\n$/);
  assert.equal(transcriptText(result), expected);
  assert.deepEqual(
    result.segments.filter((s) => s.type === "in").map((s) => s.text),
    lines.map((line) => line + "\n"),
  );
  assert.throws(() => session.submit("1"), /not waiting/);
});

await test("guess.bas: played by reading the replies, any seed", async () => {
  for (const seed of [1, 7, 12345]) {
    const session = new Session(runner, { source: read(sample("classic/guess.bas")), seed });
    let result = await session.start();
    let low = 1;
    let high = 100;
    while (result.status === "waiting") {
      assert.ok(session.runs <= 8, "binary search needs at most 7 guesses");
      const guess = Math.floor((low + high) / 2);
      const before = transcriptText(result).length;
      result = await session.submit(String(guess));
      const reply = transcriptText(result).slice(before);
      if (reply.includes("TOO LOW")) low = guess + 1;
      else if (reply.includes("TOO HIGH")) high = guess - 1;
    }
    assert.equal(result.status, "done");
    assert.match(transcriptText(result), /YOU GOT IT/);
  }
});

await test("a different seed gives a different game; the same seed the same one", () => {
  const source = '10 PRINT RND(1)\n20 INPUT A\n30 PRINT RND(1)\n';
  const first = runProgram(module, { source, seed: 5, inputs: ["1"] });
  const again = runProgram(module, { source, seed: 5, inputs: ["1"] });
  const other = runProgram(module, { source, seed: 6, inputs: ["1"] });
  assert.equal(transcriptText(first), transcriptText(again));
  assert.notEqual(transcriptText(first), transcriptText(other));
});

// 3. errors
await test("14_errors.bas: the runtime error is reported, after the output", () => {
  const program = sample("tutorial/14_errors.bas");
  const reference = spawnSync(native, [program], { encoding: "utf8" });
  assert.equal(reference.status, 1);

  const result = runProgram(module, { source: read(program), seed: 1 });
  assert.equal(result.status, "error");
  assert.equal(result.exitCode, 1);
  assert.equal(result.message, "Runtime error in line 30: Division by zero");
  assert.equal(result.message, reference.stderr.trim());
  assert.equal(transcriptText(result), "ABOUT TO DIVIDE\n");
  assert.equal(transcriptText(result), reference.stdout);
});

await test("a syntax error is reported", () => {
  const result = runProgram(module, { source: "10 PRINT (1 +\n", seed: 1 });
  assert.equal(result.status, "error");
  assert.match(result.message, /error.*line 10/i);
  assert.equal(transcriptText(result), "");
});

// 4. programs that do not stop
await test("10 GOTO 10 is stopped by the step limit", () => {
  const started = Date.now();
  const result = runProgram(module, { source: "10 GOTO 10\n", seed: 1 });
  assert.equal(result.status, "step-limit");
  assert.equal(result.message, "Runtime error in line 10: Step limit exceeded");
  console.log(`     (20,000,000 steps took ${Date.now() - started} ms)`);
});

await test("an endless loop after an INPUT is stopped too", async () => {
  const session = new Session(runner, {
    source: '10 INPUT "GO"; A\n20 GOTO 20\n',
    seed: 1,
    maxSteps: 100_000,
  });
  assert.equal((await session.start()).status, "waiting");
  const result = await session.submit("1");
  assert.equal(result.status, "step-limit");
  assert.equal(transcriptText(result), "GO? 1\n");
});

await test("endless printing is stopped by the output limit", () => {
  const result = runProgram(module, { source: '10 PRINT "SPAM"\n20 GOTO 10\n', seed: 1 });
  assert.equal(result.status, "output-limit");
  assert.ok(transcriptText(result).startsWith("SPAM\nSPAM\n"));
});

await test("the 1964 manual's quadratic example needs Dartmouth arithmetic", () => {
  const source = read(join(web, "examples", "example2.bas"));
  const later = runProgram(module, { source, seed: 1 });
  assert.equal(later.message, "Runtime error in line 40: Illegal function call: SQR");
  const dartmouth = runProgram(module, { source, seed: 1, dialect: "dartmouth" });
  assert.ok(transcriptText(dartmouth).includes("IMAGINARY PART= 1.41421356"));
});

// the bundled examples
await test("every example in web/examples runs as the native binary does", () => {
  const examples = JSON.parse(read(join(web, "examples", "index.json")));
  assert.ok(examples.length >= 140, `${examples.length} examples`);
  let compared = 0;
  const endless = [];
  for (const example of examples) {
    const path = join(web, "examples", example.file);
    const result = runProgram(module, { source: read(path), seed: 42, dialect: example.dialect });
    assert.equal(result.status === "waiting", example.input, `${example.file}: "input" in the manifest`);
    assert.ok(transcriptText(result).length > 0, `${example.file} prints something`);
    if (example.input) continue;
    // a program that reads nothing must print what the native binary prints,
    // and end the same way (two examples end by running out of DATA or on an error)
    const args = ["--seed", "42", ...(example.dialect ? ["--dialect", example.dialect] : []), path];
    const reference = spawnSync(native, args, { encoding: "utf8", input: "" });
    if (result.status === "step-limit" || result.status === "output-limit") {
      // an endless program: the page stops it sooner than the command line does
      const [a, b] = [transcriptText(result), reference.stdout];
      const shared = Math.min(a.length, b.length);
      assert.equal(a.slice(0, shared), b.slice(0, shared), example.file);
      endless.push(example.file);
    } else {
      assert.equal(transcriptText(result), reference.stdout, example.file);
      assert.equal(result.message, reference.stderr.trim(), example.file);
    }
    compared += 1;
  }
  console.log(`     (${examples.length} examples, ${compared} compared with the native binary; endless: ${endless.join(", ") || "none"})`);
});

await test("superstartrek.bas: a long line of input and several commands", async () => {
  const source = read(join(web, "examples", "superstartrek.bas"));
  const session = new Session(runner, { source, seed: 42 });
  let result = await session.start();
  // a line longer than the reader's buffer arrives over several reads
  for (const line of ["SRS", "LRS", "Q".repeat(20000), "SRS"]) {
    assert.equal(result.status, "waiting");
    result = await session.submit(line);
  }
  assert.equal(result.status, "waiting");
  assert.equal(result.linesRead, 4);
});

console.log(failed === 0 ? "\nall tests passed" : `\n${failed} test(s) failed`);
process.exit(failed === 0 ? 0 : 1);
