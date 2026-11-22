//! Token types produced by the lexer.

#[derive(Debug, Clone, PartialEq)]
pub enum Tok {
    /// Line number at the start of a line.
    LineNo(u32),
    Num(f64),
    Str(String),
    /// Variable, array, built-in or user function name (upper-cased).
    Ident(String),
    /// `REM` (or `'`) with the comment text.
    Rem(String),
    /// `DATA` with its raw, unparsed item list.
    Data(String),

    // keywords
    Let,
    Print,
    Input,
    If,
    Then,
    Else,
    For,
    To,
    Step,
    Next,
    Goto,
    Gosub,
    Return,
    On,
    Def,
    FnEnd,
    Dim,
    Read,
    Restore,
    End,
    Stop,
    Randomize,
    And,
    Or,
    Not,

    // punctuation and operators
    Comma,
    Semi,
    Colon,
    LParen,
    RParen,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Plus,
    Minus,
    Star,
    Slash,
    Caret,

    /// End of a source line.
    Eol,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub tok: Tok,
    /// 1-based line in the source text.
    pub src_line: usize,
}

/// Map an upper-cased word to its keyword token, if it is one.
pub(super) fn keyword(word: &str) -> Option<Tok> {
    Some(match word {
        "LET" => Tok::Let,
        "PRINT" => Tok::Print,
        "INPUT" => Tok::Input,
        "IF" => Tok::If,
        "THEN" => Tok::Then,
        "ELSE" => Tok::Else,
        "FOR" => Tok::For,
        "TO" => Tok::To,
        "STEP" => Tok::Step,
        "NEXT" => Tok::Next,
        "GOTO" => Tok::Goto,
        "GOSUB" => Tok::Gosub,
        "RETURN" => Tok::Return,
        "ON" => Tok::On,
        "DEF" => Tok::Def,
        "FNEND" => Tok::FnEnd,
        "DIM" => Tok::Dim,
        "READ" => Tok::Read,
        "RESTORE" => Tok::Restore,
        "END" => Tok::End,
        "STOP" => Tok::Stop,
        "RANDOMIZE" => Tok::Randomize,
        "AND" => Tok::And,
        "OR" => Tok::Or,
        "NOT" => Tok::Not,
        _ => return None,
    })
}
