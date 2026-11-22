//! The state of a running compiled program.

use std::cell::RefCell;
use std::io::{BufReader, LineWriter};

use chuk_basic::error::{Error, ErrorKind, Result};
use chuk_basic::rt::{Arrays, Console, DataPool, Dialect, Rnd, Value};

/// Each BASIC call nests WebAssembly frames; kept the same as the
/// interpreter's limit so that both stop at the same depth.
pub const MAX_FN_DEPTH: usize = 64;
pub const MAX_GOSUB_DEPTH: usize = 100_000;

#[derive(Clone, Copy)]
pub struct ForFrame {
    /// The loop variable.
    pub slot: usize,
    pub limit: f64,
    pub step: f64,
    /// The first operation of the loop body.
    pub body: i32,
}

pub struct GosubFrame {
    pub ret: i32,
    /// Height of the FOR stack on entry; loops opened inside the subroutine
    /// are discarded by RETURN.
    pub for_base: usize,
}

/// Strings, by handle. Handle 0 is the empty string.
pub struct Strings {
    slab: Vec<String>,
    free: Vec<i32>,
    /// Handles to free when the statement that made them is over.
    temps: Vec<i32>,
}

impl Strings {
    fn new() -> Self {
        Strings {
            slab: vec![String::new()],
            free: Vec::new(),
            temps: Vec::new(),
        }
    }

    pub fn get(&self, handle: i32) -> &str {
        &self.slab[handle as usize]
    }

    /// Store a string for the rest of the run.
    pub fn keep(&mut self, text: String) -> i32 {
        match self.free.pop() {
            Some(handle) => {
                self.slab[handle as usize] = text;
                handle
            }
            None => {
                self.slab.push(text);
                (self.slab.len() - 1) as i32
            }
        }
    }

    /// Store a string until the current statement is over.
    pub fn temp(&mut self, text: String) -> i32 {
        let handle = self.keep(text);
        self.temps.push(handle);
        handle
    }

    pub fn mark(&self) -> usize {
        self.temps.len()
    }

    /// Free the temporary strings made since `mark`.
    pub fn release(&mut self, mark: usize) {
        for handle in self.temps.drain(mark..) {
            self.slab[handle as usize] = String::new();
            self.free.push(handle);
        }
    }
}

pub struct Runtime {
    pub console: Console<'static>,
    pub rnd: Rnd,
    pub arrays: Arrays,
    pub data: DataPool,
    pub strings: Strings,
    /// Numeric variables; compiled code reads and writes these directly.
    pub numbers: Vec<f64>,
    pub string_vars: Vec<String>,
    pub array_names: Vec<String>,

    pub for_stack: Vec<ForFrame>,
    pub gosub_stack: Vec<GosubFrame>,
    /// Stack heights on entry to the running multi-line function.
    pub fn_gosub_base: usize,
    pub fn_for_base: usize,
    pub saved_bases: Vec<(usize, usize, Option<u32>)>,
    pub fn_depth: usize,

    /// Bytes of text on their way in (see `rt_bytes`).
    pub staged: Vec<u8>,
    /// Subscripts and arguments of the call being built up.
    pub indices: Vec<usize>,
    pub args: Vec<Value>,
    pub input_kinds: Vec<bool>,
    pub input_values: Vec<Value>,

    pub line: Option<u32>,
    pub steps: u64,
    pub step_limit: Option<u64>,
    pub dialect: Dialect,
}

thread_local! {
    static RUNTIME: RefCell<Option<Runtime>> = const { RefCell::new(None) };
}

/// Run `f` with the runtime.
pub fn with<T>(f: impl FnOnce(&mut Runtime) -> T) -> T {
    RUNTIME.with_borrow_mut(|runtime| f(runtime.as_mut().expect("rt_init has been called")))
}

/// The options a compiled program understands on its command line.
struct Options {
    seed: Option<u64>,
    step_limit: Option<u64>,
    echo: bool,
    dialect: Dialect,
}

