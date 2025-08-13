use crate::parser::{Range, RawToken};

use super::{
    DiagnosticLocation, IsRelatedDiagnosticInformation, IsSomeDisplayableDiagnostic, SeverityLevel,
};

struct RelatedDisplayedDiagnostic {
    description: String,
    token: RawToken,
}

impl DiagnosticLocation for RelatedDisplayedDiagnostic {
    fn file(&self) -> uuid::Uuid {
        self.token.file()
    }
    fn range(&self) -> Range {
        self.token.range()
    }
    fn raw_text(&self) -> String {
        self.token.raw_text()
    }
}

impl IsRelatedDiagnosticInformation for RelatedDisplayedDiagnostic {
    fn get_description(&self) -> String {
        self.description.clone()
    }
}

/// A class that can hold a generic error.
///
/// This type is used to hold an error. Any type that implements
/// the correct traits can be used as an error. This is useful
/// for passing errors or creating generic errors with messages
/// on the fly.
struct SimpleDisplayedDiagnostic {
    code_name: &'static str,
    title: &'static str,
    severity: SeverityLevel,
    token: RawToken,
    long_description: Option<String>,
    related_information: Vec<Box<dyn IsRelatedDiagnosticInformation>>,
    certainty: DiagnosticCertainty,
}

impl DiagnosticLocation for SimpleDisplayedDiagnostic {
    fn range(&self) -> Range {
        self.token.range()
    }

    fn file(&self) -> uuid::Uuid {
        self.token.file()
    }

    fn raw_text(&self) -> String {
        self.token.raw_text()
    }
}

impl IsSomeDisplayableDiagnostic for SimpleDisplayedDiagnostic {
    fn get_title(&self) -> &'static str {
        self.title
    }

    fn get_severity(&self) -> SeverityLevel {
        self.severity.clone()
    }

    fn get_error_code(&self) -> &'static str {
        self.code_name
    }

    fn get_long_description(&self) -> String {
        self.long_description.clone().unwrap_or_default()
    }
    fn get_related_information<'a>(
        &'a self,
    ) -> Option<Box<dyn Iterator<Item = &'a dyn IsRelatedDiagnosticInformation> + 'a>> {
        if self.related_information.is_empty() {
            None
        } else {
            Some(Box::new(
                self.related_information
                    .iter()
                    .map(std::convert::AsRef::as_ref),
            ))
        }
    }
}

pub enum DiagnosticCertainty {
    /// This diagnostic is emitted with 100% certainty
    IsTrue,
    /// This diagnostic is true if assumptions are met
    IsTrueIfAssumptionsAreMet,
    /// This diagnostic might not be true, but we are emitting
    /// to be overly cautious; e.g. if a value is Unknown/UnknownConst
    MightBeTrue,
    /// Not sure how certain this diagnostic is
    Unknown,
}

pub struct DiagnosticBuilder {
    code_name: &'static str,
    title: &'static str,
    long_description: Option<String>,
    related_information: Vec<Box<dyn IsRelatedDiagnosticInformation>>,
    certainty: DiagnosticCertainty,
}

impl DiagnosticBuilder {
    #[must_use]
    pub fn new(code_name: &'static str, title: &'static str) -> Self {
        Self {
            code_name,
            title,
            long_description: None,
            related_information: Vec::new(),
            certainty: DiagnosticCertainty::Unknown,
        }
    }

    fn on(
        self,
        annotated_item: &impl DiagnosticLocation,
        severity: SeverityLevel,
    ) -> impl IsSomeDisplayableDiagnostic {
        SimpleDisplayedDiagnostic {
            code_name: self.code_name,
            title: self.title,
            severity,
            token: annotated_item.as_raw_token(),
            long_description: self.long_description,
            related_information: self.related_information,
            certainty: self.certainty,
        }
    }

    pub fn is_error_on(
        self,
        annotated_item: &impl DiagnosticLocation,
    ) -> impl IsSomeDisplayableDiagnostic {
        self.on(annotated_item, SeverityLevel::Error)
    }

    pub fn is_hint_on(
        self,
        annotated_item: &impl DiagnosticLocation,
    ) -> impl IsSomeDisplayableDiagnostic {
        self.on(annotated_item, SeverityLevel::Hint)
    }

    pub fn is_warning_on(
        self,
        annotated_item: &impl DiagnosticLocation,
    ) -> impl IsSomeDisplayableDiagnostic {
        self.on(annotated_item, SeverityLevel::Warning)
    }

    pub fn is_information_on(
        self,
        annotated_item: &impl DiagnosticLocation,
    ) -> impl IsSomeDisplayableDiagnostic {
        self.on(annotated_item, SeverityLevel::Information)
    }

    #[must_use]
    pub fn description(mut self, long_description: impl Into<String>) -> Self {
        self.long_description = Some(long_description.into());
        self
    }

    #[must_use]
    pub fn related(mut self, description: impl Into<String>, token: RawToken) -> Self {
        self.related_information
            .push(Box::new(RelatedDisplayedDiagnostic {
                description: description.into(),
                token,
            }) as Box<dyn IsRelatedDiagnosticInformation>);
        self
    }

    #[must_use]
    pub fn with_certainty(mut self, certainty: DiagnosticCertainty) -> Self {
        self.certainty = certainty;
        self
    }
}
