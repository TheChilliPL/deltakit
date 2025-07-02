use compact_str::{format_compact, CompactString, ToCompactString};

// Extracted from gml_GlobalScript_scr_iteminfo using UndertaleModTool
pub fn try_get_item_name(item_id: u32) -> Option<&'static str> {
    match item_id {
        0 => Some("---"),
        1 => Some("Dark Candy"),
        2 => Some("ReviveMint"),
        3 => Some("Glowshard"),
        4 => Some("Manual"),
        5 => Some("BrokenCake"),
        6 => Some("TopCake"),
        7 => Some("SpinCake"),
        8 => Some("Darkburger"),
        9 => Some("LancerCookie"),
        10 => Some("GigaSalad"),
        11 => Some("Clubswich"),
        12 => Some("HeartsDonut"),
        13 => Some("ChocDiamond"),
        14 => Some("FavSandwich"),
        15 => Some("RouxlsRoux"),
        16 => Some("CD Bagel"),
        17 => Some("Mannequin"),
        18 => Some("Kris Tea"),
        19 => Some("Noelle Tea"),
        20 => Some("Ralsei Tea"),
        21 => Some("Susie Tea"),
        22 => Some("DD-Burger"),
        23 => Some("LightCandy"),
        24 => Some("ButJuice"),
        25 => Some("SpagettiCode"),
        26 => Some("JavaCookie"),
        27 => Some("TensionBit"),
        28 => Some("TensionGem"),
        29 => Some("TensionMax"),
        30 => Some("ReviveDust"),
        31 => Some("ReviveBrite"),
        32 => Some("S.POISON"),
        33 => Some("DogDollar"),
        34 => Some("TVDinner"),
        35 => Some("Pipis"),
        36 => Some("FlatSoda"),
        37 => Some("TVSlop"),
        38 => Some("ExecBuffet"),
        39 => Some("DeluxeDinner"),
        60 => Some("AncientSweet"),
        61 => Some("Rhapsotea"),
        62 => Some("Scarlixir"),
        63 => Some("BitterTear"),
        _ => None
    }
}

pub fn display_item(item_id: u32) -> CompactString {
    let item_name = try_get_item_name(item_id);
    item_name.map(|n| n.to_compact_string()).unwrap_or_else(|| format_compact!("Item {}", 
        item_id))
}
