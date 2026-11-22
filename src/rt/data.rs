//! The DATA items of a program, and the READ pointer.

use super::fail;
use super::format::parse_number;
use super::value::Value;
use crate::ast::DataItem;
use crate::error::Result;

#[derive(Default)]
pub struct DataPool {
    items: Vec<DataItem>,
    /// (line number, index of the first item on or after that line)
    lines: Vec<(u32, usize)>,
    next: usize,
}

impl DataPool {
    /// Note that line `number` starts here, so that `RESTORE number` can
    /// find it. Lines must be noted in order.
    pub fn mark_line(&mut self, number: u32) {
        self.lines.push((number, self.items.len()));
    }

    pub fn push(&mut self, item: DataItem) {
        self.items.push(item);
    }

    /// The items, and the (line number, index of first item) marks.
    pub fn contents(&self) -> (&[DataItem], &[(u32, usize)]) {
        (&self.items, &self.lines)
    }

    /// Whether every item has been read.
    pub fn exhausted(&self) -> bool {
        self.next >= self.items.len()
    }

    /// `READ`: the next item, as a string or as a number.
    pub fn read(&mut self, string: bool) -> Result<Value> {
        let Some(item) = self.items.get(self.next) else {
            return fail("Out of DATA");
        };
        self.next += 1;
        if string {
            return Ok(Value::Str(item.text.clone()));
        }
        // an empty item (`DATA 1,,3`) reads as zero
        let number = if item.quoted {
            None
        } else if item.text.is_empty() {
            Some(0.0)
        } else {
            parse_number(&item.text)
        };
        match number {
            Some(n) => Ok(Value::Num(n)),
            None => fail(format!("Bad number in DATA: {}", item.text)),
        }
    }

    /// `RESTORE [line]`: read from the start again, or from the first DATA
    /// on or after `line`.
    pub fn restore(&mut self, line: Option<u32>) {
        self.next = match line {
            None => 0,
            Some(line) => self
                .lines
                .iter()
                .find(|(n, _)| *n >= line)
                .map_or(self.items.len(), |&(_, index)| index),
        };
    }
}
