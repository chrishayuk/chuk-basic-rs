//! Arrays.

use std::collections::HashMap;

use super::fail;
use super::value::{default_value, Value};
use crate::error::Result;

const MAX_ELEMENTS: usize = 10_000_000;

/// The upper bound that an array gets when it is used without a DIM.
pub const DEFAULT_BOUND: usize = 10;

pub struct Array {
    /// Size of each dimension (upper bound + 1, since subscripts start at 0).
    pub dims: Vec<usize>,
    /// Elements in row-major order.
    pub data: Vec<Value>,
}

impl Array {
    /// A new array called `name` (which decides whether it holds strings),
    /// with the given upper bounds.
    pub fn new(name: &str, bounds: &[usize]) -> Result<Array> {
        let dims: Vec<usize> = bounds.iter().map(|bound| bound + 1).collect();
        let len = dims
            .iter()
            .try_fold(1usize, |acc, &d| acc.checked_mul(d))
            .filter(|&len| len <= MAX_ELEMENTS);
        match len {
            Some(len) => Ok(Array {
                dims,
                data: vec![default_value(name); len],
            }),
            None => fail(format!("Out of memory: array {name} is too large")),
        }
    }

    /// The upper bound of each dimension.
    pub fn bounds(&self) -> Vec<usize> {
        self.dims.iter().map(|d| d - 1).collect()
    }

    fn offset(&self, indices: &[usize]) -> Result<usize> {
        if self.dims.len() != indices.len() || indices.iter().zip(&self.dims).any(|(i, d)| i >= d) {
            return fail("Subscript out of range");
        }
        Ok(indices
            .iter()
            .zip(&self.dims)
            .fold(0, |acc, (i, d)| acc * d + i))
    }
}

/// Convert a subscript to an index: rounded to the nearest whole number,
/// and not negative.
pub fn subscript(n: f64) -> Result<usize> {
    let n = n.round();
    if !(0.0..=u32::MAX as f64).contains(&n) {
        return fail("Subscript out of range");
    }
    Ok(n as usize)
}

/// All the arrays of a running program, by name.
#[derive(Default)]
pub struct Arrays(HashMap<String, Array>);

impl Arrays {
    /// `DIM name(bounds)`
    pub fn dim(&mut self, name: &str, bounds: &[usize]) -> Result<()> {
        if self.0.contains_key(name) {
            return fail(format!("Array {name} is already dimensioned"));
        }
        self.0.insert(name.to_string(), Array::new(name, bounds)?);
        Ok(())
    }

    /// The element `name(indices)`. An array that was never dimensioned
    /// gets bounds of 10, as in Dartmouth and Microsoft BASIC.
    pub fn element(&mut self, name: &str, indices: &[usize]) -> Result<&mut Value> {
        if !self.0.contains_key(name) {
            self.0.insert(
                name.to_string(),
                Array::new(name, &vec![DEFAULT_BOUND; indices.len()])?,
            );
        }
        let array = self.0.get_mut(name).expect("just inserted");
        let offset = array.offset(indices)?;
        Ok(&mut array.data[offset])
    }

    pub fn get(&self, name: &str) -> Option<&Array> {
        self.0.get(name)
    }

    /// Replace (or create) a whole array.
    pub fn set(&mut self, name: &str, array: Array) {
        self.0.insert(name.to_string(), array);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn elements_are_addressed_in_row_major_order() {
        let mut arrays = Arrays::default();
        arrays.dim("G", &[2, 3]).unwrap();
        *arrays.element("G", &[1, 2]).unwrap() = Value::Num(7.0);
        assert_eq!(arrays.get("G").unwrap().data[6], Value::Num(7.0));
        assert!(arrays.element("G", &[3, 0]).is_err());
        assert!(arrays.element("G", &[1]).is_err());
        assert!(arrays.dim("G", &[1]).is_err());
    }

    #[test]
    fn undeclared_arrays_go_up_to_ten() {
        let mut arrays = Arrays::default();
        assert_eq!(
            *arrays.element("N$", &[10]).unwrap(),
            Value::Str(String::new())
        );
        assert!(arrays.element("N$", &[11]).is_err());
        assert_eq!(subscript(1.6).unwrap(), 2);
        assert!(subscript(-1.0).is_err());
    }
}
