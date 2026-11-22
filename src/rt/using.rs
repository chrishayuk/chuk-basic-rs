//! `PRINT USING`: formatting values to a picture such as `"###.##"`.
//!
//! A picture is made of fields and ordinary characters:
//!
//! | In the picture | Prints |
//! | --- | --- |
//! | `#` | a digit position; `.` marks the decimal point |
//! | `,` (before the point) | commas between thousands |
//! | `+` before or after | the sign, always |
//! | `-` after | a trailing minus for negative numbers |
//! | `**` | leading spaces filled with asterisks |
//! | `$$` | a dollar sign just before the number |
//! | `^^^^` | exponent form |
//! | `!` | the first character of a string |
//! | `\  \` | as many characters as the field is wide |
//! | `&` | the whole string |
//! | `_x` | the character `x` itself |
//!
//! A number that does not fit is printed in full with a `%` in front.

use super::value::Value;
use super::{fail, type_mismatch};
use crate::error::Result;

/// A numeric field.
#[derive(Debug, Default)]
struct Picture {
    /// Positions before the decimal point, counting `#`, `,`, `**` and `$$`.
    before: usize,
    /// Digits after the point; `None` if the picture has no point.
    decimals: Option<usize>,
    commas: bool,
    stars: bool,
    dollar: bool,
    exponent: bool,
    plus_before: bool,
    plus_after: bool,
    minus_after: bool,
}

enum Field {
    Number(Picture),
    FirstChar,
    Whole,
    Fixed(usize),
}

struct Scanner<'a> {
    chars: &'a [char],
    pos: usize,
}

impl Scanner<'_> {
    fn at(&self, offset: usize) -> Option<char> {
        self.chars.get(self.pos + offset).copied()
    }

    fn starts(&self, text: &str) -> bool {
        text.chars().enumerate().all(|(i, c)| self.at(i) == Some(c))
    }

    /// Does a numeric field start `offset` characters ahead?
    fn number_ahead(&self, offset: usize) -> bool {
        match self.at(offset) {
            Some('#') => true,
            Some('.') => self.at(offset + 1) == Some('#'),
            Some('*') => self.at(offset + 1) == Some('*'),
            Some('$') => self.at(offset + 1) == Some('$'),
            _ => false,
        }
    }

    /// The field that starts here, if one does (consuming it).
    fn field(&mut self) -> Option<Field> {
        match self.at(0)? {
            '!' => {
                self.pos += 1;
                Some(Field::FirstChar)
            }
            '&' => {
                self.pos += 1;
                Some(Field::Whole)
            }
            '\\' => {
                let spaces = self.chars[self.pos + 1..]
                    .iter()
                    .take_while(|&&c| c == ' ')
                    .count();
                (self.at(spaces + 1) == Some('\\')).then(|| {
                    self.pos += spaces + 2;
                    Field::Fixed(spaces + 2)
                })
            }
            '+' if self.number_ahead(1) => Some(Field::Number(self.number())),
            _ if self.number_ahead(0) => Some(Field::Number(self.number())),
            _ => None,
        }
    }

    fn number(&mut self) -> Picture {
        let mut p = Picture::default();
        if self.at(0) == Some('+') {
            p.plus_before = true;
            self.pos += 1;
        }
        if self.starts("**$") {
            (p.stars, p.dollar, p.before) = (true, true, 3);
            self.pos += 3;
        } else if self.starts("**") {
            (p.stars, p.before) = (true, 2);
            self.pos += 2;
        } else if self.starts("$$") {
            (p.dollar, p.before) = (true, 2);
            self.pos += 2;
        }
        while let Some(c @ ('#' | ',')) = self.at(0) {
            p.commas |= c == ',';
            p.before += 1;
            self.pos += 1;
        }
        if self.at(0) == Some('.') {
            self.pos += 1;
            let digits = self.chars[self.pos..]
                .iter()
                .take_while(|&&c| c == '#')
                .count();
            p.decimals = Some(digits);
            self.pos += digits;
        }
        if self.starts("^^^^") {
            p.exponent = true;
            self.pos += 4;
        }
        if !p.plus_before && self.at(0) == Some('+') {
            p.plus_after = true;
            self.pos += 1;
        } else if self.at(0) == Some('-') {
            p.minus_after = true;
            self.pos += 1;
        }
        p
    }

    /// Copy ordinary characters to `out` up to the next field, and return
    /// that field (or `None` at the end of the picture).
    fn literal_then_field(&mut self, out: &mut String) -> Option<Field> {
        loop {
            if let Some(field) = self.field() {
                return Some(field);
            }
            match self.at(0)? {
                // an underscore makes the next character ordinary
                '_' if self.at(1).is_some() => {
                    out.push(self.chars[self.pos + 1]);
                    self.pos += 2;
                }
                c => {
                    out.push(c);
                    self.pos += 1;
                }
            }
        }
    }
}

