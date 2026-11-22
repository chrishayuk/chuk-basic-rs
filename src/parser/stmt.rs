//! Statement parsing.

use super::data::parse_data;
use super::{describe, Parser};
use crate::ast::{Builtin, Expr, LValue, PrintItem, Stmt};
use crate::error::Result;
use crate::lexer::Tok;

impl Parser {
    pub(super) fn stmt(&mut self) -> Result<Stmt> {
        match self.next() {
            Tok::Let => self.let_stmt(),
            Tok::Ident(_) => {
                self.pos -= 1; // implicit LET
                self.let_stmt()
            }
            Tok::Print if self.eat(&Tok::Using) => self.print_using_stmt(),
            Tok::Print => self.print_stmt(),
            Tok::LineInput => self.line_input_stmt(),
            Tok::While => Ok(Stmt::While(self.expr()?)),
            Tok::Wend => Ok(Stmt::Wend),
            Tok::Mat => self.mat_stmt(),
            Tok::Input => self.input_stmt(),
            Tok::If => self.if_stmt(),
            Tok::For => self.for_stmt(),
            Tok::Next => {
                let mut vars = Vec::new();
                if !self.at_stmt_end() {
                    loop {
                        vars.push(self.ident("variable after NEXT")?);
                        if !self.eat(&Tok::Comma) {
                            break;
                        }
                    }
                }
                Ok(Stmt::Next(vars))
            }
            Tok::Goto => Ok(Stmt::Goto(self.line_target()?)),
            Tok::Gosub => Ok(Stmt::Gosub(self.line_target()?)),
            Tok::Return => Ok(Stmt::Return),
            Tok::On => self.on_stmt(),
            Tok::Def => self.def_stmt(),
            Tok::FnEnd => Ok(Stmt::FnEnd),
            Tok::Dim => self.dim_stmt(),
            Tok::Read => {
                let mut targets = vec![self.lvalue()?];
                while self.eat(&Tok::Comma) {
                    targets.push(self.lvalue()?);
                }
                Ok(Stmt::Read(targets))
            }
            Tok::Data(raw) => Ok(Stmt::Data(parse_data(&raw))),
            Tok::Restore => {
                let target = if self.at_stmt_end() {
                    None
                } else {
                    Some(self.line_target()?)
                };
                Ok(Stmt::Restore(target))
            }
            Tok::Randomize => {
                let seed = if self.at_stmt_end() {
                    None
                } else {
                    Some(self.expr()?)
                };
                Ok(Stmt::Randomize(seed))
            }
            Tok::Rem(text) => Ok(Stmt::Rem(text)),
            Tok::End => Ok(Stmt::End),
            Tok::Stop => Ok(Stmt::Stop),
            other => {
                self.pos -= 1;
                Err(self.err(format!("Unknown statement: {}", describe(&other))))
            }
        }
    }

    pub(super) fn lvalue(&mut self) -> Result<LValue> {
        let name = self.ident("a variable")?;
        if Builtin::from_name(&name).is_some() {
            self.pos -= 1;
            return Err(self.err(format!(
                "{name} is a built-in function and cannot be assigned"
            )));
        }
        let indices = if *self.peek() == Tok::LParen {
            self.args()?
        } else {
            Vec::new()
        };
        Ok(LValue { name, indices })
    }

    pub(super) fn let_stmt(&mut self) -> Result<Stmt> {
        let target = self.lvalue()?;
        self.expect(Tok::Eq, "'='")?;
        Ok(Stmt::Let {
            target,
            expr: self.expr()?,
        })
    }

    pub(super) fn print_stmt(&mut self) -> Result<Stmt> {
        let mut items = Vec::new();
        while !self.at_stmt_end() {
            let item = match self.peek() {
                Tok::Comma => {
                    self.pos += 1;
                    PrintItem::Comma
                }
                Tok::Semi => {
                    self.pos += 1;
                    PrintItem::Semi
                }
                Tok::Ident(name)
                    if (name == "TAB" || name == "SPC") && *self.peek2() == Tok::LParen =>
                {
                    let tab = name == "TAB";
                    self.pos += 2;
                    let expr = self.expr()?;
                    self.expect(Tok::RParen, "')'")?;
                    if tab {
                        PrintItem::Tab(expr)
                    } else {
                        PrintItem::Spc(expr)
                    }
                }
                // `PRINT "OWED $"-X`: a string cannot be subtracted from, so
                // the minus starts the next item
                Tok::Str(text) if *self.peek2() == Tok::Minus => {
                    let text = text.clone();
                    self.pos += 1;
                    PrintItem::Expr(Expr::Str(text))
                }
                _ => PrintItem::Expr(self.expr()?),
            };
            items.push(item);
        }
        Ok(Stmt::Print(items))
    }

    /// `PRINT USING format$; value [, value ...] [;]`
    pub(super) fn print_using_stmt(&mut self) -> Result<Stmt> {
        let format = self.expr()?;
        self.expect(Tok::Semi, "';' after the format")?;
        let mut values = Vec::new();
        let mut newline = true;
        while !self.at_stmt_end() {
            values.push(self.expr()?);
            newline = !(self.eat(&Tok::Semi) || self.eat(&Tok::Comma));
            if newline {
                break;
            }
        }
        if values.is_empty() {
            return Err(self.err("PRINT USING needs something to print"));
        }
        Ok(Stmt::PrintUsing {
            format,
            values,
            newline,
        })
    }

