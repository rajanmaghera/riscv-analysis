use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};

use super::{Register, Token, TokenType, With};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub struct LabelString(String);

impl LabelString {
    pub fn new<S: Into<String>>(value: S) -> Self {
        LabelString(value.into())
    }

    #[must_use] pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'a> From<&'a LabelString> for &'a str {
    fn from(label: &'a LabelString) -> &'a str {
        label.0.as_str()
    }
}

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

impl TryFrom<Token> for LabelString {
    type Error = ();

    fn try_from(value: Token) -> Result<Self, Self::Error> {
        match value.token_type() {
            TokenType::Symbol(s) => LabelString::try_from(s.clone()),
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

pub type LabelStringToken = With<LabelString>;
