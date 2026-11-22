//! Runtime support: everything a running BASIC program needs that does not
//! depend on how the program itself is represented.
//!
//! The interpreter (`crate::interp`) and programs compiled to WebAssembly
//! share this code, which is what keeps the two printing the same thing.
//! Errors raised here carry no line number; the caller knows which line is
//! running and adds it.

pub mod arrays;
pub mod builtins;
pub mod console;
pub mod data;
pub mod format;
pub mod mat;
pub mod rng;
pub mod using;
pub mod value;

pub use arrays::{subscript, Array, Arrays};
pub use console::Console;
pub use data::DataPool;
pub use format::format_number;
pub use rng::Rnd;
pub use value::{default_value, Value};

use crate::error::{Error, Result};

/// A runtime error with no line number yet.
pub(crate) fn fail<T>(message: impl Into<String>) -> Result<T> {
    Err(Error::runtime(message, None))
}

pub(crate) fn type_mismatch<T>() -> Result<T> {
    fail("Type mismatch")
}

/// Check that `value` can be stored in a variable called `name`: strings go
/// in `$` variables, numbers in the others, and `%` variables hold whole
/// numbers.
pub fn coerce(name: &str, value: Value) -> Result<Value> {
    match (name.ends_with('$'), value) {
        (true, value @ Value::Str(_)) => Ok(value),
        (false, Value::Num(n)) if name.ends_with('%') => Ok(Value::Num(n.round())),
        (false, value @ Value::Num(_)) => Ok(value),
        _ => type_mismatch(),
    }
}
