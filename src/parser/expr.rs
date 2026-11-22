//! Expression parsing.
//!
//! Operator precedence, lowest to highest:
//! `OR`, `AND`, `NOT`, relational, `+ -`, `* /`, unary `+ -`, `^`.

use super::{describe, Parser};
use crate::ast::{BinOp, Builtin, Expr, UnOp};
use crate::error::Result;
use crate::lexer::Tok;

impl Parser {
    /// Parenthesised, comma-separated expression list.
    pub(super) fn args(&mut self) -> Result<Vec<Expr>> {
        self.expect(Tok::LParen, "'('")?;
        let mut args = Vec::new();
        if self.eat(&Tok::RParen) {
            return Ok(args);
        }
        loop {
            args.push(self.expr()?);
            if !self.eat(&Tok::Comma) {
                break;
            }
        }
        self.expect(Tok::RParen, "')'")?;
        Ok(args)
    }

    pub(super) fn expr(&mut self) -> Result<Expr> {
        let mut left = self.and_expr()?;
        while self.eat(&Tok::Or) {
            left = binary(BinOp::Or, left, self.and_expr()?);
        }
        Ok(left)
    }

    pub(super) fn and_expr(&mut self) -> Result<Expr> {
        let mut left = self.not_expr()?;
        while self.eat(&Tok::And) {
            left = binary(BinOp::And, left, self.not_expr()?);
        }
        Ok(left)
    }

    pub(super) fn not_expr(&mut self) -> Result<Expr> {
        if self.eat(&Tok::Not) {
            return Ok(Expr::Unary(UnOp::Not, Box::new(self.not_expr()?)));
        }
        self.relational()
    }

    pub(super) fn relational(&mut self) -> Result<Expr> {
        let mut left = self.additive()?;
        loop {
            let op = match self.peek() {
                Tok::Eq => BinOp::Eq,
                Tok::Ne => BinOp::Ne,
                Tok::Lt => BinOp::Lt,
                Tok::Le => BinOp::Le,
                Tok::Gt => BinOp::Gt,
                Tok::Ge => BinOp::Ge,
                _ => return Ok(left),
            };
            self.pos += 1;
            left = binary(op, left, self.additive()?);
        }
    }

    pub(super) fn additive(&mut self) -> Result<Expr> {
        let mut left = self.term()?;
        loop {
            let op = match self.peek() {
                Tok::Plus => BinOp::Add,
                Tok::Minus => BinOp::Sub,
                _ => return Ok(left),
            };
            self.pos += 1;
            left = binary(op, left, self.term()?);
        }
    }

    pub(super) fn term(&mut self) -> Result<Expr> {
        let mut left = self.unary()?;
        loop {
            let op = match self.peek() {
                Tok::Star => BinOp::Mul,
                Tok::Slash => BinOp::Div,
                _ => return Ok(left),
            };
            self.pos += 1;
            left = binary(op, left, self.unary()?);
        }
    }

    pub(super) fn unary(&mut self) -> Result<Expr> {
        if self.eat(&Tok::Minus) {
            if matches!(self.peek(), Tok::Minus | Tok::Plus) {
                return Ok(Expr::Unary(UnOp::Neg, Box::new(self.unary()?)));
            }
            // `-X^2`, not `-(X^2)`: the dialects disagree about it
            let (operand, power) = self.power_written()?;
            let op = if power { UnOp::NegPower } else { UnOp::Neg };
            return Ok(Expr::Unary(op, Box::new(operand)));
        }
        if self.eat(&Tok::Plus) {
            return self.unary();
        }
        self.power()
    }

    /// `^` binds tighter than unary minus on its left (`-2^2` is -4) but
    /// allows a signed exponent (`2^-1`).
    pub(super) fn power(&mut self) -> Result<Expr> {
        Ok(self.power_written()?.0)
    }

    /// [`Self::power`], and whether a `^` was written at this level (as in
    /// `X^2`, and not in `(X^2)`).
    fn power_written(&mut self) -> Result<(Expr, bool)> {
        let mut written = false;
        let mut left = self.primary()?;
        while self.eat(&Tok::Caret) {
            let right = if self.eat(&Tok::Minus) {
                Expr::Unary(UnOp::Neg, Box::new(self.primary()?))
            } else {
                self.eat(&Tok::Plus);
                self.primary()?
            };
            left = binary(BinOp::Pow, left, right);
            written = true;
        }
        Ok((left, written))
    }

    pub(super) fn primary(&mut self) -> Result<Expr> {
        match self.next() {
            Tok::Num(n) => Ok(Expr::Num(n)),
            Tok::Str(s) => Ok(Expr::Str(s)),
            Tok::LParen => {
                let inner = self.expr()?;
                self.expect(Tok::RParen, "')'")?;
                Ok(inner)
            }
            Tok::Ident(mut name) => {
                if let Some(builtin) = Builtin::from_name(&name) {
                    let args = if *self.peek() == Tok::LParen {
                        self.args()?
                    } else {
                        Vec::new()
                    };
                    let (min, max) = builtin.arity();
                    if args.len() < min || args.len() > max {
                        return Err(self.err(format!("Wrong number of arguments for {name}")));
                    }
                    return Ok(Expr::Call(builtin, args));
                }
                if name == "FN" {
                    name.push_str(&self.ident("a function name after FN")?); // FN A(X)
                }
                if *self.peek() != Tok::LParen {
                    Ok(Expr::Var(name))
                } else if name.starts_with("FN") && name.len() > 2 {
                    Ok(Expr::FnCall(name, self.args()?))
                } else {
                    let indices = self.args()?;
                    if indices.is_empty() {
                        return Err(self.err("Expected an array subscript"));
                    }
                    Ok(Expr::Array(name, indices))
                }
            }
            other => {
                self.pos -= 1;
                Err(self.err(format!(
                    "Expected an expression, found {}",
                    describe(&other)
                )))
            }
        }
    }
}

fn binary(op: BinOp, left: Expr, right: Expr) -> Expr {
    Expr::Binary(op, Box::new(left), Box::new(right))
}
