//! Where the BASICs disagree about arithmetic and about printing numbers.

use super::format::{format_digits, format_number};

/// Which BASIC's arithmetic to follow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Dialect {
    /// The Microsoft BASICs of the 1970s and after: the square root of a
    /// negative number is an error.
    #[default]
    Microsoft,
    /// Dartmouth BASIC as its first manual (May 1964) describes it: `SQR`
    /// takes "the square-root ... of the absolute value", and `A ↑ B` "is
    /// actually interpreted as |A|^B" (p. 5). When `READ` finds no data left,
    /// "the computer stops" (p. 15) - quietly, as the results on p. 22 show.
    /// Numbers are printed as in the results on pp. 21-23: a number that is
    /// not whole to six significant figures, and a point after a whole number
    /// (`3.`).
    ///
    /// The manual leaves `INT` and `RND` to its Part II, which the reprint
    /// does not include, and says nothing of how a leading minus binds. For
    /// those this follows Thomas Kurtz's account of the original BASIC
    /// (dartmouth.edu/basicfifty/commands.html): `INT` truncates toward 0,
    /// `RND`'s argument is ignored, and `-X↑2` is `(-X)↑2`.
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

    /// `INT(x)`: the whole number below `x` in later BASICs; in Kurtz's
    /// account of 1964 BASIC, "the integer part (truncating toward 0)".
    pub fn int(self, x: f64) -> f64 {
        match self {
            Dialect::Microsoft => x.floor(),
            Dialect::Dartmouth => x.trunc(),
        }
    }

    /// The argument `RND(x)` is taken to have. Kurtz: in 1964 "the argument
    /// is ignored" - every call gives the next number, `RND(0)` included.
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

    /// A number as `PRINT` shows it, without the sign column.
    pub fn number(self, n: f64) -> String {
        match self {
            Dialect::Microsoft => format_number(n),
            Dialect::Dartmouth => {
                let text = format_digits(n, 6);
                let plain = text.bytes().all(|b| b.is_ascii_digit() || b == b'-');
                if plain {
                    text + "."
                } else {
                    text
                }
            }
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
    fn dartmouth_prints_as_the_1964_results_do() {
        // p. 21: 1.5797; p. 22: 3. -1. 0. and 1.41421; p. 23: .837585 and .693176
        let d = Dialect::Dartmouth;
        assert_eq!(d.number(1.57970004), "1.5797");
        assert_eq!(d.number(3.0), "3.");
        assert_eq!(d.number(-1.0), "-1.");
        assert_eq!(d.number(0.0), "0.");
        assert_eq!(d.number(2f64.sqrt()), "1.41421");
        assert_eq!(d.number(0.837585449), ".837585");
        assert_eq!(d.number(0.69317627), ".693176");
        assert_eq!(d.number(1e20), "1E+20");
        assert_eq!(Dialect::Microsoft.number(3.0), "3");
        assert_eq!(Dialect::Microsoft.number(2f64.sqrt()), "1.41421356");
        // our choice where the pages show nothing: a whole number in full
        assert_eq!(d.number(1234567.0), "1234567.");
    }

    #[test]
    fn dartmouth_int_rnd_and_minus_follow_kurtz() {
        assert_eq!(Dialect::Microsoft.int(-1.5), -2.0);
        assert_eq!(Dialect::Dartmouth.int(-1.5), -1.0);
        assert_eq!(Dialect::Dartmouth.int(2.7), 2.0);
        assert_eq!(Dialect::Microsoft.rnd_argument(0.0), 0.0);
        assert_eq!(Dialect::Dartmouth.rnd_argument(0.0), 1.0);
        assert_eq!(Dialect::Microsoft.leading_minus(4.0), -4.0);
        assert_eq!(Dialect::Dartmouth.leading_minus(4.0), 4.0);
    }
}
