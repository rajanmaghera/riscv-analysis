use crate::parser::token::SymbolData;

// TODO fill
pub enum Inst {
    Add,
    Addi,
    Sub,
    Jalr,
    Jal,
}

pub enum InstType {
    RType,
    IType,
    //SType,
    //SBType,
    //UType,
    UJType,
    JalrType,
}

impl TryFrom<&SymbolData> for Inst {
    type Error = ();

    fn try_from(value: &SymbolData) -> Result<Self, Self::Error> {
        match value.0.to_lowercase().as_str() {
            "add" => Ok(Inst::Add),
            "addi" => Ok(Inst::Addi),
            "sub" => Ok(Inst::Sub),
            _ => Err(()),
        }
    }
}

impl From<&Inst> for InstType {
    fn from(value: &Inst) -> Self {
        match value {
            Inst::Add | Inst::Sub => InstType::RType,
            Inst::Addi => InstType::IType,
            Inst::Jalr => InstType::JalrType,
            Inst::Jal => InstType::UJType,
        }
    }
}
