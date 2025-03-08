use super::{IsSomeDisplayableDiagnostic, LintError};

pub struct DiagnosticManager {
    diagnostics: Vec<Box<dyn IsSomeDisplayableDiagnostic>>,
}

impl Default for DiagnosticManager {
    fn default() -> Self {
        Self::new()
    }
}

impl DiagnosticManager {
    #[must_use]
    pub fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
        }
    }

    pub fn push_real(&mut self, diagnostic: Box<dyn IsSomeDisplayableDiagnostic>) {
        self.diagnostics.push(diagnostic);
    }

    pub fn push(&mut self, fake_diag: LintError) {
        self.diagnostics.push(Box::new(fake_diag));
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.diagnostics.len()
    }

    pub fn iter(&self) -> std::slice::Iter<Box<dyn IsSomeDisplayableDiagnostic>> {
        self.diagnostics.iter()
    }
}

// implement indexing for DiagnosticManager
impl std::ops::Index<usize> for DiagnosticManager {
    type Output = Box<dyn IsSomeDisplayableDiagnostic>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.diagnostics[index]
    }
}

impl IntoIterator for DiagnosticManager {
    type Item = Box<dyn IsSomeDisplayableDiagnostic>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.diagnostics.into_iter()
    }
}

impl<'a> IntoIterator for &'a DiagnosticManager {
    type Item = &'a Box<dyn IsSomeDisplayableDiagnostic>;
    type IntoIter = std::slice::Iter<'a, Box<dyn IsSomeDisplayableDiagnostic>>;
    fn into_iter(self) -> Self::IntoIter {
        self.diagnostics.iter()
    }
}
