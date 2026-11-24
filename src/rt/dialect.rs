//! Where the BASICs disagree about arithmetic and about printing numbers.

use super::format::{format_digits, format_number};

/// Which BASIC's arithmetic to follow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Dialect {
    /// The Microsoft BASICs of the 1970s and after: the square root of a
    /// negative number is an error.
    #[default]
    Microsoft,
    /// Dartmouth BASIC as its manuals of May and October 1964 describe it.
    /// `SQR` and `LOG` take the absolute value of their argument (October,
    /// p. 11; May, p. 5 for `SQR`), and `A ↑ B` "is interpreted as |A| ↑ B"
    /// (October, p. 9). When `READ` finds no data left, "the computer stops"
    /// (May, p. 15). `INT` chops off the fractional part: "INT(-7.8) is -7,
    /// not -8" (October, p. 39); `RND`'s argument "has no significance"
    /// (October, p. 39). Numbers print by the four rules of October p. 34
    /// (see [`Dialect::number`]).
    ///
    /// Neither manual says how a leading minus binds; for that this follows
    /// Thomas Kurtz's account of the original BASIC
    /// (dartmouth.edu/basicfifty/commands.html): `-X↑2` is `(-X)↑2`.
    Dartmouth,
}

impl Dialect {
    /// The dialect called `name` on the command line.
    pub fn from_name(name: &str) -> Option<Dialect> {
        match name.to_ascii_lowercase().as_str() {
            "microsoft" => Some(Dialect::Microsoft),
            "dartmouth" => Some(Dialect::Dartmouth),
            _ => None,
        }
    }

    /// `SQR(x)`, or `None` where that is an error.
    pub fn sqrt(self, x: f64) -> Option<f64> {
        match self {
            Dialect::Dartmouth => Some(x.abs().sqrt()),
            Dialect::Microsoft => (x >= 0.0).then(|| x.sqrt()),
        }
    }

    /// `INT(x)`: the whole number below `x` in later BASICs; in 1964, "INT
    /// always operates by chopping off the fractional part, whether the
    /// number is positive or negative" (October 1964 manual, p. 39).
    pub fn int(self, x: f64) -> f64 {
        match self {
            Dialect::Microsoft => x.floor(),
            Dialect::Dartmouth => x.trunc(),
        }
    }

    /// The argument `RND(x)` is taken to have. In 1964 "the argument has no
    /// significance" (October 1964 manual, p. 39): every call gives the next
    /// number, `RND(0)` included.
    pub fn rnd_argument(self, x: f64) -> f64 {
        match self {
            Dialect::Microsoft => x,
            Dialect::Dartmouth => 1.0,
        }
    }

    /// `-A ↑ B` written without brackets, given the value `v` of `A ↑ B`.
    /// Later BASICs read it as `-(A ↑ B)`. Kurtz: in 1964 "a leading minus
    /// sign ... has higher precedence than exponentiation", so it is
    /// `(-A) ↑ B`, which with the absolute value in every power is `v`.
    pub fn leading_minus(self, v: f64) -> f64 {
        match self {
            Dialect::Microsoft => -v,
            Dialect::Dartmouth => v,
        }
    }

    /// Whether `READ` with no data left ends the program normally, rather
    /// than being an error.
    pub fn read_past_data_ends(self) -> bool {
        self == Dialect::Dartmouth
    }

    /// `LOG(x)`, or `None` where that is an error. In 1964 "the argument is
    /// made positive before applying the function" (October manual, p. 11);
    /// `LOG(0)` is still an error (our choice: the manual does not say).
    pub fn log(self, x: f64) -> Option<f64> {
        match self {
            Dialect::Dartmouth => (x != 0.0).then(|| x.abs().ln()),
            Dialect::Microsoft => (x > 0.0).then(|| x.ln()),
        }
    }

    /// A number as `PRINT` shows it, without the sign column.
    ///
    /// For 1964, the rules of the October manual, p. 34: no more than six
    /// significant digits; no trailing zeros after the point; below 0.1,
    /// `X.XXXXX E-Y` "unless the entire significant part of the number can be
    /// printed as a six decimal number"; and "if the number is an exact
    /// integer, the decimal point is not printed", integers of up to nine
    /// digits in full. So a point after a whole number (`16.`) says it is
    /// not exact, as in the table of powers on p. 33. Numbers of 10^6 and
    /// more that are not exact integers, and integers of ten digits or more,
    /// are printed `X.XXXXX E+Y`: the manual shows no such number, and the
    /// form is ours.
    pub fn number(self, n: f64) -> String {
        match self {
            Dialect::Microsoft => format_number(n),
            Dialect::Dartmouth => dartmouth_number(n),
        }
    }

