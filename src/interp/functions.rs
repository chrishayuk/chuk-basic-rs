//! User functions defined with DEF FN.

use super::compile::FnBody;
use super::value::{default_value, Value};
use super::{Exit, Interpreter, MAX_FN_DEPTH};
use crate::ast::Expr;
use crate::error::Result;

impl Interpreter<'_> {
    pub(super) fn call_fn(&mut self, name: &str, args: &[Expr]) -> Result<Value> {
        let Some(func) = self.funcs.get(name).cloned() else {
            return Err(self.err(format!("Undefined function {name}")));
        };
        if args.len() != func.params.len() {
            return Err(self.err(format!("Wrong number of arguments for {name}")));
        }
        let mut values = Vec::with_capacity(args.len());
        for (param, arg) in func.params.iter().zip(args) {
            let value = self.eval(arg)?;
            values.push(self.coerce(param, value)?);
        }
        if self.fn_depth >= MAX_FN_DEPTH {
            return Err(self.err("Out of memory: function calls nested too deeply"));
        }

        // parameters shadow globals of the same name for the duration of the call
        let mut saved = Vec::with_capacity(values.len() + 1);
        for (param, value) in func.params.iter().zip(values) {
            saved.push((param.as_str(), self.vars.insert(param.clone(), value)));
        }
        self.fn_depth += 1;
        let result = match &func.body {
            FnBody::Expr(expr) => self.eval(expr),
            FnBody::Block(start) => {
                // the body assigns the result to a variable named after the function
                saved.push((
                    name,
                    self.vars.insert(name.to_string(), default_value(name)),
                ));
                let line = self.line;
                let gosub_base = std::mem::replace(&mut self.fn_gosub_base, self.gosub_stack.len());
                let for_base = std::mem::replace(&mut self.fn_for_base, self.for_stack.len());
                let exit = self.exec(*start, true);
                self.gosub_stack.truncate(self.fn_gosub_base);
                self.for_stack.truncate(self.fn_for_base);
                self.fn_gosub_base = gosub_base;
                self.fn_for_base = for_base;
                exit.map(|exit| {
                    // END / STOP inside a function ends the whole program
                    self.halted |= matches!(exit, Exit::Halt);
                    self.line = line;
                    self.vars
                        .get(name)
                        .cloned()
                        .unwrap_or_else(|| default_value(name))
                })
            }
        };
        self.fn_depth -= 1;
        for (var, old) in saved.into_iter().rev() {
            match old {
                Some(value) => self.vars.insert(var.to_string(), value),
                None => self.vars.remove(var),
            };
        }
        self.coerce(name, result?)
    }
}