    /// `LINE INPUT ["prompt";] A$`
    pub(super) fn line_input_stmt(&mut self) -> Result<Stmt> {
        let mut prompt = None;
        if let (Tok::Str(text), Tok::Semi | Tok::Comma) = (self.peek(), self.peek2()) {
            prompt = Some(text.clone());
            self.pos += 2;
        }
        let target = self.lvalue()?;
        if !target.name.ends_with('$') {
            return Err(self.err("LINE INPUT needs a string variable"));
        }
        Ok(Stmt::LineInput { prompt, target })
    }

    pub(super) fn input_stmt(&mut self) -> Result<Stmt> {
        let mut prompt = None;
        let mut question_mark = true;
        if let (Tok::Str(text), Tok::Semi | Tok::Comma) = (self.peek(), self.peek2()) {
            prompt = Some(text.clone());
            question_mark = *self.peek2() == Tok::Semi;
            self.pos += 2;
        }
        let mut targets = vec![self.lvalue()?];
        while self.eat(&Tok::Comma) {
            targets.push(self.lvalue()?);
        }
        Ok(Stmt::Input {
            prompt,
            question_mark,
            targets,
        })
    }

    /// One branch of an IF: a bare line number, statements, or both.
    pub(super) fn branch(&mut self) -> Result<Vec<Stmt>> {
        let mut out = Vec::new();
        if matches!(self.peek(), Tok::Num(_)) {
            out.push(Stmt::Goto(self.line_target()?));
        }
        out.extend(self.stmts(true)?);
        if out.is_empty() {
            return Err(self.err("Expected a statement or line number"));
        }
        Ok(out)
    }

    pub(super) fn if_stmt(&mut self) -> Result<Stmt> {
        let cond = self.expr()?;
        let then = match self.peek() {
            Tok::Then => {
                self.pos += 1;
                self.branch()?
            }
            // IF X GOTO 100
            Tok::Goto => self.branch()?,
            other => return Err(self.err(format!("Expected THEN, found {}", describe(other)))),
        };
        let otherwise = if self.eat(&Tok::Else) {
            Some(self.branch()?)
        } else {
            None
        };
        Ok(Stmt::If {
            cond,
            then,
            otherwise,
        })
    }

    pub(super) fn for_stmt(&mut self) -> Result<Stmt> {
        let var = self.ident("a loop variable after FOR")?;
        if var.ends_with('$') {
            return Err(self.err("FOR needs a numeric variable"));
        }
        self.expect(Tok::Eq, "'='")?;
        let start = self.expr()?;
        self.expect(Tok::To, "TO")?;
        let end = self.expr()?;
        let step = if self.eat(&Tok::Step) {
            Some(self.expr()?)
        } else {
            None
        };
        Ok(Stmt::For {
            var,
            start,
            end,
            step,
        })
    }

    pub(super) fn on_stmt(&mut self) -> Result<Stmt> {
        let expr = self.expr()?;
        let gosub = match self.next() {
            Tok::Goto => false,
            Tok::Gosub => true,
            other => {
                self.pos -= 1;
                return Err(self.err(format!(
                    "Expected GOTO or GOSUB after ON, found {}",
                    describe(&other)
                )));
            }
        };
        let mut targets = vec![self.line_target()?];
        while self.eat(&Tok::Comma) {
            targets.push(self.line_target()?);
        }
        Ok(Stmt::On {
            expr,
            targets,
            gosub,
        })
    }

    pub(super) fn def_stmt(&mut self) -> Result<Stmt> {
        let mut name = self.ident("a function name after DEF")?;
        if name == "FN" {
            name.push_str(&self.ident("a function name after FN")?); // DEF FN A(X)
        }
        if !name.starts_with("FN") || name.len() < 3 {
            return Err(self.err("Function names must start with FN"));
        }
        let mut params = Vec::new();
        if self.eat(&Tok::LParen) && !self.eat(&Tok::RParen) {
            loop {
                params.push(self.ident("a parameter name")?);
                if !self.eat(&Tok::Comma) {
                    break;
                }
            }
            self.expect(Tok::RParen, "')'")?;
        }
        let body = if self.eat(&Tok::Eq) {
            Some(self.expr()?)
        } else {
            None
        };
        Ok(Stmt::Def { name, params, body })
    }

    pub(super) fn dim_stmt(&mut self) -> Result<Stmt> {
        let mut arrays = Vec::new();
        loop {
            let name = self.ident("an array name after DIM")?;
            if *self.peek() != Tok::LParen {
                return Err(self.err("Expected '(' after array name"));
            }
            let dims = self.args()?;
            if dims.is_empty() {
                return Err(self.err("DIM needs at least one dimension"));
            }
            arrays.push((name, dims));
            if !self.eat(&Tok::Comma) {
                return Ok(Stmt::Dim(arrays));
            }
        }
    }
}
