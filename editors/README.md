# BASIC in your editor

Two pieces give an editor an understanding of line-numbered BASIC:

- **`chuk-basic --lsp`**, a language server. It is the interpreter itself, so there is nothing separate to build, and it works with any editor that speaks the Language Server Protocol.
- **A Visual Studio Code extension** in [`vscode`](vscode), which carries the interpreter and server with it and adds syntax highlighting, a run button and a 1964 Dartmouth setting. Its [README](vscode/README.md) lists the features.

Both are built on [`src/analysis`](../src/analysis), the same code behind `chuk-basic --check` and `chuk-basic --renumber`.

## Visual Studio Code

Download `chuk-basic-0.1.3.vsix` from the [releases page](https://github.com/chrishayuk/chuk-basic-rs/releases), then:

```sh
code --install-extension chuk-basic-0.1.3.vsix
```

Open any `.bas` file. Nothing else has to be installed: the extension runs its own copy of the interpreter and language server, compiled to WebAssembly.

## The language server

```sh
cargo install --git https://github.com/chrishayuk/chuk-basic-rs chuk-basic
```

or, in a clone of this repository, `cargo build --release`. Start it as `chuk-basic --lsp`; it talks on standard input and output (a `--stdio` argument is accepted and ignored).

It provides diagnostics, hover, go to definition, find references and document symbols, and one command, `chukBasic.server.renumber`, which takes the file's URI and answers with a `workspace/applyEdit`. It syncs whole documents.

To have it also warn about what the BASIC of 1964 did not have, send `{ "dialect": "dartmouth" }` as the initialization options, or `{ "settings": { "chukBasic": { "dialect": "dartmouth" } } }` in a `workspace/didChangeConfiguration`. The other value is `"later"`.

### Neovim (0.11 or later)

```lua
vim.filetype.add({ extension = { bas = "basic" } })
vim.lsp.config("chuk-basic", {
  cmd = { "chuk-basic", "--lsp" },
  filetypes = { "basic" },
  init_options = { dialect = "later" },  -- or "dartmouth"
})
vim.lsp.enable("chuk-basic")
```

### Helix

In `languages.toml`:

```toml
[language-server.chuk-basic]
command = "chuk-basic"
args = ["--lsp"]
config = { dialect = "later" }

[[language]]
name = "basic"
scope = "source.basic"
file-types = ["bas"]
comment-tokens = ["REM", "'"]
language-servers = ["chuk-basic"]
```

These two configurations follow each editor's documented format but have not been run here. The server itself is tested over standard input and output by [`tests/lsp.rs`](../tests/lsp.rs) and, in the WebAssembly build the extension carries, by [`vscode/test/server.test.js`](vscode/test/server.test.js).

## From the command line

```sh
chuk-basic --check program.bas
chuk-basic --check --dialect dartmouth program.bas
chuk-basic --renumber program.bas > renumbered.bas
```

`--check` prints one line per problem, as `file:line:column: severity: message`, and exits with status 1 if any is an error. With `--dialect dartmouth` it adds, as warnings, whatever 1964 BASIC did not have. `--renumber` prints the program numbered in tens with every jump changed to match.
