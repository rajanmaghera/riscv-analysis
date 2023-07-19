use std::{fmt::Display, str::FromStr};

use super::{Info, Register, Token};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LabelString(pub String);

impl PartialEq<str> for LabelString {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

impl PartialEq<&str> for LabelString {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

impl FromStr for LabelString {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // ensure labelstring cannot be a register
        if Register::from_str(s).is_ok() {
            return Err(());
        }

        // ensure string only starts with a letter or underscore
        let first = s.chars().next().ok_or(())?;
        if !first.is_alphabetic() && first != '_' {
            return Err(());
        }

        // ensure string only contains safe characters (including numbers)
        if !s
            .chars()
            .all(|c| c.is_ascii_digit() || c.is_alphabetic() || c == '_' || c == '.' || c == '$')
        {
            return Err(());
        }
        Ok(LabelString(s.to_string()))
    }
}

impl Display for LabelString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl TryFrom<Info> for LabelString {
    type Error = ();

    fn try_from(value: Info) -> Result<Self, Self::Error> {
        match value.token {
            Token::Symbol(s) => LabelString::try_from(s),
            _ => Err(()),
        }
    }
}

impl TryFrom<String> for LabelString {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        LabelString::from_str(&value)
    }
}
