//! Keeps the tutorial in `tutorial/` honest.
//!
//! Every ```` ```basic ```` block in a chapter is a complete program and must
//! be followed by an ```` ```output ```` block holding exactly what it prints
//! (with an ```` ```input ```` block in between if it asks questions). Every
//! ```` ```repl ```` block is a session at the interactive prompt. All of
//! them are run here; `tutorial/check_tutorial.py` does the same through the
//! command-line program, and can compile them to WebAssembly as well.
//!
//! The chapters' main programs are also files in `samples/tutorial`, and the
//! listings shown must match those files.

mod common;

use common::{programs, read, run_opts, Options, SEED};

/// A fenced block: its language, the rest of the fence line, and its text.
struct Block {
    lang: String,
    info: String,
    body: String,
    /// The file named in bold just above the block, if any.
    file: Option<String>,
    chapter: String,
}

/// The chapters, in order.
fn chapters() -> Vec<std::path::PathBuf> {
    let mut found: Vec<_> = std::fs::read_dir("tutorial")
        .expect("tutorial/")
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "md"))
        .collect();
    found.sort();
    assert!(found.len() >= 20, "expected the tutorial's chapters");
    found
}

/// Every fenced block of every chapter, in reading order.
fn blocks() -> Vec<Block> {
    let mut found = Vec::new();
    for path in chapters() {
        let chapter = path.file_name().unwrap().to_string_lossy().into_owned();
        let text = read(&path);
        let mut lines = text.lines();
        let mut file = None;
        while let Some(line) = lines.next() {
            if let Some(name) = line
                .strip_prefix("**`")
                .and_then(|rest| rest.strip_suffix("`**"))
            {
                file = Some(name.to_string());
                continue;
            }
            let Some(fence) = line.strip_prefix("```") else {
                continue;
            };
            let (lang, info) = fence.split_once(' ').unwrap_or((fence, ""));
            let mut body = String::new();
            for line in lines.by_ref().take_while(|line| *line != "```") {
                body.push_str(line);
                body.push('\n');
            }
            found.push(Block {
                lang: lang.to_string(),
                info: info.to_string(),
                body,
                file: file.take(),
                chapter: chapter.clone(),
            });
        }
    }
    found
}

/// The value of `key=value` on a fence line.
fn setting<'a>(info: &'a str, key: &str) -> Option<&'a str> {
    info.split_whitespace()
        .filter_map(|word| word.split_once('='))
        .find(|(name, _)| *name == key)
        .map(|(_, value)| value)
}

/// What a program prints, with its error message if it stops with one - the
/// way `chuk-basic --seed 42 --echo` shows it.
fn transcript(block: &Block, input: &str) -> String {
    let seed = setting(&block.info, "seed").map_or(SEED, |seed| seed.parse().unwrap());
    let (mut output, result) = match setting(&block.info, "dialect") {
        None => run_opts(
            &block.body,
            &Options {
                input,
                seed,
                echo: true,
                ..Options::default()
            },
        ),
        Some(name) => {
            let dialect = chuk_basic::Dialect::from_name(name).expect("a known dialect");
            let mut printed = Vec::new();
            let mut typed = input.as_bytes();
            let result = chuk_basic::parse(&block.body).and_then(|program| {
                let mut interpreter =
                    chuk_basic::Interpreter::new(program, &mut typed, &mut printed, Some(seed))?;
                interpreter.set_echo_input(true);
                interpreter.set_dialect(dialect);
                interpreter.run()
            });
            (String::from_utf8_lossy(&printed).into_owned(), result)
        }
    };
    if let Err(error) = result {
        output.push_str(&error.to_string());
    }
    output
}

/// Everything the interactive prompt prints when `typed` is typed at it.
fn session(typed: &str) -> String {
    let mut output = Vec::new();
    chuk_basic::repl::repl(
        &mut typed.as_bytes(),
        &mut output,
        &chuk_basic::repl::Options::default(),
    )
    .unwrap();
    String::from_utf8_lossy(&output).into_owned()
}

#[test]
fn every_program_in_the_tutorial_prints_what_the_tutorial_says() {
    let blocks = blocks();
    let mut checked = 0;
    let mut index = 0;
    while index < blocks.len() {
        let block = &blocks[index];
        index += 1;
        if block.lang != "basic" && block.lang != "repl" {
            continue;
        }
        let mut input = "";
        if block.lang == "basic" && blocks.get(index).is_some_and(|next| next.lang == "input") {
            input = &blocks[index].body;
            index += 1;
        }
        let expected = match blocks.get(index) {
            Some(next) if next.lang == "output" => &next.body,
            _ => panic!("{}: no output block after:\n{}", block.chapter, block.body),
        };
        index += 1;
        let actual = if block.lang == "repl" {
            session(&block.body)
        } else {
            transcript(block, input)
        };
        assert_eq!(
            actual.trim_end_matches('\n'),
            expected.trim_end_matches('\n'),
            "{}: output of:\n{}",
            block.chapter,
            block.body
        );
        checked += 1;
    }
    assert!(checked >= 200, "only {checked} programs found");
}

#[test]
fn tutorial_shows_every_sample_program_as_it_is_on_disk() {
    let shown: Vec<_> = blocks()
        .into_iter()
        .filter_map(|block| {
            let path = block.file?;
            assert_eq!(block.lang, "basic", "{path} should be a basic block");
            assert_eq!(block.body, read(&path), "listing of {path}");
            Some(std::path::PathBuf::from(path))
        })
        .collect();
    assert_eq!(
        shown,
        programs("samples/tutorial"),
        "the tutorial should show every tutorial program, in order"
    );
}

/// The error message quoted in the "When things go wrong" chapter.
#[test]
fn tutorial_syntax_error_example_is_real() {
    let message = "Syntax error in line 20: Expected THEN, found PRINT";
    assert!(read("tutorial/16-when-things-go-wrong.md").contains(message));
    let err = chuk_basic::parse("10 X = 1\n20 IF X PRINT 1").unwrap_err();
    assert_eq!(err.to_string(), message);
}
