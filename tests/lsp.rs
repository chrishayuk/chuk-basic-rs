//! Talks to `chuk-basic --lsp` as an editor would: framed JSON-RPC over
//! the process's standard input and output.

#![cfg(not(target_os = "wasi"))]

use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use chuk_basic::lsp::Json;

struct Editor {
    child: Child,
    to: ChildStdin,
    from: BufReader<ChildStdout>,
    next_id: u32,
}

const URI: &str = "file:///tmp/program.bas";

impl Editor {
    fn start(dialect: &str) -> Editor {
        let mut child = Command::new(env!("CARGO_BIN_EXE_chuk-basic"))
            .arg("--lsp")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("the server starts");
        let to = child.stdin.take().unwrap();
        let from = BufReader::new(child.stdout.take().unwrap());
        let mut editor = Editor {
            child,
            to,
            from,
            next_id: 1,
        };
        let options = format!(
            r#"{{"processId":null,"rootUri":null,"capabilities":{{}},"initializationOptions":{{"dialect":"{dialect}"}}}}"#
        );
        let reply = editor.request("initialize", &options);
        let capabilities = reply.get("result").get("capabilities");
        assert_eq!(capabilities.get("hoverProvider"), &Json::Bool(true));
        assert_eq!(
            capabilities.get("textDocumentSync").get("change").as_u32(),
            Some(1)
        );
        assert_eq!(
            capabilities
                .get("executeCommandProvider")
                .get("commands")
                .at(0)
                .as_str(),
            Some("chukBasic.server.renumber")
        );
        editor.notify("initialized", "{}");
        editor
    }

    fn send(&mut self, body: &str) {
        write!(self.to, "Content-Length: {}\r\n\r\n{body}", body.len()).unwrap();
        self.to.flush().unwrap();
    }

    fn read(&mut self) -> Json {
        let mut length = 0;
        loop {
            let mut header = String::new();
            assert!(
                self.from.read_line(&mut header).unwrap() > 0,
                "the server went away"
            );
            if header.trim().is_empty() {
                break;
            }
            if let Some(value) = header.strip_prefix("Content-Length:") {
                length = value.trim().parse().unwrap();
            }
        }
        let mut body = vec![0; length];
        self.from.read_exact(&mut body).unwrap();
        Json::parse(std::str::from_utf8(&body).unwrap()).expect("the server sends JSON")
    }

    fn notify(&mut self, method: &str, params: &str) {
        self.send(&format!(
            r#"{{"jsonrpc":"2.0","method":"{method}","params":{params}}}"#
        ));
    }

    /// Send a request and return its answer, skipping anything else the
    /// server sends first.
    fn request(&mut self, method: &str, params: &str) -> Json {
        let id = self.next_id;
        self.next_id += 1;
        self.send(&format!(
            r#"{{"jsonrpc":"2.0","id":{id},"method":"{method}","params":{params}}}"#
        ));
        loop {
            let message = self.read();
            if message.get("id").as_u32() == Some(id) && message.get("method") == &Json::Null {
                return message;
            }
        }
    }

    /// Open or change the file, and return the diagnostics that follow as
    /// (line, column, severity, message).
    fn edit(&mut self, method: &str, text: &str) -> Vec<(u32, u32, u32, String)> {
        let text = Json::str(text);
        let params = if method == "textDocument/didOpen" {
            format!(
                r#"{{"textDocument":{{"uri":"{URI}","languageId":"basic","version":1,"text":{text}}}}}"#
            )
        } else {
            format!(
                r#"{{"textDocument":{{"uri":"{URI}","version":2}},"contentChanges":[{{"text":{text}}}]}}"#
            )
        };
        self.notify(method, &params);
        self.diagnostics()
    }

    fn diagnostics(&mut self) -> Vec<(u32, u32, u32, String)> {
        let message = self.read();
        assert_eq!(
            message.get("method").as_str(),
            Some("textDocument/publishDiagnostics")
        );
        let params = message.get("params");
        assert_eq!(params.get("uri").as_str(), Some(URI));
        let Json::Arr(list) = params.get("diagnostics") else {
            panic!("diagnostics is a list")
        };
        list.iter()
            .map(|d| {
                let start = d.get("range").get("start");
                (
                    start.get("line").as_u32().unwrap(),
                    start.get("character").as_u32().unwrap(),
                    d.get("severity").as_u32().unwrap(),
                    d.get("message").as_str().unwrap().to_string(),
                )
            })
            .collect()
    }

    fn at(&mut self, method: &str, line: u32, character: u32) -> Json {
        let params = format!(
            r#"{{"textDocument":{{"uri":"{URI}"}},"position":{{"line":{line},"character":{character}}},"context":{{"includeDeclaration":true}}}}"#
        );
        self.request(method, &params).get("result").clone()
    }

    fn stop(mut self) -> i32 {
        assert_eq!(self.request("shutdown", "null").get("result"), &Json::Null);
        self.notify("exit", "null");
        self.child.wait().unwrap().code().unwrap()
    }
}

const PROGRAM: &str = "10 LET X = 3\n20 GOSUB 100\n30 IF X > 4 THEN 10\n40 END\n100 LET X = SQR(X↑2 + 1)\n110 RETURN\n";

#[test]
fn diagnostics_follow_the_text() {
    let mut editor = Editor::start("later");
    assert_eq!(editor.edit("textDocument/didOpen", PROGRAM), []);
    let broken = PROGRAM
        .replace("GOSUB 100", "GOSUB 105")
        .replace("LET X = 3", "LET X = (3");
    assert_eq!(
        editor.edit("textDocument/didChange", &broken),
        [
            (0, 3, 1, "Expected ')', found end of line".to_string()),
            (1, 9, 1, "There is no line 105".to_string()),
        ]
    );
    assert_eq!(editor.edit("textDocument/didChange", PROGRAM), []);
    assert_eq!(editor.stop(), 0);
}

