use std::str::FromStr;

use hex_color::{HexColor, ParseHexColorError};

#[derive(Debug, PartialEq)]
pub struct Color {
    pub red: f64,
    pub green: f64,
    pub blue: f64,
}

impl FromStr for Color {
    type Err = ParseHexColorError;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let s = s.trim_start_matches('#');
        let parsed = HexColor::parse(format!("#{s}").as_str())?;
        Ok(Color {
            red: f64::from(parsed.r) / 255.0,
            green: f64::from(parsed.g) / 255.0,
            blue: f64::from(parsed.b) / 255.0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_no_hash() {
        let result = Color::from_str("1e1e2e");
        let expected = Color {
            red: 30.0 / 255.0,
            green: 30.0 / 255.0,
            blue: 46.0 / 255.0,
        };
        assert_eq!(result, Ok(expected));
    }

    #[test]
    fn parse_with_hash() {
        let result = Color::from_str("#179299");
        let expected = Color {
            red: 23.0 / 255.0,
            green: 146.0 / 255.0,
            blue: 153.0 / 255.0,
        };
        assert_eq!(result, Ok(expected));
    }
}
