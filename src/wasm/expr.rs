//! Code for expressions and assignments.

use super::code::Op as I;
use super::gen::{Body, Gen, Ty};
use crate::ast::{BinOp, Builtin, Expr, LValue, UnOp};

impl Gen {
    /// Stop with the runtime error `message`. Code after this point in the
    /// same block is never reached.
    pub(super) fn fail(&mut self, body: &mut Body, message: &str) {
        let handle = self.literal(message);
        body.code.i32_const(handle);
        body.code.call(self.rt.rt_fail);
        body.code.op(I::Unreachable);
    }

    /// The program has used a string as a number or the reverse. The
    /// interpreter finds this out when the statement runs, so that is when
    /// the compiled program reports it too.
    fn mismatch(&mut self, body: &mut Body) {
        body.code.call(self.rt.rt_type_mismatch);
        body.code.op(I::Unreachable);
    }

    /// Generate `expr`, which must be of type `want`.
    pub(super) fn typed(&mut self, body: &mut Body, expr: &Expr, want: Ty) {
        if self.expr(body, expr) != want {
            self.mismatch(body);
        }
    }

    pub(super) fn num(&mut self, body: &mut Body, expr: &Expr) {
        self.typed(body, expr, Ty::Num);
    }

    /// Generate `expr`, leaving its value on the stack.
    pub(super) fn expr(&mut self, body: &mut Body, expr: &Expr) -> Ty {
        match expr {
            Expr::Num(n) => {
                body.code.f64_const(*n);
                Ty::Num
            }
            Expr::Str(text) => {
                let handle = self.literal(text);
                body.code.i32_const(handle);
                Ty::Str
            }
            Expr::Var(name) => {
                match Ty::of(name) {
                    Ty::Num => {
                        let slot = self.numbers.get(name);
                        body.code.local_get(body.numbers_at);
                        body.code.f64_load(slot * 8);
                    }
                    Ty::Str => {
                        let slot = self.strings.get(name);
                        body.code.i32_const(slot as i32);
                        body.code.call(self.rt.rt_sget);
                    }
                }
                Ty::of(name)
            }
            Expr::Array(name, indices) => {
                self.subscripts(body, indices);
                body.code.i32_const(self.arrays.get(name) as i32);
                body.code.i32_const(indices.len() as i32);
                body.code.call(match Ty::of(name) {
                    Ty::Num => self.rt.rt_aget_num,
                    Ty::Str => self.rt.rt_aget_str,
                });
                Ty::of(name)
            }
            Expr::Call(builtin, args) => self.builtin(body, *builtin, args),
            Expr::FnCall(name, args) => self.call_user_function(body, name, args),
            Expr::Unary(op, operand) => {
                self.num(body, operand);
                match op {
                    UnOp::Neg => body.code.op(I::F64Neg),
                    UnOp::Not => body.code.call(self.rt.rt_not),
                }
                Ty::Num
            }
            Expr::Binary(op, left, right) => {
                let left = self.expr(body, left);
                let right = self.expr(body, right);
                match (left, right) {
                    (Ty::Num, Ty::Num) => self.numeric(body, *op),
                    (Ty::Str, Ty::Str) => return self.stringy(body, *op),
                    _ => self.mismatch(body),
                }
                Ty::Num
            }
        }
    }

    /// Evaluate subscripts (or DIM bounds) and pass each to the runtime.
    pub(super) fn subscripts(&mut self, body: &mut Body, indices: &[Expr]) {
        for index in indices {
            self.num(body, index);
            body.code.call(self.rt.rt_idx);
        }
    }

    /// Turn the i32 condition on the stack into BASIC's -1 or 0.
    fn truth(&mut self, body: &mut Body) {
        body.code.local_set(body.scratch);
        body.code.f64_const(-1.0);
        body.code.f64_const(0.0);
        body.code.local_get(body.scratch);
        body.code.op(I::Select);
    }

