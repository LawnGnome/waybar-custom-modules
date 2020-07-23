use std::convert::From;
use std::fmt;

#[derive(Default, Debug, Clone, Copy, Eq, Ord, PartialOrd, PartialEq)]
pub struct Percentage(u8);

impl Percentage {
    pub fn calculate<T>(numerator: T, denominator: T) -> Self
    where
        T: Into<f64>,
    {
        let n: f64 = numerator.into();
        let d: f64 = denominator.into();

        Self(((100. * n) / d) as u8)
    }

    pub fn as_u8(&self) -> u8 {
        self.0
    }
}

impl From<f64> for Percentage {
    fn from(v: f64) -> Self {
        Self((100. * v).round() as u8)
    }
}

impl fmt::Display for Percentage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}%", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate() {
        assert_eq!(100, Percentage::calculate(100u32, 100u32).as_u8());
        assert_eq!(100, Percentage::calculate(1f64, 1f64).as_u8());
        assert_eq!(0, Percentage::calculate(0u16, 40000u16).as_u8());
        assert_eq!(10, Percentage::calculate(4000u16, 40000u16).as_u8());
    }
}