    /// `a ^ b`, or `None` where that is an error: in the Microsoft BASICs,
    /// a negative number to a power that is not whole.
    pub fn power(self, a: f64, b: f64) -> Option<f64> {
        match self {
            Dialect::Dartmouth => Some(a.abs().powf(b)),
            Dialect::Microsoft => (a >= 0.0 || b.fract() == 0.0).then(|| a.powf(b)),
        }
    }
}

/// The October 1964 rules for printing a number (see [`Dialect::number`]).
fn dartmouth_number(n: f64) -> String {
    if !n.is_finite() {
        return format_digits(n, 6);
    }
    let e_form = |n: f64| {
        let sci = format!("{n:.5e}");
        let (mantissa, exponent) = sci.split_once('e').expect("exponent format");
        let exponent: i32 = exponent.parse().expect("exponent digits");
        let sign = if exponent < 0 { '-' } else { '+' };
        format!("{mantissa} E{sign}{}", exponent.abs())
    };
    if n.fract() == 0.0 {
        return if n.abs() < 1e9 {
            format!("{}", n as i64)
        } else {
            e_form(n)
        };
    }
    if n.abs() < 0.1 {
        // printed as a decimal only if six decimals hold all of it
        let micro = n * 1e6;
        if (micro - micro.round()).abs() <= micro.abs() * 1e-12 {
            return format_digits(n, 6);
        }
        return e_form(n);
    }
    let text = format_digits(n, 6);
    if text.contains('E') || n.abs() >= 999_999.5 {
        return e_form(n);
    }
    if text.contains('.') {
        text
    } else {
        // not exact, but whole to six figures: the point says so
        text + "."
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dartmouth_takes_absolute_values() {
        assert_eq!(Dialect::Microsoft.sqrt(-4.0), None);
        assert_eq!(Dialect::Dartmouth.sqrt(-4.0), Some(2.0));
        assert_eq!(Dialect::Microsoft.power(-2.0, 3.0), Some(-8.0));
        assert_eq!(Dialect::Dartmouth.power(-2.0, 3.0), Some(8.0));
        // GW-BASIC: "Illegal function call" for (-8)^(1/3)
        assert_eq!(Dialect::Microsoft.power(-8.0, 1.0 / 3.0), None);
        assert_eq!(Dialect::Dartmouth.power(-8.0, 1.0 / 3.0), Some(2.0));
        assert_eq!(Dialect::from_name("Dartmouth"), Some(Dialect::Dartmouth));
        assert_eq!(Dialect::from_name("cobol"), None);
    }

    #[test]
    fn dartmouth_prints_by_the_october_1964_rules() {
        let d = Dialect::Dartmouth;
        // May, pp. 21-23, and October rules 1 and 2
        assert_eq!(d.number(1.57970004), "1.5797");
        assert_eq!(d.number(2f64.sqrt()), "1.41421");
        assert_eq!(d.number(0.837585449), ".837585");
        assert_eq!(d.number(0.69317627), ".693176");
        // rule 4: an exact integer has no point, up to nine digits in full
        assert_eq!(d.number(3.0), "3");
        assert_eq!(d.number(-1.0), "-1");
        assert_eq!(d.number(0.0), "0");
        assert_eq!(d.number(1_000_000.0), "1000000");
        assert_eq!(d.number(823_543.0), "823543");
        // not exact but whole to six figures: the point says so (p. 33: 16.)
        assert_eq!(d.number(16.000000001), "16.");
        assert_eq!(d.number(279_935.999_999), "279936.");
        // rule 3: .03456 is exact to six decimals; otherwise X.XXXXX E-Y
        assert_eq!(d.number(0.03456), ".03456");
        assert_eq!(d.number(0.0345600001), "3.45600 E-2");
        // ours: where the manual shows nothing
        assert_eq!(d.number(1e20), "1.00000 E+20");
        assert_eq!(d.number(1234567.8), "1.23457 E+6");
        assert_eq!(Dialect::Microsoft.number(3.0), "3");
        assert_eq!(Dialect::Microsoft.number(2f64.sqrt()), "1.41421356");
    }

    #[test]
    fn dartmouth_log_takes_the_absolute_value() {
        assert_eq!(Dialect::Dartmouth.log(-1.0), Some(0.0));
        assert_eq!(Dialect::Dartmouth.log(0.0), None);
        assert_eq!(Dialect::Microsoft.log(-1.0), None);
    }

    #[test]
    fn dartmouth_int_rnd_and_minus() {
        assert_eq!(Dialect::Microsoft.int(-1.5), -2.0);
        assert_eq!(Dialect::Dartmouth.int(-1.5), -1.0);
        assert_eq!(Dialect::Dartmouth.int(2.7), 2.0);
        assert_eq!(Dialect::Microsoft.rnd_argument(0.0), 0.0);
        assert_eq!(Dialect::Dartmouth.rnd_argument(0.0), 1.0);
        assert_eq!(Dialect::Microsoft.leading_minus(4.0), -4.0);
        assert_eq!(Dialect::Dartmouth.leading_minus(4.0), 4.0);
    }
}
