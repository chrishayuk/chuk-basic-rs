//! Compiles a BASIC program to a WebAssembly module.
//!
//! The program's arithmetic and control flow become WebAssembly
//! instructions; strings, PRINT and INPUT, arrays and the built-in functions
//! are calls into a prebuilt runtime module (the `chuk-basic-rt` crate, in
//! `runtime/`), which the compiler copies into its output. The result is one
//! self-contained module for any WASI runtime:
//!
//! ```text
//! chuk-basic --compile hello.bas        writes hello.wasm
//! wasmtime hello.wasm
//! ```
//!
//! How a program is laid out:
//!
//! - The program is first lowered to the same flat list of operations that
//!   the interpreter executes (`crate::interp::lower`), in which IF and
//!   WHILE are already jumps.
//! - All of it becomes one function, `run`. BASIC can jump anywhere, so the
//!   function is a loop around a `br_table` on a "program counter" local,
//!   with one block per operation; falling off the end of an operation's
//!   code runs the next one, and a jump sets the counter and goes round
//!   the loop.
//! - Numeric variables live in memory at fixed offsets and are read and
//!   written directly. Strings are handles managed by the runtime.
//! - Each `DEF FN` becomes a function of its own.
//!
//! Because the runtime is the interpreter's own (`crate::rt`), a compiled
//! program prints what the interpreter prints; the tests hold the two to
//! that for every sample program.

mod code;
mod expr;
mod func;
mod gen;
mod leb;
mod module;
mod stmt;

use crate::error::{Error, Result};
use crate::parser::parse;

/// Compile `source`, linking it against the runtime module `runtime` (the
/// bytes of `chuk_basic_rt.wasm`).
pub fn compile(source: &str, runtime: &[u8]) -> Result<Vec<u8>> {
    let lowered = crate::interp::lower::lower(parse(source)?)?;
    gen::Gen::new(lowered, runtime)
        .and_then(gen::Gen::finish)
        .map_err(|message| Error::syntax(message, None, None))
}