fn options() -> Options {
    let mut options = Options {
        seed: None,
        step_limit: None,
        echo: false,
        dialect: Dialect::default(),
    };
    let mut args = std::env::args().skip(1);
    let number = |args: &mut dyn Iterator<Item = String>, flag: &str| match args
        .next()
        .map(|value| value.parse())
    {
        Some(Ok(n)) => Some(n),
        _ => {
            eprintln!("{flag} needs a whole number");
            std::process::exit(2);
        }
    };
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--seed" => options.seed = number(&mut args, "--seed"),
            "--max-steps" => options.step_limit = number(&mut args, "--max-steps"),
            "--echo" => options.echo = true,
            "--dialect" => match args.next().as_deref().and_then(Dialect::from_name) {
                Some(dialect) => options.dialect = dialect,
                None => {
                    eprintln!("--dialect needs microsoft or dartmouth");
                    std::process::exit(2);
                }
            },
            other => {
                eprintln!("Unknown option: {other}\nOptions: --seed <N>, --max-steps <N>, --echo, --dialect <NAME>");
                std::process::exit(2);
            }
        }
    }
    options
}

pub fn init(numbers: usize, strings: usize) {
    let options = options();
    let input = Box::leak(Box::new(BufReader::new(std::io::stdin())));
    let output = Box::leak(Box::new(LineWriter::new(std::io::stdout())));
    let mut console = Console::new(input, output);
    console.set_echo(options.echo);
    console.set_dialect(options.dialect);
    let runtime = Runtime {
        console,
        rnd: Rnd::new(options.seed),
        arrays: Arrays::default(),
        data: DataPool::default(),
        strings: Strings::new(),
        // one spare so that the address is valid even with no variables
        numbers: vec![0.0; numbers + 1],
        string_vars: vec![String::new(); strings],
        array_names: Vec::new(),
        for_stack: Vec::new(),
        gosub_stack: Vec::new(),
        fn_gosub_base: 0,
        fn_for_base: 0,
        saved_bases: Vec::new(),
        fn_depth: 0,
        staged: Vec::new(),
        indices: Vec::new(),
        args: Vec::new(),
        input_kinds: Vec::new(),
        input_values: Vec::new(),
        line: None,
        steps: 0,
        step_limit: options.step_limit,
        dialect: options.dialect,
    };
    RUNTIME.set(Some(runtime));
}

impl Runtime {
    /// Flush the output and leave.
    pub fn exit(&mut self, code: i32) -> ! {
        let _ = self.console.flush();
        std::process::exit(code)
    }

    /// End the program with a runtime error on the current line.
    pub fn fail(&mut self, message: &str) -> ! {
        self.die(Error::runtime(message, None))
    }

    fn die(&mut self, error: Error) -> ! {
        // the reader went away (`program | head`): not an error
        if error.kind == ErrorKind::Io(std::io::ErrorKind::BrokenPipe) {
            std::process::exit(0);
        }
        let _ = self.console.flush();
        eprintln!("{}", error.at_line(self.line));
        std::process::exit(1)
    }

    /// The value of `result`, or the end of the program if it is an error.
    pub fn check<T>(&mut self, result: Result<T>) -> T {
        match result {
            Ok(value) => value,
            Err(error) => self.die(error),
        }
    }

    pub fn take_staged(&mut self) -> String {
        String::from_utf8_lossy(&std::mem::take(&mut self.staged)).into_owned()
    }

    /// The last `count` subscripts passed with `rt_idx`.
    pub fn take_indices(&mut self, count: i32) -> Vec<usize> {
        let at = self.indices.len() - count as usize;
        self.indices.split_off(at)
    }

    /// The last `count` arguments passed with `rt_arg_*`.
    pub fn take_args(&mut self, count: i32) -> Vec<Value> {
        let at = self.args.len() - count as usize;
        self.args.split_off(at)
    }

    pub fn element(&mut self, array: i32, count: i32) -> Value {
        let indices = self.take_indices(count);
        let element = self
            .arrays
            .element(&self.array_names[array as usize], &indices)
            .cloned();
        self.check(element)
    }

    pub fn set_element(&mut self, array: i32, count: i32, value: Value) {
        let indices = self.take_indices(count);
        let element = self
            .arrays
            .element(&self.array_names[array as usize], &indices)
            .map(|slot| *slot = value);
        self.check(element)
    }

    /// Loops below this height belong to the caller of the running
    /// subroutine or function.
    pub fn for_base(&self) -> usize {
        self.gosub_stack
            .last()
            .map_or(0, |frame| frame.for_base)
            .max(self.fn_for_base)
    }
}
