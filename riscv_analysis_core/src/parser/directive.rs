use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum DirectiveToken {
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

impl Display for DirectiveToken {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            DirectiveToken::Align => write!(f, "align"),
            DirectiveToken::Ascii => write!(f, "ascii"),
            DirectiveToken::Asciz => write!(f, "asciz"),
            DirectiveToken::Byte => write!(f, "byte"),
            DirectiveToken::Data => write!(f, "data"),
            DirectiveToken::Double => write!(f, "double"),
            DirectiveToken::Dword => write!(f, "dword"),
            DirectiveToken::EndMacro => write!(f, "endmacro"),
            DirectiveToken::Eqv => write!(f, "eqv"),
            DirectiveToken::Extern => write!(f, "extern"),
            DirectiveToken::Float => write!(f, "float"),
            DirectiveToken::Global => write!(f, "global"),
            DirectiveToken::Globl => write!(f, "globl"),
            DirectiveToken::Half => write!(f, "half"),
            DirectiveToken::Include => write!(f, "include"),
            DirectiveToken::Macro => write!(f, "macro"),
            DirectiveToken::Section => write!(f, "section"),
            DirectiveToken::Space => write!(f, "space"),
            DirectiveToken::String => write!(f, "string"),
            DirectiveToken::Text => write!(f, "text"),
            DirectiveToken::Word => write!(f, "word"),
        }
    }
}

impl FromStr for DirectiveToken {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // ensure first char is a "."
        match s.to_lowercase().as_str() {
            "align" => Ok(DirectiveToken::Align),
            "ascii" => Ok(DirectiveToken::Ascii),
            "asciz" => Ok(DirectiveToken::Asciz),
            "byte" => Ok(DirectiveToken::Byte),
            "data" => Ok(DirectiveToken::Data),
            "double" => Ok(DirectiveToken::Double),
            "dword" => Ok(DirectiveToken::Dword),
            "endmacro" => Ok(DirectiveToken::EndMacro),
            "eqv" => Ok(DirectiveToken::Eqv),
            "extern" => Ok(DirectiveToken::Extern),
            "float" => Ok(DirectiveToken::Float),
            "global" => Ok(DirectiveToken::Global),
            "globl" => Ok(DirectiveToken::Globl),
            "half" => Ok(DirectiveToken::Half),
            "include" => Ok(DirectiveToken::Include),
            "macro" => Ok(DirectiveToken::Macro),
            "section" => Ok(DirectiveToken::Section),
            "space" => Ok(DirectiveToken::Space),
            "string" => Ok(DirectiveToken::String),
            "text" => Ok(DirectiveToken::Text),
            "word" => Ok(DirectiveToken::Word),
            _ => Err(()),
        }
    }
}
