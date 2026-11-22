// Tokenises BASIC with the extension's grammar, outside VS Code, using the
// same engine VS Code uses (vscode-textmate + Oniguruma).

const fs = require("node:fs");
const path = require("node:path");
const oniguruma = require("vscode-oniguruma");
const textmate = require("vscode-textmate");

const grammarFile = path.join(__dirname, "..", "syntaxes", "basic.tmLanguage.json");

let loading = null;
function loadGrammar() {
  loading ??= (async () => {
    const wasm = fs.readFileSync(require.resolve("vscode-oniguruma/release/onig.wasm"));
    await oniguruma.loadWASM(wasm.buffer.slice(wasm.byteOffset, wasm.byteOffset + wasm.byteLength));
    const registry = new textmate.Registry({
      onigLib: Promise.resolve({
        createOnigScanner: (patterns) => new oniguruma.OnigScanner(patterns),
        createOnigString: (text) => new oniguruma.OnigString(text),
      }),
      loadGrammar: async (scope) =>
        scope === "source.basic" ? textmate.parseRawGrammar(fs.readFileSync(grammarFile, "utf8"), grammarFile) : null,
    });
    return registry.loadGrammar("source.basic");
  })();
  return loading;
}

/** The tokens of a program: [{ line, text, scope }], scope being the innermost one. */
async function tokenize(source) {
  const grammar = await loadGrammar();
  const out = [];
  let stack = textmate.INITIAL;
  source.split(/\r?\n/).forEach((text, line) => {
    const result = grammar.tokenizeLine(text, stack);
    stack = result.ruleStack;
    for (const token of result.tokens) {
      const piece = text.slice(token.startIndex, token.endIndex);
      if (piece.trim() === "") continue;
      out.push({ line, text: piece, scope: token.scopes[token.scopes.length - 1], scopes: token.scopes });
    }
  });
  return out;
}

const SHORT = [
  [/\.basic$/, ""],
  ["entity.name.label.line-number.reference", "target"],
  ["entity.name.label.line-number", "lineno"],
  ["keyword.control", "control"],
  ["keyword.other.data", "keyword"],
  ["keyword.other", "keyword"],
  ["keyword.operator.logical", "logical"],
  [/keyword\.operator\.\w+/, "op"],
  ["support.function.matrix", "function"],
  ["support.function", "function"],
  ["entity.name.function", "fn"],
  ["variable.other.string", "strvar"],
  ["variable.other", "var"],
  ["constant.numeric", "number"],
  ["string.quoted.double", "string"],
  [/comment\.line\.\w+/, "comment"],
  [/punctuation\.\w+(\.\w+)*/, "punct"],
  ["meta.data", "data"],
  ["source", "plain"],
];

/** "text:short-scope" for each token of one line, for compact assertions. */
async function shape(lineOfBasic) {
  const short = (scope) => SHORT.reduce((name, [from, to]) => name.replace(from, to), scope);
  return (await tokenize(lineOfBasic)).map((token) => `${token.text.trim()}:${short(token.scope)}`).join(" ");
}

module.exports = { tokenize, shape };
