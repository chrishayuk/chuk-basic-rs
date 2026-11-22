// The interpreter and language server the extension carries, run as VS Code
// runs them: `node server/run.js` and `node server/lsp.js`, with the
// WebAssembly build in server/chuk-basic.wasm (`npm run build` makes it).
//
//   npm test

const assert = require("node:assert/strict");
const { spawn, spawnSync } = require("node:child_process");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const test = require("node:test");

const here = path.join(__dirname, "..");
const repo = path.join(here, "..", "..");
const runner = path.join(here, "server", "run.js");
const wasm = path.join(here, "server", "chuk-basic.wasm");
const native = path.join(repo, "target", "release", "chuk-basic");
const built = fs.existsSync(wasm);
const skip = built ? false : "server/chuk-basic.wasm is not built: run `npm run build`";

const run = (args, input = "") => spawnSync(process.execPath, [runner, ...args], { input, encoding: "utf8" });

test("the built-in interpreter runs the classic and tutorial samples as their saved transcripts", { skip }, () => {
  let ran = 0;
  for (const folder of ["classic", "tutorial"]) {
    const dir = path.join(repo, "samples", folder);
    for (const file of fs.readdirSync(dir).filter((name) => name.endsWith(".bas")).sort()) {
      const base = path.join(dir, file.slice(0, -4));
      const input = fs.existsSync(`${base}.in`) ? fs.readFileSync(`${base}.in`, "utf8") : "";
      const result = run(["--seed", "42", "--echo", `${base}.bas`], input);
      const expected = fs.readFileSync(`${base}.out`, "utf8");
      // the transcripts hold what was printed; an error message goes elsewhere
      assert.equal(result.stdout, expected, `${folder}/${file}`);
      ran += 1;
    }
  }
  assert.ok(ran >= 35, `${ran} programs`);
});

test("the 1964 dialect, an error exit, and a file that is not there", { skip }, () => {
  const manual = path.join(repo, "samples", "dartmouth");
  assert.equal(run(["--dialect", "dartmouth", path.join(manual, "example1.bas")]).stdout, "AVERAGE OF 100 LOGS\n 1.5797 \n");
  assert.equal(run([path.join(manual, "example1.bas")]).stdout, "AVERAGE OF 100 LOGS\n 1.57970004 \n");

  const later = run([path.join(manual, "example2.bas")]);
  assert.equal(later.status, 1);
  assert.equal(later.stderr, "Runtime error in line 40: Illegal function call: SQR\n");

  const missing = run([path.join(manual, "no-such.bas")]);
  assert.equal(missing.status, 2);
  assert.match(missing.stderr, /cannot read .*no-such\.bas/);
});

test("--check names the file as it is called on disk", { skip }, () => {
  const file = path.join(fs.mkdtempSync(path.join(os.tmpdir(), "chuk-basic-")), "broken.bas");
  fs.writeFileSync(file, '10 PRINT "A"\n20 GOTO 500\n30 END\n');
  const result = run(["--check", file]);
  assert.equal(result.status, 1);
  assert.equal(result.stdout, "broken.bas:2:9: error: There is no line 500\n");
});

// ---- the language server, spoken to as an editor speaks to it

function connect(command, args) {
  const child = spawn(command, args, { stdio: ["pipe", "pipe", "inherit"] });
  let buffer = Buffer.alloc(0);
  const waiting = [];
  const arrived = [];
  child.stdout.on("data", (chunk) => {
    buffer = Buffer.concat([buffer, chunk]);
    for (;;) {
      const header = buffer.indexOf("\r\n\r\n");
      if (header === -1) return;
      const length = Number(/Content-Length: (\d+)/i.exec(buffer.subarray(0, header).toString())[1]);
      if (buffer.length < header + 4 + length) return;
      const message = JSON.parse(buffer.subarray(header + 4, header + 4 + length).toString());
      buffer = buffer.subarray(header + 4 + length);
      const next = waiting.shift();
      if (next) next(message);
      else arrived.push(message);
    }
  });
  const send = (message) => {
    const body = JSON.stringify({ jsonrpc: "2.0", ...message });
    child.stdin.write(`Content-Length: ${Buffer.byteLength(body)}\r\n\r\n${body}`);
  };
  const receive = () => new Promise((resolve) => (arrived.length ? resolve(arrived.shift()) : waiting.push(resolve)));
  let id = 0;
  const request = async (method, params) => {
    send({ id: ++id, method, params });
    const reply = await receive();
    assert.equal(reply.id, id);
    return reply.result;
  };
  const exited = new Promise((resolve) => child.on("exit", resolve));
  return { send, receive, request, exited };
}

