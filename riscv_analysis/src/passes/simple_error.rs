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

pub struct DiagnosticBuilder {
    code_name: &'static str,
    title: &'static str,
    long_description: Option<String>,
    related_information: Vec<Box<dyn IsRelatedDiagnosticInformation>>,
}

impl DiagnosticBuilder {
    #[must_use]
    pub fn new(code_name: &'static str, title: &'static str) -> Self {
        Self {
            code_name,
            title,
            long_description: None,
            related_information: Vec::new(),
        }
    }

    fn on<T: AsRef<impl DiagnosticLocation>>(
        self,
        annotated_item: T,
        severity: SeverityLevel,
    ) -> Box<dyn IsSomeDisplayableDiagnostic> {
        Box::new(SimpleDisplayedDiagnostic {
            code_name: self.code_name,
            title: self.title,
            severity,
            token: annotated_item.as_ref().as_raw_token(),
            long_description: self.long_description,
            related_information: self.related_information,
        })
    }

    pub fn is_error_on<T: AsRef<impl DiagnosticLocation>>(
        self,
        annotated_item: T,
    ) -> Box<dyn IsSomeDisplayableDiagnostic> {
        self.on(annotated_item, SeverityLevel::Error)
    }

    pub fn is_hint_on<T: AsRef<impl DiagnosticLocation>>(
        self,
        annotated_item: T,
    ) -> Box<dyn IsSomeDisplayableDiagnostic> {
        self.on(annotated_item, SeverityLevel::Hint)
    }

    pub fn is_warning_on<T: AsRef<impl DiagnosticLocation>>(
        self,
        annotated_item: T,
    ) -> Box<dyn IsSomeDisplayableDiagnostic> {
        self.on(annotated_item, SeverityLevel::Warning)
    }

    pub fn is_information_on<T: AsRef<impl DiagnosticLocation>>(
        self,
        annotated_item: T,
    ) -> Box<dyn IsSomeDisplayableDiagnostic> {
        self.on(annotated_item, SeverityLevel::Information)
    }

    #[must_use]
    pub fn description<S: Into<String>>(mut self, long_description: S) -> Self {
        self.long_description = Some(long_description.into());
        self
    }

    #[must_use]
    pub fn related<S: Into<String>>(mut self, description: S, token: RawToken) -> Self {
        self.related_information
            .push(Box::new(RelatedDisplayedDiagnostic {
                description: description.into(),
                token,
            }) as Box<dyn IsRelatedDiagnosticInformation>);
        self
    }
}
