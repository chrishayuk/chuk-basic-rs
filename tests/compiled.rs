//! The compiler against the interpreter: every program is compiled to
//! WebAssembly, run under wasmtime, and must print exactly what the
//! interpreter prints - output, error message and all.
//!
//! These tests need `wasmtime` on the PATH. Without it they are skipped
//! (with a note), unless `CHUK_BASIC_REQUIRE_WASMTIME` is set, as it is in
//! CI. They cannot run inside a WASI runtime themselves.
#![cfg(not(target_os = "wasi"))]

mod common;

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};

use common::{programs, read, run_opts, Options, SEED};

const STEP_LIMIT: u64 = 1_000_000;

fn wasmtime_available() -> bool {
    let found = Command::new("wasmtime")
        .arg("--version")
        .output()
        .is_ok_and(|out| out.status.success());
    if !found {
        assert!(
            std::env::var_os("CHUK_BASIC_REQUIRE_WASMTIME").is_none(),
            "wasmtime is not on the PATH"
        );
        eprintln!("skipped: wasmtime is not on the PATH");
    }
    found
}

fn compile(source: &str) -> Vec<u8> {
    let runtime = std::fs::read("runtime/chuk_basic_rt.wasm").expect("runtime/chuk_basic_rt.wasm");
    chuk_basic::wasm::compile(source, &runtime).unwrap_or_else(|e| panic!("{e}"))
}

/// Run a compiled module; returns what it printed and its error message.
fn run_compiled(module: &[u8], input: &str, seed: u64) -> (String, String) {
    run_compiled_with(module, input, seed, &[])
}

