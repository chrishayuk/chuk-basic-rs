//! Interactive mode: type numbered lines to build a program, then `RUN` it.
//!
//! ```text
//! 10 PRINT "HELLO"
//! 20 GOTO 10          a numbered line is stored (retyping replaces it;
//! 20                   a number on its own deletes it)
//! LIST                show the program          LIST 10-50
//! RUN                 run it
//! PRINT 2 + 2         anything else is run at once
//! NEW                 forget the program
//! LOAD "FILE.BAS"     SAVE "FILE.BAS"
//! BYE                 leave
//! ```

use std::collections::BTreeMap;
use std::io::{BufRead, Write};

use crate::error::{Error, ErrorKind, Result};
use crate::interp::Interpreter;
use crate::parser::parse;

const HELP: &str = "\
Type a line starting with a number to add it to the program:
  10 PRINT \"HELLO\"     store line 10 (typing it again replaces it)
  10                   delete line 10
Commands:
  LIST [from[-to]]     show the program, or part of it
  RUN                  run the program
  NEW                  forget the program
  LOAD \"file\"          read a program from a file
  SAVE \"file\"          write the program to a file
  HELP                 this text
  BYE                  leave
Anything else is run straight away, for example: PRINT 2 + 2
";

/// Settings for programs run from the prompt.
#[derive(Debug, Clone, Default)]
pub struct Options {
    /// Seed for RND; `None` gives a different sequence on every RUN.
    pub seed: Option<u64>,
    /// Stop a program after this many operations.
    pub step_limit: Option<u64>,
    /// Copy lines read by INPUT to the output.
    pub echo: bool,
}

/// The program being edited: its lines as typed, by line number.
#[derive(Default)]
struct Listing(BTreeMap<u32, String>);

impl Listing {
    fn source(&self) -> String {
        self.0
            .iter()
            .map(|(number, text)| format!("{number} {text}\n"))
            .collect()
    }

    /// Store (or, with nothing after the number, delete) a numbered line.
    fn enter(&mut self, number: u32, text: &str) {
        if text.trim().is_empty() {
            self.0.remove(&number);
        } else {
            self.0.insert(number, text.to_string());
        }
    }

    /// Replace the listing with the program in `source`. Every line must
    /// start with a line number.
    fn load(&mut self, source: &str) -> std::result::Result<(), String> {
        let mut lines = BTreeMap::new();
        for (i, line) in source
            .lines()
            .enumerate()
            .filter(|(_, line)| !line.trim().is_empty())
        {
            match numbered(line) {
                Some((number, text)) if !text.trim().is_empty() => {
                    lines.insert(number, text.to_string())
                }
                Some(_) => None,
                None => return Err(format!("Line {} of the file has no line number", i + 1)),
            };
        }
        self.0 = lines;
        Ok(())
    }
}

/// Split `10 PRINT X` into the number and the rest.
fn numbered(line: &str) -> Option<(u32, &str)> {
    let line = line.trim();
    let digits = line.len() - line.trim_start_matches(|c: char| c.is_ascii_digit()).len();
    let number = line[..digits].parse().ok()?;
    // keep any indentation beyond the one space that separates the number
    let text = &line[digits..];
    Some((number, text.strip_prefix(' ').unwrap_or(text)))
}

/// The text between double quotes in `"NAME"` (the quotes are optional).
fn file_name(argument: &str) -> Option<&str> {
    let name = argument.trim().trim_matches('"').trim();
    (!name.is_empty()).then_some(name)
}

/// The range of a `LIST` command: nothing, `n`, `n-`, `-m` or `n-m`.
fn list_range(argument: &str) -> Option<(u32, u32)> {
    let argument = argument.trim();
    if argument.is_empty() {
        return Some((0, u32::MAX));
    }
    let bound = |text: &str, default: u32| match text.trim() {
        "" => Some(default),
        text => text.parse().ok(),
    };
    match argument.split_once('-') {
        Some((from, to)) => Some((bound(from, 0)?, bound(to, u32::MAX)?)),
        None => argument.parse().ok().map(|line| (line, line)),
    }
}

