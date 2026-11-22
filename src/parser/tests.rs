use super::*;
use crate::ast::*;

fn stmts(src: &str) -> Vec<Stmt> {
    parse(src)
        .unwrap()
        .lines
        .into_iter()
        .flat_map(|l| l.stmts)
        .collect()
}

fn expr(src: &str) -> Expr {
    match stmts(&format!("X = {src}")).remove(0) {
        Stmt::Let { expr, .. } => expr,
        other => panic!("not a LET: {other:?}"),
    }
}

fn num(n: f64) -> Box<Expr> {
    Box::new(Expr::Num(n))
}

fn var(name: &str) -> LValue {
    LValue {
        name: name.to_string(),
        indices: vec![],
    }
}

#[test]
fn precedence() {
    assert_eq!(
        expr("1 + 2 * 3"),
        Expr::Binary(
            BinOp::Add,
            num(1.0),
            Box::new(Expr::Binary(BinOp::Mul, num(2.0), num(3.0)))
        )
    );
    // -2^2 is -(2^2), but marked: in 1964's BASIC it was (-2)^2
    assert_eq!(
        expr("-2^2"),
        Expr::Unary(
            UnOp::NegPower,
            Box::new(Expr::Binary(BinOp::Pow, num(2.0), num(2.0)))
        )
    );
    // with brackets it is an ordinary minus in every dialect
    assert_eq!(
        expr("-(2^2)"),
        Expr::Unary(
            UnOp::Neg,
            Box::new(Expr::Binary(BinOp::Pow, num(2.0), num(2.0)))
        )
    );
    assert!(matches!(expr("-(2)^2"), Expr::Unary(UnOp::NegPower, _)));
    assert_eq!(
        expr("-2*3"),
        Expr::Binary(
            BinOp::Mul,
            Box::new(Expr::Unary(UnOp::Neg, num(2.0))),
            num(3.0)
        )
    );
    // relational binds tighter than AND/OR
    assert!(matches!(
        expr("A < 1 OR B > 2"),
        Expr::Binary(BinOp::Or, _, _)
    ));
}

#[test]
fn let_with_and_without_keyword() {
    let expected = Stmt::Let {
        target: var("X"),
        expr: Expr::Num(5.0),
    };
    assert_eq!(stmts("10 LET X = 5"), vec![expected.clone()]);
    assert_eq!(stmts("10 x = 5"), vec![expected]);
}

#[test]
fn if_then_line_number_and_else() {
    assert_eq!(
        stmts("10 IF X THEN 100 ELSE PRINT"),
        vec![Stmt::If {
            cond: Expr::Var("X".into()),
            then: vec![Stmt::Goto(100)],
            otherwise: Some(vec![Stmt::Print(vec![])]),
        }]
    );
}

#[test]
fn if_swallows_the_rest_of_the_line() {
    let parsed = stmts("10 IF X THEN PRINT 1: PRINT 2");
    assert_eq!(parsed.len(), 1);
    assert!(matches!(&parsed[0], Stmt::If { then, .. } if then.len() == 2));
}

#[test]
fn print_items() {
    assert_eq!(
        stmts("PRINT TAB(5);\"A\",B"),
        vec![Stmt::Print(vec![
            PrintItem::Tab(Expr::Num(5.0)),
            PrintItem::Semi,
            PrintItem::Expr(Expr::Str("A".into())),
            PrintItem::Comma,
            PrintItem::Expr(Expr::Var("B".into())),
        ])]
    );
}

#[test]
fn functions_arrays_and_builtins() {
    assert_eq!(
        expr("FNA(1)"),
        Expr::FnCall("FNA".into(), vec![Expr::Num(1.0)])
    );
    assert_eq!(expr("A(1)"), Expr::Array("A".into(), vec![Expr::Num(1.0)]));
    assert_eq!(
        expr("INT(1)"),
        Expr::Call(Builtin::Int, vec![Expr::Num(1.0)])
    );
    assert_eq!(
        stmts("DEF FNA(X) = X"),
        vec![Stmt::Def {
            name: "FNA".into(),
            params: vec!["X".into()],
            body: Some(Expr::Var("X".into()))
        }]
    );
}

#[test]
fn data_items() {
    assert_eq!(
        stmts("DATA 1, \"A,B\", HELLO WORLD"),
        vec![Stmt::Data(vec![
            DataItem {
                text: "1".into(),
                quoted: false
            },
            DataItem {
                text: "A,B".into(),
                quoted: true
            },
            DataItem {
                text: "HELLO WORLD".into(),
                quoted: false
            },
        ])]
    );
}

#[test]
fn lines_are_sorted_by_number() {
    let program = parse("20 PRINT 2\n10 PRINT 1").unwrap();
    assert_eq!(
        program.lines.iter().map(|l| l.number).collect::<Vec<_>>(),
        vec![Some(10), Some(20)]
    );
}

#[test]
fn errors_carry_the_line_number() {
    let err = parse("10 PRINT 1\n20 FOO BAR").unwrap_err();
    assert_eq!(err.line, Some(20));
    assert!(parse("10 PRINT 1\n10 PRINT 2").is_err());
    assert!(parse("10 IF X PRINT 1").is_err());
    assert!(parse("10 X = (1 + 2").is_err());
}
