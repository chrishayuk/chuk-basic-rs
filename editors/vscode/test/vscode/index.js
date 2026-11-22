// Runs inside VS Code (see ../run-vscode.js). Each check prints a line; any
// failure fails the run.

const assert = require("node:assert/strict");
const { execFileSync } = require("node:child_process");
const path = require("node:path");
const vscode = require("vscode");

const repo = process.env.CHUK_BASIC_REPO;
const server = process.env.CHUK_BASIC_SERVER;
const sample = (name) => vscode.Uri.file(path.join(repo, "samples", name));

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
async function until(what, probe, timeout = 15000) {
  const end = Date.now() + timeout;
  for (;;) {
    const value = await probe();
    if (value) return value;
    if (Date.now() > end) throw new Error(`timed out waiting for ${what}`);
    await sleep(100);
  }
}

/** Does this chuk-basic have the language server? */
function serverSpeaksLsp() {
  if (!server) return false;
  try {
    return execFileSync(server, ["--help"], { encoding: "utf8" }).includes("--lsp");
  } catch {
    return false;
  }
}

/** The grammar's scopes for a document, from VS Code's own tokeniser. */
async function scopesOf(uri) {
  const tokens = await vscode.commands.executeCommand("_workbench.captureSyntaxTokens", uri);
  return tokens.map((token) => ({ text: token.c, scopes: token.t }));
}

const checks = [];
const check = (name, body) => checks.push({ name, body });

check("a .bas file opens as BASIC, not Visual Basic", async () => {
  const document = await vscode.workspace.openTextDocument(sample("dartmouth/example2.bas"));
  await vscode.window.showTextDocument(document);
  assert.equal(document.languageId, "basic");
  const languages = await vscode.languages.getLanguages();
  assert.ok(languages.includes("basic"));
  // the built-in Visual Basic extension claims .bas too
  return `languageId=${document.languageId}; Visual Basic present: ${languages.includes("vb")}`;
});

check("the extension activates for it", async () => {
  const extension = vscode.extensions.getExtension("chrishayuk.chuk-basic");
  assert.ok(extension, "extension is installed");
  await until("activation", () => extension.isActive);
  const commands = await vscode.commands.getCommands(true);
  for (const command of ["chukBasic.run", "chukBasic.renumber", "chukBasic.switchDialect", "chukBasic.restartServer"]) {
    assert.ok(commands.includes(command), command);
  }
  return `version ${extension.packageJSON.version}`;
});

check("the grammar colours the manual's program 2", async () => {
  const tokens = await scopesOf(sample("dartmouth/example2.bas"));
  const has = (text, scope) =>
    assert.ok(
      tokens.some((token) => token.text.trim() === text && token.scopes.includes(scope)),
      `"${text}" as ${scope}; got ${JSON.stringify(tokens.filter((token) => token.text.trim() === text))}`,
    );
  has("40", "entity.name.label.line-number.basic");
  has("LET", "keyword.other.basic");
  has("SQR", "support.function.basic");
  has("IF", "keyword.control.basic");
  has("THEN", "keyword.control.basic");
  has("100", "entity.name.label.line-number.reference.basic");
  has('"QUADRATIC EQUATION"', "string.quoted.double.basic");
  has("DATA", "keyword.other.data.basic");
  return `${tokens.length} tokens`;
});

