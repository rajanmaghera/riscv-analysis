use uuid::Uuid;

use crate::parser::Range;

use super::{IsSomeDisplayableDiagnostic, SeverityLevel};

pub trait DiagnosticLocation {
    fn range(&self) -> Range;
    fn file(&self) -> Uuid;
    fn raw_text(&self) -> String;
}

pub trait DiagnosticMessage {
    fn title(&self) -> String;
    fn description(&self) -> String;
    fn long_description(&self) -> String;
    fn level(&self) -> SeverityLevel;
    fn related(&self) -> Option<Vec<RelatedDiagnosticItem>>;
}

#[derive(Clone)]
pub struct RelatedDiagnosticItem {
    pub file: Uuid,
    pub range: Range,
    pub description: String,
}

#[derive(Clone)]
pub struct DiagnosticItem {
    pub file: Uuid,
    pub range: Range,
    pub title: String,
    pub description: String,
    pub long_description: String,
    pub level: SeverityLevel,
    pub related: Option<Vec<RelatedDiagnosticItem>>,
}

impl DiagnosticItem {
    pub fn from_displayable(item: &dyn IsSomeDisplayableDiagnostic) -> Self {
        let level = item.get_severity();
        let range = item.range();
        let file = item.file();
        let title = item.to_string();
        let description = item.get_long_description_as_string();
        let related = None;
        DiagnosticItem {
            file,
            range,
            title: title.clone(),
            description: title,
            long_description: description,
            level,
            related,
        }
    }
}

impl PartialEq for DiagnosticItem {
    fn eq(&self, other: &Self) -> bool {
        self.range == other.range && self.file == other.file
    }
}
impl Eq for DiagnosticItem {}

impl PartialOrd for DiagnosticItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
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
        let related = val.related();
        DiagnosticItem {
            file,
            range,
            title,
            description,
            long_description,
            level,
            related,
        }
    }
}
