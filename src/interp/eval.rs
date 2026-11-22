//! Expression evaluation.

use super::value::{default_value, Value};
use super::Interpreter;
use crate::ast::{BinOp, Expr, UnOp};
use crate::error::Result;

impl Interpreter<'_> {
    pub(super) fn eval_num(&mut self, expr: &Expr) -> Result<f64> {
        match self.eval(expr)? {
            Value::Num(n) => Ok(n),
            Value::Str(_) => Err(self.type_mismatch()),
        }
    }

    pub(super) fn eval_str(&mut self, expr: &Expr) -> Result<String> {
        match self.eval(expr)? {
            Value::Str(s) => Ok(s),
            Value::Num(_) => Err(self.type_mismatch()),
        }
    }

    /// Evaluate a non-negative integer argument (rounded, as MS BASIC does).
    pub(super) fn eval_count(&mut self, expr: &Expr) -> Result<usize> {
        let n = self.eval_num(expr)?.round();
        if !(0.0..=u32::MAX as f64).contains(&n) {
            return Err(self.err("Illegal function call"));
        }
        Ok(n as usize)
    }

    pub(super) fn eval(&mut self, expr: &Expr) -> Result<Value> {
        match expr {
            Expr::Num(n) => Ok(Value::Num(*n)),
            Expr::Str(s) => Ok(Value::Str(s.clone())),
            Expr::Var(name) => Ok(self
                .vars
                .get(name)
                .cloned()
                .unwrap_or_else(|| default_value(name))),
            Expr::Array(name, indices) => {
                let indices = self.eval_indices(indices)?;
                Ok(self.element(name, &indices)?.clone())
            }
            Expr::Call(builtin, args) => self.call_builtin(*builtin, args),
            Expr::FnCall(name, args) => self.call_fn(name, args),
            Expr::Unary(op, operand) => {
                let n = self.eval_num(operand)?;
                Ok(Value::Num(match op {
                    UnOp::Neg => -n,
                    UnOp::Not => !(n as i64) as f64,
                }))
            }
            Expr::Binary(op, left, right) => {
                let left = self.eval(left)?;
                let right = self.eval(right)?;
                self.binary(*op, left, right)
            }
        }
    }

    pub(super) fn binary(&self, op: BinOp, left: Value, right: Value) -> Result<Value> {
        let truth = |b: bool| Value::Num(if b { -1.0 } else { 0.0 });
        match (left, right) {
            (Value::Num(a), Value::Num(b)) => Ok(match op {
                BinOp::Add => Value::Num(a + b),
                BinOp::Sub => Value::Num(a - b),
                BinOp::Mul => Value::Num(a * b),
                BinOp::Div if b == 0.0 => return Err(self.err("Division by zero")),
                BinOp::Div => Value::Num(a / b),
                BinOp::Pow => Value::Num(a.powf(b)),
                BinOp::Eq => truth(a == b),
                BinOp::Ne => truth(a != b),
                BinOp::Lt => truth(a < b),
                BinOp::Le => truth(a <= b),
                BinOp::Gt => truth(a > b),
                BinOp::Ge => truth(a >= b),
                BinOp::And => Value::Num(((a as i64) & (b as i64)) as f64),
                BinOp::Or => Value::Num(((a as i64) | (b as i64)) as f64),
            }),
            (Value::Str(a), Value::Str(b)) => Ok(match op {
                BinOp::Add => Value::Str(a + &b),
                BinOp::Eq => truth(a == b),
                BinOp::Ne => truth(a != b),
                BinOp::Lt => truth(a < b),
                BinOp::Le => truth(a <= b),
                BinOp::Gt => truth(a > b),
                BinOp::Ge => truth(a >= b),
                _ => return Err(self.type_mismatch()),
            }),
            _ => Err(self.type_mismatch()),
        }
    }
}
