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
    pub primary: Color,
    pub border: Color,
    pub select: Color,
    pub background: Color,
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

pub struct Charset {
    pub null: char,
    pub ascii_whitespace: char,
    pub ascii_other: char,
    pub non_ascii: char,
}

impl Charset {
    pub fn get_char(&self, byte: &Byte) -> char {
        match byte.get_bytetype() {
            ByteType::Null => self.null,
            ByteType::AsciiPrintable => byte.value() as char,
            ByteType::AsciiWhitespace if byte.value() as char == ' ' => ' ',
            ByteType::AsciiWhitespace => self.ascii_whitespace,
            ByteType::AsciiOther => self.ascii_other,
            ByteType::NonAscii => self.non_ascii,
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
                ascii_whitespace: Color::Cyan,
                ascii_other: Color::Yellow,
                non_ascii: Color::Green,
                accent: Color::Blue,
                select: Color::DarkGray,
                border: Color::White,
                primary: Color::White,
                background: Color::Reset,
            },
            charset: Charset {
                null: '.',
                ascii_whitespace: '·',
                ascii_other: '°',
                non_ascii: '×',
            },
        }
    }
}

impl Config {
    pub fn toml_value_to_color(value: &toml::Value) -> Result<Color, String> {
        if let Some(s) = value.as_str() {
            Color::from_str(s).map_err(|_| "Invalid color name".into())
        } else if let Some(i) = value.as_integer() {
            if (0..=255).contains(&i) {
                Ok(Color::Indexed(i as u8))
            } else {
                Err("Invalid color index".into())
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
            Ok(Color::Rgb(r, g, b))
        } else {
            Err("Invalid color format".into())
        }
    }

    fn set_color_field(table: &Table, field: &str, current: &mut Color) -> Result<(), String> {
        if let Some(value) = table.get(field) {
            let color = Config::toml_value_to_color(value);
            if color.is_err() {
                return Err(format!(
                    "Invalid color for field '{}' - {}",
                    field,
                    color.err().unwrap()
                ));
            } else {
                *current = color.unwrap()
            }
        }
        Ok(())
    }

    fn set_charset_field(table: &Table, field: &str, current: &mut char) -> Result<(), String> {
        if let Some(value) = table.get(field) {
            if let Some(s) = value.as_str() {
                let mut chars = s.chars();
                if let Some(c) = chars.next() {
                    if chars.next().is_none() {
                        *current = c;
                    } else {
                        return Err(format!("Field '{field}' must be a single character"));
                    }
                } else {
                    return Err(format!("Field '{field}' cannot be empty"));
                }
            } else {
                return Err(format!("Field '{field}' must be a string"));
            }
        }
        Ok(())
    }

    pub fn read_config(path: &str) -> Result<Self, String> {
        let mut config = Config::default();

        let config_file = read_to_string(path);

        if config_file.is_err() {
            return Ok(config);
        }

        let values = config_file.unwrap().parse::<Table>().unwrap();

        if let Some(colors) = values.get("theme") {
            if let Some(table) = colors.as_table() {
                Config::set_color_field(table, "null", &mut config.colorscheme.null)?;
                Config::set_color_field(
                    table,
                    "ascii_printable",
                    &mut config.colorscheme.ascii_printable,
                )?;
                Config::set_color_field(
                    table,
                    "ascii_whitespace",
                    &mut config.colorscheme.ascii_whitespace,
                )?;
                Config::set_color_field(table, "ascii_other", &mut config.colorscheme.ascii_other)?;
                Config::set_color_field(table, "non_ascii", &mut config.colorscheme.non_ascii)?;
                Config::set_color_field(table, "accent", &mut config.colorscheme.accent)?;
                Config::set_color_field(table, "select", &mut config.colorscheme.select)?;
                Config::set_color_field(table, "primary", &mut config.colorscheme.primary)?;
                Config::set_color_field(table, "border", &mut config.colorscheme.border)?;
                Config::set_color_field(table, "background", &mut config.colorscheme.background)?;
            }
        }

        if let Some(charset) = values.get("charset") {
            if let Some(table) = charset.as_table() {
                Config::set_charset_field(table, "null", &mut config.charset.null)?;
                Config::set_charset_field(
                    table,
                    "ascii_whitespace",
                    &mut config.charset.ascii_whitespace,
                )?;
                Config::set_charset_field(table, "ascii_other", &mut config.charset.ascii_other)?;
                Config::set_charset_field(table, "non_ascii", &mut config.charset.non_ascii)?;
                Config::set_charset_field(table, "non_ascii", &mut config.charset.non_ascii)?;
            }
        }

        Ok(config)
    }
}
