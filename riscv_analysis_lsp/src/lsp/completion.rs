use lsp_types::CompletionItem;

use riscv_analysis::parser::{LabelString, RVInst, RVRegister};

pub enum RVCompletionItem {
    Inst(RVInst),
    Register(RVRegister),
    Label(LabelString),
    FunctionLabel(LabelString),
}

impl From<RVRegister> for RVCompletionItem {
    fn from(value: RVRegister) -> Self {
        RVCompletionItem::Register(value)
    }
}

impl From<RVInst> for RVCompletionItem {
    fn from(value: RVInst) -> Self {
        RVCompletionItem::Inst(value)
    }
}

impl From<LabelString> for RVCompletionItem {
    fn from(value: LabelString) -> Self {
        RVCompletionItem::Label(value)
    }
}

impl From<RVCompletionItem> for CompletionItem {
    fn from(value: RVCompletionItem) -> Self {
        match value {
            RVCompletionItem::Register(r) => {
                let mut item = CompletionItem::new_simple(
                    r.to_string(),
                    format!(
                        "Register {}",
                        r.all_representations()
                            .into_iter()
                            .collect::<Vec<String>>()
                            .join(", ")
                    ),
                );
                item.kind = Some(lsp_types::CompletionItemKind::VARIABLE);
                item.filter_text = Some(
                    r.all_representations()
                        .into_iter()
                        .collect::<Vec<String>>()
                        .join(" "),
                );
                item
            }
            RVCompletionItem::Inst(i) => {
                let mut item =
                    CompletionItem::new_simple(i.to_string(), format!("Instruction {i}"));
                item.kind = Some(lsp_types::CompletionItemKind::FUNCTION);
                item
            }
            RVCompletionItem::Label(l) => {
                CompletionItem::new_simple(l.to_string(), format!("Label {l}"))
            }
            RVCompletionItem::FunctionLabel(l) => {
                CompletionItem::new_simple(l.to_string(), format!("Function {l}"))
            }
        }
    }
}

impl RVCompletionItem {
    fn get_registers() -> Vec<CompletionItem> {
        RVRegister::all()
            .into_iter()
            .map(|r| RVCompletionItem::from(r).into())
            .collect()
    }

    fn get_instructions() -> Vec<CompletionItem> {
        RVInst::all()
            .into_iter()
            .map(|i| RVCompletionItem::from(i).into())
            .collect()
    }

    pub fn get_all() -> Vec<CompletionItem> {
        let mut items = RVCompletionItem::get_registers();
        items.extend(RVCompletionItem::get_instructions());
        items
    }
}
