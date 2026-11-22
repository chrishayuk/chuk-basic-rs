//! Number formatting and parsing.

/// A number as PRINT and STR$ show it: a leading space stands in for `+`.
pub fn signed(n: f64) -> String {
    let text = format_number(n);
    if text.starts_with('-') {
        text
    } else {
        format!(" {text}")
    }
}

/// Format a number the way classic BASICs do: integers without a decimal
/// point, at most 9 significant digits, no leading zero (`.5`), and
/// exponent notation for very large or very small values.
pub fn format_number(n: f64) -> String {
    format_digits(n, 9)
}

/// [`format_number`] with another number of significant digits for values
/// that are not whole.
pub fn format_digits(n: f64, digits: usize) -> String {
    if n.is_nan() {
        return "NAN".to_string();
    }
    if n.is_infinite() {
        return if n > 0.0 { "INF" } else { "-INF" }.to_string();
    }
    if n == 0.0 {
        return "0".to_string();
    }
    if n.fract() == 0.0 && n.abs() < 1e15 {
        return format!("{}", n as i64);
    }
    let trim = |s: &str| -> String {
        if s.contains('.') {
            s.trim_end_matches('0').trim_end_matches('.').to_string()
        } else {
            s.to_string()
        }
    };
    let sci = format!("{n:.*e}", digits - 1);
    let (mantissa, exponent) = sci.split_once('e').expect("exponent format");
    let exponent: i32 = exponent.parse().expect("exponent digits");
    if !(-7..15).contains(&exponent) {
        let sign = if exponent < 0 { '-' } else { '+' };
        return format!("{}E{sign}{:02}", trim(mantissa), exponent.abs());
    }
    let decimals = (digits as i32 - 1 - exponent).max(0) as usize;
    let fixed = trim(&format!("{n:.decimals$}"));
    match fixed.strip_prefix("-0.") {
        Some(rest) => format!("-.{rest}"),
        None => match fixed.strip_prefix("0.") {
            Some(rest) => format!(".{rest}"),
            None => fixed,
        },
    }
}

/// Parse a complete numeric literal (as typed at INPUT or written in DATA).
pub fn parse_number(text: &str) -> Option<f64> {
    let text: String = text.chars().filter(|c| !c.is_whitespace()).collect();
    // f64::from_str also accepts "inf", "nan" and friends - BASIC does not
    if text.is_empty()
        || !text
            .chars()
            .all(|c| c.is_ascii_digit() || "+-.eE".contains(c))
    {
        return None;
    }
    text.parse().ok()
}

/// VAL: the value of the longest numeric prefix, or 0.
pub fn val(text: &str) -> f64 {
    let text: String = text.chars().filter(|c| !c.is_whitespace()).collect();
    let end = text
        .find(|c: char| !(c.is_ascii_digit() || "+-.eE".contains(c)))
        .unwrap_or(text.len());
    (1..=end)
        .rev()
        .find_map(|len| parse_number(&text[..len]))
        .unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn number_formatting() {
        assert_eq!(format_number(0.0), "0");
        assert_eq!(format_number(42.0), "42");
        assert_eq!(format_number(-7.0), "-7");
        assert_eq!(format_number(0.5), ".5");
        assert_eq!(format_number(-0.5), "-.5");
        assert_eq!(format_number(0.1 + 0.2), ".3");
        assert_eq!(format_number(2.0 / 3.0), ".666666667");
        assert_eq!(format_number(-22.0 / 6.0), "-3.66666667");
        assert_eq!(format_number(1234.5), "1234.5");
        assert_eq!(format_number(1e20), "1E+20");
        assert_eq!(format_number(1.5e-10), "1.5E-10");
        assert_eq!(format_number(1234567890.0), "1234567890");
    }

    #[test]
    fn val_takes_the_numeric_prefix() {
        assert_eq!(val("12"), 12.0);
        assert_eq!(val(" 3.5 apples"), 3.5);
        assert_eq!(val("-2E3x"), -2000.0);
        assert_eq!(val("abc"), 0.0);
        assert_eq!(val(""), 0.0);
    }

    #[test]
    fn numbers_typed_by_the_user() {
        assert_eq!(parse_number(" 12 "), Some(12.0));
        assert_eq!(parse_number("-.5"), Some(-0.5));
        assert_eq!(parse_number("inf"), None);
        assert_eq!(parse_number("1x"), None);
    }
}
