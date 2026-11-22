// Runs test/vscode/index.js inside a real VS Code, with this folder loaded as
// the extension under development.
//
//   npm run test:vscode
//   CHUK_BASIC=none npm run test:vscode     without the language server
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
  // CHUK_BASIC=none runs the checks with no server, as on a machine without chuk-basic
  const server =
    process.env.CHUK_BASIC === "none"
      ? undefined
      : [process.env.CHUK_BASIC, path.join(repo, "target", "release", "chuk-basic")].find(
          (candidate) => candidate && fs.existsSync(candidate),
        );
  fs.mkdirSync(path.join(profile, "User"), { recursive: true });
  fs.writeFileSync(
    path.join(profile, "User", "settings.json"),
    JSON.stringify({
      // a path that does not exist exercises "highlighting still works without the server"
      "chukBasic.path": server || path.join(profile, "no-such-chuk-basic"),
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
      extensionTestsEnv: { CHUK_BASIC_REPO: repo, CHUK_BASIC_SERVER: server || "" },
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
