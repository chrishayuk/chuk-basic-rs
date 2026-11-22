//! CHUK-BASIC: an interpreter for classic line-numbered BASIC, covering the
//! Dartmouth, Vintage and Microsoft (Altair) flavours.
//!
//! The pipeline is `lexer` -> `parser` -> `ast` -> `interp`. The crate has no
//! dependencies and builds for `wasm32-wasip1` as well as native targets.
//!
//! ```
//! let mut output = Vec::new();
//! chuk_basic::run("10 PRINT \"HELLO\"", &mut std::io::empty(), &mut output, None).unwrap();
//! assert_eq!(output, b"HELLO\n");
//! ```

pub mod analysis;
pub mod ast;
pub mod error;
pub mod interp;
pub mod lexer;
pub mod lsp;
pub mod parser;
pub mod repl;
pub mod rt;
pub mod wasm;

use std::io::{BufRead, Write};

pub use error::{Error, ErrorKind, Result};
pub use interp::Interpreter;
pub use lexer::{tokenize, tokenize_line, Spacing};
pub use parser::parse;
pub use rt::Dialect;

/// Parse and run a program, reading INPUT from `input` and printing to
/// `output`. `seed` fixes the RND sequence; `None` seeds from the clock.
pub fn run(
    source: &str,
    input: &mut dyn BufRead,
    output: &mut dyn Write,
    seed: Option<u64>,
) -> Result<()> {
    Interpreter::new(parse(source)?, input, output, seed)?.run()
}

/// Compiles and runs the Rust examples in the README as doctests.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeExamples;
