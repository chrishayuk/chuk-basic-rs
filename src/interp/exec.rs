//! The fetch-execute loop and statement dispatch.

use std::rc::Rc;

use super::lower::Op;
use super::{Exit, Flow, Interpreter};
use crate::ast::Stmt;
use crate::error::{AtLine, Result};

impl Interpreter<'_> {
    pub(super) fn exec(&mut self, mut pc: usize, in_fn: bool) -> Result<Exit> {
        let ops = Rc::clone(&self.ops);
        loop {
            let Some(op) = ops.get(pc) else {
                return Ok(Exit::Halt);
            };
            self.line = self.op_line[pc];
            self.steps += 1;
            if self.step_limit.is_some_and(|limit| self.steps > limit) {
                return Err(self.err("Step limit exceeded"));
            }
            match op {
                Op::Jump(target) => pc = *target,
                Op::JumpIfFalse(cond, target) => {
                    pc = if self.eval_num(cond)? != 0.0 {
                        pc + 1
                    } else {
                        *target
                    };
                }
                Op::Next(var) => pc = self.next(var.as_deref(), pc)?,
                Op::FnEnd if in_fn => return Ok(Exit::FnReturn),
                Op::FnEnd => return Err(self.err("FNEND without DEF")),
                Op::Stmt(stmt) => match self.stmt(stmt, pc)? {
                    Flow::Next => pc += 1,
                    Flow::Jump(target) => pc = target,
                    Flow::Halt => return Ok(Exit::Halt),
                },
            }
            if self.halted {
                return Ok(Exit::Halt);
            }
        }
    }

    pub(super) fn stmt(&mut self, stmt: &Stmt, pc: usize) -> Result<Flow> {
        match stmt {
            Stmt::Let { target, expr } => {
                let value = self.eval(expr)?;
                self.assign(target, value)?;
            }
            Stmt::Print(items) => self.print(items)?,
            Stmt::Input {
                prompt,
                question_mark,
                targets,
            } => {
                self.input(prompt.as_deref(), *question_mark, targets)?;
            }
            Stmt::For {
                var,
                start,
                end,
                step,
            } => return self.for_loop(var, start, end, step.as_ref(), pc),
            Stmt::Goto(line) => return Ok(Flow::Jump(self.target(*line)?)),
            Stmt::Gosub(line) => return self.gosub(*line, pc + 1),
            Stmt::Return => {
                if self.gosub_stack.len() <= self.fn_gosub_base {
                    return Err(self.err("RETURN without GOSUB"));
                }
                let frame = self.gosub_stack.pop().expect("checked above");
                self.for_stack.truncate(frame.for_base);
                return Ok(Flow::Jump(frame.ret));
            }
            Stmt::On {
                expr,
                targets,
                gosub,
            } => {
                let n = self.eval_num(expr)?.round();
                if n >= 1.0 && n <= targets.len() as f64 {
                    let line = targets[n as usize - 1];
                    return if *gosub {
                        self.gosub(line, pc + 1)
                    } else {
                        Ok(Flow::Jump(self.target(line)?))
                    };
                }
            }
            Stmt::Dim(arrays) => {
                for (name, dims) in arrays {
                    let bounds = self.eval_indices(dims)?;
                    self.arrays.dim(name, &bounds).at(self.line)?;
                }
            }
            Stmt::Read(targets) => {
                for target in targets {
                    let value = self.data.read(target.name.ends_with('$')).at(self.line)?;
                    self.assign(target, value)?;
                }
            }
            Stmt::Restore(line) => self.data.restore(*line),
            Stmt::Randomize(seed) => {
                let seed = match seed {
                    Some(expr) => Some(self.eval_num(expr)?),
                    None => None,
                };
                self.rnd.randomize(seed);
            }
            Stmt::PrintUsing {
                format,
                values,
                newline,
            } => self.print_using(format, values, *newline)?,
            Stmt::LineInput { prompt, target } => self.line_input(prompt.as_deref(), target)?,
            Stmt::Mat(statement) => self.mat(statement)?,
            Stmt::End | Stmt::Stop => return Ok(Flow::Halt),
            // removed or rewritten by `lower`
            Stmt::If { .. }
            | Stmt::Next(_)
            | Stmt::Def { .. }
            | Stmt::FnEnd
            | Stmt::While(_)
            | Stmt::Wend
            | Stmt::Data(_)
            | Stmt::Rem(_) => {}
        }
        Ok(Flow::Next)
    }
}
