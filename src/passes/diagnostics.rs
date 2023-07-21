use uuid::Uuid;

use crate::parser::Range;

use super::WarningLevel;

pub trait DiagnosticLocation {
    fn range(&self) -> Range;
    fn file(&self) -> Uuid;
}

pub trait DiagnosticMessage {
    fn title(&self) -> String;
    fn description(&self) -> String;
    fn long_description(&self) -> String;
    fn level(&self) -> WarningLevel;
}

pub struct DiagnosticItem {
    pub file: Uuid,
    pub range: Range,
    pub title: String,
    pub description: String,
    pub long_description: String,
    pub level: WarningLevel,
}

impl PartialEq for DiagnosticItem {
    fn eq(&self, other: &Self) -> bool {
        self.range == other.range && self.file == other.file
    }
}
impl Eq for DiagnosticItem {}

impl PartialOrd for DiagnosticItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.file == other.file {
            self.range.partial_cmp(&other.range)
        } else {
            None
        }
    }
}

impl Ord for DiagnosticItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.file == other.file {
            self.range.cmp(&other.range)
        } else {
            self.file.cmp(&other.file)
        }
    }
}

impl<T> From<T> for DiagnosticItem
where
    T: DiagnosticMessage + DiagnosticLocation,
{
    fn from(val: T) -> Self {
        let level = val.level();
        let range = val.range();
        let file = val.file();
        let title = val.title();
        let description = val.description();
        let long_description = val.long_description();
        DiagnosticItem {
            file,
            range,
            title,
            description,
            long_description,
            level,
        }
    }
}
