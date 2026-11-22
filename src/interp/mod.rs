//! Interpreter: lowers the AST to a flat list of ops and executes it.
//!
//! Semantics follow Microsoft / Vintage BASIC where the dialects differ:
//! numbers print with a sign column and a trailing space, print zones are 14
//! columns wide, true is -1, `AND` / `OR` / `NOT` are bitwise, and arrays are
//! auto-dimensioned to 10. `FOR` follows Dartmouth / ANSI: a loop whose range
//! is empty is skipped entirely.

mod builtins;
mod compile;
mod control;
mod eval;
mod exec;
mod functions;
mod io;
mod mat;
mod vars;

use std::collections::HashMap;
use std::io::{BufRead, Write};
use std::rc::Rc;

use crate::ast::Program;
use crate::error::{AtLine, Error, Result};
use crate::rt::{Arrays, Console, DataPool, Rnd, Value};
use compile::{compile, Func, Op};

/// Each BASIC call nests several interpreter frames on the native stack, so
/// this is kept well inside a 1 MiB WebAssembly stack.
const MAX_FN_DEPTH: usize = 64;
const MAX_GOSUB_DEPTH: usize = 100_000;

struct ForFrame {
    var: String,
    limit: f64,
    step: f64,
    /// First op of the loop body.
    body: usize,
}

struct GosubFrame {
    ret: usize,
    /// Height of the FOR stack on entry; loops opened inside the subroutine
    /// are discarded by RETURN.
    for_base: usize,
}

enum Flow {
    Next,
    Jump(usize),
    Halt,
}

enum Exit {
    Halt,
    FnReturn,
}

pub struct Interpreter<'a> {
    ops: Rc<Vec<Op>>,
    op_line: Vec<Option<u32>>,
    line_map: HashMap<u32, usize>,
    funcs: HashMap<String, Rc<Func>>,

    vars: HashMap<String, Value>,
    arrays: Arrays,
    data: DataPool,
    console: Console<'a>,
    rnd: Rnd,

    for_stack: Vec<ForFrame>,
    gosub_stack: Vec<GosubFrame>,
    /// Stack heights on entry to the running multi-line function.
    fn_gosub_base: usize,
    fn_for_base: usize,
    fn_depth: usize,
    halted: bool,
    /// Ops executed so far, and the most that may be executed.
    steps: u64,
    step_limit: Option<u64>,
    /// The line being executed, for error messages.
    line: Option<u32>,
}

impl<'a> Interpreter<'a> {
    /// Prepare `program` for execution. `seed` fixes the random number
    /// sequence; `None` seeds from the clock.
    pub fn new(
        program: Program,
        input: &'a mut dyn BufRead,
        output: &'a mut dyn Write,
        seed: Option<u64>,
    ) -> Result<Self> {
        let c = compile(program)?;
        Ok(Interpreter {
            ops: Rc::new(c.ops),
            op_line: c.op_line,
            line_map: c.line_map,
            funcs: c.funcs,
            vars: HashMap::new(),
            arrays: Arrays::default(),
            data: c.data,
            console: Console::new(input, output),
            rnd: Rnd::new(seed),
            for_stack: Vec::new(),
            gosub_stack: Vec::new(),
            fn_gosub_base: 0,
            fn_for_base: 0,
            fn_depth: 0,
            halted: false,
            steps: 0,
            step_limit: None,
            line: None,
        })
    }

    /// Stop with an error once `limit` operations have run. Useful for
    /// running untrusted programs, which may loop forever.
    pub fn set_step_limit(&mut self, limit: Option<u64>) {
        self.step_limit = limit;
    }

    /// Copy every line read by INPUT to the output. A terminal already shows
    /// what is typed; this makes the transcript read the same way when input
    /// comes from a file or a pipe.
    pub fn set_echo_input(&mut self, echo: bool) {
        self.console.set_echo(echo);
    }

    /// The column the output cursor is in (0 at the start of a line).
    pub fn column(&self) -> usize {
        self.console.column()
    }

    /// Run the program to completion.
    pub fn run(&mut self) -> Result<()> {
        let result = self.exec(0, false).map(|_| ());
        let flushed = self.console.flush().at(self.line);
        result.and(flushed)
    }

    fn err(&self, message: impl Into<String>) -> Error {
        Error::runtime(message, self.line)
    }

    fn type_mismatch(&self) -> Error {
        self.err("Type mismatch")
    }
}
