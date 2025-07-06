use compact_str::{CompactString, ToCompactString, format_compact};

// Extracted from gml_GlobalScript_scr_spellinfo using UndertaleModTool
pub fn try_get_spell_name(spell_id: i32) -> Option<&'static str> {
    match spell_id {
        0 => Some("---"),
        1 => Some("Rude Sword"),
        2 => Some("Heal Prayer"),
        3 => Some("Pacify"),
        4 => Some("Rude Buster"),
        5 => Some("Red Buster"),
        6 => Some("Dual Heal"),
        7 => Some("ACT"),
        8 => Some("Sleep Mist"),
        9 => Some("Ice Shock"),
        10 => Some("SnowGrave"),
        // Depending on the chapter and flags, can be:
        // UltimateHeal, UltraHeal, Heal, OKHeal, BetterHeal
        11 => Some("* Heal"),
        _ => None,
    }
}

pub fn display_spell(spell_id: i32) -> CompactString {
    let spell_name = try_get_spell_name(spell_id);
    spell_name
        .map(|n| n.to_compact_string())
        .unwrap_or_else(|| format_compact!("Spell {}", spell_id))
}