/// Format `values` according to `picture`. If there are more values than
/// fields, the picture is used again from the start.
pub fn format_using(picture: &str, values: &[Value]) -> Result<String> {
    let chars: Vec<char> = picture.chars().collect();
    let mut scanner = Scanner {
        chars: &chars,
        pos: 0,
    };
    let mut out = String::new();
    for value in values {
        let field = match scanner.literal_then_field(&mut out) {
            Some(field) => field,
            None => {
                // used up: go round again (a picture with no fields is an error)
                scanner.pos = 0;
                match scanner.literal_then_field(&mut out) {
                    Some(field) => field,
                    None => {
                        return fail("Illegal function call: PRINT USING needs a field such as #")
                    }
                }
            }
        };
        match (field, value) {
            (Field::Number(picture), Value::Num(n)) => out.push_str(&number(&picture, *n)),
            (Field::FirstChar, Value::Str(s)) => out.push(s.chars().next().unwrap_or(' ')),
            (Field::Whole, Value::Str(s)) => out.push_str(s),
            (Field::Fixed(width), Value::Str(s)) => {
                out.extend(s.chars().chain(std::iter::repeat(' ')).take(width));
            }
            _ => return type_mismatch(),
        }
    }
    // the text after the last field used, up to the next field
    scanner.literal_then_field(&mut out);
    Ok(out)
}

fn with_commas(digits: &str) -> String {
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

fn number(p: &Picture, n: f64) -> String {
    let negative = n < 0.0;
    let decimals = p.decimals.unwrap_or(0);
    let sign_after = p.plus_after || p.minus_after;

    let (mut digits, fraction, exponent) = if p.exponent {
        // one position before the point is kept for the sign unless the
        // sign has a place of its own
        let own_sign = p.plus_before || sign_after;
        let lead = if own_sign {
            p.before
        } else {
            p.before.saturating_sub(1)
        };
        let (mantissa, exp) = scientific(n.abs(), lead, decimals);
        let (whole, fraction) = mantissa.split_once('.').unwrap_or((&mantissa, ""));
        let sign = if exp < 0 { '-' } else { '+' };
        (
            whole.to_string(),
            fraction.to_string(),
            format!("E{sign}{:02}", exp.abs()),
        )
    } else {
        let fixed = format!("{:.decimals$}", n.abs());
        let (whole, fraction) = fixed.split_once('.').unwrap_or((&fixed, ""));
        (whole.to_string(), fraction.to_string(), String::new())
    };
    if p.commas && !p.exponent {
        digits = with_commas(&digits);
    }

    let mut lead = String::new();
    if p.plus_before {
        lead.push(if negative { '-' } else { '+' });
    } else if negative && !sign_after {
        lead.push('-');
    }
    if p.dollar {
        lead.push('$');
    }

    // room for the sign and the digits; a leading `+` has a place of its own
    let width = p.before + usize::from(p.plus_before);
    // a lone zero before the point is dropped when there is no room for it
    if digits == "0" && p.decimals.is_some() && lead.len() + 1 > width {
        digits.clear();
    }
    let mut body = lead + &digits;
    let overflow = body.chars().count() > width;
    if !overflow {
        let fill = if p.stars { "*" } else { " " };
        body = fill.repeat(width - body.chars().count()) + &body;
    }
    if p.decimals.is_some() {
        body.push('.');
        body.push_str(&fraction);
    }
    body.push_str(&exponent);
    if p.plus_after {
        body.push(if negative { '-' } else { '+' });
    } else if p.minus_after {
        body.push(if negative { '-' } else { ' ' });
    }
    if overflow {
        body.insert(0, '%');
    }
    body
}

/// `x` as a mantissa with `lead` digits before the point and `decimals`
/// after, and the power of ten that goes with it.
fn scientific(x: f64, lead: usize, decimals: usize) -> (String, i32) {
    if x == 0.0 || !x.is_finite() {
        return (format!("{:.decimals$}", 0.0), 0);
    }
    let lead = lead as i32;
    let mut exp = x.log10().floor() as i32 - (lead - 1);
    let mut mantissa = format!("{:.decimals$}", x / 10f64.powi(exp));
    // rounding can carry into an extra digit (9.99 -> 10.0)
    let whole = mantissa.split('.').next().unwrap_or("").len() as i32;
    if whole > lead.max(1) {
        exp += 1;
        mantissa = format!("{:.decimals$}", x / 10f64.powi(exp));
    }
    (mantissa, exp)
}
