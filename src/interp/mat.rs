//! The MAT statements: works out the expressions, then hands over to the
//! shared matrix code.

use super::Interpreter;
use crate::ast::{Mat, MatExpr, MatTarget};
use crate::error::{AtLine, Result};
use crate::rt::mat::{self, MatOp};

impl Interpreter<'_> {
    pub(super) fn mat(&mut self, statement: &Mat) -> Result<()> {
        match statement {
            Mat::Read(targets) => {
                for MatTarget { name, dims } in targets {
                    let dims = self.eval_indices(dims)?;
                    mat::read(&mut self.arrays, &mut self.data, name, &dims).at(self.line)?;
                }
            }
            Mat::Input(targets) => {
                for MatTarget { name, dims } in targets {
                    let dims = self.eval_indices(dims)?;
                    mat::input(&mut self.arrays, &mut self.console, name, &dims).at(self.line)?;
                }
            }
            Mat::Print(items) => {
                for (name, packed) in items {
                    mat::print(&self.arrays, &mut self.console, name, *packed).at(self.line)?;
                }
            }
            Mat::Assign { target, value } => {
                let dims;
                let op = match value {
                    MatExpr::Copy(a) => MatOp::Copy(a),
                    MatExpr::Add(a, b) => MatOp::Add(a, b),
                    MatExpr::Sub(a, b) => MatOp::Sub(a, b),
                    MatExpr::Mul(a, b) => MatOp::Mul(a, b),
                    MatExpr::Scale(factor, a) => MatOp::Scale(self.eval_num(factor)?, a),
                    MatExpr::Trn(a) => MatOp::Trn(a),
                    MatExpr::Inv(a) => MatOp::Inv(a),
                    MatExpr::Zer(exprs) | MatExpr::Con(exprs) | MatExpr::Idn(exprs) => {
                        dims = self.eval_indices(exprs)?;
                        match value {
                            MatExpr::Zer(_) => MatOp::Zer(&dims),
                            MatExpr::Con(_) => MatOp::Con(&dims),
                            _ => MatOp::Idn(&dims),
                        }
                    }
                };
                mat::assign(&mut self.arrays, target, op).at(self.line)?;
            }
        }
        Ok(())
    }
}
