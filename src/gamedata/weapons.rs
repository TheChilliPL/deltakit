use compact_str::{format_compact, CompactString, ToCompactString};

// Extracted from gml_GlobalScript_scr_weaponinfo
pub fn try_get_weapon_name(weapon_id: u32) -> Option<&'static str> {
    match weapon_id {
        0 => Some("---"),
        1 => Some("Wood Blade"),
        2 => Some("Mane Ax"),
        3 => Some("Red Scarf"),
        4 => Some("EverybodyWeapon"),
        5 => Some("Spookysword"),
        6 => Some("Brave Ax"),
        7 => Some("Devilsknife"),
        8 => Some("Trefoil"),
        9 => Some("Ragger"),
        10 => Some("DaintyScarf"),
        11 => Some("TwistedSwd"),
        12 => Some("SnowRing"),
        13 => Some("ThornRing"),
        14 => Some("BounceBlade"),
        15 => Some("CheerScarf"),
        16 => Some("MechaSaber"),
        17 => Some("AutoAxe"),
        18 => Some("FiberScarf"),
        19 => Some("Ragger2"),
        20 => Some("BrokenSwd"),
        21 => Some("PuppetScarf"),
        22 => Some("FreezeRing"),
        23 => Some("Saber10"),
        24 => Some("ToxicAxe"),
        25 => Some("FlexScarf"),
        26 => Some("BlackShard"),
        50 => Some("JingleBlade"),
        51 => Some("ScarfMark"),
        52 => Some("JusticeAxe"),
        53 => Some("Winglade"),
        54 => Some("AbsorbAx"),
        _ => None
    }
}

pub fn display_weapon(weapon_id: u32) -> CompactString {
    let weapon_name = try_get_weapon_name(weapon_id);
    weapon_name.map(|n| n.to_compact_string()).unwrap_or_else(|| format_compact!("Weapon {}", 
        weapon_id))
}
