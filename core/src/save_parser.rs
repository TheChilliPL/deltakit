use std::fmt::Display;
use compact_str::ToCompactString;
use crate::iter::ResultArrayExt;
use crate::savefile::{ItemStats, LightworldStats, Stats};
use thiserror::Error;
use crate::save_parser::ParseErrorKind::{EofExpected, EofUnexpected, IntParse};

#[derive(Debug, Error)]
pub struct ParseError {
    pub kind: ParseErrorKind,
    pub line: Option<usize>,
}

impl ParseError {
    pub fn new(kind: ParseErrorKind, line: Option<usize>) -> ParseError {
        ParseError { kind, line }
    }
}

impl From<ParseErrorKind> for ParseError {
    fn from(kind: ParseErrorKind) -> Self {
        ParseError::new(kind, None)
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.kind)?;
        if let Some(line) = self.line {
            write!(f, " on line {}", line)?;
        }
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum ParseErrorKind {
    #[error("integer parse error")]
    IntParse,
    #[error("float parse error")]
    FloatParse,
    #[error("file ended unexpectedly")]
    EofUnexpected,
    #[error("expected end of file")]
    EofExpected,
}

impl From<std::num::ParseIntError> for ParseErrorKind {
    fn from(_: std::num::ParseIntError) -> Self {
        ParseErrorKind::IntParse
    }
}

impl From<std::num::ParseFloatError> for ParseErrorKind {
    fn from(_: std::num::ParseFloatError) -> Self {
        ParseErrorKind::FloatParse
    }
}

pub struct SaveParser<'a> {
    chapter: i32,
    save_lines: &'a [&'a str],
    current_line: usize,
}

impl<'a> SaveParser<'a> {
    pub fn new(chapter: i32, save_lines: &'a [&'a str]) -> SaveParser<'a> {
        SaveParser {
            chapter,
            save_lines,
            current_line: 0,
        }
    }

    pub fn parse_string(&mut self) -> Result<&'a str, ParseError> {
        if self.current_line >= self.save_lines.len() {
            return Err(ParseError::new(EofUnexpected, Some(self.current_line)));
        }

        let line = self.save_lines[self.current_line];
        self.current_line += 1;
        Ok(line)
    }

    pub fn parse_int(&mut self) -> Result<i32, ParseError> {
        self.parse_string()?.trim().parse::<i32>()
                .map_err(|e| ParseError::new(e.into(), Some(self.current_line)))
    }

    pub fn parse_float(&mut self) -> Result<f32, ParseError> {
        self.parse_string()?.trim().parse::<f32>()
                .map_err(|e| ParseError::new(e.into(), Some(self.current_line)))
    }

    pub fn parse_bool(&mut self) -> Result<bool, ParseError> {
        match self.parse_int()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(ParseError::new(IntParse, Some(self.current_line))),
        }
    }

    pub fn parse_stats(&mut self) -> Result<Stats, ParseError> {
        Ok(Stats {
            hp: self.parse_int()?,
            max_hp: self.parse_int()?,
            attack: self.parse_int()?,
            defense: self.parse_int()?,
            magic: self.parse_int()?,
            guts: self.parse_int()?,

            weapon: self.parse_int()?,
            armor1: self.parse_int()?,
            armor2: self.parse_int()?,
            weapon_style: self.parse_string()?.to_compact_string(),

            item_stats: [(); 4]
                .map(|_| {
                    Ok::<ItemStats, ParseError>(ItemStats {
                        attack: self.parse_int()?,
                        defense: self.parse_int()?,
                        magic: self.parse_int()?,
                        bolts: self.parse_int()?,
                        graze_amount: self.parse_int()?,
                        graze_size: self.parse_int()?,
                        bolts_speed: self.parse_int()?,
                        item_special: self.parse_int()?,

                        item_element: if self.chapter >= 2 {
                            self.parse_int()?
                        } else {
                            0
                        },
                        item_element_amount: if self.chapter >= 2 {
                            self.parse_float()?
                        } else {
                            0.0
                        },
                    })
                })
                .flatten_ok()?,
            spells: [(); 12].map(|_| self.parse_int()).flatten_ok()?,
        })
    }

    pub fn parse_lightworld_stats(&mut self) -> Result<LightworldStats, ParseError> {
        Ok(LightworldStats {
            weapon: self.parse_int()?,
            armor: self.parse_int()?,
            xp: self.parse_int()?,
            lv: self.parse_int()?,
            gold: self.parse_int()?,
            hp: self.parse_int()?,
            max_hp: self.parse_int()?,
            attack: self.parse_int()?,
            defense: self.parse_int()?,
            wstrength: self.parse_int()?,
            adef: self.parse_int()?,
        })
    }

    pub fn expect_eof(&self) -> Result<(), ParseError> {
        if self.current_line < self.save_lines.len() {
            // Allow the last line to be empty
            // This shouldn't happen with unedited saves [?]
            if self.current_line + 1 == self.save_lines.len()
                && self.save_lines[self.current_line].is_empty()
            {
                return Ok(());
            }

            Err(ParseError::new(EofExpected, Some(self.current_line)))
        } else {
            Ok(())
        }
    }
}
