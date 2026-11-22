// Runs test/vscode/index.js inside a real VS Code, with this folder loaded as
// the extension under development.
//
//   npm run test:vscode                       the server the extension carries
//   CHUK_BASIC=native npm run test:vscode     target/release/chuk-basic instead
//   CHUK_BASIC=/path/to/chuk-basic npm run test:vscode
//
// The first is what someone who installs the extension gets: no chuk-basic
// program anywhere, the WebAssembly build in server/ (`npm run build`).
//
// It uses the VS Code installed on this machine if there is one (set
// VSCODE_EXECUTABLE to choose), and downloads one otherwise. The instance has
// its own empty profile and extensions folder, so it does not touch yours and
// can run while your own VS Code is open.

const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const { runTests } = require("@vscode/test-electron");

const extension = path.join(__dirname, "..");
const repo = path.join(extension, "..", "..");

const installed = [
  process.env.VSCODE_EXECUTABLE,
  "/Applications/Visual Studio Code.app/Contents/MacOS/Code",
  "/Applications/Visual Studio Code.app/Contents/MacOS/Electron",
  "/usr/share/code/code",
].find((candidate) => candidate && fs.existsSync(candidate));

async function main() {
  const profile = fs.mkdtempSync(path.join(os.tmpdir(), "chuk-basic-vscode-"));
  const asked = process.env.CHUK_BASIC || "";
  const server = asked === "native" ? path.join(repo, "target", "release", "chuk-basic") : asked === "none" ? "" : asked;
  if (server && !fs.existsSync(server)) throw new Error(`${server} does not exist`);
  if (!server && !fs.existsSync(path.join(extension, "server", "chuk-basic.wasm"))) {
    throw new Error("server/chuk-basic.wasm is not built: run `npm run build`");
  }
  fs.mkdirSync(path.join(profile, "User"), { recursive: true });
  fs.writeFileSync(
    path.join(profile, "User", "settings.json"),
    JSON.stringify({
      // empty: the server and interpreter the extension carries
      "chukBasic.path": server,
      "security.workspace.trust.enabled": false,
      "telemetry.telemetryLevel": "off",
      "update.mode": "none",
    }),
  );
  try {
    await runTests({
      ...(installed ? { vscodeExecutablePath: installed } : {}),
      extensionDevelopmentPath: extension,
      extensionTestsPath: path.join(__dirname, "vscode", "index.js"),
      // no chuk-basic can be found by name either: the built-in run must not lean on one
      extensionTestsEnv: {
        CHUK_BASIC_REPO: repo,
        CHUK_BASIC_SERVER: server,
        PATH: server ? process.env.PATH : "/usr/bin:/bin",
      },
      launchArgs: [
        path.join(repo, "samples", "dartmouth"),
        `--user-data-dir=${profile}`,
        `--extensions-dir=${path.join(profile, "extensions")}`,
        "--disable-workspace-trust",
        "--skip-welcome",
        "--skip-release-notes",
        "--disable-gpu",
      ],
    });
  } finally {
    fs.rmSync(profile, { recursive: true, force: true });
  }
}

main().catch((error) => {
  console.error(String((error && error.message) || error));
  process.exit(1);
});
