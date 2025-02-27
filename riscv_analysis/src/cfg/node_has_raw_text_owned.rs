use super::{CfgNode, HasRawTextOwned};

impl HasRawTextOwned for CfgNode {
    fn raw_text_owned(&self) -> String {
        self.node().raw_text_owned()
    }
}
