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
