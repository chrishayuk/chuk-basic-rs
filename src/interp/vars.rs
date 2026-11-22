//! Variables and arrays.

use super::value::{default_value, Value};
use super::{Array, Interpreter, MAX_ARRAY_ELEMENTS};
use crate::ast::{Expr, LValue};
use crate::error::{Error, Result};

impl Interpreter<'_> {
    /// Check that `value` can be stored in a variable called `name`.
    pub(super) fn coerce(&self, name: &str, value: Value) -> Result<Value> {
        match (name.ends_with('$'), value) {
            (true, value @ Value::Str(_)) => Ok(value),
            (false, Value::Num(n)) if name.ends_with('%') => Ok(Value::Num(n.round())),
            (false, value @ Value::Num(_)) => Ok(value),
            _ => Err(self.type_mismatch()),
        }
    }

    pub(super) fn assign(&mut self, target: &LValue, value: Value) -> Result<()> {
        let value = self.coerce(&target.name, value)?;
        if target.indices.is_empty() {
            self.vars.insert(target.name.clone(), value);
        } else {
            let indices = self.eval_indices(&target.indices)?;
            *self.element(&target.name, &indices)? = value;
        }
        Ok(())
    }

    pub(super) fn eval_indices(&mut self, exprs: &[Expr]) -> Result<Vec<usize>> {
        let mut out = Vec::with_capacity(exprs.len());
        for expr in exprs {
            let n = self.eval_num(expr)?.round();
            if !(0.0..=u32::MAX as f64).contains(&n) {
                return Err(self.err("Subscript out of range"));
            }
            out.push(n as usize);
        }
        Ok(out)
    }

    pub(super) fn new_array(&self, name: &str, dims: Vec<usize>) -> Result<Array> {
        let len = dims
            .iter()
            .try_fold(1usize, |acc, &d| acc.checked_mul(d))
            .filter(|&len| len <= MAX_ARRAY_ELEMENTS)
            .ok_or_else(|| self.err(format!("Out of memory: array {name} is too large")))?;
        Ok(Array {
            dims,
            data: vec![default_value(name); len],
        })
    }

    pub(super) fn element(&mut self, name: &str, indices: &[usize]) -> Result<&mut Value> {
        if !self.arrays.contains_key(name) {
            // undeclared arrays get bounds of 10, as in Dartmouth and MS BASIC
            let array = self.new_array(name, vec![11; indices.len()])?;
            self.arrays.insert(name.to_string(), array);
        }
        let line = self.line;
        let array = self.arrays.get_mut(name).expect("just inserted");
        if array.dims.len() != indices.len() || indices.iter().zip(&array.dims).any(|(i, d)| i >= d)
        {
            return Err(Error::runtime("Subscript out of range", line));
        }
        let offset = indices
            .iter()
            .zip(&array.dims)
            .fold(0, |acc, (i, d)| acc * d + i);
        Ok(&mut array.data[offset])
    }
}
