use crate::gamedata::items::display_item;
use crate::gamedata::key_items::display_key_item;
use crate::gamedata::rooms::display_room;
use crate::iter::{ResultArrayExt, ResultVecExt};
use crate::save_parser::{ParseError, SaveParser};
use compact_str::CompactString;
use indoc::indoc;
use log::warn;
use std::time::Duration;

#[derive(Debug)]
pub struct SaveData<'a> {
    pub chapter: i32,
    pub true_name: &'a str,
    pub vessel_names: [&'a str; 6],
    pub party: [i32; 3],
    pub dark_dollars: i32,
    pub xp: i32,
    pub level: i32,
    pub inv: i32,
    pub invc: i32,
    pub is_darkworld: bool,
    pub stats: Vec<Stats>,
    pub bolt_speed: i32,
    pub graze_amount: i32,
    pub graze_size: i32,
    pub inventory: [i32; 13],
    pub key_items: [i32; 13],
    pub weapons: Vec<i32>,
    pub armors: Vec<i32>,
    pub storage: Option<Vec<i32>>,
    pub tension: f32,
    pub max_tension: f32,
    pub lightworld_stats: LightworldStats,
    pub lightworld_items: [i32; 8],
    pub lightworld_phone: [i32; 8],
    pub flags: [f32; 2500],
    pub plot_value: i32,
    pub room_id: i32,
    pub time_played: Duration,
}

