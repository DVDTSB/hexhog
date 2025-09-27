use std::{fs::read_to_string, str::FromStr};

use crate::byte::{Byte, ByteType};
use ratatui::style::{Color, Style};
use toml::Table;

pub struct ColorScheme {
    pub null: Color,
    pub ascii_printable: Color,
    pub ascii_whitespace: Color,
    pub ascii_other: Color,
    pub non_ascii: Color,
    pub accent: Color,
}

impl ColorScheme {
    pub fn get_style(&self, byte: &Byte) -> Style {
        match byte.get_bytetype() {
            ByteType::Null => Style::default().fg(self.null),
            ByteType::AsciiPrintable => Style::default().fg(self.ascii_printable),
            ByteType::AsciiWhitespace => Style::default().fg(self.ascii_whitespace),
            ByteType::AsciiOther => Style::default().fg(self.ascii_other),
            ByteType::NonAscii => Style::default().fg(self.non_ascii),
        }
    }
}

pub struct BasicCharset {
    null: char,
    ascii_whitespace: char,
    ascii_other: char,
    non_ascii: char,
}

pub enum Charset {
    Basic(BasicCharset),
    Custom([char; 256]),
}

impl Charset {
    pub fn get_char(&self, byte: &Byte) -> char {
        match self {
            Charset::Basic(basic) => match byte.get_bytetype() {
                ByteType::Null => basic.null,
                ByteType::AsciiPrintable => byte.value() as char,
                ByteType::AsciiWhitespace if byte.value() as char == ' ' => ' ',
                ByteType::AsciiWhitespace => basic.ascii_whitespace,
                ByteType::AsciiOther => basic.ascii_other,
                ByteType::NonAscii => basic.non_ascii,
            },
            Charset::Custom(array) => array[byte.value() as usize],
        }
    }
}

pub struct Config {
    pub colorscheme: ColorScheme,
    pub charset: Charset,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            colorscheme: ColorScheme {
                null: Color::DarkGray,
                ascii_printable: Color::Blue,
                ascii_whitespace: Color::Green,
                ascii_other: Color::Green,
                non_ascii: Color::Yellow,
                accent: Color::Blue,
            },
            charset: Charset::Basic(BasicCharset {
                null: '.',
                ascii_whitespace: '·',
                ascii_other: '°',
                non_ascii: '×',
            }),
        }
    }
}

impl Config {
    pub fn toml_value_to_color(value: &toml::Value) -> Option<Color> {
        if let Some(s) = value.as_str() {
            Color::from_str(s).ok()
        } else if let Some(i) = value.as_integer() {
            if i >= 0 && i <= 255 {
                Some(Color::Indexed(i as u8))
            } else {
                None
            }
        } else if let Some((r, g, b)) = value.as_array().and_then(|arr| {
            if arr.len() == 3 {
                let r = arr[0].as_integer()?;
                let g = arr[1].as_integer()?;
                let b = arr[2].as_integer()?;
                if (0..=255).contains(&r) && (0..=255).contains(&g) && (0..=255).contains(&b) {
                    Some((r as u8, g as u8, b as u8))
                } else {
                    None
                }
            } else {
                None
            }
        }) {
            Some(Color::Rgb(r, g, b))
        } else {
            None
        }
    }

    pub fn read_config(path: &str) -> Self {
        let mut config = Config::default();

        let config_file = read_to_string(path);

        if config_file.is_err() {
            return config;
        }

        let values = config_file.unwrap().parse::<Table>().unwrap();

        if let Some(colors) = values.get("theme") {
            if let Some(null) = colors.get("null").and_then(Config::toml_value_to_color) {
                config.colorscheme.null = null;
            }
            if let Some(ascii_printable) = colors
                .get("ascii_printable")
                .and_then(Config::toml_value_to_color)
            {
                config.colorscheme.ascii_printable = ascii_printable;
            }
            if let Some(ascii_whitespace) = colors
                .get("ascii_whitespace")
                .and_then(Config::toml_value_to_color)
            {
                config.colorscheme.ascii_whitespace = ascii_whitespace;
            }
            if let Some(ascii_other) = colors
                .get("ascii_other")
                .and_then(Config::toml_value_to_color)
            {
                config.colorscheme.ascii_other = ascii_other;
            }
            if let Some(non_ascii) = colors
                .get("non_ascii")
                .and_then(Config::toml_value_to_color)
            {
                config.colorscheme.non_ascii = non_ascii;
            }
            if let Some(accent) = colors.get("accent").and_then(Config::toml_value_to_color) {
                config.colorscheme.accent = accent;
            }
        }

        config
    }
}
