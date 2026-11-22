//! PRINT and INPUT.

use super::format::{parse_number, signed};
use super::value::Value;
use super::{Interpreter, ZONE_WIDTH};

/// Widest gap a single TAB or SPC may produce.
const MAX_SPACING: f64 = 255.0;
use crate::ast::{Expr, LValue, PrintItem};
use crate::error::Result;

impl Interpreter<'_> {
    pub(super) fn write(&mut self, text: &str) -> Result<()> {
        self.output
            .write_all(text.as_bytes())
            .map_err(|e| self.io_err(e))?;
        match text.rfind('\n') {
            Some(at) => self.col = text[at + 1..].chars().count(),
            None => self.col += text.chars().count(),
        }
        Ok(())
    }

    pub(super) fn pad_to(&mut self, col: usize) -> Result<()> {
        if col > self.col {
            self.write(&" ".repeat(col - self.col))?;
        }
        Ok(())
    }

    /// The argument of TAB or SPC. Old programs compute these freely, so
    /// out-of-range values are clamped rather than treated as errors.
    fn eval_spacing(&mut self, expr: &Expr) -> Result<usize> {
        Ok(self.eval_num(expr)?.round().clamp(0.0, MAX_SPACING) as usize)
    }

    pub(super) fn print(&mut self, items: &[PrintItem]) -> Result<()> {
        for item in items {
            match item {
                PrintItem::Expr(expr) => {
                    let text = match self.eval(expr)? {
                        Value::Num(n) => signed(n) + " ",
                        Value::Str(s) => s,
                    };
                    self.write(&text)?;
                }
                PrintItem::Tab(expr) => {
                    let col = self.eval_spacing(expr)?;
                    self.pad_to(col)?;
                }
                PrintItem::Spc(expr) => {
                    let n = self.eval_spacing(expr)?;
                    self.write(&" ".repeat(n))?;
                }
                PrintItem::Comma => self.pad_to((self.col / ZONE_WIDTH + 1) * ZONE_WIDTH)?,
                PrintItem::Semi => {}
            }
        }
        if !matches!(items.last(), Some(PrintItem::Comma | PrintItem::Semi)) {
            self.write("\n")?;
        }
        Ok(())
    }

    pub(super) fn read_line(&mut self) -> Result<String> {
        self.output.flush().map_err(|e| self.io_err(e))?;
        let mut line = String::new();
        let n = self
            .input
            .read_line(&mut line)
            .map_err(|e| self.io_err(e))?;
        if n == 0 {
            return Err(self.err("Out of input"));
        }
        let line = line.trim_end_matches(['\r', '\n']).to_string();
        if self.echo_input {
            self.write(&line)?;
            self.write("\n")?;
        }
        self.col = 0; // the user's RETURN moved the cursor to a new line
        Ok(line)
    }

    pub(super) fn input(
        &mut self,
        prompt: Option<&str>,
        question_mark: bool,
        targets: &[LValue],
    ) -> Result<()> {
        'retry: loop {
            if let Some(prompt) = prompt {
                self.write(prompt)?;
            }
            if question_mark {
                self.write("? ")?;
            }
            let line = self.read_line()?;

            // a lone string variable takes the whole line, commas and all
            let mut fields = match targets {
                [only] if only.name.ends_with('$') => vec![unquote(&line)],
                _ => split_fields(&line),
            };
            while fields.len() < targets.len() {
                self.write("?? ")?;
                let more = self.read_line()?;
                fields.extend(split_fields(&more));
            }
            if fields.len() > targets.len() {
                self.write("?EXTRA IGNORED\n")?;
            }

            let mut values = Vec::with_capacity(targets.len());
            for (target, field) in targets.iter().zip(fields) {
                if target.name.ends_with('$') {
                    values.push(Value::Str(field));
                } else if field.is_empty() {
                    values.push(Value::Num(0.0));
                } else if let Some(n) = parse_number(&field) {
                    values.push(Value::Num(n));
                } else {
                    self.write("?REDO FROM START\n")?;
                    continue 'retry;
                }
            }
            for (target, value) in targets.iter().zip(values) {
                self.assign(target, value)?;
            }
            return Ok(());
        }
    }
}

fn unquote(field: &str) -> String {
    let field = field.trim();
    match field.strip_prefix('"') {
        Some(rest) => rest.strip_suffix('"').unwrap_or(rest).to_string(),
        None => field.to_string(),
    }
}

/// Split a line typed at INPUT into comma-separated fields; commas inside
/// double quotes do not split.
fn split_fields(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut quoted = false;
    for c in line.chars() {
        match c {
            '"' => {
                quoted = !quoted;
                current.push(c);
            }
            ',' if !quoted => fields.push(unquote(&std::mem::take(&mut current))),
            _ => current.push(c),
        }
    }
    fields.push(unquote(&current));
    fields
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_fields() {
        assert_eq!(split_fields("1, 2 ,three"), vec!["1", "2", "three"]);
        assert_eq!(split_fields("\"a,b\",c"), vec!["a,b", "c"]);
        assert_eq!(split_fields(""), vec![""]);
    }
}
