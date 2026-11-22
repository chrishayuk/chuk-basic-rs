// BASIC for VS Code.
//
// Highlighting needs nothing but the grammar in syntaxes/. Everything else -
// errors as you type, line numbers as links, hover, renumbering, running a
// program - comes from the interpreter, which is also the language server
// (`chuk-basic --lsp`, speaking the Language Server Protocol on standard
// input and output).
//
// The extension carries the interpreter compiled to WebAssembly, in server/,
// and runs it with the Node that VS Code is built on, so nothing else has to
// be installed. Setting `chukBasic.path` uses a chuk-basic program instead.

const childProcess = require("node:child_process");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const vscode = require("vscode");
const { LanguageClient, State, TransportKind } = require("vscode-languageclient/node");

const LANGUAGE = "basic";
const INSTALL = "https://github.com/chrishayuk/chuk-basic-rs#install";
const RENUMBER = "chukBasic.server.renumber";
const START_TIMEOUT = 10000;

/** @type {LanguageClient | undefined} */
let client;
/** @type {vscode.StatusBarItem | undefined} */
let status;
let warned = false;
/** @type {vscode.ExtensionContext} */
let extension;

const settings = () => vscode.workspace.getConfiguration("chukBasic");
/** The chuk-basic program the user asked for, or "" for the one carried here. */
const program = () => (settings().get("path", "") || "").trim();
const carried = (file) => extension.asAbsolutePath(path.join("server", file));
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
  const which = program() ? `\`${program()} --lsp\`, from the chukBasic.path setting` : "the one built into the extension";
  const choice = await vscode.window.showWarningMessage(
    `BASIC: could not start the language server (${which}): ${reason}. ` +
      "Highlighting still works. Clear chukBasic.path to use the built-in server.",
    install,
    choose,
  );
  if (choice === install) vscode.env.openExternal(vscode.Uri.parse(INSTALL));
  if (choice === choose) vscode.commands.executeCommand("workbench.action.openSettings", "chukBasic.path");
}

async function startServer() {
  // The built-in server is a Node module, started by the client with VS Code's
  // own Node; it adds a `--stdio` argument, which server/lsp.js ignores. For a
  // chuk-basic program no transport is named: standard input and output are
  // the default, and naming them would add that argument to its command line.
  const builtIn = { module: carried("lsp.js"), transport: TransportKind.stdio };
  const server = program() ? { command: program(), args: ["--lsp"] } : { run: builtIn, debug: builtIn };
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

/**
 * Runs the interpreter the extension carries, in a terminal the extension
 * draws itself.
 *
 * VS Code's own program doubles as Node, which is how the interpreter is
 * started; but started that way directly as a task's process it never begins
 * on Windows. So the extension starts it as a child and joins it to the
 * terminal by hand: what the program prints is shown, and what is typed is
 * echoed and sent on a line at a time, as a terminal would. Ctrl+C ends it.
 */
function carriedExecution(args, cwd) {
  return new vscode.CustomExecution(async () => {
    const shown = new vscode.EventEmitter();
    const closed = new vscode.EventEmitter();
    let child;
    let line = "";
    let over = false;
    const end = (status) => {
      if (over) return;
      over = true;
      closed.fire(status);
    };
    return {
      onDidWrite: shown.event,
      onDidClose: closed.event,
      open() {
        child = childProcess.spawn(process.execPath, [carried("run.js"), ...args], {
          cwd,
          env: { ...process.env, ELECTRON_RUN_AS_NODE: "1" },
          stdio: ["pipe", "pipe", "pipe"],
        });
        for (const stream of [child.stdout, child.stderr]) {
          stream.setEncoding("utf8");
          // a terminal wants a carriage return with each new line
          stream.on("data", (text) => shown.fire(text.replace(/\r?\n/g, "\r\n")));
        }
        child.stdin.on("error", () => {}); // the program may end while a line is on its way
        child.on("error", (error) => {
          shown.fire(`chuk-basic: ${error.message}\r\n`);
          end(1);
        });
        // 130 is what a shell reports for a program stopped with Ctrl+C
        child.on("close", (status, signal) => end(status === null ? (signal ? 130 : 1) : status));
      },
      close() {
        if (child && !over) child.kill();
      },
      handleInput(data) {
        if (!child || over) return;
        for (const key of data) {
          if (key === "\x03") {
            shown.fire("^C\r\n");
            child.kill();
            return;
          }
          if (key === "\x1b") return; // an arrow or function key: nothing to do with it
          if (key === "\r" || key === "\n") {
            shown.fire("\r\n");
            child.stdin.write(line + "\n");
            line = "";
          } else if (key === "\x7f" || key === "\b") {
            if (line.length > 0) {
              line = line.slice(0, -1);
              shown.fire("\b \b");
            }
          } else if (key >= " ") {
            line += key;
            shown.fire(key);
          }
        }
      },
    };
  });
}

async function run() {
  const document = await activeProgram();
  if (!document) return;
  let file = document.uri.fsPath;
  if (document.isUntitled || document.uri.scheme !== "file") {
    // not on disk yet: run a copy
    file = path.join(os.tmpdir(), "untitled.bas");
    fs.writeFileSync(file, document.getText());
  } else if (document.isDirty && !(await document.save())) {
    return;
  }
  const options = dialect() === "dartmouth" ? ["--dialect", "dartmouth"] : [];
  const cwd = path.dirname(file);
  // A task, not text sent to a shell: it gets a terminal of its own, where
  // INPUT is typed and Ctrl+C stops a program that never ends.
  const execution = program()
    ? new vscode.ProcessExecution(program(), [...options, file], { cwd })
    : carriedExecution([...options, file], cwd);
  const task = new vscode.Task(
    { type: "chuk-basic" },
    vscode.TaskScope.Workspace,
    `Run ${path.basename(file)}`,
    "BASIC",
    execution,
    [],
  );
  task.presentationOptions = {
    reveal: vscode.TaskRevealKind.Always,
    panel: vscode.TaskPanelKind.Dedicated,
    focus: true,
    clear: true,
    // without this the terminal first prints the whole command line
    echo: false,
    showReuseMessage: false,
  };
  return vscode.tasks.executeTask(task);
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
  extension = context;
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
