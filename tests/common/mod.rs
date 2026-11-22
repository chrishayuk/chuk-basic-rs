//! Helpers shared by the integration tests.
//!
//! Files are opened by paths relative to the package root (where `cargo
//! test` runs), so the same tests work under a WASI runtime that has only
//! been given that directory.
#![allow(dead_code)]

use std::cell::RefCell;
use std::io::{BufReader, Read, Write};
use std::path::Path;
use std::rc::Rc;

use chuk_basic::{Error, Interpreter};

/// The seed used wherever a test needs repeatable random numbers.
pub const SEED: u64 = 42;

/// How a test run is set up.
pub struct Options<'a> {
    pub input: &'a str,
    pub seed: u64,
    pub echo: bool,
    pub step_limit: Option<u64>,
}

impl Default for Options<'_> {
    fn default() -> Self {
        Options {
            input: "",
            seed: SEED,
            echo: false,
            step_limit: Some(50_000_000),
        }
    }
}

/// Run a program; returns everything printed and how the run ended.
pub fn run_opts(source: &str, options: &Options) -> (String, Result<(), Error>) {
    let mut output = Vec::new();
    let mut input = options.input.as_bytes();
    let result = chuk_basic::parse(source).and_then(|program| {
        let mut interpreter =
            Interpreter::new(program, &mut input, &mut output, Some(options.seed))?;
        interpreter.set_echo_input(options.echo);
        interpreter.set_step_limit(options.step_limit);
        interpreter.run()
    });
    (String::from_utf8_lossy(&output).into_owned(), result)
}

/// Run a program with the given keyboard input.
pub fn run_with(source: &str, input: &str) -> (String, Result<(), Error>) {
    run_opts(
        source,
        &Options {
            input,
            ..Options::default()
        },
    )
}

/// Run a program that must succeed; returns everything printed.
pub fn run(source: &str) -> String {
    let (output, result) = run_with(source, "");
    result.unwrap_or_else(|e| panic!("{e}\noutput so far:\n{output}"));
    output
}

/// Run a program that must fail; returns the error.
pub fn run_err(source: &str) -> Error {
    run_with(source, "").1.expect_err("program should fail")
}

/// Read a file below the package root.
pub fn read(path: impl AsRef<Path>) -> String {
    let path = path.as_ref();
    let bytes = std::fs::read(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    String::from_utf8_lossy(&bytes).into_owned()
}

/// The `.bas` files directly inside `dir`, sorted by name.
pub fn programs(dir: &str) -> Vec<std::path::PathBuf> {
    let mut found: Vec<_> = std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("{dir}: {e}"))
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "bas"))
        .collect();
    found.sort();
    assert!(!found.is_empty(), "no programs in {dir}");
    found
}

/// Play a program interactively: each time it asks for input, `player` is
/// shown everything printed so far and returns the line to type (or `None`
/// to stop typing). Returns the transcript and how the run ended.
pub fn play(
    source: &str,
    seed: u64,
    mut player: impl FnMut(&str) -> Option<String>,
) -> (String, Result<(), Error>) {
    struct Screen(Rc<RefCell<Vec<u8>>>);
    impl Write for Screen {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0.borrow_mut().extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    struct Keyboard<F>(Rc<RefCell<Vec<u8>>>, F);
    impl<F: FnMut(&str) -> Option<String>> Read for Keyboard<F> {
        fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
            let seen = String::from_utf8_lossy(&self.0.borrow()).into_owned();
            let Some(line) = (self.1)(&seen) else {
                return Ok(0);
            };
            let line = format!("{line}\n");
            buffer[..line.len()].copy_from_slice(line.as_bytes());
            Ok(line.len())
        }
    }

    let screen = Rc::new(RefCell::new(Vec::new()));
    let mut output = Screen(Rc::clone(&screen));
    let mut input = BufReader::new(Keyboard(Rc::clone(&screen), &mut player));
    let result = chuk_basic::parse(source).and_then(|program| {
        let mut interpreter = Interpreter::new(program, &mut input, &mut output, Some(seed))?;
        interpreter.set_echo_input(true);
        interpreter.set_step_limit(Some(50_000_000));
        interpreter.run()
    });
    let transcript = String::from_utf8_lossy(&screen.borrow()).into_owned();
    (transcript, result)
}
