//! The built-in functions, as plain functions of their argument values.

use super::dialect::Dialect;
use super::format::{signed, val};
use super::rng::Rnd;
use super::value::Value;
use super::{fail, type_mismatch};
use crate::ast::Builtin;
use crate::error::Result;

/// Longest string SPACE$ or STRING$ will build.
const MAX_STRING: usize = 65_535;

/// The arguments of one call, with typed access.
struct Args<'a> {
    builtin: Builtin,
    values: &'a [Value],
}

impl Args<'_> {
    fn illegal<T>(&self) -> Result<T> {
        fail(format!("Illegal function call: {}", self.builtin.name()))
    }

    fn num(&self, at: usize) -> Result<f64> {
        match &self.values[at] {
            Value::Num(n) => Ok(*n),
            Value::Str(_) => type_mismatch(),
        }
    }

    fn str(&self, at: usize) -> Result<&str> {
        match &self.values[at] {
            Value::Str(s) => Ok(s),
            Value::Num(_) => type_mismatch(),
        }
    }

    /// A count or position: rounded, as Microsoft BASIC does, and not
    /// negative.
    fn count(&self, at: usize) -> Result<usize> {
        let n = self.num(at)?.round();
        if !(0.0..=u32::MAX as f64).contains(&n) {
            return self.illegal();
        }
        Ok(n as usize)
    }
}

/// Call `builtin`. The parser has already checked the number of arguments.
pub fn call(builtin: Builtin, values: &[Value], rnd: &mut Rnd, dialect: Dialect) -> Result<Value> {
    use Builtin::*;
    let args = Args { builtin, values };
    Ok(match builtin {
        Sin | Cos | Tan | Atn | Exp | Abs | Log | Sqr | Int | Fix | Sgn => {
            let x = args.num(0)?;
            Value::Num(match builtin {
                Sin => x.sin(),
                Cos => x.cos(),
                Tan => x.tan(),
                Atn => x.atan(),
                Exp => x.exp(),
                Abs => x.abs(),
                Log if x <= 0.0 => return args.illegal(),
                Log => x.ln(),
                Sqr => match dialect.sqrt(x) {
                    Some(root) => root,
                    None => return args.illegal(),
                },
                Int => x.floor(),
                Fix => x.trunc(),
                _ if x > 0.0 => 1.0,
                _ if x < 0.0 => -1.0,
                _ => 0.0,
            })
        }
        // RND with no argument is RND(1)
        Rnd => Value::Num(rnd.rnd(if values.is_empty() { 1.0 } else { args.num(0)? })),
        Val => Value::Num(val(args.str(0)?)),
        Len => Value::Num(args.str(0)?.chars().count() as f64),
        Asc => match args.str(0)?.chars().next() {
            Some(c) => Value::Num(c as u32 as f64),
            None => return args.illegal(),
        },
        Chr => match char::from_u32(args.count(0)? as u32) {
            Some(c) => Value::Str(c.to_string()),
            None => return args.illegal(),
        },
        Str => Value::Str(signed(args.num(0)?)),
        Left => Value::Str(args.str(0)?.chars().take(args.count(1)?).collect()),
        Right => {
            let text = args.str(0)?;
            let len = text.chars().count();
            Value::Str(
                text.chars()
                    .skip(len.saturating_sub(args.count(1)?))
                    .collect(),
            )
        }
        Mid => {
            // MID$(text$, start [, length]), counting from 1
            let start = args.count(1)?;
            if start == 0 {
                return args.illegal();
            }
            let len = if values.len() > 2 {
                args.count(2)?
            } else {
                usize::MAX
            };
            Value::Str(args.str(0)?.chars().skip(start - 1).take(len).collect())
        }
        Instr => {
            // INSTR([start,] text$, find$): the position of find$, or 0
            let (start, text, find) = match values.len() {
                3 => (args.count(0)?, args.str(1)?, args.str(2)?),
                _ => (1, args.str(0)?, args.str(1)?),
            };
            if start == 0 {
                return args.illegal();
            }
            let text: Vec<char> = text.chars().collect();
            let find: Vec<char> = find.chars().collect();
            let found = if start > text.len() {
                None
            } else if find.is_empty() {
                Some(start - 1)
            } else {
                (start - 1..text.len()).find(|&at| text[at..].starts_with(&find))
            };
            Value::Num(found.map_or(0.0, |at| (at + 1) as f64))
        }
        Space => Value::Str(" ".repeat(args.count(0)?.min(MAX_STRING))),
        StringOf => {
            // STRING$(n, code) or STRING$(n, text$): n copies of one character
            let c = match &values[1] {
                Value::Str(s) => match s.chars().next() {
                    Some(c) => c,
                    None => return args.illegal(),
                },
                Value::Num(code) if (0.0..=255.0).contains(&code.round()) => {
                    char::from(code.round() as u8)
                }
                Value::Num(_) => return args.illegal(),
            };
            Value::Str(c.to_string().repeat(args.count(0)?.min(MAX_STRING)))
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn call1(builtin: Builtin, values: &[Value]) -> Result<Value> {
        call(builtin, values, &mut Rnd::new(Some(1)), Dialect::default())
    }

    fn s(text: &str) -> Value {
        Value::Str(text.to_string())
    }

    #[test]
    fn instr() {
        let n = |x: f64| Ok(Value::Num(x));
        assert_eq!(call1(Builtin::Instr, &[s("HELLO"), s("L")]), n(3.0));
        assert_eq!(
            call1(Builtin::Instr, &[Value::Num(4.0), s("HELLO"), s("L")]),
            n(4.0)
        );
        assert_eq!(call1(Builtin::Instr, &[s("HELLO"), s("Z")]), n(0.0));
        assert_eq!(call1(Builtin::Instr, &[s("HELLO"), s("")]), n(1.0));
        assert_eq!(
            call1(Builtin::Instr, &[Value::Num(9.0), s("HELLO"), s("L")]),
            n(0.0)
        );
        assert!(call1(Builtin::Instr, &[Value::Num(0.0), s("HELLO"), s("L")]).is_err());
    }

    #[test]
    fn string_builders() {
        assert_eq!(call1(Builtin::Space, &[Value::Num(3.0)]), Ok(s("   ")));
        assert_eq!(
            call1(Builtin::StringOf, &[Value::Num(3.0), Value::Num(42.0)]),
            Ok(s("***"))
        );
        assert_eq!(
            call1(Builtin::StringOf, &[Value::Num(2.0), s("AB")]),
            Ok(s("AA"))
        );
    }

    #[test]
    fn wrong_types_and_ranges_are_errors() {
        assert_eq!(
            call1(Builtin::Len, &[Value::Num(1.0)]).unwrap_err().message,
            "Type mismatch"
        );
        assert_eq!(
            call1(Builtin::Sqr, &[Value::Num(-1.0)])
                .unwrap_err()
                .message,
            "Illegal function call: SQR"
        );
        assert_eq!(
            call1(Builtin::Left, &[s("A"), Value::Num(-1.0)])
                .unwrap_err()
                .message,
            "Illegal function call: LEFT$"
        );
    }
}
