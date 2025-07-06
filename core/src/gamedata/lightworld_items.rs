use compact_str::{CompactString, ToCompactString, format_compact};

// Extracted from gml_GlobalScript_scr_litemname using UndertaleModTool
pub fn try_get_lightworld_item_name(item_id: i32) -> Option<&'static str> {
    match item_id {
        0 => Some("---"),
        1 => Some("Hot Chocolate"),
        2 => Some("Pencil"),
        3 => Some("Bandage"),
        4 => Some("Bouquet"),
        5 => Some("Ball of Junk"),
        6 => Some("Halloween Pencil"),
        7 => Some("Lucky Pencil"),
        8 => Some("Egg"),
        9 => Some("Cards"),
        10 => Some("Box of Heart Candy"),
        11 => Some("Glass"),
        12 => Some("Eraser"),
        13 => Some("Mech. Pencil"),
        14 => Some("Wristwatch"),
        15 => Some("Holiday Pencil"),
        16 => Some("CactusNeedle"),
        17 => Some("BlackShard"),
        18 => Some("QuillPen"),
        _ => None,
    }
}

pub fn display_lightworld_item(item_id: i32) -> CompactString {
    let item_name = try_get_lightworld_item_name(item_id);
    item_name
        .map(|n| n.to_compact_string())
        .unwrap_or_else(|| format_compact!("LW Item {}", item_id))
}
