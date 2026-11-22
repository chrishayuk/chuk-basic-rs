//! Runtime values.

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Num(f64),
    Str(String),
}

/// The value of a variable that has never been assigned.
pub fn default_value(name: &str) -> Value {
    if name.ends_with('$') {
        Value::Str(String::new())
    } else {
        Value::Num(0.0)
    }
}
