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
    /// actually interpreted as |A|^B". Numbers are printed as in the results
    /// on its pp. 21-23: six significant figures, and a point after a whole
    /// number (`3.`).
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
    }
}
