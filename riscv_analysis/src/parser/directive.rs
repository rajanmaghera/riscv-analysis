use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum DirectiveType {
    Align,
    Ascii,
    Asciz,
    Byte,
    Data,
    Double,
    Dword,
    EndMacro,
    Eqv,
    Extern,
    Float,
    Global,
    Globl,
    Half,
    Include,
    Macro,
    Section,
    Space,
    String,
    Text,
    Word,
}

impl Display for DirectiveType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            DirectiveType::Align => write!(f, "align"),
            DirectiveType::Ascii => write!(f, "ascii"),
            DirectiveType::Asciz => write!(f, "asciz"),
            DirectiveType::Byte => write!(f, "byte"),
            DirectiveType::Data => write!(f, "data"),
            DirectiveType::Double => write!(f, "double"),
            DirectiveType::Dword => write!(f, "dword"),
            DirectiveType::EndMacro => write!(f, "endmacro"),
            DirectiveType::Eqv => write!(f, "eqv"),
            DirectiveType::Extern => write!(f, "extern"),
            DirectiveType::Float => write!(f, "float"),
            DirectiveType::Global => write!(f, "global"),
            DirectiveType::Globl => write!(f, "globl"),
            DirectiveType::Half => write!(f, "half"),
            DirectiveType::Include => write!(f, "include"),
            DirectiveType::Macro => write!(f, "macro"),
            DirectiveType::Section => write!(f, "section"),
            DirectiveType::Space => write!(f, "space"),
            DirectiveType::String => write!(f, "string"),
            DirectiveType::Text => write!(f, "text"),
            DirectiveType::Word => write!(f, "word"),
        }
    }
}

impl FromStr for DirectiveType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // ensure first char is a "."
        match s.to_lowercase().as_str() {
            "align" => Ok(DirectiveType::Align),
            "ascii" => Ok(DirectiveType::Ascii),
            "asciz" => Ok(DirectiveType::Asciz),
            "byte" => Ok(DirectiveType::Byte),
            "data" => Ok(DirectiveType::Data),
            "double" => Ok(DirectiveType::Double),
            "dword" => Ok(DirectiveType::Dword),
            "endmacro" => Ok(DirectiveType::EndMacro),
            "eqv" => Ok(DirectiveType::Eqv),
            "extern" => Ok(DirectiveType::Extern),
            "float" => Ok(DirectiveType::Float),
            "global" => Ok(DirectiveType::Global),
            "globl" => Ok(DirectiveType::Globl),
            "half" => Ok(DirectiveType::Half),
            "include" => Ok(DirectiveType::Include),
            "macro" => Ok(DirectiveType::Macro),
            "section" => Ok(DirectiveType::Section),
            "space" => Ok(DirectiveType::Space),
            "string" => Ok(DirectiveType::String),
            "text" => Ok(DirectiveType::Text),
            "word" => Ok(DirectiveType::Word),
            _ => Err(()),
        }
    }
}
