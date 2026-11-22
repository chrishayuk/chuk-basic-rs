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
    /// `PRINT USING format$; value, ...` - `newline` is false when the
    /// statement ends with a separator.
    PrintUsing {
        format: Expr,
        values: Vec<Expr>,
        newline: bool,
    },
    Input {
        prompt: Option<String>,
        question_mark: bool,
        targets: Vec<LValue>,
    },
    /// `LINE INPUT "prompt"; A$` - reads a whole line, commas and all.
    LineInput {
        prompt: Option<String>,
        target: LValue,
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
    While(Expr),
    Wend,
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
    Mat(Mat),
    Rem(String),
    End,
    Stop,
}

/// The Dartmouth matrix statements. Matrices are ordinary arrays, of which
/// row 0 and column 0 are left alone.
#[derive(Debug, Clone, PartialEq)]
pub enum Mat {
    /// `MAT READ A, B(2, 3)`
    Read(Vec<MatTarget>),
    /// `MAT INPUT A`
    Input(Vec<MatTarget>),
    /// `MAT PRINT A; B` - the flag is true for a matrix followed by `;`,
    /// which is printed packed rather than in zones.
    Print(Vec<(String, bool)>),
    /// `MAT C = ...`
    Assign { target: String, value: MatExpr },
}

/// A matrix to fill, optionally giving it new dimensions first.
#[derive(Debug, Clone, PartialEq)]
pub struct MatTarget {
    pub name: String,
    pub dims: Vec<Expr>,
}

/// The right-hand side of a `MAT` assignment.
#[derive(Debug, Clone, PartialEq)]
pub enum MatExpr {
    Copy(String),
    Add(String, String),
    Sub(String, String),
    Mul(String, String),
    /// `(k) * A`
    Scale(Expr, String),
    /// All zeros; all ones; the identity matrix. Dimensions are optional.
    Zer(Vec<Expr>),
    Con(Vec<Expr>),
    Idn(Vec<Expr>),
    /// Transpose.
    Trn(String),
    /// Inverse.
    Inv(String),
}
