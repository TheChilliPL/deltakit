use compact_str::{format_compact, CompactString, ToCompactString};

// Extracted from gml_GlobalScript_scr_keyiteminfo using UndertaleModTool
pub fn try_get_key_item_name(key_item_id: u32) -> Option<&'static str> {
    match key_item_id {
        0 => Some("---"),
        1 => Some("Cell Phone"),
        2 => Some("Egg"),
        3 => Some("BrokenCake"),
        4 => Some("Broken Key A"),
        5 => Some("Door Key"),
        6 => Some("Broken Key B"),
        7 => Some("Broken Key C"),
        8 => Some("Lancer"),
        9 => Some("Rouxls Kaard"),
        10 => Some("EmptyDisk"),
        11 => Some("LoadedDisk"),
        12 => Some("KeyGen"),
        // Amount counted using scr_get_total_shadow_crystal_amount(),
        // counting flags set to one: 1646, 1647, 1648, 1649, up to
        // the current chapter.
        13 => Some("ShadowCrystal"),
        14 => Some("Starwalker"),
        15 => Some("PureCrystal"),
        16 => Some("OddController"),
        17 => Some("BackstagePass"),
        18 => Some("TripTicket"),
        19 => Some("LancerCon"), // Amount in global flag 1099
        30 => Some("SheetMusic"),
        31 => Some("ClaimbClaws"),
        _ => None
    }
}

pub fn display_key_item(key_item_id: u32) -> CompactString {
    let key_item_name = try_get_key_item_name(key_item_id);
    key_item_name.map(|n| n.to_compact_string()).unwrap_or_else(|| format_compact!("Key Item {}", 
        key_item_id))
}
