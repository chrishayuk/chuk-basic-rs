//! Command-line front end.

use std::io::{self, LineWriter, Write};
use std::path::Path;
use std::process::ExitCode;

use chuk_basic::{repl, Dialect, ErrorKind};

/// The runtime that compiled programs are linked against: the `runtime`
/// crate built for WASI (see `scripts/build-runtime.sh`).
#[cfg(feature = "compiler")]
const RUNTIME: &[u8] = include_bytes!("../runtime/chuk_basic_rt.wasm");
#[cfg(not(feature = "compiler"))]
const RUNTIME: &[u8] = &[];

/// What an editor uses: the language server and the checks behind it.
#[cfg(feature = "editor")]
mod editor {
    use std::io::{self, Write};

    use chuk_basic::analysis::{self, Lint, Severity};
    use chuk_basic::Dialect;

    pub fn serve() -> ! {
        let status = chuk_basic::lsp::serve(&mut io::stdin().lock(), &mut io::stdout().lock());
        std::process::exit(status)
    }

    /// Print what is wrong as `file:line:column: kind: message`.
    pub fn check(
        path: &str,
        source: &str,
        dialect: Dialect,
        output: &mut dyn Write,
    ) -> Result<(), String> {
        let lint = match dialect {
            Dialect::Dartmouth => Lint::Dartmouth1964,
            Dialect::Microsoft => Lint::Later,
        };
        let found = analysis::diagnostics(source, lint);
        for d in &found {
            let kind = match d.severity {
                Severity::Error => "error",
                Severity::Warning => "warning",
            };
            let (line, column) = (d.at.line + 1, d.at.span.start + 1);
            let _ = writeln!(output, "{path}:{line}:{column}: {kind}: {}", d.message);
        }
        let _ = output.flush();
        match found
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .count()
        {
            0 => Ok(()),
            1 => Err("1 error".to_string()),
            n => Err(format!("{n} errors")),
        }
    }

    pub fn renumber(source: &str, output: &mut dyn Write) -> Result<(), String> {
        let text = analysis::renumbered(source, 10, 10)?;
        let _ = write!(output, "{text}");
        output.flush().map_err(|e| e.to_string())
    }
}

#[cfg(not(feature = "editor"))]
mod editor {
    use std::io::Write;

    use chuk_basic::Dialect;

    const LEFT_OUT: &str = "This build of chuk-basic does not include the editor tools";

    pub fn serve() -> ! {
        eprintln!("{LEFT_OUT}");
        std::process::exit(2)
    }

    pub fn check(_: &str, _: &str, _: Dialect, _: &mut dyn Write) -> Result<(), String> {
        Err(LEFT_OUT.to_string())
    }

    pub fn renumber(_: &str, _: &mut dyn Write) -> Result<(), String> {
        Err(LEFT_OUT.to_string())
    }
}

const USAGE: &str = "\
Usage: chuk-basic [OPTIONS] [FILE.bas]

Runs a BASIC program, or with no file starts the interactive prompt.

Options:
      --tokens         Print the token stream instead of running
      --ast            Print the syntax tree as JSON instead of running
      --compile        Compile to WebAssembly instead of running (FILE.wasm)
      --check          List the mistakes that can be found without running
      --renumber       Print the program renumbered in tens
      --lsp            Be a language server for an editor, on stdin and stdout
  -o, --output <FILE>  Where --compile writes its output
      --seed <N>       Seed the random number generator (repeatable runs)
      --max-steps <N>  Stop with an error after N operations
      --echo           Echo INPUT lines to the output (for piped input)
      --dialect <NAME> BASIC to follow: microsoft (default) or dartmouth
  -h, --help           Show this help
  -V, --version        Show the version";

enum Mode {
    Run,
    Compile,
    Tokens,
    Ast,
    Check,
    Renumber,
    Lsp,
}

struct Args {
    mode: Mode,
    seed: Option<u64>,
    max_steps: Option<u64>,
    echo: bool,
    dialect: Dialect,
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
    let mut dialect = Dialect::default();
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
            "--check" => mode = Mode::Check,
            "--renumber" => mode = Mode::Renumber,
            "--lsp" => mode = Mode::Lsp,
            // editors' clients add this to say how they will talk; it is the only way we do
            "--stdio" => {}
            "--echo" => echo = true,
            "--dialect" => {
                let name = args.next().ok_or("--dialect needs a name")?;
                dialect = Dialect::from_name(&name)
                    .ok_or(format!("Unknown dialect: {name} (microsoft or dartmouth)"))?;
            }
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
        dialect,
        output,
        path,
    }))
}

fn run(args: &Args) -> Result<(), String> {
    let mut output = LineWriter::new(io::stdout().lock());
    let result = match (&args.path, &args.mode) {
        (_, Mode::Lsp) => {
            drop(output);
            editor::serve()
        }
        (None, Mode::Run) => {
            let options = repl::Options {
                seed: args.seed,
                step_limit: args.max_steps,
                echo: args.echo,
                dialect: args.dialect,
            };
            repl::repl(&mut io::stdin().lock(), &mut output, &options)
        }
        (None, _) => return Err("This option needs a program file".to_string()),
        (Some(path), mode) => {
            let source = std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
            match mode {
                Mode::Tokens => chuk_basic::tokenize(&source).map(|tokens| {
                    for token in tokens {
                        let _ = writeln!(output, "{:?}", token.tok);
                    }
                }),
                Mode::Lsp => unreachable!("handled above"),
                Mode::Check => return editor::check(path, &source, args.dialect, &mut output),
                Mode::Renumber => return editor::renumber(&source, &mut output),
                Mode::Ast => chuk_basic::parse(&source).map(|program| {
                    let _ = writeln!(output, "{}", program.to_json());
                }),
                Mode::Compile if RUNTIME.is_empty() => {
                    return Err("This build of chuk-basic does not include the compiler".to_string())
                }
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
                    interpreter.set_dialect(args.dialect);
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
