use compact_str::{CompactString, ToCompactString, format_compact};

// Extracted from gml_GlobalScript_scr_phonename using UndertaleModTool
pub fn try_get_phone_number_name(number_id: i32) -> Option<&'static str> {
    match number_id {
        201 => Some("Call Home"),     // Sometimes as "Call Toriel"
        202 => Some("Sans's Number"), // Sometimes as "Not Sans's Number"
        _ => None,
    }
}

pub fn display_phone_number(number_id: i32) -> CompactString {
    let phone_name = try_get_phone_number_name(number_id);
    phone_name
        .map(|n| n.to_compact_string())
        .unwrap_or_else(|| format_compact!("Phone #{}", number_id))
}
