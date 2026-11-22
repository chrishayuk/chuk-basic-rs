//! Command-line front end.

use std::io::{self, LineWriter, Write};
use std::path::Path;
use std::process::ExitCode;

use chuk_basic::{repl, ErrorKind};

/// The runtime that compiled programs are linked against: the `runtime`
/// crate built for WASI (see `scripts/build-runtime.sh`).
const RUNTIME: &[u8] = include_bytes!("../runtime/chuk_basic_rt.wasm");

const USAGE: &str = "\
Usage: chuk-basic [OPTIONS] [FILE.bas]

Runs a BASIC program, or with no file starts the interactive prompt.

Options:
      --tokens         Print the token stream instead of running
      --ast            Print the syntax tree as JSON instead of running
      --compile        Compile to WebAssembly instead of running (FILE.wasm)
  -o, --output <FILE>  Where --compile writes its output
      --seed <N>       Seed the random number generator (repeatable runs)
      --max-steps <N>  Stop with an error after N operations
      --echo           Echo INPUT lines to the output (for piped input)
  -h, --help           Show this help
  -V, --version        Show the version";

enum Mode {
    Run,
    Compile,
    Tokens,
    Ast,
}

struct Args {
    mode: Mode,
    seed: Option<u64>,
    max_steps: Option<u64>,
    echo: bool,
    /// Where `--compile` writes; by default, beside the program.
    output: Option<String>,
    /// The program to run; `None` starts the interactive prompt.
    path: Option<String>,
}

fn parse_args() -> Result<Option<Args>, String> {
    let mut mode = Mode::Run;
    let mut seed = None;
    let mut max_steps = None;
    let mut echo = false;
    let mut output = None;
    let mut path = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                println!("{USAGE}");
                return Ok(None);
            }
            "-V" | "--version" => {
                println!("chuk-basic {}", env!("CARGO_PKG_VERSION"));
                return Ok(None);
            }
            "--tokens" => mode = Mode::Tokens,
            "--ast" => mode = Mode::Ast,
            "--echo" => echo = true,
            "--compile" => mode = Mode::Compile,
            "-o" | "--output" => output = Some(args.next().ok_or("--output needs a file name")?),
            "--seed" => {
                let value = args.next().ok_or("--seed needs a value")?;
                seed = Some(
                    value
                        .parse()
                        .map_err(|_| format!("Invalid seed: {value}"))?,
                );
            }
            "--max-steps" => {
                let value = args.next().ok_or("--max-steps needs a value")?;
                max_steps = Some(
                    value
                        .parse()
                        .map_err(|_| format!("Invalid step count: {value}"))?,
                );
            }
            flag if flag.starts_with('-') => return Err(format!("Unknown option: {flag}")),
            _ if path.is_some() => return Err("Only one program file can be given".to_string()),
            _ => path = Some(arg),
        }
    }
    Ok(Some(Args {
        mode,
        seed,
        max_steps,
        echo,
        output,
        path,
    }))
}

fn run(args: &Args) -> Result<(), String> {
    let mut output = LineWriter::new(io::stdout().lock());
    let result = match (&args.path, &args.mode) {
        (None, Mode::Run) => {
            let options = repl::Options {
                seed: args.seed,
                step_limit: args.max_steps,
                echo: args.echo,
            };
            repl::repl(&mut io::stdin().lock(), &mut output, &options)
        }
        (None, _) => return Err("--tokens, --ast and --compile need a program file".to_string()),
        (Some(path), mode) => {
            let source = std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
            match mode {
                Mode::Tokens => chuk_basic::tokenize(&source).map(|tokens| {
                    for token in tokens {
                        let _ = writeln!(output, "{:?}", token.tok);
                    }
                }),
                Mode::Ast => chuk_basic::parse(&source).map(|program| {
                    let _ = writeln!(output, "{}", program.to_json());
                }),
                Mode::Compile => chuk_basic::wasm::compile(&source, RUNTIME).and_then(|module| {
                    let to = args.output.clone().unwrap_or_else(|| {
                        Path::new(path).with_extension("wasm").display().to_string()
                    });
                    std::fs::write(&to, module).map_err(|e| chuk_basic::Error::io(&e))
                }),
                Mode::Run => chuk_basic::parse(&source).and_then(|program| {
                    let mut input = io::stdin().lock();
                    let mut interpreter =
                        chuk_basic::Interpreter::new(program, &mut input, &mut output, args.seed)?;
                    interpreter.set_step_limit(args.max_steps);
                    interpreter.set_echo_input(args.echo);
                    interpreter.run()
                }),
            }
        }
    };
    let _ = output.flush();
    match result {
        // the reader went away (`chuk-basic prog.bas | head`): not an error
        Err(e) if e.kind == ErrorKind::Io(io::ErrorKind::BrokenPipe) => Ok(()),
        other => other.map_err(|e| e.to_string()),
    }
}

fn main() -> ExitCode {
    let args = match parse_args() {
        Ok(Some(args)) => args,
        Ok(None) => return ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}\n\n{USAGE}");
            return ExitCode::from(2);
        }
    };
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}
