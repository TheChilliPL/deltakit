use compact_str::{format_compact, CompactString};

pub trait Serializable {
    fn serialize(&self) -> CompactString;
}

impl Serializable for str {
    fn serialize(&self) -> CompactString {
        CompactString::from(self)
    }
}

impl Serializable for i32 {
    fn serialize(&self) -> CompactString {
        format_compact!("{} ", self)
    }
}

impl Serializable for f32 {
    fn serialize(&self) -> CompactString {
        format_compact!("{} ", self)
    }   
}

impl Serializable for u64 {
    fn serialize(&self) -> CompactString {
        format_compact!("{} ", self)
    }
}
