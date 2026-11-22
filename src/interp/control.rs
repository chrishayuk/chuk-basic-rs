//! Control flow: GOTO / GOSUB targets and FOR / NEXT loops.

use super::compile::Op;
use super::{Flow, ForFrame, GosubFrame, Interpreter, MAX_GOSUB_DEPTH};
use crate::ast::{Expr, Stmt};
use crate::error::Result;
use crate::rt::Value;

impl Interpreter<'_> {
    pub(super) fn target(&self, line: u32) -> Result<usize> {
        self.line_map
            .get(&line)
            .copied()
            .ok_or_else(|| self.err(format!("Undefined line {line}")))
    }

    pub(super) fn for_base(&self) -> usize {
        self.gosub_stack
            .last()
            .map_or(0, |f| f.for_base)
            .max(self.fn_for_base)
    }

    pub(super) fn gosub(&mut self, line: u32, ret: usize) -> Result<Flow> {
        let target = self.target(line)?;
        if self.gosub_stack.len() >= MAX_GOSUB_DEPTH {
            return Err(self.err("Out of memory: GOSUB nested too deeply"));
        }
        self.gosub_stack.push(GosubFrame {
            ret,
            for_base: self.for_stack.len(),
        });
        Ok(Flow::Jump(target))
    }

    pub(super) fn for_loop(
        &mut self,
        var: &str,
        start: &Expr,
        end: &Expr,
        step: Option<&Expr>,
        pc: usize,
    ) -> Result<Flow> {
        let start = self.eval_num(start)?;
        let limit = self.eval_num(end)?;
        let step = match step {
            Some(expr) => self.eval_num(expr)?,
            None => 1.0,
        };
        self.vars.insert(var.to_string(), Value::Num(start));

        // re-entering a loop that is still open replaces it
        let base = self.for_base();
        if let Some(at) = self.for_stack[base..].iter().rposition(|f| f.var == var) {
            self.for_stack.truncate(base + at);
        }

        if (step > 0.0 && start > limit) || (step < 0.0 && start < limit) {
            // empty range: skip to just past the matching NEXT
            let mut depth = 0;
            for (i, op) in self.ops.iter().enumerate().skip(pc + 1) {
                match op {
                    Op::Stmt(Stmt::For { .. }) => depth += 1,
                    Op::Next(_) if depth == 0 => return Ok(Flow::Jump(i + 1)),
                    Op::Next(_) => depth -= 1,
                    _ => {}
                }
            }
            return Err(self.err("FOR without NEXT"));
        }
        self.for_stack.push(ForFrame {
            var: var.to_string(),
            limit,
            step,
            body: pc + 1,
        });
        Ok(Flow::Next)
    }

    /// Execute NEXT; returns the next pc.
    pub(super) fn next(&mut self, var: Option<&str>, pc: usize) -> Result<usize> {
        let base = self.for_base();
        // NEXT with a variable closes any inner loops that were jumped out of
        while self.for_stack.len() > base {
            let top = self.for_stack.last().expect("non-empty");
            if var.is_none() || var == Some(top.var.as_str()) {
                break;
            }
            self.for_stack.pop();
        }
        if self.for_stack.len() <= base {
            return Err(self.err("NEXT without FOR"));
        }
        let frame = self.for_stack.last().expect("non-empty");
        let (limit, step, body) = (frame.limit, frame.step, frame.body);
        let name = frame.var.clone();
        let value = match self.vars.get(&name) {
            Some(Value::Num(n)) => n + step,
            _ => step,
        };
        self.vars.insert(name, Value::Num(value));
        if (step >= 0.0 && value <= limit) || (step < 0.0 && value >= limit) {
            Ok(body)
        } else {
            self.for_stack.pop();
            Ok(pc + 1)
        }
    }
}