check("the language server: errors, hover, definition, references, symbols", async () => {
  if (!serverSpeaksLsp()) return "NOT RUN: no chuk-basic with --lsp was found";
  const api = vscode.extensions.getExtension("chrishayuk.chuk-basic").exports;
  await until("the server to start", () => api.client() && api.client().state === 2 /* Running */);

  // a jump to a line that does not exist
  const broken = await vscode.workspace.openTextDocument({
    language: "basic",
    content: '10 PRINT "A"\n20 GOTO 500\n30 END\n',
  });
  await vscode.window.showTextDocument(broken);
  const diagnostics = await until("a diagnostic", () => {
    const found = vscode.languages.getDiagnostics(broken.uri);
    return found.length > 0 && found;
  });
  assert.ok(
    diagnostics.some((diagnostic) => diagnostic.range.start.line === 1),
    `a diagnostic on the GOTO line; got ${JSON.stringify(diagnostics.map((d) => [d.range.start.line, d.message]))}`,
  );
  const report = [`diagnostic: "${diagnostics[0].message}"`];

  const uri = sample("dartmouth/example2.bas");
  const document = await vscode.workspace.openTextDocument(uri);
  await vscode.window.showTextDocument(document);
  const line = (number) => {
    for (let index = 0; index < document.lineCount; index++) {
      if (document.lineAt(index).text.startsWith(`${number} `)) return index;
    }
    throw new Error(`no line ${number}`);
  };
  // the 100 in `50 IF D = 0 THEN 100`
  const at = new vscode.Position(line(50), document.lineAt(line(50)).text.indexOf("100") + 1);

  const definition = await until("a definition", async () => {
    const found = await vscode.commands.executeCommand("vscode.executeDefinitionProvider", uri, at);
    return found && found.length > 0 && found;
  });
  const target = definition[0].targetRange || definition[0].range;
  assert.equal(target.start.line, line(100), "THEN 100 goes to line 100");
  report.push("definition: THEN 100 -> line 100");

  const references = await vscode.commands.executeCommand(
    "vscode.executeReferenceProvider",
    uri,
    new vscode.Position(line(300), 1),
  );
  assert.ok(references && references.length >= 2, `lines that jump to 300; got ${references && references.length}`);
  report.push(`references to line 300: ${references.length}`);

  const sqr = new vscode.Position(line(40), document.lineAt(line(40)).text.indexOf("SQR") + 1);
  const hovers = await vscode.commands.executeCommand("vscode.executeHoverProvider", uri, sqr);
  assert.ok(hovers && hovers.length > 0, "a hover on SQR");
  const text = hovers
    .flatMap((hover) => hover.contents)
    .map((content) => (typeof content === "string" ? content : content.value))
    .join(" ");
  assert.match(text, /square/i);
  report.push("hover on SQR mentions the square root");

  // the outline lists what a program defines: its functions and subroutines.
  // Program 2 has neither; program 1 defines FNL.
  const first = sample("dartmouth/example1.bas");
  await vscode.window.showTextDocument(await vscode.workspace.openTextDocument(first));
  const symbols = await until("document symbols", async () => {
    const found = await vscode.commands.executeCommand("vscode.executeDocumentSymbolProvider", first);
    return found && found.length > 0 && found;
  });
  assert.deepEqual(symbols.map((symbol) => symbol.name), ["FNL"]);
  report.push("outline of program 1: FNL");
  return report.join("; ");
});

check("renumbering rewrites the lines and what points at them", async () => {
  if (!serverSpeaksLsp()) return "NOT RUN: no chuk-basic with --lsp was found";
  const document = await vscode.workspace.openTextDocument({
    language: "basic",
    content: '5 PRINT "A"\n6 GOTO 9\n9 END\n',
  });
  await vscode.window.showTextDocument(document);
  await sleep(300);
  await vscode.commands.executeCommand("chukBasic.renumber");
  await until("the edit", () => document.getText().startsWith("10 "));
  assert.equal(document.getText(), '10 PRINT "A"\n20 GOTO 30\n30 END\n');
  return "5,6,9 -> 10,20,30 and GOTO 9 -> GOTO 30";
});

check("switching the dialect changes the setting and the 1964 warnings", async () => {
  const before = vscode.workspace.getConfiguration("chukBasic").get("dialect");
  assert.equal(before, "later");
  await vscode.commands.executeCommand("chukBasic.switchDialect");
  await until("the setting", () => vscode.workspace.getConfiguration("chukBasic").get("dialect") === "dartmouth");
  let report = "later -> dartmouth";
  if (serverSpeaksLsp()) {
    const document = await vscode.workspace.openTextDocument({
      language: "basic",
      content: '10 INPUT N$\n20 PRINT N$\n30 END\n',
    });
    await vscode.window.showTextDocument(document);
    const found = await until("a 1964 warning", () => {
      const list = vscode.languages.getDiagnostics(document.uri);
      return list.length > 0 && list;
    });
    report += `; 1964 warning: "${found[0].message}"`;
  } else {
    report += "; warnings NOT RUN (no server)";
  }
  await vscode.commands.executeCommand("chukBasic.switchDialect");
  await until("the setting", () => vscode.workspace.getConfiguration("chukBasic").get("dialect") === "later");
  return report;
});

exports.run = async function run() {
  let failed = 0;
  for (const { name, body } of checks) {
    try {
      const note = await body();
      console.log(`ok   ${name}${note ? ` (${note})` : ""}`);
    } catch (error) {
      failed += 1;
      console.log(`FAIL ${name}\n     ${String((error && error.stack) || error).replace(/\n/g, "\n     ")}`);
    }
  }
  await vscode.commands.executeCommand("workbench.action.closeAllEditors");
  if (failed > 0) throw new Error(`${failed} check(s) failed`);
};
