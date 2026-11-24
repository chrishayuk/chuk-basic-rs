//! PRINT, PRINT USING, INPUT and LINE INPUT.

use super::Interpreter;
use crate::ast::{Expr, LValue, PrintItem};
use crate::error::{AtLine, Result};
use crate::rt::using::format_using;
use crate::rt::Value;

impl Interpreter<'_> {
    pub(super) fn print(&mut self, items: &[PrintItem]) -> Result<()> {
        for item in items {
            match item {
                PrintItem::Expr(expr) => {
                    let value = self.eval(expr)?;
                    self.console.print_value(&value)
                }
                PrintItem::Tab(expr) => {
                    let col = self.eval_num(expr)?;
                    self.console.tab(col)
                }
                PrintItem::Spc(expr) => {
                    let n = self.eval_num(expr)?;
                    self.console.spc(n)
                }
                PrintItem::Comma => self.console.next_zone(),
                PrintItem::Semi => self.console.packed(),
            }
            .at(self.line)?;
        }
        if !matches!(items.last(), Some(PrintItem::Comma | PrintItem::Semi)) {
            self.console.newline().at(self.line)?;
        }
        Ok(())
    }

    pub(super) fn print_using(
        &mut self,
        format: &Expr,
        values: &[Expr],
        newline: bool,
    ) -> Result<()> {
        let format = self.eval_str(format)?;
        let values = values
            .iter()
            .map(|value| self.eval(value))
            .collect::<Result<Vec<Value>>>()?;
        let mut text = format_using(&format, &values).at(self.line)?;
        if newline {
            text.push('\n');
        }
        self.console.write(&text).at(self.line)
    }

    pub(super) fn input(
        &mut self,
        prompt: Option<&str>,
        question_mark: bool,
        targets: &[LValue],
    ) -> Result<()> {
        let strings: Vec<bool> = targets
            .iter()
            .map(|target| target.name.ends_with('$'))
            .collect();
        let values = self
            .console
            .input(prompt, question_mark, &strings)
            .at(self.line)?;
        for (target, value) in targets.iter().zip(values) {
            self.assign(target, value)?;
        }
        Ok(())
    }

    pub(super) fn line_input(&mut self, prompt: Option<&str>, target: &LValue) -> Result<()> {
        let line = self.console.line_input(prompt).at(self.line)?;
        self.assign(target, Value::Str(line))
    }
}
