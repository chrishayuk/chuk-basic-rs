//! The Dartmouth MAT statements.
//!
//! A matrix is an ordinary numeric array used from subscript 1: row 0 and
//! column 0 are left alone. A one-dimensional array is a column vector.

use super::arrays::{Array, Arrays};
use super::console::Console;
use super::data::DataPool;
use super::value::Value;
use super::{fail, type_mismatch};
use crate::error::Result;

/// The right-hand side of `MAT target = ...`, with its expressions already
/// worked out.
pub enum MatOp<'a> {
    Copy(&'a str),
    Add(&'a str, &'a str),
    Sub(&'a str, &'a str),
    Mul(&'a str, &'a str),
    Scale(f64, &'a str),
    /// All zeros, all ones, and the identity; the dimensions may be empty,
    /// meaning "the shape the target already has".
    Zer(&'a [usize]),
    Con(&'a [usize]),
    Idn(&'a [usize]),
    Trn(&'a str),
    Inv(&'a str),
}

/// The part of an array that MAT works on.
#[derive(Debug, Clone, PartialEq)]
struct Matrix {
    rows: usize,
    cols: usize,
    /// True if this came from (or should become) a one-dimensional array.
    vector: bool,
    /// `rows * cols` numbers, row by row.
    cells: Vec<f64>,
}

impl Matrix {
    fn filled(rows: usize, cols: usize, vector: bool, value: f64) -> Matrix {
        Matrix {
            rows,
            cols,
            vector,
            cells: vec![value; rows * cols],
        }
    }

    fn at(&self, row: usize, col: usize) -> f64 {
        self.cells[row * self.cols + col]
    }

    fn at_mut(&mut self, row: usize, col: usize) -> &mut f64 {
        &mut self.cells[row * self.cols + col]
    }
}

fn numeric(name: &str) -> Result<()> {
    if name.ends_with('$') {
        return type_mismatch();
    }
    Ok(())
}

/// The shape given by `dims` (one or two upper bounds).
fn shape(name: &str, dims: &[usize]) -> Result<(usize, usize, bool)> {
    match *dims {
        [rows] if rows > 0 => Ok((rows, 1, true)),
        [rows, cols] if rows > 0 && cols > 0 => Ok((rows, cols, false)),
        _ => fail(format!(
            "Matrix {name} needs one or two dimensions of at least 1"
        )),
    }
}

fn load(arrays: &Arrays, name: &str) -> Result<Matrix> {
    numeric(name)?;
    let Some(array) = arrays.get(name) else {
        return fail(format!("Matrix {name} has not been dimensioned"));
    };
    let (rows, cols, vector) = shape(name, &array.bounds())?;
    let stride = if vector { 1 } else { cols + 1 };
    let mut matrix = Matrix::filled(rows, cols, vector, 0.0);
    for row in 0..rows {
        for col in 0..cols {
            let offset = (row + 1) * stride + if vector { 0 } else { col + 1 };
            if let Value::Num(n) = array.data[offset] {
                *matrix.at_mut(row, col) = n;
            }
        }
    }
    Ok(matrix)
}

fn store(arrays: &mut Arrays, name: &str, matrix: &Matrix) -> Result<()> {
    numeric(name)?;
    let vector = matrix.vector && matrix.cols == 1;
    let bounds = if vector {
        vec![matrix.rows]
    } else {
        vec![matrix.rows, matrix.cols]
    };
    let mut array = Array::new(name, &bounds)?;
    let stride = if vector { 1 } else { matrix.cols + 1 };
    for row in 0..matrix.rows {
        for col in 0..matrix.cols {
            let offset = (row + 1) * stride + if vector { 0 } else { col + 1 };
            array.data[offset] = Value::Num(matrix.at(row, col));
        }
    }
    arrays.set(name, array);
    Ok(())
}

/// An empty matrix to fill: with the shape `dims` if given, otherwise with
/// the shape `name` already has.
fn blank(arrays: &Arrays, name: &str, dims: &[usize]) -> Result<Matrix> {
    if dims.is_empty() {
        let mut matrix = load(arrays, name)?;
        matrix.cells.fill(0.0);
        return Ok(matrix);
    }
    numeric(name)?;
    let (rows, cols, vector) = shape(name, dims)?;
    Ok(Matrix::filled(rows, cols, vector, 0.0))
}

/// `MAT READ name[(dims)]`: fill from DATA, row by row.
pub fn read(arrays: &mut Arrays, data: &mut DataPool, name: &str, dims: &[usize]) -> Result<()> {
    let mut matrix = blank(arrays, name, dims)?;
    for cell in &mut matrix.cells {
        if let Value::Num(n) = data.read(false)? {
            *cell = n;
        }
    }
    store(arrays, name, &matrix)
}

/// `MAT INPUT name[(dims)]`: one line of numbers for each row.
pub fn input(arrays: &mut Arrays, console: &mut Console, name: &str, dims: &[usize]) -> Result<()> {
    let mut matrix = blank(arrays, name, dims)?;
    let wanted = vec![false; matrix.cols];
    for row in 0..matrix.rows {
        for (col, value) in console.input(None, true, &wanted)?.into_iter().enumerate() {
            if let Value::Num(n) = value {
                *matrix.at_mut(row, col) = n;
            }
        }
    }
    store(arrays, name, &matrix)
}

/// `MAT PRINT name`: each row on a line, in print zones or (when `packed`)
/// close together, with a blank line after the matrix. A vector is printed
/// as one row.
pub fn print(arrays: &Arrays, console: &mut Console, name: &str, packed: bool) -> Result<()> {
    let matrix = load(arrays, name)?;
    let (rows, cols) = if matrix.vector {
        (1, matrix.rows)
    } else {
        (matrix.rows, matrix.cols)
    };
    for row in 0..rows {
        for col in 0..cols {
            if col > 0 && !packed {
                console.next_zone()?;
            }
            console.print_value(&Value::Num(matrix.cells[row * cols + col]))?;
        }
        console.newline()?;
    }
    console.newline()
}

/// `MAT target = op`. The target takes the shape of the result.
pub fn assign(arrays: &mut Arrays, target: &str, op: MatOp) -> Result<()> {
    let result = match op {
        MatOp::Copy(a) => load(arrays, a)?,
        MatOp::Add(a, b) => zip(load(arrays, a)?, &load(arrays, b)?, |x, y| x + y)?,
        MatOp::Sub(a, b) => zip(load(arrays, a)?, &load(arrays, b)?, |x, y| x - y)?,
        MatOp::Mul(a, b) => multiply(&load(arrays, a)?, &load(arrays, b)?)?,
        MatOp::Scale(k, a) => {
            let mut matrix = load(arrays, a)?;
            matrix.cells.iter_mut().for_each(|cell| *cell *= k);
            matrix
        }
        MatOp::Zer(dims) => blank(arrays, target, dims)?,
        MatOp::Con(dims) => {
            let mut matrix = blank(arrays, target, dims)?;
            matrix.cells.fill(1.0);
            matrix
        }
        MatOp::Idn(dims) => {
            let mut matrix = blank(arrays, target, dims)?;
            if matrix.rows != matrix.cols {
                return fail("An identity matrix must be square");
            }
            for i in 0..matrix.rows {
                *matrix.at_mut(i, i) = 1.0;
            }
            matrix
        }
        MatOp::Trn(a) => {
            let from = load(arrays, a)?;
            let mut matrix = Matrix::filled(from.cols, from.rows, false, 0.0);
            for row in 0..from.rows {
                for col in 0..from.cols {
                    *matrix.at_mut(col, row) = from.at(row, col);
                }
            }
            matrix
        }
        MatOp::Inv(a) => invert(load(arrays, a)?)?,
    };
    store(arrays, target, &result)
}

fn zip(mut a: Matrix, b: &Matrix, f: impl Fn(f64, f64) -> f64) -> Result<Matrix> {
    if (a.rows, a.cols) != (b.rows, b.cols) {
        return fail("Matrix dimensions do not match");
    }
    for (x, y) in a.cells.iter_mut().zip(&b.cells) {
        *x = f(*x, *y);
    }
    Ok(a)
}

fn multiply(a: &Matrix, b: &Matrix) -> Result<Matrix> {
    if a.cols != b.rows {
        return fail("Matrix dimensions do not match");
    }
    let mut product = Matrix::filled(a.rows, b.cols, b.vector, 0.0);
    for row in 0..a.rows {
        for col in 0..b.cols {
            *product.at_mut(row, col) = (0..a.cols).map(|k| a.at(row, k) * b.at(k, col)).sum();
        }
    }
    Ok(product)
}

/// Gauss-Jordan elimination with partial pivoting.
fn invert(mut a: Matrix) -> Result<Matrix> {
    if a.rows != a.cols {
        return fail("Only a square matrix can be inverted");
    }
    let n = a.rows;
    let mut inverse = Matrix::filled(n, n, false, 0.0);
    for i in 0..n {
        *inverse.at_mut(i, i) = 1.0;
    }
    for col in 0..n {
        let pivot = (col..n)
            .max_by(|&x, &y| a.at(x, col).abs().total_cmp(&a.at(y, col).abs()))
            .expect("at least one row");
        if a.at(pivot, col).abs() < 1e-12 {
            return fail("Matrix is singular");
        }
        for k in 0..n {
            a.cells.swap(col * n + k, pivot * n + k);
            inverse.cells.swap(col * n + k, pivot * n + k);
        }
        let scale = a.at(col, col);
        for k in 0..n {
            *a.at_mut(col, k) /= scale;
            *inverse.at_mut(col, k) /= scale;
        }
        for row in (0..n).filter(|&row| row != col) {
            let factor = a.at(row, col);
            for k in 0..n {
                *a.at_mut(row, k) -= factor * a.at(col, k);
                *inverse.at_mut(row, k) -= factor * inverse.at(col, k);
            }
        }
    }
    Ok(inverse)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn matrix(rows: usize, cols: usize, cells: &[f64]) -> Matrix {
        Matrix {
            rows,
            cols,
            vector: false,
            cells: cells.to_vec(),
        }
    }

    #[test]
    fn stores_from_subscript_one() {
        let mut arrays = Arrays::default();
        store(&mut arrays, "A", &matrix(2, 2, &[1.0, 2.0, 3.0, 4.0])).unwrap();
        assert_eq!(*arrays.element("A", &[1, 1]).unwrap(), Value::Num(1.0));
        assert_eq!(*arrays.element("A", &[2, 1]).unwrap(), Value::Num(3.0));
        assert_eq!(*arrays.element("A", &[0, 0]).unwrap(), Value::Num(0.0));
        assert_eq!(
            load(&arrays, "A").unwrap(),
            matrix(2, 2, &[1.0, 2.0, 3.0, 4.0])
        );
    }

    #[test]
    fn multiplies_and_inverts() {
        let a = matrix(2, 2, &[4.0, 7.0, 2.0, 6.0]);
        let inverse = invert(a.clone()).unwrap();
        let expected = [0.6, -0.7, -0.2, 0.4];
        assert!(inverse
            .cells
            .iter()
            .zip(expected)
            .all(|(x, y)| (x - y).abs() < 1e-12));
        let product = multiply(&a, &inverse).unwrap();
        let identity = [1.0, 0.0, 0.0, 1.0];
        assert!(product
            .cells
            .iter()
            .zip(identity)
            .all(|(x, y)| (x - y).abs() < 1e-12));
        assert!(invert(matrix(2, 2, &[1.0, 2.0, 2.0, 4.0])).is_err());
        assert!(multiply(&a, &matrix(3, 1, &[1.0, 2.0, 3.0])).is_err());
    }
}
