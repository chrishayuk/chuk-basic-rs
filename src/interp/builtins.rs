//! Built-in functions.

use super::format::{signed, val};
use super::rng::Rng;
use super::value::Value;
use super::Interpreter;
use crate::ast::{Builtin, Expr};
use crate::error::Result;

impl Interpreter<'_> {
    pub(super) fn call_builtin(&mut self, builtin: Builtin, args: &[Expr]) -> Result<Value> {
        use Builtin::*;
        let illegal = |me: &Self| me.err(format!("Illegal function call: {}", builtin.name()));
        Ok(match builtin {
            Sin | Cos | Tan | Atn | Exp | Abs | Log | Sqr | Int | Fix | Sgn => {
                let x = self.eval_num(&args[0])?;
                Value::Num(match builtin {
                    Sin => x.sin(),
                    Cos => x.cos(),
                    Tan => x.tan(),
                    Atn => x.atan(),
                    Exp => x.exp(),
                    Abs => x.abs(),
                    Log if x <= 0.0 => return Err(illegal(self)),
                    Log => x.ln(),
                    Sqr if x < 0.0 => return Err(illegal(self)),
                    Sqr => x.sqrt(),
                    Int => x.floor(),
                    Fix => x.trunc(),
                    _ if x > 0.0 => 1.0,
                    _ if x < 0.0 => -1.0,
                    _ => 0.0,
                })
            }
            Rnd => {
                // RND(x): x > 0 next number, x = 0 repeat the last, x < 0 reseed
                let x = match args.first() {
                    Some(arg) => self.eval_num(arg)?,
                    None => 1.0,
                };
                if x < 0.0 {
                    self.rng = Rng::new(x.to_bits());
                }
                if x != 0.0 {
                    self.last_rnd = self.rng.next();
                }
                Value::Num(self.last_rnd)
            }
            Val => Value::Num(val(&self.eval_str(&args[0])?)),
            Len => Value::Num(self.eval_str(&args[0])?.chars().count() as f64),
            Asc => {
                let s = self.eval_str(&args[0])?;
                Value::Num(s.chars().next().ok_or_else(|| illegal(self))? as u32 as f64)
            }
            Chr => {
                let code = self.eval_count(&args[0])?;
                let c = char::from_u32(code as u32).ok_or_else(|| illegal(self))?;
                Value::Str(c.to_string())
            }
            Str => {
                let n = self.eval_num(&args[0])?;
                Value::Str(signed(n))
            }
            Left => {
                let s = self.eval_str(&args[0])?;
                let n = self.eval_count(&args[1])?;
                Value::Str(s.chars().take(n).collect())
            }
            Right => {
                let s = self.eval_str(&args[0])?;
                let n = self.eval_count(&args[1])?;
                let len = s.chars().count();
                Value::Str(s.chars().skip(len.saturating_sub(n)).collect())
            }
            Mid => {
                let s = self.eval_str(&args[0])?;
                let start = self.eval_count(&args[1])?;
                if start == 0 {
                    return Err(illegal(self));
                }
                let len = match args.get(2) {
                    Some(arg) => self.eval_count(arg)?,
                    None => usize::MAX,
                };
                Value::Str(s.chars().skip(start - 1).take(len).collect())
            }
        })
    }
}
