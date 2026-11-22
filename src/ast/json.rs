//! JSON rendering of the AST, for `--ast` and for tooling.

use super::{DataItem, Expr, LValue, Mat, MatExpr, MatTarget, PrintItem, Program, Stmt, UnOp};

/// A minimal JSON document; object keys keep their insertion order.
enum Json {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Arr(Vec<Json>),
    Obj(Vec<(&'static str, Json)>),
}

impl Json {
    fn str(s: &str) -> Json {
        Json::Str(s.to_string())
    }

    fn node(kind: &str, mut fields: Vec<(&'static str, Json)>) -> Json {
        fields.insert(0, ("type", Json::str(kind)));
        Json::Obj(fields)
    }

    fn write(&self, out: &mut String, indent: usize) {
        let pad = |out: &mut String, n: usize| out.push_str(&"  ".repeat(n));
        match self {
            Json::Null => out.push_str("null"),
            Json::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
            Json::Num(n) if n.is_finite() => out.push_str(&n.to_string()),
            Json::Num(_) => out.push_str("null"),
            Json::Str(s) => {
                out.push('"');
                for c in s.chars() {
                    match c {
                        '"' => out.push_str("\\\""),
                        '\\' => out.push_str("\\\\"),
                        '\n' => out.push_str("\\n"),
                        '\r' => out.push_str("\\r"),
                        '\t' => out.push_str("\\t"),
                        c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
                        c => out.push(c),
                    }
                }
                out.push('"');
            }
            Json::Arr(items) if items.is_empty() => out.push_str("[]"),
            Json::Arr(items) => {
                out.push_str("[\n");
                for (i, item) in items.iter().enumerate() {
                    pad(out, indent + 1);
                    item.write(out, indent + 1);
                    out.push_str(if i + 1 < items.len() { ",\n" } else { "\n" });
                }
                pad(out, indent);
                out.push(']');
            }
            Json::Obj(fields) => {
                out.push_str("{\n");
                for (i, (key, value)) in fields.iter().enumerate() {
                    pad(out, indent + 1);
                    out.push_str(&format!("\"{key}\": "));
                    value.write(out, indent + 1);
                    out.push_str(if i + 1 < fields.len() { ",\n" } else { "\n" });
                }
                pad(out, indent);
                out.push('}');
            }
        }
    }
}

fn list<T>(items: &[T], f: impl Fn(&T) -> Json) -> Json {
    Json::Arr(items.iter().map(f).collect())
}

fn option<T>(item: &Option<T>, f: impl Fn(&T) -> Json) -> Json {
    item.as_ref().map_or(Json::Null, f)
}

fn line_number(n: &u32) -> Json {
    Json::Num(*n as f64)
}

impl Program {
    /// Pretty-printed JSON for the whole program.
    pub fn to_json(&self) -> String {
        let lines = list(&self.lines, |line| {
            Json::Obj(vec![
                ("line_number", option(&line.number, line_number)),
                ("statements", list(&line.stmts, stmt)),
            ])
        });
        let mut out = String::new();
        Json::node("Program", vec![("lines", lines)]).write(&mut out, 0);
        out
    }
}

fn lvalue(target: &LValue) -> Json {
    let mut fields = vec![("name", Json::str(&target.name))];
    if !target.indices.is_empty() {
        fields.push(("indices", list(&target.indices, expr)));
    }
    Json::node("Variable", fields)
}

fn print_item(item: &PrintItem) -> Json {
    match item {
        PrintItem::Expr(e) => expr(e),
        PrintItem::Tab(e) => Json::node("Tab", vec![("column", expr(e))]),
        PrintItem::Spc(e) => Json::node("Spc", vec![("count", expr(e))]),
        PrintItem::Comma => Json::node("Separator", vec![("value", Json::str(","))]),
        PrintItem::Semi => Json::node("Separator", vec![("value", Json::str(";"))]),
    }
}

fn data_item(item: &DataItem) -> Json {
    Json::Obj(vec![
        ("value", Json::str(&item.text)),
        ("quoted", Json::Bool(item.quoted)),
    ])
}

fn stmt(s: &Stmt) -> Json {
    match s {
        Stmt::Let { target, expr: e } => Json::node(
            "LetStatement",
            vec![("variable", lvalue(target)), ("expression", expr(e))],
        ),
        Stmt::Print(items) => {
            Json::node("PrintStatement", vec![("items", list(items, print_item))])
        }
        Stmt::Input {
            prompt,
            question_mark,
            targets,
        } => Json::node(
            "InputStatement",
            vec![
                ("prompt", option(prompt, |p| Json::str(p))),
                ("questionMark", Json::Bool(*question_mark)),
                ("variables", list(targets, lvalue)),
            ],
        ),
        Stmt::If {
            cond,
            then,
            otherwise,
        } => Json::node(
            "IfStatement",
            vec![
                ("condition", expr(cond)),
                ("thenPart", list(then, stmt)),
                ("elsePart", option(otherwise, |branch| list(branch, stmt))),
            ],
        ),
        Stmt::For {
            var,
            start,
            end,
            step,
        } => Json::node(
            "ForStatement",
            vec![
                ("variable", Json::str(var)),
                ("start", expr(start)),
                ("end", expr(end)),
                ("step", option(step, expr)),
            ],
        ),
        Stmt::Next(vars) => Json::node(
            "NextStatement",
            vec![("variables", list(vars, |v| Json::str(v)))],
        ),
        Stmt::Goto(line) => Json::node("GotoStatement", vec![("target", line_number(line))]),
        Stmt::Gosub(line) => Json::node("GosubStatement", vec![("target", line_number(line))]),
        Stmt::Return => Json::node("ReturnStatement", vec![]),
        Stmt::On {
            expr: e,
            targets,
            gosub,
        } => Json::node(
            "OnStatement",
            vec![
                ("expression", expr(e)),
                ("targets", list(targets, line_number)),
                ("isGosub", Json::Bool(*gosub)),
            ],
        ),
        Stmt::Def { name, params, body } => Json::node(
            "DefStatement",
            vec![
                ("name", Json::str(name)),
                ("parameters", list(params, |p| Json::str(p))),
                ("body", option(body, expr)),
            ],
        ),
        Stmt::FnEnd => Json::node("FnEndStatement", vec![]),
        Stmt::Dim(arrays) => Json::node(
            "DimStatement",
            vec![(
                "arrays",
                list(arrays, |(name, dims)| {
                    Json::Obj(vec![
                        ("name", Json::str(name)),
                        ("dimensions", list(dims, expr)),
                    ])
                }),
            )],
        ),
        Stmt::Read(targets) => {
            Json::node("ReadStatement", vec![("variables", list(targets, lvalue))])
        }
        Stmt::Data(items) => Json::node("DataStatement", vec![("items", list(items, data_item))]),
        Stmt::Restore(line) => Json::node(
            "RestoreStatement",
            vec![("target", option(line, line_number))],
        ),
        Stmt::Randomize(seed) => {
            Json::node("RandomizeStatement", vec![("seed", option(seed, expr))])
        }
        Stmt::PrintUsing {
            format,
            values,
            newline,
        } => Json::node(
            "PrintUsingStatement",
            vec![
                ("format", expr(format)),
                ("values", list(values, expr)),
                ("newline", Json::Bool(*newline)),
            ],
        ),
        Stmt::LineInput { prompt, target } => Json::node(
            "LineInputStatement",
            vec![
                ("prompt", option(prompt, |p| Json::str(p))),
                ("variable", lvalue(target)),
            ],
        ),
        Stmt::While(cond) => Json::node("WhileStatement", vec![("condition", expr(cond))]),
        Stmt::Wend => Json::node("WendStatement", vec![]),
        Stmt::Mat(m) => mat(m),
        Stmt::Rem(text) => Json::node("RemStatement", vec![("comment", Json::str(text))]),
        Stmt::End => Json::node("EndStatement", vec![]),
        Stmt::Stop => Json::node("StopStatement", vec![]),
    }
}

fn mat_target(target: &MatTarget) -> Json {
    Json::Obj(vec![
        ("name", Json::str(&target.name)),
        ("dimensions", list(&target.dims, expr)),
    ])
}

fn mat(m: &Mat) -> Json {
    match m {
        Mat::Read(targets) => Json::node(
            "MatReadStatement",
            vec![("matrices", list(targets, mat_target))],
        ),
        Mat::Input(targets) => Json::node(
            "MatInputStatement",
            vec![("matrices", list(targets, mat_target))],
        ),
        Mat::Print(items) => Json::node(
            "MatPrintStatement",
            vec![(
                "matrices",
                list(items, |(name, packed)| {
                    Json::Obj(vec![
                        ("name", Json::str(name)),
                        ("packed", Json::Bool(*packed)),
                    ])
                }),
            )],
        ),
        Mat::Assign { target, value } => {
            let one =
                |op: &str, a: &str| vec![("operation", Json::str(op)), ("matrix", Json::str(a))];
            let two = |op: &str, a: &str, b: &str| {
                vec![
                    ("operation", Json::str(op)),
                    ("left", Json::str(a)),
                    ("right", Json::str(b)),
                ]
            };
            let fill = |op: &str, dims: &[Expr]| {
                vec![
                    ("operation", Json::str(op)),
                    ("dimensions", list(dims, expr)),
                ]
            };
            let mut fields = match value {
                MatExpr::Copy(a) => one("copy", a),
                MatExpr::Add(a, b) => two("+", a, b),
                MatExpr::Sub(a, b) => two("-", a, b),
                MatExpr::Mul(a, b) => two("*", a, b),
                MatExpr::Scale(k, a) => {
                    vec![
                        ("operation", Json::str("scale")),
                        ("factor", expr(k)),
                        ("matrix", Json::str(a)),
                    ]
                }
                MatExpr::Zer(dims) => fill("ZER", dims),
                MatExpr::Con(dims) => fill("CON", dims),
                MatExpr::Idn(dims) => fill("IDN", dims),
                MatExpr::Trn(a) => one("TRN", a),
                MatExpr::Inv(a) => one("INV", a),
            };
            fields.insert(0, ("target", Json::str(target)));
            Json::node("MatAssignStatement", fields)
        }
    }
}

fn expr(e: &Expr) -> Json {
    match e {
        Expr::Num(n) => Json::node("LiteralExpression", vec![("value", Json::Num(*n))]),
        Expr::Str(s) => Json::node("LiteralExpression", vec![("value", Json::str(s))]),
        Expr::Var(name) => Json::node("Variable", vec![("name", Json::str(name))]),
        Expr::Array(name, indices) => Json::node(
            "Variable",
            vec![("name", Json::str(name)), ("indices", list(indices, expr))],
        ),
        Expr::Call(builtin, args) => Json::node(
            "BuiltinCall",
            vec![
                ("name", Json::str(builtin.name())),
                ("arguments", list(args, expr)),
            ],
        ),
        Expr::FnCall(name, args) => Json::node(
            "FnExpression",
            vec![("name", Json::str(name)), ("arguments", list(args, expr))],
        ),
        Expr::Unary(op, operand) => {
            let symbol = match op {
                UnOp::Neg | UnOp::NegPower => "-",
                UnOp::Not => "NOT",
            };
            Json::node(
                "UnaryExpression",
                vec![("operator", Json::str(symbol)), ("operand", expr(operand))],
            )
        }
        Expr::Binary(op, left, right) => Json::node(
            "BinaryExpression",
            vec![
                ("operator", Json::str(op.symbol())),
                ("left", expr(left)),
                ("right", expr(right)),
            ],
        ),
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::parse;

    #[test]
    fn renders_a_program() {
        let json = parse("10 LET X = 5\n20 PRINT \"A\\B\"; X + 1")
            .unwrap()
            .to_json();
        assert!(json.starts_with("{\n  \"type\": \"Program\","));
        assert!(json.contains("\"line_number\": 10"));
        assert!(json.contains("\"type\": \"LetStatement\""));
        assert!(json.contains("\"value\": 5"));
        assert!(json.contains("\"value\": \"A\\\\B\""));
        assert!(json.contains("\"operator\": \"+\""));
    }
}
