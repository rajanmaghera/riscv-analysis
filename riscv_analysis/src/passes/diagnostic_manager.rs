use super::IsSomeDisplayableDiagnostic;

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

    pub fn push(&mut self, diagnostic: impl IsSomeDisplayableDiagnostic + 'static) {
        self.diagnostics.push(Box::new(diagnostic));
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.diagnostics.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &dyn IsSomeDisplayableDiagnostic> {
        self.diagnostics.iter().map(AsRef::as_ref)
    }

    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&Box<dyn IsSomeDisplayableDiagnostic>) -> bool,
    {
        self.diagnostics.retain(f);
    }
}

// implement indexing for DiagnosticManager
#[cfg(test)]
impl std::ops::Index<usize> for DiagnosticManager {
    type Output = Box<dyn IsSomeDisplayableDiagnostic>;

    fn index(&self, index: usize) -> &Self::Output {
        #[allow(clippy::indexing_slicing)]
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
