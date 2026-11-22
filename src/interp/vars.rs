//! Variables and arrays.

use super::Interpreter;
use crate::ast::{Expr, LValue};
use crate::error::{AtLine, Result};
use crate::rt::{self, Value};

impl Interpreter<'_> {
    /// Check that `value` can be stored in a variable called `name`.
    pub(super) fn coerce(&self, name: &str, value: Value) -> Result<Value> {
        rt::coerce(name, value).at(self.line)
    }

    pub(super) fn assign(&mut self, target: &LValue, value: Value) -> Result<()> {
        let value = self.coerce(&target.name, value)?;
        if target.indices.is_empty() {
            self.vars.insert(target.name.clone(), value);
        } else {
            let indices = self.eval_indices(&target.indices)?;
            *self.arrays.element(&target.name, &indices).at(self.line)? = value;
        }
        Ok(())
    }

    /// Evaluate subscripts (or DIM bounds).
    pub(super) fn eval_indices(&mut self, exprs: &[Expr]) -> Result<Vec<usize>> {
        let mut out = Vec::with_capacity(exprs.len());
        for expr in exprs {
            let n = self.eval_num(expr)?;
            out.push(rt::subscript(n).at(self.line)?);
        }
        Ok(out)
    }

    pub(super) fn element(&mut self, name: &str, indices: &[usize]) -> Result<&mut Value> {
        self.arrays.element(name, indices).at(self.line)
    }
}
