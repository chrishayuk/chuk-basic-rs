//! Lowering: flattens the AST into a list of ops with resolved jump
//! targets, and collects DATA items and function definitions.

use std::collections::HashMap;
use std::rc::Rc;

use crate::ast::{Expr, Program, Stmt};
use crate::error::{Error, Result};
use crate::rt::DataPool;

pub(crate) enum Op {
    Stmt(Stmt),
    JumpIfFalse(Expr, usize),
    Jump(usize),
    /// One loop variable of a NEXT (`NEXT J,I` becomes two ops).
    Next(Option<String>),
    FnEnd,
}

pub(crate) enum FnBody {
    Expr(Expr),
    /// Index of the first op of a multi-line function.
    Block(usize),
}

pub(crate) struct Func {
    pub(crate) params: Vec<String>,
    pub(crate) body: FnBody,
}

#[derive(Default)]
pub(crate) struct Lowered {
    pub(crate) ops: Vec<Op>,
    /// BASIC line number of each op, for error messages.
    pub(crate) op_line: Vec<Option<u32>>,
    /// Line number -> index of the first op of that line.
    pub(crate) line_map: HashMap<u32, usize>,
    pub(crate) data: DataPool,
    pub(crate) funcs: HashMap<String, Rc<Func>>,
}

struct Lowering {
    out: Lowered,
    line: Option<u32>,
    /// Open multi-line DEFs: index of the jump that skips the body.
    open_defs: Vec<usize>,
    /// Open WHILE loops: index of the jump that leaves the loop.
    open_whiles: Vec<usize>,
}

impl Lowering {
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
            Stmt::Data(items) => items.into_iter().for_each(|item| self.out.data.push(item)),
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
            Stmt::While(cond) => {
                let exit = self.emit(Op::JumpIfFalse(cond, 0));
                self.open_whiles.push(exit);
            }
            Stmt::Wend => {
                let Some(exit) = self.open_whiles.pop() else {
                    return Err(Error::syntax("WEND without WHILE", self.line, None));
                };
                self.emit(Op::Jump(exit)); // back to test the condition again
                self.patch(exit);
            }
            Stmt::Rem(_) => {}
            other => {
                self.emit(Op::Stmt(other));
            }
        }
        Ok(())
    }
}

pub(crate) fn lower(program: Program) -> Result<Lowered> {
    let mut c = Lowering {
        out: Lowered::default(),
        line: None,
        open_defs: Vec::new(),
        open_whiles: Vec::new(),
    };
    for line in program.lines {
        c.line = line.number;
        if let Some(n) = line.number {
            c.out.line_map.insert(n, c.out.ops.len());
            c.out.data.mark_line(n);
        }
        for stmt in line.stmts {
            c.stmt(stmt)?;
        }
    }
    if !c.open_whiles.is_empty() {
        return Err(Error::syntax("WHILE without WEND", None, None));
    }
    if !c.open_defs.is_empty() {
        return Err(Error::syntax("DEF without FNEND", None, None));
    }
    Ok(c.out)
}