    fn numeric(&mut self, body: &mut Body, op: BinOp) {
        let compare = |body: &mut Body, me: &mut Gen, instruction| {
            body.code.op(instruction);
            me.truth(body);
        };
        match op {
            BinOp::Add => body.code.op(I::F64Add),
            BinOp::Sub => body.code.op(I::F64Sub),
            BinOp::Mul => body.code.op(I::F64Mul),
            // the runtime checks for division by zero
            BinOp::Div => body.code.call(self.rt.rt_div),
            BinOp::Pow => body.code.call(self.rt.rt_pow),
            BinOp::And => body.code.call(self.rt.rt_and),
            BinOp::Or => body.code.call(self.rt.rt_or),
            BinOp::Eq => compare(body, self, I::F64Eq),
            BinOp::Ne => compare(body, self, I::F64Ne),
            BinOp::Lt => compare(body, self, I::F64Lt),
            BinOp::Le => compare(body, self, I::F64Le),
            BinOp::Gt => compare(body, self, I::F64Gt),
            BinOp::Ge => compare(body, self, I::F64Ge),
        }
    }

    fn stringy(&mut self, body: &mut Body, op: BinOp) -> Ty {
        if op == BinOp::Add {
            body.code.call(self.rt.rt_concat);
            return Ty::Str;
        }
        // the runtime compares, giving -1, 0 or 1, which is tested against 0
        let test = match op {
            BinOp::Eq => I::I32Eq,
            BinOp::Ne => I::I32Ne,
            BinOp::Lt => I::I32LtS,
            BinOp::Le => I::I32LeS,
            BinOp::Gt => I::I32GtS,
            BinOp::Ge => I::I32GeS,
            _ => {
                self.mismatch(body);
                return Ty::Num;
            }
        };
        body.code.call(self.rt.rt_scmp);
        body.code.i32_const(0);
        body.code.op(test);
        self.truth(body);
        Ty::Num
    }

    /// Pass `args` to the runtime as the arguments of a call it will make.
    pub(super) fn pass_arguments(&mut self, body: &mut Body, args: &[Expr]) {
        for arg in args {
            let pass = match self.expr(body, arg) {
                Ty::Num => self.rt.rt_arg_num,
                Ty::Str => self.rt.rt_arg_str,
            };
            body.code.call(pass);
        }
    }

    /// Built-in functions run in the runtime, which checks the types of
    /// their arguments as the interpreter does.
    fn builtin(&mut self, body: &mut Body, builtin: Builtin, args: &[Expr]) -> Ty {
        // the three that are single WebAssembly instructions are done in line
        let instruction = match builtin {
            Builtin::Int => Some(I::F64Floor),
            Builtin::Fix => Some(I::F64Trunc),
            Builtin::Abs => Some(I::F64Abs),
            _ => None,
        };
        if let Some(instruction) = instruction {
            self.num(body, &args[0]);
            body.code.op(instruction);
            return Ty::Num;
        }
        self.pass_arguments(body, args);
        let number = Builtin::NAMES
            .iter()
            .position(|name| *name == builtin.name())
            .expect("a built-in");
        let result = Ty::of(builtin.name());
        body.code.i32_const(number as i32);
        body.code.i32_const(args.len() as i32);
        body.code.call(match result {
            Ty::Num => self.rt.rt_call_num,
            Ty::Str => self.rt.rt_call_str,
        });
        result
    }

    /// Store the value that `value` generates in `target`.
    pub(super) fn assign(
        &mut self,
        body: &mut Body,
        target: &LValue,
        value: impl FnOnce(&mut Gen, &mut Body) -> Ty,
    ) {
        let want = Ty::of(&target.name);
        let whole_number = target.name.ends_with('%');
        // the value is worked out before the subscripts, as the interpreter does
        let checked = |me: &mut Gen, body: &mut Body| {
            if value(me, body) != want {
                me.mismatch(body);
            }
            if whole_number {
                body.code.call(me.rt.rt_round);
            }
        };
        if !target.indices.is_empty() {
            checked(self, body);
            self.subscripts(body, &target.indices);
            body.code.i32_const(self.arrays.get(&target.name) as i32);
            body.code.i32_const(target.indices.len() as i32);
            body.code.call(match want {
                Ty::Num => self.rt.rt_aset_num,
                Ty::Str => self.rt.rt_aset_str,
            });
            return;
        }
        match want {
            Ty::Num => {
                let slot = self.numbers.get(&target.name);
                body.code.local_get(body.numbers_at);
                checked(self, body);
                body.code.f64_store(slot * 8);
            }
            Ty::Str => {
                let slot = self.strings.get(&target.name);
                body.code.i32_const(slot as i32);
                checked(self, body);
                body.code.call(self.rt.rt_sset);
            }
        }
    }
}
