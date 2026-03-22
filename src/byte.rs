use ratatui::style::Style;

use crate::config::Config;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Byte(u8);

pub enum ByteType {
    Null,
    AsciiPrintable,
    AsciiWhitespace,
    AsciiOther,
    NonAscii,
}

impl Byte {
    pub fn new(value: u8) -> Self {
        Byte(value)
    }

    pub fn value(&self) -> u8 {
        self.0
    }

    pub fn get_bytetype(self) -> ByteType {
        match self.0 {
            0 => ByteType::Null,
            c if c.is_ascii_graphic() => ByteType::AsciiPrintable,
            c if c.is_ascii_whitespace() => ByteType::AsciiWhitespace,
            c if c.is_ascii() => ByteType::AsciiOther,
            _ => ByteType::NonAscii,
        }
    }

    pub fn get_hex(self) -> String {
        format!("{:02X}", self.0)
    }

    pub fn get_char(self, config: &Config) -> char {
        config.charset.get_char(&self)
    }

    pub fn get_style(&self, config: &Config) -> Style {
        config.colorscheme.get_style(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bytetype_null() {
        assert!(matches!(Byte::new(0).get_bytetype(), ByteType::Null));
    }

    #[test]
    fn bytetype_ascii_printable() {
        assert!(matches!(
            Byte::new(b'A').get_bytetype(),
            ByteType::AsciiPrintable
        ));
    }

    #[test]
    fn bytetype_ascii_whitespace() {
        assert!(matches!(
            Byte::new(b'\t').get_bytetype(),
            ByteType::AsciiWhitespace
        ));
        assert!(matches!(
            Byte::new(b' ').get_bytetype(),
            ByteType::AsciiWhitespace
        ));
    }

    #[test]
    fn bytetype_ascii_other() {
        // SOH (0x01) is ASCII control but not graphic, not whitespace, not null
        assert!(matches!(
            Byte::new(1).get_bytetype(),
            ByteType::AsciiOther
        ));
    }

    #[test]
    fn bytetype_non_ascii() {
        assert!(matches!(
            Byte::new(128).get_bytetype(),
            ByteType::NonAscii
        ));
        assert!(matches!(
            Byte::new(255).get_bytetype(),
            ByteType::NonAscii
        ));
    }

    #[test]
    fn get_hex_zero() {
        assert_eq!(Byte::new(0).get_hex(), "00");
    }

    #[test]
    fn get_hex_single_digit() {
        assert_eq!(Byte::new(10).get_hex(), "0A");
    }

    #[test]
    fn get_hex_max() {
        assert_eq!(Byte::new(255).get_hex(), "FF");
    }

    #[test]
    fn value_roundtrip() {
        assert_eq!(Byte::new(42).value(), 42);
        assert_eq!(Byte::new(0).value(), 0);
        assert_eq!(Byte::new(255).value(), 255);
    }
}
