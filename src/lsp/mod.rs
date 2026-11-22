//! A language server: the editor's side of `analysis`, spoken as the
//! Language Server Protocol over standard input and output.
//!
//! `chuk-basic --lsp` starts it. It keeps the text of each open file and
//! answers from that: diagnostics whenever a file changes, and hover,
//! go to definition, find references, the outline, and renumbering on
//! request. No dependencies: the JSON is in `json`.

mod json;

use std::collections::HashMap;
use std::io::{BufRead, Write};

use crate::analysis::{self, CompletionKind, Lint, Location, Severity, SymbolKind, TokenType};
pub use json::Json;

/// The command an editor sends to renumber a file; its argument is the
/// file's URI. The server answers with a `workspace/applyEdit` request.
pub const RENUMBER: &str = "chukBasic.server.renumber";

struct Server<'a> {
    output: &'a mut dyn Write,
    documents: HashMap<String, String>,
    lint: Lint,
    next_id: u32,
    shutdown: bool,
}

/// Serve one editor until it says `exit`. Returns the exit status: 0 after
/// a proper `shutdown`, 1 otherwise, as the protocol asks.
pub fn serve(input: &mut dyn BufRead, output: &mut dyn Write) -> i32 {
    let mut server = Server {
        output,
        documents: HashMap::new(),
        lint: Lint::default(),
        next_id: 1,
        shutdown: false,
    };
    while let Some(message) = read_message(input) {
        let method = message.get("method").as_str().unwrap_or("").to_string();
        if method == "exit" {
            return i32::from(!server.shutdown);
        }
        server.handle(&method, &message);
    }
    1
}

/// One message: headers, a blank line, then `Content-Length` bytes of JSON.
fn read_message(input: &mut dyn BufRead) -> Option<Json> {
    let mut length = None;
    loop {
        let mut header = String::new();
        if input.read_line(&mut header).ok()? == 0 {
            return None;
        }
        let header = header.trim();
        if header.is_empty() {
            break;
        }
        if let Some((name, value)) = header.split_once(':') {
            if name.eq_ignore_ascii_case("content-length") {
                length = value.trim().parse::<usize>().ok();
            }
        }
    }
    let mut body = vec![0; length?];
    input.read_exact(&mut body).ok()?;
    // a message that is not JSON is skipped, not fatal
    Some(Json::parse(&String::from_utf8_lossy(&body)).unwrap_or(Json::Null))
}

fn lint_named(name: Option<&str>) -> Option<Lint> {
    match name? {
        "dartmouth" => Some(Lint::Dartmouth1964),
        "later" => Some(Lint::Later),
        _ => None,
    }
}

fn range(at: Location) -> Json {
    let position = |character: u32| {
        Json::obj([
            ("line", Json::num(at.line as u32)),
            ("character", Json::num(character)),
        ])
    };
    Json::obj([
        ("start", position(at.span.start)),
        ("end", position(at.span.end)),
    ])
}