const servers = [];
if (built) servers.push(["built-in (WebAssembly)", process.execPath, [path.join(here, "server", "lsp.js"), "--stdio"]]);
if (fs.existsSync(native)) servers.push(["native", native, ["--lsp"]]);

const uri = "file:///tmp/program.bas";
const program = '10 LET X = 3\n20 GOSUB 100\n30 IF X > 4 THEN 10\n40 END\n100 LET X = SQR(X↑2 + 1)\n110 RETURN\n';

test("there is a server to test", { skip }, () => assert.ok(servers.length > 0));

for (const [name, command, args] of servers) {
  test(`${name} server: a session from initialize to exit`, async () => {
    const server = connect(command, args);
    const hello = await server.request("initialize", {
      processId: null,
      rootUri: null,
      capabilities: {},
      initializationOptions: { dialect: "later" },
    });
    assert.equal(hello.capabilities.hoverProvider, true);
    assert.deepEqual(hello.capabilities.executeCommandProvider.commands, ["chukBasic.server.renumber"]);
    server.send({ method: "initialized", params: {} });

    const open = (text, version) =>
      version === 1
        ? server.send({ method: "textDocument/didOpen", params: { textDocument: { uri, languageId: "basic", version, text } } })
        : server.send({ method: "textDocument/didChange", params: { textDocument: { uri, version }, contentChanges: [{ text }] } });
    const diagnostics = async () => {
      const message = await server.receive();
      assert.equal(message.method, "textDocument/publishDiagnostics");
      return message.params.diagnostics.map((d) => [d.range.start.line, d.range.start.character, d.severity, d.message]);
    };

    open(program, 1);
    assert.deepEqual(await diagnostics(), []);

    // a jump to a line that is not there is marked on the number
    open(program.replace("GOSUB 100", "GOSUB 105"), 2);
    assert.deepEqual(await diagnostics(), [[1, 9, 1, "There is no line 105"]]);
    open(program, 3);
    assert.deepEqual(await diagnostics(), []);

    const at = (line, character) => ({ textDocument: { uri }, position: { line, character } });
    const hover = await server.request("textDocument/hover", at(4, 13));
    assert.match(hover.contents.value, /absolute value/);
    const target = await server.request("textDocument/definition", at(1, 10));
    assert.equal(target.range.start.line, 4, "GOSUB 100 leads to line 100");
    const uses = await server.request("textDocument/references", { ...at(0, 1), context: { includeDeclaration: false } });
    assert.deepEqual(uses.map((use) => use.range.start.line), [2], "line 10 is named by the THEN");
    const outline = await server.request("textDocument/documentSymbol", { textDocument: { uri } });
    assert.deepEqual(outline.map((symbol) => symbol.name), ["100 (subroutine)"]);

    // the 1964 setting, switched while running
    open("10 INPUT N\n20 PRINT N\n30 END\n", 4);
    assert.deepEqual(await diagnostics(), []);
    server.send({ method: "workspace/didChangeConfiguration", params: { settings: { chukBasic: { dialect: "dartmouth" } } } });
    assert.deepEqual(await diagnostics(), [[0, 3, 2, "INPUT is not one of the fifteen statements of 1964 BASIC"]]);

    // renumbering comes back as an edit for the editor to apply
    open("5 GOTO 7\n7 END\n", 5);
    await diagnostics();
    server.send({ id: 99, method: "workspace/executeCommand", params: { command: "chukBasic.server.renumber", arguments: [uri] } });
    const apply = await server.receive();
    assert.equal(apply.method, "workspace/applyEdit");
    assert.deepEqual(apply.params.edit.changes[uri].map((edit) => edit.newText), ["10", "20", "20"]);
    server.send({ id: apply.id, result: { applied: true } });
    assert.equal((await server.receive()).id, 99);

    assert.equal(await server.request("shutdown", null), null);
    server.send({ method: "exit", params: null });
    assert.equal(await server.exited, 0);
  });
}
