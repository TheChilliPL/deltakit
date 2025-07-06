pub fn try_get_party_member_name(index: i32) -> Option<&'static str> {
    match index {
        1 => Some("Kris"),
        2 => Some("Susie"),
        3 => Some("Ralsei"),
        4 => Some("Noelle"),
        _ => None,
    }
}

pub fn get_party_member(index: usize) -> Option<&'static str> {
    try_get_party_member_name(index as i32)
}
