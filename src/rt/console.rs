//! The terminal: PRINT's cursor, print zones, and INPUT.

use std::io::{BufRead, Write};

use super::fail;
use super::format::{parse_number, signed};
use super::value::Value;
use crate::error::{Error, Result};

/// Width of the print zones that a comma in PRINT moves between.
pub const ZONE_WIDTH: usize = 14;

/// Widest gap a single TAB or SPC may produce.
const MAX_SPACING: f64 = 255.0;

pub struct Console<'a> {
    input: &'a mut dyn BufRead,
    output: &'a mut dyn Write,
    /// Current output column, for TAB and print zones.
    col: usize,
    /// Copy each line read from the keyboard to the output.
    echo: bool,
}

impl<'a> Console<'a> {
    pub fn new(input: &'a mut dyn BufRead, output: &'a mut dyn Write) -> Self {
        Console {
            input,
            output,
            col: 0,
            echo: false,
        }
    }

    /// Copy every line that is read to the output. A terminal already shows
    /// what is typed; this makes a transcript read the same way when input
    /// comes from a file or a pipe.
    pub fn set_echo(&mut self, echo: bool) {
        self.echo = echo;
    }

    /// The column the cursor is in (0 at the start of a line).
    pub fn column(&self) -> usize {
        self.col
    }

    pub fn flush(&mut self) -> Result<()> {
        self.output.flush().map_err(|e| Error::io(&e))
    }

    // ---- output ------------------------------------------------------------

    pub fn write(&mut self, text: &str) -> Result<()> {
        self.output
            .write_all(text.as_bytes())
            .map_err(|e| Error::io(&e))?;
        match text.rfind('\n') {
            Some(at) => self.col = text[at + 1..].chars().count(),
            None => self.col += text.chars().count(),
        }
        Ok(())
    }

    pub fn newline(&mut self) -> Result<()> {
        self.write("\n")
    }

    /// Print a value the way PRINT does: a number gets a sign column in
    /// front and a space after.
    pub fn print_value(&mut self, value: &Value) -> Result<()> {
        match value {
            Value::Num(n) => self.write(&(signed(*n) + " ")),
            Value::Str(s) => self.write(s),
        }
    }

    fn pad_to(&mut self, col: usize) -> Result<()> {
        if col > self.col {
            self.write(&" ".repeat(col - self.col))?;
        }
        Ok(())
    }

    /// A comma in PRINT: move to the start of the next zone.
    pub fn next_zone(&mut self) -> Result<()> {
        self.pad_to((self.col / ZONE_WIDTH + 1) * ZONE_WIDTH)
    }

    /// `TAB(n)`: move right to column `n` (counting from 0); no effect if
    /// the cursor is already there or beyond. Old programs compute these
    /// freely, so out-of-range values are clamped rather than errors.
    pub fn tab(&mut self, n: f64) -> Result<()> {
        self.pad_to(n.round().clamp(0.0, MAX_SPACING) as usize)
    }

    /// `SPC(n)`: print `n` spaces.
    pub fn spc(&mut self, n: f64) -> Result<()> {
        self.write(&" ".repeat(n.round().clamp(0.0, MAX_SPACING) as usize))
    }

    // ---- input -------------------------------------------------------------

    /// Read one line from the keyboard, without its line ending.
    pub fn read_line(&mut self) -> Result<String> {
        self.flush()?;
        let mut line = String::new();
        let n = self.input.read_line(&mut line).map_err(|e| Error::io(&e))?;
        if n == 0 {
            return fail("Out of input");
        }
        let line = line.trim_end_matches(['\r', '\n']).to_string();
        if self.echo {
            self.write(&line)?;
            self.newline()?;
        }
        self.col = 0; // the user's RETURN moved the cursor to a new line
        Ok(line)
    }

    /// `INPUT`: prompt, then read one value for each entry of `strings`
    /// (true where a string is wanted, false for a number). Asks again
    /// until the answers make sense.
    pub fn input(
        &mut self,
        prompt: Option<&str>,
        question_mark: bool,
        strings: &[bool],
    ) -> Result<Vec<Value>> {
        'retry: loop {
            if let Some(prompt) = prompt {
                self.write(prompt)?;
            }
            if question_mark {
                self.write("? ")?;
            }
            let line = self.read_line()?;

            // a lone string variable takes the whole line, commas and all
            let mut fields = match strings {
                [true] => vec![unquote(&line)],
                _ => split_fields(&line),
            };
            while fields.len() < strings.len() {
                self.write("?? ")?;
                let more = self.read_line()?;
                fields.extend(split_fields(&more));
            }
            if fields.len() > strings.len() {
                self.write("?EXTRA IGNORED\n")?;
            }

            let mut values = Vec::with_capacity(strings.len());
            for (&string, field) in strings.iter().zip(fields) {
                if string {
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
            return Ok(values);
        }
    }

    /// `LINE INPUT`: prompt (with no question mark) and read a whole line
    /// exactly as typed.
    pub fn line_input(&mut self, prompt: Option<&str>) -> Result<String> {
        if let Some(prompt) = prompt {
            self.write(prompt)?;
        }
        self.read_line()
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

    #[test]
    fn zones_and_tabs_follow_the_cursor() {
        let mut out = Vec::new();
        let mut input = std::io::empty();
        let mut console = Console::new(&mut input, &mut out);
        console.write("AB").unwrap();
        console.next_zone().unwrap();
        console.write("C").unwrap();
        console.tab(20.0).unwrap();
        console.tab(3.0).unwrap(); // already past: no effect
        console.print_value(&Value::Num(-1.5)).unwrap();
        console.spc(2.0).unwrap();
        console.print_value(&Value::Num(2.0)).unwrap();
        console.newline().unwrap();
        assert_eq!(
            String::from_utf8(out).unwrap(),
            "AB            C     -1.5    2 \n"
        );
    }
}
