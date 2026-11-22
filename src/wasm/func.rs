//! User functions (`DEF FN`): calling them, and their bodies.

use super::gen::{Body, Gen, Ty};
use crate::ast::Expr;
use crate::interp::lower::FnBody;

impl Gen {
    pub(super) fn call_user_function(&mut self, body: &mut Body, name: &str, args: &[Expr]) -> Ty {
        let result = Ty::of(name);
        let Some(function) = self.functions.get(name) else {
            self.fail(body, &format!("Undefined function {name}"));
            return result;
        };
        let (index, def) = (function.index, function.def.clone());
        if args.len() != def.params.len() {
            self.fail(body, &format!("Wrong number of arguments for {name}"));
            return result;
        }
        for (param, arg) in def.params.iter().zip(args) {
            self.typed(body, arg, Ty::of(param));
            if param.ends_with('%') {
                body.code.call(self.rt.rt_round);
            }
        }
        body.code.call(index);
        result
    }

    /// Load variable `name` onto the stack.
    fn load(&mut self, body: &mut Body, name: &str) {
        self.expr(body, &Expr::Var(name.to_string()));
    }

    /// Store the local `from` in variable `name`.
    fn store(&mut self, body: &mut Body, name: &str, from: u32) {
        match Ty::of(name) {
            Ty::Num => {
                let slot = self.numbers.get(name);
                body.code.local_get(body.numbers_at);
                body.code.local_get(from);
                body.code.f64_store(slot * 8);
            }
            Ty::Str => {
                let slot = self.strings.get(name);
                body.code.i32_const(slot as i32);
                body.code.local_get(from);
                body.code.call(self.rt.rt_sset);
            }
        }
    }

    /// The WebAssembly function for the user function `name`. Its
    /// parameters are ordinary variables that get their old values back
    /// when the function returns.
    pub(super) fn user_function(&mut self, name: &str) {
        let function = &self.functions[name];
        let (index, def) = (function.index, function.def.clone());
        let mut body = Body::new(def.params.len() as u32, &self.rt);
        body.code.call(self.rt.rt_fn_enter);

        // a multi-line function returns the value of the variable named
        // after it, which is also put back afterwards
        let block = matches!(def.body, FnBody::Block(_));
        let mut shadowed: Vec<&str> = def.params.iter().map(String::as_str).collect();
        if block {
            shadowed.push(name);
        }
        let saved: Vec<u32> = shadowed
            .iter()
            .map(|var| body.code.local(Ty::of(var).val()))
            .collect();
        for (at, var) in shadowed.iter().enumerate() {
            self.load(&mut body, var);
            body.code.local_set(saved[at]);
            match def.params.get(at) {
                Some(_) => self.store(&mut body, var, at as u32),
                // the function's own variable starts out empty
                None => {
                    let empty = body.code.local(Ty::of(var).val());
                    self.store(&mut body, var, empty);
                }
            }
        }

        let result = body.code.local(Ty::of(name).val());
        match &def.body {
            FnBody::Expr(expr) => self.typed(&mut body, expr, Ty::of(name)),
            FnBody::Block(start) => {
                body.code.call(self.rt.rt_block_enter);
                body.code.i32_const(*start as i32);
                body.code.i32_const(1);
                body.code.call(self.run);
                body.code.call(self.rt.rt_block_exit);
                self.load(&mut body, name);
            }
        }
        if name.ends_with('%') {
            body.code.call(self.rt.rt_round);
        }
        body.code.local_set(result);

        for (at, var) in shadowed.iter().enumerate().rev() {
            self.store(&mut body, var, saved[at]);
        }
        body.code.call(self.rt.rt_fn_exit);
        body.code.local_get(result);

        let (locals, instructions) = body.code.finish();
        self.module.define(index, &locals, &instructions);
    }
}
