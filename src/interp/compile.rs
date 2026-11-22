//! Lowering: flattens the AST into a list of ops with resolved jump
//! targets, and collects DATA items and function definitions.

use std::collections::HashMap;
use std::rc::Rc;

use crate::ast::{DataItem, Expr, Program, Stmt};
use crate::error::{Error, Result};

pub(super) enum Op {
    Stmt(Stmt),
    JumpIfFalse(Expr, usize),
    Jump(usize),
    /// One loop variable of a NEXT (`NEXT J,I` becomes two ops).
    Next(Option<String>),
    FnEnd,
}

pub(super) enum FnBody {
    Expr(Expr),
    /// Index of the first op of a multi-line function.
    Block(usize),
}

pub(super) struct Func {
    pub(super) params: Vec<String>,
    pub(super) body: FnBody,
}

#[derive(Default)]
pub(super) struct Compiled {
    pub(super) ops: Vec<Op>,
    /// BASIC line number of each op, for error messages.
    pub(super) op_line: Vec<Option<u32>>,
    /// Line number -> index of the first op of that line.
    pub(super) line_map: HashMap<u32, usize>,
    pub(super) data: Vec<DataItem>,
    /// (line number, index of the first DATA item on or after that line)
    pub(super) data_lines: Vec<(u32, usize)>,
    pub(super) funcs: HashMap<String, Rc<Func>>,
}

struct Compiler {
    out: Compiled,
    line: Option<u32>,
    /// Open multi-line DEFs: index of the jump that skips the body.
    open_defs: Vec<usize>,
}

impl Compiler {
    fn emit(&mut self, op: Op) -> usize {
        self.out.ops.push(op);
        self.out.op_line.push(self.line);
        self.out.ops.len() - 1
    }

    fn patch(&mut self, at: usize) {
        let target = self.out.ops.len();
        match &mut self.out.ops[at] {
            Op::Jump(t) | Op::JumpIfFalse(_, t) => *t = target,
            _ => unreachable!("patching a non-jump"),
        }
    }

    fn define(&mut self, name: String, func: Func) -> Result<()> {
        if self.out.funcs.insert(name.clone(), Rc::new(func)).is_some() {
            return Err(Error::syntax(
                format!("Function {name} is defined twice"),
                self.line,
                None,
            ));
        }
        Ok(())
    }

    fn stmt(&mut self, stmt: Stmt) -> Result<()> {
        match stmt {
            Stmt::If {
                cond,
                then,
                otherwise,
            } => {
                let skip_then = self.emit(Op::JumpIfFalse(cond, 0));
                for s in then {
                    self.stmt(s)?;
                }
                if let Some(otherwise) = otherwise {
                    let skip_else = self.emit(Op::Jump(0));
                    self.patch(skip_then);
                    for s in otherwise {
                        self.stmt(s)?;
                    }
                    self.patch(skip_else);
                } else {
                    self.patch(skip_then);
                }
            }
            Stmt::Next(vars) if vars.is_empty() => {
                self.emit(Op::Next(None));
            }
            Stmt::Next(vars) => {
                for var in vars {
                    self.emit(Op::Next(Some(var)));
                }
            }
            Stmt::Data(items) => self.out.data.extend(items),
            Stmt::Def {
                name,
                params,
                body: Some(expr),
            } => {
                self.define(
                    name,
                    Func {
                        params,
                        body: FnBody::Expr(expr),
                    },
                )?;
            }
            Stmt::Def {
                name,
                params,
                body: None,
            } => {
                let skip = self.emit(Op::Jump(0));
                self.open_defs.push(skip);
                self.define(
                    name,
                    Func {
                        params,
                        body: FnBody::Block(skip + 1),
                    },
                )?;
            }
            Stmt::FnEnd => {
                let Some(skip) = self.open_defs.pop() else {
                    return Err(Error::syntax("FNEND without DEF", self.line, None));
                };
                self.emit(Op::FnEnd);
                self.patch(skip);
            }
            Stmt::Rem(_) => {}
            other => {
                self.emit(Op::Stmt(other));
            }
        }
        Ok(())
    }
}

pub(super) fn compile(program: Program) -> Result<Compiled> {
    let mut c = Compiler {
        out: Compiled::default(),
        line: None,
        open_defs: Vec::new(),
    };
    for line in program.lines {
        c.line = line.number;
        if let Some(n) = line.number {
            c.out.line_map.insert(n, c.out.ops.len());
            c.out.data_lines.push((n, c.out.data.len()));
        }
        for stmt in line.stmts {
            c.stmt(stmt)?;
        }
    }
    if !c.open_defs.is_empty() {
        return Err(Error::syntax("DEF without FNEND", None, None));
    }
    Ok(c.out)
}
