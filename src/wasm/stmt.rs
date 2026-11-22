//! Code for each operation of the lowered program.

use super::code::Op as I;
use super::gen::{Body, Gen, Ty};
use crate::ast::{Expr, Mat, MatExpr, MatTarget, PrintItem, Stmt};
use crate::interp::lower::Op;

/// The operation codes of `rt_mat_assign`.
mod mat_op {
    pub const COPY: i32 = 0;
    pub const ADD: i32 = 1;
    pub const SUB: i32 = 2;
    pub const MUL: i32 = 3;
    pub const SCALE: i32 = 4;
    pub const ZER: i32 = 5;
    pub const CON: i32 = 6;
    pub const IDN: i32 = 7;
    pub const TRN: i32 = 8;
    pub const INV: i32 = 9;
}

impl Gen {
    /// Jump to line `line`, or fail as the interpreter would if the program
    /// has no such line.
    fn jump_to_line(&mut self, body: &mut Body, line: u32) {
        match self.lowered.line_map.get(&line).copied() {
            Some(target) => self.jump(body, target),
            None => self.fail(body, &format!("Undefined line {line}")),
        }
    }

    /// GOSUB `line`, returning to operation `ret`.
    fn gosub(&mut self, body: &mut Body, line: u32, ret: usize) {
        if self.lowered.line_map.contains_key(&line) {
            body.code.i32_const(ret as i32);
            body.code.call(self.rt.rt_gosub);
        }
        self.jump_to_line(body, line);
    }

    /// The code for operation `index` of `ops`.
    pub(super) fn op(&mut self, body: &mut Body, ops: &[Op], index: usize, op: &Op) {
        match op {
            Op::Jump(target) => self.jump(body, *target),
            Op::JumpIfFalse(cond, target) => {
                self.num(body, cond);
                body.code.f64_const(0.0);
                body.code.op(I::F64Eq);
                body.code.if_();
                self.jump(body, *target);
                body.code.op(I::End);
            }
            Op::Next(var) => {
                let slot = var
                    .as_ref()
                    .map_or(-1, |name| self.numbers.get(name) as i32);
                body.code.i32_const(slot);
                body.code.call(self.rt.rt_next);
                // the loop body to go back to, or -1 when the loop is over
                body.code.local_tee(body.scratch);
                body.code.i32_const(0);
                body.code.op(I::I32GeS);
                body.code.if_();
                body.code.local_get(body.scratch);
                self.jump_to_stack(body);
                body.code.op(I::End);
            }
            Op::FnEnd => {
                self.in_function(body);
                body.code.if_();
                body.code.op(I::Return);
                body.code.op(I::End);
                self.fail(body, "FNEND without DEF");
            }
            Op::Stmt(stmt) => self.stmt(body, ops, index, stmt),
        }
    }

    fn stmt(&mut self, body: &mut Body, ops: &[Op], index: usize, stmt: &Stmt) {
        match stmt {
            Stmt::Let { target, expr } => self.assign(body, target, |me, body| me.expr(body, expr)),
            Stmt::Print(items) => self.print(body, items),
            Stmt::PrintUsing {
                format,
                values,
                newline,
            } => {
                // the picture is worked out first, then the values
                self.typed(body, format, Ty::Str);
                self.pass_arguments(body, values);
                body.code.i32_const(values.len() as i32);
                body.code.i32_const(i32::from(*newline));
                body.code.call(self.rt.rt_using);
            }
            Stmt::Input {
                prompt,
                question_mark,
                targets,
            } => {
                for target in targets {
                    body.code
                        .i32_const(i32::from(Ty::of(&target.name) == Ty::Str));
                    body.code.call(self.rt.rt_input_want);
                }
                self.prompt(body, prompt.as_deref());
                body.code.i32_const(i32::from(*question_mark));
                body.code.call(self.rt.rt_input);
                for (at, target) in targets.iter().enumerate() {
                    self.assign(body, target, |me, body| {
                        let ty = Ty::of(&target.name);
                        body.code.i32_const(at as i32);
                        body.code.call(match ty {
                            Ty::Num => me.rt.rt_input_num,
                            Ty::Str => me.rt.rt_input_str,
                        });
                        ty
                    });
                }
            }
            Stmt::LineInput { prompt, target } => self.assign(body, target, |me, body| {
                me.prompt(body, prompt.as_deref());
                body.code.call(me.rt.rt_line_input);
                Ty::Str
            }),
            Stmt::For {
                var,
                start,
                end,
                step,
            } => {
                body.code.i32_const(self.numbers.get(var) as i32);
                self.num(body, start);
                self.num(body, end);
                match step {
                    Some(step) => self.num(body, step),
                    None => body.code.f64_const(1.0),
                }
                body.code.i32_const(index as i32 + 1);
                body.code.call(self.rt.rt_for);
                // an empty range skips to just past the matching NEXT
                body.code.if_();
                match Gen::after_matching_next(ops, index) {
                    Some(target) => self.jump(body, target),
                    None => self.fail(body, "FOR without NEXT"),
                }
                body.code.op(I::End);
            }
            Stmt::Goto(line) => self.jump_to_line(body, *line),
            Stmt::Gosub(line) => self.gosub(body, *line, index + 1),
            Stmt::Return => {
                body.code.call(self.rt.rt_return);
                self.jump_to_stack(body);
            }
            Stmt::On {
                expr,
                targets,
                gosub,
            } => {
                self.num(body, expr);
                body.code.i32_const(targets.len() as i32);
                body.code.call(self.rt.rt_on);
                body.code.local_set(body.scratch);
                for (at, line) in targets.iter().enumerate() {
                    body.code.local_get(body.scratch);
                    body.code.i32_const(at as i32 + 1);
                    body.code.op(I::I32Eq);
                    body.code.if_();
                    if *gosub {
                        self.gosub(body, *line, index + 1);
                    } else {
                        self.jump_to_line(body, *line);
                    }
                    body.code.op(I::End);
                }
            }
            Stmt::Dim(arrays) => {
                for (name, bounds) in arrays {
                    self.subscripts(body, bounds);
                    body.code.i32_const(self.arrays.get(name) as i32);
                    body.code.i32_const(bounds.len() as i32);
                    body.code.call(self.rt.rt_dim);
                }
            }
            Stmt::Read(targets) => {
                for target in targets {
                    self.assign(body, target, |me, body| {
                        let ty = Ty::of(&target.name);
                        body.code.call(match ty {
                            Ty::Num => me.rt.rt_read_num,
                            Ty::Str => me.rt.rt_read_str,
                        });
                        ty
                    });
                }
            }
            Stmt::Restore(line) => {
                body.code.i32_const(line.map_or(-1, |line| line as i32));
                body.code.call(self.rt.rt_restore);
            }
            Stmt::Randomize(seed) => {
                body.code.i32_const(i32::from(seed.is_some()));
                match seed {
                    Some(seed) => self.num(body, seed),
                    None => body.code.f64_const(0.0),
                }
                body.code.call(self.rt.rt_randomize);
            }
            Stmt::Mat(mat) => self.mat(body, mat),
            Stmt::End | Stmt::Stop => {
                body.code.call(self.rt.rt_end);
                body.code.op(I::Unreachable);
            }
            // removed or rewritten when the program was lowered
            Stmt::If { .. }
            | Stmt::Next(_)
            | Stmt::Def { .. }
            | Stmt::FnEnd
            | Stmt::While(_)
            | Stmt::Wend
            | Stmt::Data(_)
            | Stmt::Rem(_) => {}
        }
    }

