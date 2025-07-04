use std::time::Duration;
use compact_str::{format_compact, CompactString};
use indoc::indoc;
use crate::gamedata::items::display_item;
use crate::gamedata::key_items::display_key_item;
use crate::gamedata::rooms::display_room;

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
    pub fn read<'a>(chapter: u32, save_lines: &'a [&'a str]) -> SaveMetadata<'a> {
        let len = save_lines.len();
        let (items_offset, item_mult) = match chapter {
            1 => (Some(236 - 1), 4),
            2 => (Some(330 - 1), 2),
            _ => (None, 0),
        };
        let (items, key_items) = if let Some(offset) = items_offset {
            let mut item_ids = [0; 13];
            let mut key_item_ids = [0; 13];
            for i in 0..13 {
                item_ids[i] = save_lines[offset + item_mult * i].trim().parse().unwrap();
                key_item_ids[i] = save_lines[offset + item_mult * i + 1].trim().parse().unwrap();
            }
            (Some(item_ids), Some(key_item_ids))
        } else { (None, None) };
        SaveMetadata {
            chapter,
            player_name: save_lines[0],
            vessel_name: save_lines[1],
            dark_dollars: save_lines[10].trim().parse().unwrap(),
            level: save_lines[12].trim().parse().unwrap(),
            is_darkworld: save_lines[16].trim() == "1",
            story_flag: save_lines[len - 3].trim().parse().unwrap(),
            room_id: save_lines[len - 2].trim().parse().unwrap(),
            time_played: Duration::from_secs_f64(save_lines[len - 1].trim().parse::<f64>().unwrap() / 30.0),
            item_ids: items,
            key_item_ids: key_items,
        }
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

