//! Calls to the built-in functions.

use super::Interpreter;
use crate::ast::{Builtin, Expr};
use crate::error::{AtLine, Result};
use crate::rt::{builtins, Value};

impl Interpreter<'_> {
    pub(super) fn call_builtin(&mut self, builtin: Builtin, args: &[Expr]) -> Result<Value> {
        let values = args
            .iter()
            .map(|arg| self.eval(arg))
            .collect::<Result<Vec<Value>>>()?;
        builtins::call(builtin, &values, &mut self.rnd, self.dialect).at(self.line)
    }
}
