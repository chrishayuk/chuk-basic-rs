//! Keeps `docs/TUTORIAL.md` honest: every program listing in it must match
//! the file in `samples/tutorial`, and the output shown beneath it must be
//! what the program really prints (as recorded by the golden tests).

mod common;

use common::{programs, read};

/// The contents of the next fenced code block that opens with ```` ```lang ````.
fn fenced<'a>(lines: &mut impl Iterator<Item = &'a str>, lang: &str) -> String {
    let fence = format!("```{lang}");
    assert!(
        lines.by_ref().any(|line| line == fence),
        "missing {fence} block"
    );
    let mut block = String::new();
    for line in lines.by_ref().take_while(|line| *line != "```") {
        block.push_str(line);
        block.push('\n');
    }
    block
}

#[test]
fn tutorial_listings_and_outputs_are_real() {
    let tutorial = read("docs/TUTORIAL.md");
    let mut lines = tutorial.lines();
    let mut shown = Vec::new();
    while let Some(line) = lines.next() {
        let Some(path) = line
            .strip_prefix("**`")
            .and_then(|rest| rest.strip_suffix("`**"))
        else {
            continue;
        };
        let expected = std::path::Path::new(path);

        assert_eq!(fenced(&mut lines, "basic"), read(path), "listing of {path}");

        let mut output = read(expected.with_extension("out"));
        if expected.with_extension("err").exists() {
            output.push_str(&read(expected.with_extension("err")));
        }
        assert_eq!(
            fenced(&mut lines, "text").trim_end(),
            output.trim_end(),
            "output of {path}"
        );
        shown.push(expected.to_path_buf());
    }
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
    assert!(read("docs/TUTORIAL.md").contains(message));
    let err = chuk_basic::parse("10 X = 1\n20 IF X PRINT 1").unwrap_err();
    assert_eq!(err.to_string(), message);
}
