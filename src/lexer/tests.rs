use super::*;

fn toks(src: &str) -> Vec<Tok> {
    tokenize(src).unwrap().into_iter().map(|t| t.tok).collect()
}

fn id(s: &str) -> Tok {
    Tok::Ident(s.to_string())
}

#[test]
fn line_numbers_and_keywords() {
    assert_eq!(
        toks("10 LET X = 5"),
        vec![
            Tok::LineNo(10),
            Tok::Let,
            id("X"),
            Tok::Eq,
            Tok::Num(5.0),
            Tok::Eol
        ]
    );
}

#[test]
fn indented_line_numbers_and_case() {
    assert_eq!(
        toks("   20 print \"hi\""),
        vec![Tok::LineNo(20), Tok::Print, Tok::Str("hi".into()), Tok::Eol]
    );
}

#[test]
fn keywords_need_word_boundaries() {
    assert_eq!(
        toks("TOTAL = FORT"),
        vec![id("TOTAL"), Tok::Eq, id("FORT"), Tok::Eol]
    );
}

#[test]
fn goto_and_gosub_spellings() {
    assert_eq!(toks("GOTO 10"), vec![Tok::Goto, Tok::Num(10.0), Tok::Eol]);
    assert_eq!(toks("GO TO 10"), vec![Tok::Goto, Tok::Num(10.0), Tok::Eol]);
    assert_eq!(
        toks("GO SUB 10"),
        vec![Tok::Gosub, Tok::Num(10.0), Tok::Eol]
    );
    assert_eq!(toks("gosub 10"), vec![Tok::Gosub, Tok::Num(10.0), Tok::Eol]);
}

#[test]
fn numbers() {
    assert_eq!(
        toks("X=.5"),
        vec![id("X"), Tok::Eq, Tok::Num(0.5), Tok::Eol]
    );
    assert_eq!(
        toks("X=1.5E-3"),
        vec![id("X"), Tok::Eq, Tok::Num(0.0015), Tok::Eol]
    );
    assert_eq!(
        toks("X=6."),
        vec![id("X"), Tok::Eq, Tok::Num(6.0), Tok::Eol]
    );
}

#[test]
fn operators() {
    assert_eq!(
        toks("<> >< <= =< >= => < > ="),
        vec![
            Tok::Ne,
            Tok::Ne,
            Tok::Le,
            Tok::Le,
            Tok::Ge,
            Tok::Ge,
            Tok::Lt,
            Tok::Gt,
            Tok::Eq,
            Tok::Eol
        ]
    );
}

#[test]
fn string_suffix_and_functions() {
    assert_eq!(
        toks("A$=LEFT$(B$,2)"),
        vec![
            id("A$"),
            Tok::Eq,
            id("LEFT$"),
            Tok::LParen,
            id("B$"),
            Tok::Comma,
            Tok::Num(2.0),
            Tok::RParen,
            Tok::Eol
        ]
    );
}

#[test]
fn rem_and_data_take_the_rest_of_the_line() {
    assert_eq!(
        toks("10 REM hello: world"),
        vec![Tok::LineNo(10), Tok::Rem("hello: world".into()), Tok::Eol]
    );
    assert_eq!(
        toks("20 DATA 1, \"a:b\", 3: PRINT"),
        vec![
            Tok::LineNo(20),
            Tok::Data("1, \"a:b\", 3".into()),
            Tok::Colon,
            Tok::Print,
            Tok::Eol
        ]
    );
}

#[test]
fn question_mark_is_print() {
    assert_eq!(toks("? 1"), vec![Tok::Print, Tok::Num(1.0), Tok::Eol]);
}

#[test]
fn unexpected_character() {
    let err = tokenize("10 PRINT @").unwrap_err();
    assert_eq!(err.line, Some(10));
    assert!(err.message.contains('@'));
}

#[test]
fn crunched_keywords_are_split_where_no_variable_could_be() {
    // everything after the `=` of `I=...`
    let after_eq = |src: &str| toks(&format!("I={src}"))[2..].to_vec();
    assert_eq!(
        after_eq("1TO8"),
        vec![Tok::Num(1.0), Tok::To, Tok::Num(8.0), Tok::Eol]
    );
    assert_eq!(
        after_eq("1TOQ1"),
        vec![Tok::Num(1.0), Tok::To, id("Q1"), Tok::Eol]
    );
    assert_eq!(
        after_eq("22STEP3"),
        vec![Tok::Num(22.0), Tok::Step, Tok::Num(3.0), Tok::Eol]
    );
    assert_eq!(
        after_eq("1ANDC1"),
        vec![Tok::Num(1.0), Tok::And, id("C1"), Tok::Eol]
    );
    assert_eq!(
        after_eq("\"Y\"THEN100"),
        vec![Tok::Str("Y".into()), Tok::Then, Tok::Num(100.0), Tok::Eol]
    );
    assert_eq!(
        after_eq("A(1)THENX")[1..],
        [
            Tok::LParen,
            Tok::Num(1.0),
            Tok::RParen,
            Tok::Then,
            id("X"),
            Tok::Eol
        ]
    );
    assert_eq!(
        toks("GOSUB500"),
        vec![Tok::Gosub, Tok::Num(500.0), Tok::Eol]
    );
}

#[test]
fn crunching_leaves_ordinary_names_alone() {
    assert_eq!(
        toks("TOTAL ORDER ANDY"),
        vec![id("TOTAL"), id("ORDER"), id("ANDY"), Tok::Eol]
    );
    // after a string or `)` only control keywords are split off
    assert_eq!(
        toks("\"A\"TOTAL"),
        vec![Tok::Str("A".into()), id("TOTAL"), Tok::Eol]
    );
    assert_eq!(toks(")ORDER"), vec![Tok::RParen, id("ORDER"), Tok::Eol]);
    assert_eq!(
        toks("X=1E5"),
        vec![id("X"), Tok::Eq, Tok::Num(100000.0), Tok::Eol]
    );
}

#[test]
fn rem_prefix() {
    assert_eq!(
        toks("10 REMARK HI"),
        vec![Tok::LineNo(10), Tok::Rem("ARK HI".into()), Tok::Eol]
    );
    assert_eq!(
        toks("X=1:REMEMBER"),
        vec![
            id("X"),
            Tok::Eq,
            Tok::Num(1.0),
            Tok::Colon,
            Tok::Rem("EMBER".into()),
            Tok::Eol
        ]
    );
    assert_eq!(
        toks("REMAIN=1"),
        vec![id("REMAIN"), Tok::Eq, Tok::Num(1.0), Tok::Eol]
    );
    assert_eq!(
        toks("X=REMAIN"),
        vec![id("X"), Tok::Eq, id("REMAIN"), Tok::Eol]
    );
}
