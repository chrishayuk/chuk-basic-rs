//! The MAT statements.

use super::Parser;
use crate::ast::{Mat, MatExpr, MatTarget, Stmt};
use crate::error::Result;
use crate::lexer::Tok;

impl Parser {
    /// Everything after the word `MAT`.
    pub(super) fn mat_stmt(&mut self) -> Result<Stmt> {
        let mat = match self.peek() {
            Tok::Read => {
                self.pos += 1;
                Mat::Read(self.mat_targets()?)
            }
            Tok::Input => {
                self.pos += 1;
                Mat::Input(self.mat_targets()?)
            }
            Tok::Print => {
                self.pos += 1;
                let mut items = Vec::new();
                loop {
                    let name = self.ident("a matrix name")?;
                    let packed = self.eat(&Tok::Semi);
                    let more = packed || self.eat(&Tok::Comma);
                    items.push((name, packed));
                    if !more || self.at_stmt_end() {
                        break;
                    }
                }
                Mat::Print(items)
            }
            _ => {
                let target = self.ident("READ, PRINT, INPUT or a matrix name after MAT")?;
                self.expect(Tok::Eq, "'='")?;
                Mat::Assign {
                    target,
                    value: self.mat_expr()?,
                }
            }
        };
        Ok(Stmt::Mat(mat))
    }

    /// `A, B(2, 3), ...`
    fn mat_targets(&mut self) -> Result<Vec<MatTarget>> {
        let mut targets = Vec::new();
        loop {
            let name = self.ident("a matrix name")?;
            let dims = if *self.peek() == Tok::LParen {
                self.args()?
            } else {
                Vec::new()
            };
            targets.push(MatTarget { name, dims });
            if !self.eat(&Tok::Comma) {
                return Ok(targets);
            }
        }
    }

    fn mat_expr(&mut self) -> Result<MatExpr> {
        // (k) * A
        if *self.peek() == Tok::LParen {
            self.pos += 1;
            let factor = self.expr()?;
            self.expect(Tok::RParen, "')'")?;
            self.expect(Tok::Star, "'*' after the scale factor")?;
            return Ok(MatExpr::Scale(factor, self.ident("a matrix name")?));
        }
        let name = self.ident("a matrix expression")?;
        let has_args = *self.peek() == Tok::LParen;
        match name.as_str() {
            "ZER" | "CON" | "IDN" => {
                let dims = if has_args { self.args()? } else { Vec::new() };
                Ok(match name.as_str() {
                    "ZER" => MatExpr::Zer(dims),
                    "CON" => MatExpr::Con(dims),
                    _ => MatExpr::Idn(dims),
                })
            }
            "TRN" | "INV" if has_args => {
                self.pos += 1;
                let of = self.ident("a matrix name")?;
                self.expect(Tok::RParen, "')'")?;
                Ok(if name == "TRN" {
                    MatExpr::Trn(of)
                } else {
                    MatExpr::Inv(of)
                })
            }
            _ => Ok(match self.peek() {
                Tok::Plus => {
                    self.pos += 1;
                    MatExpr::Add(name, self.ident("a matrix name")?)
                }
                Tok::Minus => {
                    self.pos += 1;
                    MatExpr::Sub(name, self.ident("a matrix name")?)
                }
                Tok::Star => {
                    self.pos += 1;
                    MatExpr::Mul(name, self.ident("a matrix name")?)
                }
                _ => MatExpr::Copy(name),
            }),
        }
    }
}
