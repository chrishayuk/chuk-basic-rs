//! Where the BASICs disagree about arithmetic.

/// Which BASIC's arithmetic to follow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Dialect {
    /// The Microsoft BASICs of the 1970s and after: the square root of a
    /// negative number is an error.
    #[default]
    Microsoft,
    /// Dartmouth BASIC as its first manual (May 1964) describes it: `SQR`
    /// takes "the square-root ... of the absolute value", and `A ↑ B` "is
    /// actually interpreted as |A|^B".
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

    /// `a ^ b`
    pub fn power(self, a: f64, b: f64) -> f64 {
        match self {
            Dialect::Dartmouth => a.abs().powf(b),
            Dialect::Microsoft => a.powf(b),
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
        assert_eq!(Dialect::Microsoft.power(-2.0, 3.0), -8.0);
        assert_eq!(Dialect::Dartmouth.power(-2.0, 3.0), 8.0);
        assert_eq!(Dialect::from_name("Dartmouth"), Some(Dialect::Dartmouth));
        assert_eq!(Dialect::from_name("cobol"), None);
    }
}
