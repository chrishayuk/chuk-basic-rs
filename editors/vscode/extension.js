// BASIC for VS Code.
//
// Highlighting needs nothing but the grammar in syntaxes/. Everything else -
// errors as you type, line numbers as links, hover, renumbering - comes from
// the language server, which is the interpreter itself: `chuk-basic --lsp`,
// speaking the Language Server Protocol on standard input and output.

const path = require("node:path");
const vscode = require("vscode");
const { LanguageClient, State } = require("vscode-languageclient/node");

const LANGUAGE = "basic";
const INSTALL = "https://github.com/chrishayuk/chuk-basic-rs#quick-start";
const RENUMBER = "chukBasic.server.renumber";
const START_TIMEOUT = 10000;

/** @type {LanguageClient | undefined} */
let client;
/** @type {vscode.StatusBarItem | undefined} */
let status;
let warned = false;

const settings = () => vscode.workspace.getConfiguration("chukBasic");
const program = () => settings().get("path", "chuk-basic") || "chuk-basic";
/** "later" or "dartmouth" */
const dialect = () => (settings().get("dialect") === "dartmouth" ? "dartmouth" : "later");

function showStatus() {
  if (!status) return;
  status.text = dialect() === "dartmouth" ? "BASIC: 1964" : "BASIC: later";
  status.tooltip =
    dialect() === "dartmouth"
      ? "Dartmouth BASIC as its May 1964 manual describes it. Click to switch."
      : "The BASICs of the 1970s and after. Click to switch to Dartmouth, 1964.";
  const editor = vscode.window.activeTextEditor;
  if (editor && editor.document.languageId === LANGUAGE) status.show();
  else status.hide();
}

async function serverMissing(reason) {
  if (warned) return;
  warned = true;
  const install = "How to install";
  const choose = "Set the path";
  const choice = await vscode.window.showWarningMessage(
    `BASIC: could not start the language server (\`${program()} --lsp\`): ${reason}. ` +
      "Highlighting still works; errors as you type, links between lines and renumbering need the chuk-basic program.",
    install,
    choose,
  );
  if (choice === install) vscode.env.openExternal(vscode.Uri.parse(INSTALL));
  if (choice === choose) vscode.commands.executeCommand("workbench.action.openSettings", "chukBasic.path");
}

async function startServer() {
  // No `transport` here: naming stdio makes the client add a `--stdio`
  // argument of its own. Standard input and output are the default.
  const server = { command: program(), args: ["--lsp"] };
  const next = new LanguageClient("chukBasic", "BASIC", server, {
    documentSelector: [
      { language: LANGUAGE, scheme: "file" },
      { language: LANGUAGE, scheme: "untitled" },
    ],
    initializationOptions: { dialect: dialect() },
    // a missing program is reported once, by serverMissing, not as a stream of popups
    initializationFailedHandler: () => false,
    errorHandler: {
      error: () => ({ action: 1 /* continue */, handled: true }),
      closed: () => ({ action: 1 /* do not restart */, handled: true }),
    },
  });
  client = next;
  try {
    // a program that starts and then leaves (an old chuk-basic with no --lsp)
    // would otherwise keep the client "starting" for ever
    let timer;
    const late = new Promise((_, reject) => {
      timer = setTimeout(() => reject(new Error("it did not answer within 10 seconds")), START_TIMEOUT);
    });
    try {
      await Promise.race([next.start(), late]);
    } finally {
      clearTimeout(timer);
    }
    warned = false;
  } catch (error) {
    if (client === next) client = undefined;
    next.stop().catch(() => {});
    serverMissing(error && error.code === "ENOENT" ? "the program was not found" : String((error && error.message) || error));
  }
}

async function stopServer() {
  const old = client;
  client = undefined;
  if (old && old.state !== State.Stopped) await old.stop().catch(() => {});
}

function sendDialect() {
  if (client && client.state === State.Running) {
    client.sendNotification("workspace/didChangeConfiguration", { settings: { chukBasic: { dialect: dialect() } } });
  }
}

/** The BASIC file in the active editor, saved, or undefined with a message. */
async function activeProgram() {
  const editor = vscode.window.activeTextEditor;
  if (!editor || editor.document.languageId !== LANGUAGE) {
    vscode.window.showInformationMessage("BASIC: open a .bas file first.");
    return undefined;
  }
  return editor.document;
}

const quote = (text) => (/^[A-Za-z0-9_\/.:=+-]+$/.test(text) ? text : `"${text.replace(/(["\\$`])/g, "\\$1")}"`);

async function run() {
  const document = await activeProgram();
  if (!document) return;
  if (document.isUntitled) {
    vscode.window.showInformationMessage("BASIC: save the program to a file before running it.");
    return;
  }
  if (document.isDirty && !(await document.save())) return;
  const terminal =
    vscode.window.terminals.find((candidate) => candidate.name === "BASIC" && candidate.exitStatus === undefined) ||
    vscode.window.createTerminal({ name: "BASIC", cwd: path.dirname(document.uri.fsPath) });
  terminal.show(true);
  const words = [program(), ...(dialect() === "dartmouth" ? ["--dialect", "dartmouth"] : []), document.uri.fsPath];
  terminal.sendText(words.map(quote).join(" "));
}

async function renumber() {
  const document = await activeProgram();
  if (!document) return;
  if (!client || client.state !== State.Running) {
    serverMissing("it is not running");
    return;
  }
  // the server answers by asking for the edit to be applied (workspace/applyEdit)
  await client.sendRequest("workspace/executeCommand", { command: RENUMBER, arguments: [document.uri.toString()] });
}

async function switchDialect() {
  const next = dialect() === "dartmouth" ? "later" : "dartmouth";
  const inspected = settings().inspect("dialect");
  const target =
    inspected && inspected.workspaceValue !== undefined
      ? vscode.ConfigurationTarget.Workspace
      : vscode.ConfigurationTarget.Global;
  await settings().update("dialect", next, target);
}

async function activate(context) {
  status = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 90);
  status.command = "chukBasic.switchDialect";
  context.subscriptions.push(
    status,
    vscode.commands.registerCommand("chukBasic.run", run),
    vscode.commands.registerCommand("chukBasic.renumber", renumber),
    vscode.commands.registerCommand("chukBasic.switchDialect", switchDialect),
    vscode.commands.registerCommand("chukBasic.restartServer", async () => {
      warned = false;
      await stopServer();
      await startServer();
    }),
    vscode.window.onDidChangeActiveTextEditor(showStatus),
    vscode.workspace.onDidChangeConfiguration(async (event) => {
      if (event.affectsConfiguration("chukBasic.path")) {
        warned = false;
        await stopServer();
        await startServer();
      } else if (event.affectsConfiguration("chukBasic.dialect")) {
        sendDialect();
      }
      showStatus();
    }),
  );
  showStatus();
  // not awaited: highlighting and the commands must not wait for the server
  startServer();
  // for tests
  return { client: () => client, dialect, program };
}

async function deactivate() {
  await stopServer();
}

module.exports = { activate, deactivate };