impl SaveData<'_> {
    /// Parses the Deltarune save data.
    ///
    /// Made based on `gml_GlobalScript_scr_saveprocess` (see `research/saveprocess` directory for
    /// decompiled code).
    pub fn read<'a>(chapter: i32, save_lines: &'a [&'a str]) -> Result<SaveData<'a>, ParseError> {
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

        let true_name = parser.parse_string()?;

        let vessel_names = [(); 6].map(|_| parser.parse_string()).flatten_ok()?;

        let party = [(); 3].map(|_| parser.parse_int()).flatten_ok()?;

        let dark_dollars = parser.parse_int()?;

        let xp = parser.parse_int()?;

        let level = parser.parse_int()?;

        // Something with invincibility frames
        let inv = parser.parse_int()?;
        let invc = parser.parse_int()?;

        let is_darkworld = parser.parse_bool()?;

        let stat_blocks = match chapter {
            1 => 4,
            _ => 5,
        };

        let stats = (0..stat_blocks)
            .map(|_i| parser.parse_stats())
            .collect::<Vec<_>>()
            .flatten_ok()?;

        let bolt_speed = parser.parse_int()?; // ?
        let graze_amount = parser.parse_int()?;
        let graze_size = parser.parse_int()?;

        let mut inventory = [0; 13];
        let mut key_items = [0; 13];

        let mut weapons = Vec::with_capacity(if is_chapter_1 { 13 } else { 48 });
        let mut armors = weapons.clone();

        let mut storage = if is_chapter_1 {
            None
        } else {
            Some(Vec::with_capacity(72))
        };

        for i in 0..13 {
            inventory[i] = parser.parse_int()?;
            key_items[i] = parser.parse_int()?;

            if is_chapter_1 {
                weapons.push(parser.parse_int()?);
                armors.push(parser.parse_int()?);
            }
        }

        if !is_chapter_1 {
            for _ in 0..48 {
                weapons.push(parser.parse_int()?);
                armors.push(parser.parse_int()?);
            }

            for _ in 0..72 {
                storage.as_mut().unwrap().push(parser.parse_int()?);
            }
        }

        let tension = parser.parse_float()?;
        let max_tension = parser.parse_float()?;

        let lightworld_stats = parser.parse_lightworld_stats()?;

        let mut lightworld_items = [0; 8];
        let mut lightworld_phone = [0; 8];

        for i in 0..8 {
            lightworld_items[i] = parser.parse_int()?;
            lightworld_phone[i] = parser.parse_int()?;
        }

        let flags = [(); 2500].map(|_| parser.parse_float()).flatten_ok()?;

        if is_chapter_1 {
            // Chapter 1 stores 9999 flags, we need to skip the rest of them
            // They should all be zero
            for _ in 2500..9999 {
                _ = parser.parse_int()?;
            }
        }

        let plot_value = parser.parse_int()?;
        let room_id = parser.parse_int()?;
        let time_played_frames = parser.parse_float()?;
        let time_played = Duration::from_secs_f64(time_played_frames as f64 / 30.0);

        parser.expect_eof()?;

        Ok(SaveData {
            chapter,
            true_name,
            vessel_names,
            party,
            dark_dollars,
            xp,
            level,
            inv,
            invc,
            is_darkworld,
            stats,
            bolt_speed,
            graze_amount,
            graze_size,
            inventory,
            key_items,
            weapons,
            armors,
            storage,
            tension,
            max_tension,
            lightworld_stats,
            lightworld_items,
            lightworld_phone,
            flags,
            plot_value,
            room_id,
            time_played,
        })
    }

    pub fn display_room(&self) -> CompactString {
        display_room(self.room_id)
    }

    pub fn display_inventory(
        inventory: Option<&[i32]>,
        name: &str,
        display: impl Fn(i32) -> CompactString,
    ) -> String {
        if inventory.is_none() {
            return "".to_string();
        }
        let items = inventory
            .unwrap()
            .iter()
            .map(|i| format!("{: <13}", display(*i)))
            .collect::<Vec<_>>();
        let chunks = items.chunks(4);
        let inventory = chunks
            .map(|chunk| chunk.join(" "))
            .collect::<Vec<_>>()
            .join("\n");
        format!("---------------\n{}:\n{}\n", name, inventory)
    }

    pub fn display_info(&self) -> String {
        let time_played_secs = self.time_played.as_secs();
        let time_played_h = time_played_secs / 3600;
        let time_played_m = (time_played_secs % 3600) / 60;
        let time_played_s = time_played_secs % 60;
        let time_played = format!(
            "{}h{:02}m{:02}s",
            time_played_h, time_played_m, time_played_s
        );

        format!(
            indoc! {"
                Save for chapter {}
                {} | {}
                D${} LV{}
                Plot value {}
                {}{}
                Played for {}
                {}{}
            "},
            self.chapter,
            self.true_name,
            self.vessel_names[0],
            self.dark_dollars,
            self.level,
            self.plot_value,
            self.display_room(),
            if self.is_darkworld {
                " (Dark World)"
            } else {
                " (Light World)"
            },
            time_played,
            Self::display_inventory(Some(&self.inventory[..]), "Items", display_item),
            Self::display_inventory(Some(&self.key_items[..]), "Keys", display_key_item),
        )
    }
}

#[derive(Debug)]
pub struct ItemStats {
    pub attack: i32,
    pub defense: i32,
    pub magic: i32,
    pub bolts: i32, // What?
    pub graze_amount: i32,
    pub graze_size: i32,
    pub bolts_speed: i32,  // ?
    pub item_special: i32, // int?

    // Chapter 2 and up
    // For chapter 1, both set to 0
    pub item_element: i32,
    pub item_element_amount: f32,
}

#[derive(Debug)]
pub struct Stats {
    pub hp: i32,
    pub max_hp: i32,
    pub attack: i32,
    pub defense: i32,
    pub magic: i32,
    pub guts: i32,

    pub weapon: i32,
    pub armor1: i32,
    pub armor2: i32,
    pub weapon_style: CompactString,

    pub item_stats: [ItemStats; 4],
    pub spells: [i32; 12],
}

#[derive(Debug)]
pub struct LightworldStats {
    pub weapon: i32,
    pub armor: i32,
    pub xp: i32,
    pub lv: i32,
    pub gold: i32,
    pub hp: i32,
    pub max_hp: i32,
    pub attack: i32,
    pub defense: i32,
    pub wstrength: i32, // ?
    pub adef: i32,      // ?
}
