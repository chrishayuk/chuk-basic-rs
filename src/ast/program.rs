//! Program structure: numbered lines of statements.

use super::Stmt;

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    /// Lines in execution order (sorted by line number).
    pub lines: Vec<Line>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Line {
    /// BASIC line number; lines may be unnumbered.
    pub number: Option<u32>,
    /// 1-based line in the source text.
    pub src_line: usize,
    pub stmts: Vec<Stmt>,
}
