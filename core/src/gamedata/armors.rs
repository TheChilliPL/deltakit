use compact_str::{CompactString, ToCompactString, format_compact};

// Extracted from gml_GlobalScript_scr_armorinfo using UndertaleModTool
pub fn try_get_armor_name(armor_id: i32) -> Option<&'static str> {
    match armor_id {
        0 => Some("---"),
        1 => Some("Amber Card"),
        2 => Some("Dice Brace"),
        3 => Some("Pink Ribbon"),
        4 => Some("White Ribbon"),
        5 => Some("IronShackle"),
        6 => Some("MouseToken"),
        7 => Some("Jevilstail"),
        8 => Some("Silver Card"),
        9 => Some("TwinRibbon"),
        10 => Some("GlowWrist"),
        11 => Some("ChainMail"),
        12 => Some("B.ShotBowtie"),
        13 => Some("SpikeBand"),
        14 => Some("Silver Watch"),
        15 => Some("TensionBow"),
        16 => Some("Mannequin"),
        17 => Some("DarkGoldBand"),
        18 => Some("SkyMantle"),
        19 => Some("SpikeShackle"),
        20 => Some("FrayedBowtie"),
        21 => Some("Dealmaker"),
        22 => Some("RoyalPin"),
        23 => Some("ShadowMantle"),
        24 => Some("LodeStone"),
        25 => Some("GingerGuard"),
        26 => Some("BlueRibbon"),
        27 => Some("TennaTie"),
        50 => Some("Waferguard"),
        51 => Some("MysticBand"),
        52 => Some("PowerBand"),
        53 => Some("PrincessRBN"),
        54 => Some("GoldWidow"),
        _ => None,
    }
}

pub fn display_armor(armor_id: i32) -> CompactString {
    let armor_name = try_get_armor_name(armor_id);
    armor_name
        .map(|n| n.to_compact_string())
        .unwrap_or_else(|| format_compact!("Armor {}", armor_id))
}