impl Server<'_> {
    fn send(&mut self, message: Json) {
        let body = message.to_string();
        let _ = write!(self.output, "Content-Length: {}\r\n\r\n{body}", body.len());
        let _ = self.output.flush();
    }

    fn reply(&mut self, request: &Json, result: Json) {
        self.send(Json::obj([
            ("jsonrpc", Json::str("2.0")),
            ("id", request.get("id").clone()),
            ("result", result),
        ]));
    }

    fn notify(&mut self, method: &str, params: Json) {
        self.send(Json::obj([
            ("jsonrpc", Json::str("2.0")),
            ("method", Json::str(method)),
            ("params", params),
        ]));
    }

    fn handle(&mut self, method: &str, message: &Json) {
        let params = message.get("params");
        let uri = params
            .get("textDocument")
            .get("uri")
            .as_str()
            .unwrap_or("")
            .to_string();
        let position = params.get("position");
        let place = (
            position.get("line").as_u32().map(|n| n as usize),
            position.get("character").as_u32(),
        );
        let text = self.documents.get(&uri).cloned();
        let here =
            |at: Location| Json::obj([("uri", Json::str(uri.clone())), ("range", range(at))]);

        match method {
            "initialize" => {
                if let Some(lint) =
                    lint_named(params.get("initializationOptions").get("dialect").as_str())
                {
                    self.lint = lint;
                }
                let capabilities = Json::obj([
                    // 1: the whole text is sent on every change
                    (
                        "textDocumentSync",
                        Json::obj([("openClose", Json::Bool(true)), ("change", Json::num(1))]),
                    ),
                    ("hoverProvider", Json::Bool(true)),
                    ("definitionProvider", Json::Bool(true)),
                    ("referencesProvider", Json::Bool(true)),
                    ("documentSymbolProvider", Json::Bool(true)),
                    ("completionProvider", Json::obj([])),
                    (
                        "semanticTokensProvider",
                        Json::obj([
                            (
                                "legend",
                                Json::obj([
                                    (
                                        "tokenTypes",
                                        Json::Arr(
                                            TokenType::NAMES
                                                .iter()
                                                .map(|name| Json::str(*name))
                                                .collect(),
                                        ),
                                    ),
                                    ("tokenModifiers", Json::Arr(vec![])),
                                ]),
                            ),
                            ("full", Json::Bool(true)),
                        ]),
                    ),
                    (
                        "executeCommandProvider",
                        Json::obj([("commands", Json::Arr(vec![Json::str(RENUMBER)]))]),
                    ),
                ]);
                let info = Json::obj([
                    ("name", Json::str("chuk-basic")),
                    ("version", Json::str(env!("CARGO_PKG_VERSION"))),
                ]);
                self.reply(
                    message,
                    Json::obj([("capabilities", capabilities), ("serverInfo", info)]),
                );
            }
            "shutdown" => {
                self.shutdown = true;
                self.reply(message, Json::Null);
            }
            "textDocument/didOpen" => {
                let text = params
                    .get("textDocument")
                    .get("text")
                    .as_str()
                    .unwrap_or("")
                    .to_string();
                self.documents.insert(uri.clone(), text);
                self.publish(&uri);
            }
            "textDocument/didChange" => {
                // full sync: the last change is the whole text
                let mut index = 0;
                while params.get("contentChanges").at(index + 1) != &Json::Null {
                    index += 1;
                }
                if let Some(text) = params.get("contentChanges").at(index).get("text").as_str() {
                    self.documents.insert(uri.clone(), text.to_string());
                    self.publish(&uri);
                }
            }
            "textDocument/didClose" => {
                self.documents.remove(&uri);
                let cleared =
                    Json::obj([("uri", Json::str(uri)), ("diagnostics", Json::Arr(vec![]))]);
                self.notify("textDocument/publishDiagnostics", cleared);
            }
            "workspace/didChangeConfiguration" => {
                let settings = params.get("settings");
                let dialect = settings
                    .get("chukBasic")
                    .get("dialect")
                    .as_str()
                    .or(settings.get("dialect").as_str());
                if let Some(lint) = lint_named(dialect) {
                    self.lint = lint;
                    let open: Vec<String> = self.documents.keys().cloned().collect();
                    for uri in open {
                        self.publish(&uri);
                    }
                }
            }
            "textDocument/hover" => {
                let found = match (&text, place) {
                    (Some(text), (Some(line), Some(column))) => analysis::hover(text, line, column),
                    _ => None,
                };
                let result = found.map_or(Json::Null, |(markdown, span)| {
                    let contents = Json::obj([
                        ("kind", Json::str("markdown")),
                        ("value", Json::str(markdown)),
                    ]);
                    let at = Location {
                        line: place.0.unwrap_or(0),
                        span,
                    };
                    Json::obj([("contents", contents), ("range", range(at))])
                });
                self.reply(message, result);
            }
            "textDocument/definition" => {
                let found = match (&text, place) {
                    (Some(text), (Some(line), Some(column))) => {
                        analysis::definition(text, line, column)
                    }
                    _ => None,
                };
                self.reply(message, found.map_or(Json::Null, here));
            }
            "textDocument/references" => {
                let declaration =
                    params.get("context").get("includeDeclaration") == &Json::Bool(true);
                let found = match (&text, place) {
                    (Some(text), (Some(line), Some(column))) => {
                        analysis::references(text, line, column, declaration)
                    }
                    _ => Vec::new(),
                };
                self.reply(message, Json::Arr(found.into_iter().map(here).collect()));
            }
            "textDocument/semanticTokens/full" => {
                // five numbers a token: lines and columns since the one before,
                // its length, its type's place in the legend, and no modifiers
                let mut data = Vec::new();
                let (mut last_line, mut last_start) = (0, 0);
                for (line, span, kind) in text.as_deref().map(analysis::tokens).unwrap_or_default()
                {
                    let start = if line == last_line {
                        span.start - last_start
                    } else {
                        span.start
                    };
                    let numbers = [
                        (line - last_line) as u32,
                        start,
                        span.end - span.start,
                        kind as u32,
                        0,
                    ];
                    data.extend(numbers.into_iter().map(Json::num));
                    (last_line, last_start) = (line, span.start);
                }
                self.reply(message, Json::obj([("data", Json::Arr(data))]));
            }
            "textDocument/completion" => {
                let found = match (&text, place) {
                    (Some(text), (Some(line), Some(column))) => {
                        analysis::complete(text, line, column, self.lint)
                    }
                    _ => Vec::new(),
                };
                let items = found
                    .into_iter()
                    .map(|item| {
                        // the protocol's numbers for Keyword, Function, Variable and Reference
                        let kind = match item.kind {
                            CompletionKind::Keyword => 14,
                            CompletionKind::Function => 3,
                            CompletionKind::Variable => 6,
                            CompletionKind::Line => 18,
                        };
                        Json::obj([
                            ("label", Json::str(item.label)),
                            ("kind", Json::num(kind)),
                            ("detail", Json::str(item.detail)),
                            ("documentation", Json::str(item.documentation)),
                        ])
                    })
                    .collect();
                self.reply(message, Json::Arr(items));
            }
            "textDocument/documentSymbol" => {
                let symbols = text.as_deref().map(analysis::symbols).unwrap_or_default();
                let list = symbols
                    .into_iter()
                    .map(|symbol| {
                        // 12 is Function and 6 is Method in the protocol's list of kinds
                        let kind = match symbol.kind {
                            SymbolKind::Function => 12,
                            SymbolKind::Subroutine => 6,
                        };
                        Json::obj([
                            ("name", Json::str(symbol.name)),
                            ("kind", Json::num(kind)),
                            ("range", range(symbol.at)),
                            ("selectionRange", range(symbol.at)),
                        ])
                    })
                    .collect();
                self.reply(message, Json::Arr(list));
            }
            "workspace/executeCommand" if params.get("command").as_str() == Some(RENUMBER) => {
                let uri = params
                    .get("arguments")
                    .at(0)
                    .as_str()
                    .unwrap_or("")
                    .to_string();
                let outcome = match self.documents.get(&uri) {
                    Some(text) => analysis::renumber(text, 10, 10),
                    None => Err("Not renumbered: the file is not open".to_string()),
                };
                match outcome {
                    Ok(edits) => {
                        let edits: Vec<Json> = edits
                            .into_iter()
                            .map(|edit| {
                                let at = Location {
                                    line: edit.line,
                                    span: edit.span,
                                };
                                Json::obj([("range", range(at)), ("newText", Json::str(edit.text))])
                            })
                            .collect();
                        let changes = Json::Obj(vec![(uri, Json::Arr(edits))]);
                        let params = Json::obj([
                            ("label", Json::str("Renumber")),
                            ("edit", Json::obj([("changes", changes)])),
                        ]);
                        let id = self.next_id;
                        self.next_id += 1;
                        self.send(Json::obj([
                            ("jsonrpc", Json::str("2.0")),
                            ("id", Json::str(format!("chuk-basic-{id}"))),
                            ("method", Json::str("workspace/applyEdit")),
                            ("params", params),
                        ]));
                    }
                    Err(why) => {
                        // 1 is Error in the protocol's message types
                        let params =
                            Json::obj([("type", Json::num(1)), ("message", Json::str(why))]);
                        self.notify("window/showMessage", params);
                    }
                }
                self.reply(message, Json::Null);
            }
            // a request we do not serve gets an answer, so the editor is not left waiting
            _ if message.get("id") != &Json::Null && !method.is_empty() => {
                let error = Json::obj([
                    ("code", Json::num(-32601)),
                    ("message", Json::str(format!("{method} is not supported"))),
                ]);
                self.send(Json::obj([
                    ("jsonrpc", Json::str("2.0")),
                    ("id", message.get("id").clone()),
                    ("error", error),
                ]));
            }
            // notifications we do not use, and the editor's answers to our requests
            _ => {}
        }
    }

    fn publish(&mut self, uri: &str) {
        let Some(text) = self.documents.get(uri) else {
            return;
        };
        let list = analysis::diagnostics(text, self.lint)
            .into_iter()
            .map(|d| {
                // 1 is Error and 2 is Warning
                let severity = match d.severity {
                    Severity::Error => 1,
                    Severity::Warning => 2,
                };
                Json::obj([
                    ("range", range(d.at)),
                    ("severity", Json::num(severity)),
                    ("source", Json::str(d.source)),
                    ("message", Json::str(d.message)),
                ])
            })
            .collect();
        let params = Json::obj([("uri", Json::str(uri)), ("diagnostics", Json::Arr(list))]);
        self.notify("textDocument/publishDiagnostics", params);
    }
}
