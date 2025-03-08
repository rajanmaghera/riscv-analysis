use crate::parser::HasIdentity;

use super::CfgNode;

impl HasIdentity for CfgNode {
    fn id(&self) -> uuid::Uuid {
        self.node().id()
    }
}