#[test]
fn hover_definition_references_and_outline() {
    let mut editor = Editor::start("later");
    editor.edit("textDocument/didOpen", PROGRAM);

    let hover = editor.at("textDocument/hover", 4, 13);
    assert!(hover
        .get("contents")
        .get("value")
        .as_str()
        .unwrap()
        .contains("absolute value"));
    assert_eq!(
        hover.get("range").get("start").get("character").as_u32(),
        Some(12)
    );
    assert_eq!(editor.at("textDocument/hover", 0, 7), Json::Null);

    // GOSUB 100 leads to line 100, the fifth line of the file
    let target = editor.at("textDocument/definition", 1, 10);
    assert_eq!(target.get("uri").as_str(), Some(URI));
    assert_eq!(
        target.get("range").get("start").get("line").as_u32(),
        Some(4)
    );

    // line 10 is named by the THEN on the third line
    let Json::Arr(uses) = editor.at("textDocument/references", 0, 1) else {
        panic!("a list")
    };
    let lines: Vec<_> = uses
        .iter()
        .map(|u| u.get("range").get("start").get("line").as_u32())
        .collect();
    assert_eq!(lines, [Some(0), Some(2)]);

    let symbols = editor.request(
        "textDocument/documentSymbol",
        &format!(r#"{{"textDocument":{{"uri":"{URI}"}}}}"#),
    );
    assert_eq!(
        symbols.get("result").at(0).get("name").as_str(),
        Some("100 (subroutine)")
    );

    // colours: the first token is the line number 10, a label, the eighth kind in the legend
    let tokens = editor.request(
        "textDocument/semanticTokens/full",
        &format!(r#"{{"textDocument":{{"uri":"{URI}"}}}}"#),
    );
    let Json::Arr(data) = tokens.get("result").get("data") else {
        panic!("token data")
    };
    let first: Vec<_> = data.iter().take(10).map(|n| n.as_u32().unwrap()).collect();
    assert_eq!(
        first,
        [0, 0, 2, 7, 0, 0, 3, 3, 5, 0],
        "10 is a label, LET a keyword"
    );
    assert_eq!(data.len() % 5, 0);

    // after GOSUB, the lines of the program
    let Json::Arr(items) = editor.at("textDocument/completion", 1, 9) else {
        panic!("completion items")
    };
    let labels: Vec<_> = items
        .iter()
        .map(|i| i.get("label").as_str().unwrap())
        .collect();
    assert_eq!(labels, ["10", "20", "30", "40", "100", "110"]);
    assert_eq!(
        items[4].get("detail").as_str(),
        Some("LET X = SQR(X↑2 + 1)")
    );

    // a request it does not serve is answered, not ignored
    let reply = editor.request("textDocument/formatting", "{}");
    assert_eq!(reply.get("error").get("code"), &Json::Num(-32601.0));
    assert_eq!(editor.stop(), 0);
}

#[test]
fn the_1964_setting_can_be_switched_while_running() {
    let source = "10 INPUT N\n20 PRINT N\n30 END\n";
    let mut editor = Editor::start("dartmouth");
    assert_eq!(
        editor.edit("textDocument/didOpen", source),
        [(
            0,
            3,
            2,
            "INPUT is not one of the fifteen statements of 1964 BASIC".to_string()
        )]
    );
    editor.notify(
        "workspace/didChangeConfiguration",
        r#"{"settings":{"chukBasic":{"dialect":"later"}}}"#,
    );
    assert_eq!(editor.diagnostics(), []);
    assert_eq!(editor.stop(), 0);
}

#[test]
fn renumber_is_sent_back_as_an_edit() {
    let mut editor = Editor::start("later");
    editor.edit("textDocument/didOpen", "5 GOTO 7\n7 END\n");
    let params = format!(r#"{{"command":"chukBasic.server.renumber","arguments":["{URI}"]}}"#);
    let id = editor.next_id;
    editor.next_id += 1;
    editor.send(&format!(
        r#"{{"jsonrpc":"2.0","id":{id},"method":"workspace/executeCommand","params":{params}}}"#
    ));

    let apply = editor.read();
    assert_eq!(apply.get("method").as_str(), Some("workspace/applyEdit"));
    let Json::Arr(edits) = apply.get("params").get("edit").get("changes").get(URI) else {
        panic!("edits for the file")
    };
    let texts: Vec<_> = edits
        .iter()
        .map(|e| e.get("newText").as_str().unwrap())
        .collect();
    assert_eq!(texts, ["10", "20", "20"]);
    // the editor answers the server's request, and the server ours
    editor.send(&format!(
        r#"{{"jsonrpc":"2.0","id":{},"result":{{"applied":true}}}}"#,
        apply.get("id")
    ));
    assert_eq!(editor.read().get("id").as_u32(), Some(id));

    // a program it cannot renumber safely is refused with a message
    editor.edit("textDocument/didChange", "10 PRINT (\n");
    editor.send(&format!(
        r#"{{"jsonrpc":"2.0","id":99,"method":"workspace/executeCommand","params":{params}}}"#
    ));
    let shown = editor.read();
    assert_eq!(shown.get("method").as_str(), Some("window/showMessage"));
    assert!(shown
        .get("params")
        .get("message")
        .as_str()
        .unwrap()
        .starts_with("Not renumbered"));
    assert_eq!(editor.read().get("id").as_u32(), Some(99));
    assert_eq!(editor.stop(), 0);
}

#[test]
fn leaving_without_shutdown_is_an_error_exit() {
    let mut editor = Editor::start("later");
    editor.notify("exit", "null");
    assert_eq!(editor.child.wait().unwrap().code(), Some(1));
}