/// Run the interactive prompt until `BYE` or the end of the input.
pub fn repl(input: &mut dyn BufRead, output: &mut dyn Write, options: &Options) -> Result<()> {
    let io = |result: std::io::Result<()>| result.map_err(|e| Error::io(&e));
    let mut listing = Listing::default();
    io(writeln!(
        output,
        "CHUK-BASIC {} - type HELP for help",
        env!("CARGO_PKG_VERSION")
    ))?;
    io(writeln!(output, "Ok"))?;
    loop {
        io(output.flush())?;
        let mut line = String::new();
        if input.read_line(&mut line).map_err(|e| Error::io(&e))? == 0 {
            return Ok(());
        }
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some((number, text)) = numbered(line) {
            listing.enter(number, text);
            continue;
        }

        let (word, argument) = line.split_once(char::is_whitespace).unwrap_or((line, ""));
        let said = match word.to_ascii_uppercase().as_str() {
            "BYE" | "QUIT" | "EXIT" | "SYSTEM" => return Ok(()),
            "HELP" => io(write!(output, "{HELP}")),
            "NEW" => {
                listing = Listing::default();
                Ok(())
            }
            "LIST" => match list_range(argument) {
                Some((from, to)) => {
                    let shown = listing
                        .0
                        .range(from..=to)
                        .map(|(number, text)| format!("{number} {text}\n"));
                    io(write!(output, "{}", shown.collect::<String>()))
                }
                None => io(writeln!(
                    output,
                    "?LIST takes a line number or a range, such as LIST 10-50"
                )),
            },
            "LOAD" | "SAVE" => {
                let outcome = match (word.to_ascii_uppercase().as_str(), file_name(argument)) {
                    (_, None) => Err(format!(
                        "{} needs a file name in quotes",
                        word.to_ascii_uppercase()
                    )),
                    ("LOAD", Some(name)) => std::fs::read_to_string(name)
                        .map_err(|e| format!("{name}: {e}"))
                        .and_then(|source| listing.load(&source)),
                    (_, Some(name)) => {
                        std::fs::write(name, listing.source()).map_err(|e| format!("{name}: {e}"))
                    }
                };
                match outcome {
                    Ok(()) => Ok(()),
                    Err(message) => io(writeln!(output, "?{message}")),
                }
            }
            // RUN runs the program; anything else is a statement to run now
            _ => {
                let source = if word.eq_ignore_ascii_case("RUN") {
                    listing.source()
                } else {
                    line.to_string()
                };
                match execute(&source, input, output, options) {
                    // the keyboard or screen has gone: nothing more to do
                    Err(error) if matches!(error.kind, ErrorKind::Io(_)) => return Err(error),
                    Err(error) => io(writeln!(output, "?{error}")),
                    Ok(()) => Ok(()),
                }
            }
        };
        said?;
        io(writeln!(output, "Ok"))?;
    }
}

fn execute(
    source: &str,
    input: &mut dyn BufRead,
    output: &mut dyn Write,
    options: &Options,
) -> Result<()> {
    let mut interpreter = Interpreter::new(parse(source)?, input, output, options.seed)?;
    interpreter.set_step_limit(options.step_limit);
    interpreter.set_echo_input(options.echo);
    let result = interpreter.run();
    // finish a line that the program left half printed
    let mid_line = interpreter.column() > 0;
    drop(interpreter);
    if mid_line {
        writeln!(output).map_err(|e| Error::io(&e))?;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session(typed: &str) -> String {
        let mut output = Vec::new();
        repl(
            &mut typed.as_bytes(),
            &mut output,
            &Options {
                seed: Some(1),
                ..Options::default()
            },
        )
        .unwrap();
        let text = String::from_utf8(output).unwrap();
        text.split_once('\n').unwrap().1.to_string() // drop the banner
    }

    #[test]
    fn stores_lists_and_runs_a_program() {
        let out = session("20 PRINT \"WORLD\"\n10 PRINT \"HELLO\"\nLIST\nRUN\n");
        assert_eq!(
            out,
            "Ok\n10 PRINT \"HELLO\"\n20 PRINT \"WORLD\"\nOk\nHELLO\nWORLD\nOk\n"
        );
    }

    #[test]
    fn lines_can_be_replaced_deleted_and_listed_by_range() {
        let out =
            session("10 A=1\n20 B=2\n30 C=3\n20 B=9\n10\nLIST\nlist 30\nLIST -20\nLIST 25-\n");
        assert_eq!(
            out,
            "Ok\n20 B=9\n30 C=3\nOk\n30 C=3\nOk\n20 B=9\nOk\n30 C=3\nOk\n"
        );
    }

    #[test]
    fn statements_without_a_number_run_at_once() {
        assert_eq!(session("PRINT 2+2\n? \"HI\"\n"), "Ok\n 4 \nOk\nHI\nOk\n");
    }

    #[test]
    fn a_running_program_reads_its_input_from_the_same_keyboard() {
        let out = session("10 INPUT \"NAME\";N$\n20 PRINT \"HI \";N$\nRUN\nADA\nPRINT 1\n");
        assert_eq!(out, "Ok\nNAME? HI ADA\nOk\n 1 \nOk\n");
    }

    #[test]
    fn errors_are_reported_and_the_prompt_carries_on() {
        let out = session("10 PRINT 1/0\nRUN\nFOO BAR\nNEW\nLIST\nBYE\nPRINT 1\n");
        assert_eq!(
            out,
            "Ok\n?Runtime error in line 10: Division by zero\nOk\n\
             ?Syntax error at source line 1: Expected '=', found 'BAR'\nOk\nOk\nOk\n"
        );
    }

    #[test]
    fn load_needs_numbered_lines() {
        let mut listing = Listing::default();
        listing.load("10 PRINT 1\n\n20 END\n").unwrap();
        assert_eq!(listing.source(), "10 PRINT 1\n20 END\n");
        assert!(listing.load("PRINT 1\n").is_err());
        assert_eq!(listing.source(), "10 PRINT 1\n20 END\n"); // unchanged by the failed load
    }
}
