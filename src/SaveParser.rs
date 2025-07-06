use thiserror::Error;
use crate::iter::ResultArrayExt;
use crate::savefile::{ItemStats, LightworldStats, Stats};

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("integer parse error")]
    IntParseError,
    #[error("float parse error")]
    FloatParseError,
    #[error("file ended unexpectedly")]
    EofUnexpected,
    #[error("expected end of file")]
    EofExpected,
}

impl From<std::num::ParseIntError> for ParseError {
    fn from(_: std::num::ParseIntError) -> Self {
        ParseError::IntParseError
    }
}

impl From<std::num::ParseFloatError> for ParseError {
    fn from(_: std::num::ParseFloatError) -> Self {
        ParseError::FloatParseError
    }
}

pub struct SaveParser<'a> {
    chapter: u32,
    save_lines: &'a [&'a str],
    current_line: usize,
}

impl<'a> SaveParser<'a> {
    pub fn new(
        chapter: u32,
        save_lines: &'a [&'a str],
    ) -> SaveParser<'a> {
        SaveParser {
            chapter,
            save_lines,
            current_line: 0,
        }
    }

    pub fn parse_string(&mut self) -> Result<&'a str, ParseError> {
        if self.current_line >= self.save_lines.len() {
            return Err(ParseError::EofUnexpected);
        }

        let line = self.save_lines[self.current_line];
        self.current_line += 1;
        Ok(line)
    }

    pub fn parse_int(&mut self) -> Result<i32, ParseError> {
        Ok(self.parse_string()?.parse::<i32>()?)
    }

    pub fn parse_uint(&mut self) -> Result<u32, ParseError> {
        Ok(self.parse_string()?.parse::<u32>()?)
    }

    pub fn parse_float(&mut self) -> Result<f32, ParseError> {
        Ok(self.parse_string()?.parse::<f32>()?)
    }
    
    pub fn parse_bool(&mut self) -> Result<bool, ParseError> {
        match self.parse_int()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(ParseError::IntParseError),
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
            
            weapon: self.parse_uint()?,
            armor1: self.parse_uint()?,
            armor2: self.parse_uint()?,
            weapon_style: self.parse_uint()?,
            
            item_stats: [(); 4].map(|_| Ok::<ItemStats, ParseError>(ItemStats {
                attack: self.parse_int()?,
                defense: self.parse_int()?,
                magic: self.parse_int()?,
                bolts: self.parse_int()?,
                graze_amount: self.parse_int()?,
                graze_size: self.parse_int()?,
                bolts_speed: self.parse_int()?,
                item_special: self.parse_int()?,
                
                item_element: if self.chapter >= 2 { self.parse_uint()? } else { 0 },
                item_element_amount: if self.chapter >= 2 { self.parse_int()? } else { 0 },
            })).flatten_ok()?,
            spells: [(); 12].map(|_| self.parse_uint()).flatten_ok()?,
        })
    }
    
    pub fn parse_lightworld_stats(&mut self) -> Result<LightworldStats, ParseError> {
        Ok(LightworldStats {
            weapon: self.parse_uint()?,
            armor: self.parse_uint()?,
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
            if self.current_line + 1 == self.save_lines.len() && self.save_lines[self.current_line].is_empty() {
                return Ok(());
            }

            Err(ParseError::EofExpected)
        } else {
            Ok(())
        }
    }
}
