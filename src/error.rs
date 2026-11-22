//! Error type shared by the lexer, parser and interpreter.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    /// The program text is not valid BASIC.
    Syntax,
    /// The program failed while running.
    Runtime,
    /// Reading input or writing output failed.
    Io(std::io::ErrorKind),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Error {
    pub kind: ErrorKind,
    pub message: String,
    /// BASIC line number, when the line has one.
    pub line: Option<u32>,
    /// 1-based line in the source file, when known.
    pub src_line: Option<usize>,
}

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    pub fn syntax(message: impl Into<String>, line: Option<u32>, src_line: Option<usize>) -> Self {
        Error {
            kind: ErrorKind::Syntax,
            message: message.into(),
            line,
            src_line,
        }
    }

    pub fn runtime(message: impl Into<String>, line: Option<u32>) -> Self {
        Error {
            kind: ErrorKind::Runtime,
            message: message.into(),
            line,
            src_line: None,
        }
    }
}

impl Error {
    /// A failure of the input or output stream.
    pub fn io(error: &std::io::Error) -> Self {
        Error {
            kind: ErrorKind::Io(error.kind()),
            message: error.to_string(),
            line: None,
            src_line: None,
        }
    }

    /// Fill in the BASIC line number, if the error does not have one yet.
    pub fn at_line(self, line: Option<u32>) -> Self {
        Error {
            line: self.line.or(line),
            ..self
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let kind = match self.kind {
            ErrorKind::Syntax => "Syntax error",
            ErrorKind::Runtime => "Runtime error",
            ErrorKind::Io(_) => "I/O error",
        };
        match (self.line, self.src_line) {
            (Some(line), _) => write!(f, "{kind} in line {line}: {}", self.message),
            (None, Some(src)) => write!(f, "{kind} at source line {src}: {}", self.message),
            (None, None) => write!(f, "{kind}: {}", self.message),
        }
    }
}

impl std::error::Error for Error {}

/// Adds the line number to an error raised by code that does not know it.
pub(crate) trait AtLine<T> {
    fn at(self, line: Option<u32>) -> Result<T>;
}

impl<T> AtLine<T> for Result<T> {
    fn at(self, line: Option<u32>) -> Result<T> {
        self.map_err(|error| error.at_line(line))
    }
}
