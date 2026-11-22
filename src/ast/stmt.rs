//! Statements.

use super::Expr;

/// An assignable location: a scalar variable or an array element.
#[derive(Debug, Clone, PartialEq)]
pub struct LValue {
    pub name: String,
    pub indices: Vec<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PrintItem {
    Expr(Expr),
    Tab(Expr),
    Spc(Expr),
    /// `,` - advance to the next print zone.
    Comma,
    /// `;` - no spacing.
    Semi,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataItem {
    pub text: String,
    pub quoted: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Let {
        target: LValue,
        expr: Expr,
    },
    Print(Vec<PrintItem>),
    Input {
        prompt: Option<String>,
        question_mark: bool,
        targets: Vec<LValue>,
    },
    If {
        cond: Expr,
        then: Vec<Stmt>,
        otherwise: Option<Vec<Stmt>>,
    },
    For {
        var: String,
        start: Expr,
        end: Expr,
        step: Option<Expr>,
    },
    Next(Vec<String>),
    Goto(u32),
    Gosub(u32),
    Return,
    On {
        expr: Expr,
        targets: Vec<u32>,
        gosub: bool,
    },
    /// `DEF FNA(X) = expr`, or with `body: None` the start of a multi-line
    /// function that runs until `FNEND`.
    Def {
        name: String,
        params: Vec<String>,
        body: Option<Expr>,
    },
    FnEnd,
    Dim(Vec<(String, Vec<Expr>)>),
    Read(Vec<LValue>),
    Data(Vec<DataItem>),
    Restore(Option<u32>),
    Randomize(Option<Expr>),
    Rem(String),
    End,
    Stop,
}