    /// Push an optional prompt: its handle, and whether there is one.
    fn prompt(&mut self, body: &mut Body, prompt: Option<&str>) {
        let handle = prompt.map_or(0, |text| self.literal(text));
        body.code.i32_const(handle);
        body.code.i32_const(i32::from(prompt.is_some()));
    }

    fn print(&mut self, body: &mut Body, items: &[PrintItem]) {
        for item in items {
            match item {
                PrintItem::Expr(expr) => {
                    let print = match self.expr(body, expr) {
                        Ty::Num => self.rt.rt_print_num,
                        Ty::Str => self.rt.rt_print_str,
                    };
                    body.code.call(print);
                }
                PrintItem::Tab(expr) => {
                    self.num(body, expr);
                    body.code.call(self.rt.rt_tab);
                }
                PrintItem::Spc(expr) => {
                    self.num(body, expr);
                    body.code.call(self.rt.rt_spc);
                }
                PrintItem::Comma => body.code.call(self.rt.rt_zone),
                PrintItem::Semi => {}
            }
        }
        if !matches!(items.last(), Some(PrintItem::Comma | PrintItem::Semi)) {
            body.code.call(self.rt.rt_newline);
        }
    }

    fn mat(&mut self, body: &mut Body, mat: &Mat) {
        match mat {
            Mat::Read(targets) | Mat::Input(targets) => {
                let function = if matches!(mat, Mat::Read(_)) {
                    self.rt.rt_mat_read
                } else {
                    self.rt.rt_mat_input
                };
                for MatTarget { name, dims } in targets {
                    self.subscripts(body, dims);
                    body.code.i32_const(self.arrays.get(name) as i32);
                    body.code.i32_const(dims.len() as i32);
                    body.code.call(function);
                }
            }
            Mat::Print(items) => {
                for (name, packed) in items {
                    body.code.i32_const(self.arrays.get(name) as i32);
                    body.code.i32_const(i32::from(*packed));
                    body.code.call(self.rt.rt_mat_print);
                }
            }
            Mat::Assign { target, value } => {
                let none = String::new();
                let no_dims: &[Expr] = &[];
                let (op, a, b, factor, dims) = match value {
                    MatExpr::Copy(a) => (mat_op::COPY, a, &none, None, no_dims),
                    MatExpr::Add(a, b) => (mat_op::ADD, a, b, None, no_dims),
                    MatExpr::Sub(a, b) => (mat_op::SUB, a, b, None, no_dims),
                    MatExpr::Mul(a, b) => (mat_op::MUL, a, b, None, no_dims),
                    MatExpr::Scale(factor, a) => (mat_op::SCALE, a, &none, Some(factor), no_dims),
                    MatExpr::Zer(dims) => (mat_op::ZER, &none, &none, None, dims.as_slice()),
                    MatExpr::Con(dims) => (mat_op::CON, &none, &none, None, dims.as_slice()),
                    MatExpr::Idn(dims) => (mat_op::IDN, &none, &none, None, dims.as_slice()),
                    MatExpr::Trn(a) => (mat_op::TRN, a, &none, None, no_dims),
                    MatExpr::Inv(a) => (mat_op::INV, a, &none, None, no_dims),
                };
                self.subscripts(body, dims);
                body.code.i32_const(self.arrays.get(target) as i32);
                body.code.i32_const(op);
                // an operand that is not used is passed as array 0
                for operand in [a, b] {
                    let id = if operand.is_empty() {
                        0
                    } else {
                        self.arrays.get(operand)
                    };
                    body.code.i32_const(id as i32);
                }
                match factor {
                    Some(factor) => self.num(body, factor),
                    None => body.code.f64_const(0.0),
                }
                body.code.i32_const(dims.len() as i32);
                body.code.call(self.rt.rt_mat_assign);
            }
        }
    }
}