fn run_compiled_with(module: &[u8], input: &str, seed: u64, options: &[&str]) -> (String, String) {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let file: PathBuf = std::env::temp_dir().join(format!(
        "chuk-basic-test-{}-{}.wasm",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::write(&file, module).unwrap();
    let mut child = Command::new("wasmtime")
        .arg("run")
        .arg(&file)
        .args([
            "--echo",
            "--seed",
            &seed.to_string(),
            "--max-steps",
            &STEP_LIMIT.to_string(),
        ])
        .args(options)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    // the program may stop reading before the input is used up
    let _ = child.stdin.take().unwrap().write_all(input.as_bytes());
    let output = child.wait_with_output().unwrap();
    let _ = std::fs::remove_file(&file);
    (
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).trim().to_string(),
    )
}

/// What the interpreter prints, and its error message.
fn run_interpreted(source: &str, input: &str, seed: u64) -> (String, String) {
    let options = Options {
        input,
        seed,
        echo: true,
        step_limit: Some(STEP_LIMIT),
    };
    let (output, result) = run_opts(source, &options);
    (
        output,
        result.err().map(|e| e.to_string()).unwrap_or_default(),
    )
}

/// Compare the two ways of running `source`; returns a description of the
/// difference, if there is one.
fn difference(name: &str, source: &str, input: &str, seed: u64) -> Option<String> {
    let interpreted = run_interpreted(source, input, seed);
    let compiled = run_compiled(&compile(source), input, seed);
    (interpreted != compiled).then(|| {
        format!(
            "{name} (seed {seed}):\n--- interpreted\n{}\n{}\n--- compiled\n{}\n{}",
            interpreted.0, interpreted.1, compiled.0, compiled.1
        )
    })
}

/// 400 lines of the sort of thing a player types, different for each seed.
fn mash(seed: u64) -> String {
    const ANSWERS: [&str; 16] = [
        "1", "2", "3", "5", "10", "0", "YES", "NO", "Y", "N", "1,1", "2,3", "50", "A", "-1", ".5",
    ];
    let mut state = seed
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    let mut lines = String::new();
    for _ in 0..400 {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        lines.push_str(ANSWERS[(state >> 33) as usize % ANSWERS.len()]);
        lines.push('\n');
    }
    lines
}

/// Check every program in `paths`, using its `.in` file as input if it has
/// one and `mash` otherwise.
fn check(paths: impl IntoIterator<Item = PathBuf>, seeds: std::ops::Range<u64>) {
    if !wasmtime_available() {
        return;
    }
    let mut differences = Vec::new();
    for path in paths {
        let source = read(&path);
        let input_file = path.with_extension("in");
        for seed in seeds.clone() {
            let input = if input_file.exists() {
                read(&input_file)
            } else {
                mash(seed)
            };
            differences.extend(difference(
                &path.display().to_string(),
                &source,
                &input,
                seed,
            ));
        }
    }
    assert!(
        differences.is_empty(),
        "{} program(s) differ:\n\n{}",
        differences.len(),
        differences.join("\n\n")
    );
}

fn all(dirs: &[&str]) -> Vec<PathBuf> {
    dirs.iter().flat_map(|dir| programs(dir)).collect()
}

#[test]
fn samples_classic_and_tutorial() {
    check(
        all(&["samples", "samples/classic", "samples/tutorial"]),
        SEED..SEED + 1,
    );
}

#[test]
fn conformance_and_extensions() {
    check(
        all(&["tests/conformance", "tests/extensions"]),
        SEED..SEED + 1,
    );
}

/// The 103 games, in four batches so that they run side by side.
fn games(batch: usize) {
    let games = programs("samples/games").into_iter().enumerate();
    check(
        games.filter(|(i, _)| i % 4 == batch).map(|(_, path)| path),
        0..2,
    );
}

#[test]
fn games_batch_0() {
    games(0);
}

#[test]
fn games_batch_1() {
    games(1);
}

#[test]
fn games_batch_2() {
    games(2);
}

#[test]
fn games_batch_3() {
    games(3);
}

/// Mistakes that the interpreter only notices when the statement runs must
/// be reported at the same moment, in the same words, by compiled code.
#[test]
fn errors_are_the_same() {
    if !wasmtime_available() {
        return;
    }
    let programs = [
        "10 PRINT \"A\"\n20 PRINT 1/0",
        "10 PRINT \"A\"\n20 GOTO 999",
        "10 PRINT \"A\"\n20 GOSUB 999",
        "10 PRINT \"A\"\n20 RETURN",
        "10 PRINT \"A\"\n20 NEXT",
        "10 PRINT \"A\"\n20 A$=5",
        "10 PRINT \"A\"\n20 X=\"S\"",
        "10 PRINT \"A\"\n20 PRINT \"X\"+1",
        "10 PRINT \"A\"\n20 PRINT -\"X\"",
        "10 PRINT \"A\"\n20 PRINT LEN(5)",
        "10 PRINT \"A\"\n20 PRINT SQR(-1)",
        "10 PRINT \"A\"\n20 PRINT INT(\"X\")",
        "10 PRINT \"A\"\n20 PRINT FNQ(1)",
        "10 DEF FNA(X)=X\n20 PRINT \"A\"\n30 PRINT FNA(1,2)",
        "10 DEF FNA(X)=X\n20 PRINT \"A\"\n30 PRINT FNA(\"S\")",
        "10 DEF FNA$(X)=X\n20 PRINT \"A\"\n30 PRINT FNA$(1)",
        "10 DEF FNR(N)=FNR(N+1)\n20 PRINT \"A\"\n30 PRINT FNR(1)",
        "10 PRINT \"A\"\n20 READ X",
        "10 PRINT \"A\"\n20 READ X\n30 DATA HELLO",
        "10 PRINT \"A\"\n20 DIM A(3)\n30 A(4)=1",
        "10 PRINT \"A\"\n20 DIM A(3): DIM A(3)",
        "10 PRINT \"A\"\n20 A(-1)=1",
        "10 PRINT \"A\"\n20 FOR I=5 TO 1\n30 PRINT \"B\"",
        "10 PRINT \"A\"\n20 ON 2 GOTO 30,999\n30 END",
        "10 PRINT \"A\"\n20 PRINT USING \"ABC\";1",
        "10 PRINT \"A\"\n20 DIM M(2,2)\n30 MAT N=INV(M)",
        "10 PRINT \"A\";\n20 GOTO 10",
        "10 IF 0 THEN PRINT \"X\"+1\n20 PRINT \"REACHED\"",
        "10 INPUT A\n20 PRINT A",
    ];
    let differences: Vec<String> = programs
        .iter()
        .filter_map(|source| difference(source, source, "", SEED))
        .collect();
    assert!(differences.is_empty(), "{}", differences.join("\n\n"));
}

/// The 1964 manual's programs, compiled, in both arithmetics.
#[test]
fn dartmouth_examples_and_dialect() {
    if !wasmtime_available() {
        return;
    }
    // and where the dialects part: INT, RND(0), a leading minus, running out of DATA
    let probe = "10 PRINT INT(-1.5); -2^2; -(2^2); RND(0) = RND(0)\n20 READ A\n30 PRINT A\n40 GOTO 20\n50 DATA 7\n60 END\n";
    let mut sources: Vec<(String, String)> = programs("samples/dartmouth")
        .into_iter()
        .map(|path| (path.display().to_string(), read(&path)))
        .collect();
    sources.push(("the dialect probe".to_string(), probe.to_string()));
    for (path, source) in sources {
        for dialect in ["microsoft", "dartmouth"] {
            let interpreted = {
                let mut output = Vec::new();
                let result = chuk_basic::parse(&source).and_then(|program| {
                    let mut input = std::io::empty();
                    let mut interpreter =
                        chuk_basic::Interpreter::new(program, &mut input, &mut output, Some(SEED))?;
                    interpreter.set_dialect(chuk_basic::Dialect::from_name(dialect).unwrap());
                    interpreter.run()
                });
                (
                    String::from_utf8(output).unwrap(),
                    result.err().map(|e| e.to_string()).unwrap_or_default(),
                )
            };
            let compiled = run_compiled_with(&compile(&source), "", SEED, &["--dialect", dialect]);
            assert_eq!(interpreted, compiled, "{path} with {dialect} arithmetic");
        }
    }
}

/// A few things about the module itself.
#[test]
fn the_module_is_a_self_contained_wasi_command() {
    let source = read("samples/classic/primes.bas");
    let module = compile(&source);
    assert_eq!(&module[..8], b"\0asm\x01\0\0\0");
    assert_eq!(
        module,
        compile(&source),
        "compiling twice gives the same bytes"
    );
    // exports its entry point and memory, and none of the runtime's internals
    let text = String::from_utf8_lossy(&module);
    assert!(text.contains("_start") && text.contains("memory"));
    assert!(!text.contains("rt_print_num"));
    // syntax errors are reported by the compiler, as by the interpreter
    let runtime = std::fs::read(Path::new("runtime/chuk_basic_rt.wasm")).unwrap();
    let error = chuk_basic::wasm::compile("10 PRINT )", &runtime).unwrap_err();
    assert_eq!(
        error.to_string(),
        "Syntax error in line 10: Expected an expression, found RPAREN"
    );
    assert!(chuk_basic::wasm::compile("10 PRINT 1", b"not wasm").is_err());
}
