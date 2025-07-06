use std::time::Duration;
use compact_str::{format_compact, CompactString};
use indoc::indoc;
use log::warn;
use crate::gamedata::items::display_item;
use crate::gamedata::key_items::display_key_item;
use crate::gamedata::rooms::display_room;
use crate::iter::{ResultArrayExt, ResultVecExt};
use crate::SaveParser::{ParseError, SaveParser};

pub struct SaveMetadata<'a> {
    chapter: u32,
    player_name: &'a str,
    vessel_name: &'a str,
    dark_dollars: u32,
    level: u32,
    is_darkworld: bool,
    story_flag: u32,
    pub room_id: u32,
    pub time_played: Duration,
    item_ids: Option<[u32; 13]>,
    key_item_ids: Option<[u32; 13]>,
}

impl SaveMetadata<'_> {
    pub fn read<'a>(chapter: u32, save_lines: &'a [&'a str]) -> Result<SaveMetadata<'a>,
        ParseError> {
        let len = save_lines.len();

        if chapter < 1 {
            panic!("Invalid chapter number");
        }

        if chapter > 4 {
            warn!(
                "Chapter {} is not supported. Will assume it's the same as chapters 2–4,\
                    but might break.",
                chapter
            );
        }
        
        let is_chapter_1 = chapter == 1;

        let mut parser = SaveParser::new(chapter, save_lines);

        let truename = parser.parse_string();

        let vesselname = parser.parse_string()?;

        for _ in 1..6 {
            _ = parser.parse_string()?; // Other 5 vessel names?
        }

        let party = [(); 3].map(|_| parser.parse_int()).flatten_ok()?;

        let dark_dollars = parser.parse_int()?;

        let xp = parser.parse_int()?;

        let level = parser.parse_int()?;

        // Something with invincibility frames
        let _inv = parser.parse_int()?;
        let _invc = parser.parse_int()?;

        let darkzone = parser.parse_bool()?;

        let stat_blocks = match chapter {
            1 => 4,
            _ => 5,
        };

        let stats = (0..stat_blocks).map(|i| parser.parse_stats())
            .collect::<Vec<_>>()
            .flatten_ok()?;

        let bolt_speed = parser.parse_int()?; // ?
        let graze_amount = parser.parse_int()?;
        let graze_size = parser.parse_int()?;
        
        let mut inventory = [0; 13];
        let mut key_items = [0; 13];
        
        let mut weapons = Vec::with_capacity(if is_chapter_1 { 13 } else { 48 });
        let mut armors = weapons.clone();
        
        let mut pocket = if is_chapter_1 { None } else { Some(Vec::with_capacity(72)) };
        
        for i in 0..13 {
            inventory[i] = parser.parse_uint()?;
            key_items[i] = parser.parse_uint()?;
            
            if is_chapter_1 {
                weapons.push(parser.parse_uint()?);
                armors.push(parser.parse_uint()?);
            }
        }
        
        if !is_chapter_1 {
            for _ in 0..48 {
                weapons.push(parser.parse_uint()?);
                armors.push(parser.parse_uint()?);
            }
            
            for _ in 0..72 {
                pocket.as_mut().unwrap().push(parser.parse_uint()?);
            }
        }
        
        let tension = parser.parse_int()?;
        let max_tension = parser.parse_int()?;
        
        let lightworld_stats = parser.parse_lightworld_stats()?;
        
        let mut lightworld_items = [0; 8];
        let mut lightworld_phone = [0; 8];
        
        for i in 0..8 {
            lightworld_items[i] = parser.parse_uint()?;
            lightworld_phone[i] = parser.parse_uint()?;
        }
        
        let flags = [(); 2500].map(|_| parser.parse_int()).flatten_ok()?; // Int?
        
        if is_chapter_1 {
            // Chapter 1 stores 9999 flags, we need to skip the rest of them
            // They should all be zero
            for _ in 2500..9999 {
                _ = parser.parse_int()?;
            }
        }
        
        let plot_value = parser.parse_int()?;
        let current_room = parser.parse_uint()?;
        let played_time = parser.parse_uint()?;
        
        parser.expect_eof()?;
        
        todo!();
    }

    pub fn display_room(&self) -> CompactString {
        display_room(self.room_id)
    }

    pub fn display_inventory(
        inventory: Option<&[u32]>,
        name: &str,
        display: impl Fn(u32) -> CompactString
    ) -> String {
        if inventory.is_none() {
            return "".to_string();
        }
        let items = inventory.unwrap().iter().map(|i| format!("{: <13}", display(*i)))
            .collect::<Vec<_>>();
        let chunks = items.chunks(4);
        let inventory = chunks.map(|chunk| chunk.join(" ")).collect::<Vec<_>>().join("\n");
        format!("---------------\n{}:\n{}", name, inventory)
    }

    pub fn display_info(&self) -> String {
        let time_played_secs = self.time_played.as_secs();
        let time_played_h = time_played_secs / 3600;
        let time_played_m = (time_played_secs % 3600) / 60;
        let time_played_s = time_played_secs % 60;
        let time_played = format!("{}h{:02}m{:02}s", time_played_h, time_played_m, time_played_s);

        format!(
            indoc!{"
                Save for chapter {}
                {} | {}
                D${} LV{}
                Plot value {}
                {}{}
                Played for {}
                {}{}
            "},
            self.chapter,
            self.player_name, self.vessel_name,
            self.dark_dollars, self.level,
            self.story_flag,
            self.display_room(), if self.is_darkworld { " (Dark World)" } else { " (Light World)" },
            time_played,
            Self::display_inventory(self.item_ids.as_ref().map(|a| &a[..]), "Items", display_item),
            Self::display_inventory(self.key_item_ids.as_ref().map(|a| &a[..]), "Keys", 
                                    display_key_item),
        )
    }
}

pub struct ItemStats {
    pub attack: i32,
    pub defense: i32,
    pub magic: i32,
    pub bolts: i32, // What?
    pub graze_amount: i32,
    pub graze_size: i32,
    pub bolts_speed: i32, // ?
    pub item_special: i32, // int?

    // Chapter 2 and up
    // For chapter 1, both set to 0
    pub item_element: u32,
    pub item_element_amount: i32,
}

pub struct Stats {
    pub hp: i32,
    pub max_hp: i32,
    pub attack: i32,
    pub defense: i32,
    pub magic: i32,
    pub guts: i32,

    pub weapon: u32,
    pub armor1: u32,
    pub armor2: u32,
    pub weapon_style: u32,
    
    pub item_stats: [ItemStats; 4],
    pub spells: [u32; 12],
}

pub struct LightworldStats {
    pub weapon: u32,
    pub armor: u32,
    pub xp: i32,
    pub lv: i32,
    pub gold: i32,
    pub hp: i32,
    pub max_hp: i32,
    pub attack: i32,
    pub defense: i32,
    pub wstrength: i32, // ?
    pub adef: i32, // ?
}