use crate::passes::DiagnosticLocation;

use super::CfgNode;

impl DiagnosticLocation for CfgNode {
    fn range(&self) -> crate::parser::Range {
        self.node().range()
    }

    fn file(&self) -> uuid::Uuid {
        self.node().file()
    }

    fn raw_text(&self) -> String {
        self.node().raw_text()
    }
}
