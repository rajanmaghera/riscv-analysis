use std::fmt::Display;

use crate::parser::token::SymbolData;

#[derive(Debug, Clone, PartialEq)]
pub enum BasicType {
    Ebreak,
    Ecall,
    Uret,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArithType {
    Add,
    Addw,
    And,
    Or,
    Sll,
    Sllw,
    Slt,
    Sltu,
    Sra,
    Sraw,
    Srl,
    Srlw,
    Sub,
    Xor,
    Mul,
    Mulh,
    Mulhsu,
    Mulhu,
    Div,
    Divu,
    Divw,
    Rem,
    Remu,
    Remw,
    Remuw,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BranchType {
    Beq,
    Bge,
    Bgeu,
    Blt,
    Bltu,
    Bne,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IArithType {
    Addi,
    Addiw,
    Andi,
    Ori,
    Slli,
    Slliw,
    Slti,
    Sltiu,
    Srai,
    Sraiw,
    Srli,
    Srliw,
    Xori,
    Auipc, // Same as this
}

// TODO how is pseudo instruction handled for the FromStr trait?

#[derive(Debug, Clone, PartialEq)]
pub enum LoadType {
    Lb,
    Lbu,
    Lh,
    Lhu,
    Lw,
    Lwu,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StoreType {
    Sb,
    Sh,
    Sw,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CSRType {
    Csrrw,
    Csrrs,
    Csrrc,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CSRIType {
    Csrrwi,
    Csrrsi,
    Csrrci,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IgnoreType {
    Fence,
    Fencei,
}

#[derive(Debug, Clone, PartialEq)]
pub enum JumpLinkType {
    Jal,
}

#[derive(Debug, Clone, PartialEq)]
pub enum JumpLinkRType {
    Jalr,
}

pub enum Inst {
    Ret,
    Ebreak,
    Ecall,
    Nop,
    Add,
    Addw,
    And,
    Or,
    Sll,
    Sllw,
    Slt,
    Sltu,
    Sra,
    Sraw,
    Srl,
    Srlw,
    Sub,
    Xor,
    Mul,
    Mulh,
    Mulhsu,
    Mulhu,
    Div,
    Divu,
    Divw,
    Rem,
    Remu,
    Remw,
    Remuw,
    Beq,
    Bge,
    Bgeu,
    Blt,
    Bltu,
    Bne,
    Addi,
    Addiw,
    Andi,
    Ori,
    Slli,
    Slliw,
    Slti,
    Sltiu,
    Srai,
    Sraiw,
    Srli,
    Srliw,
    Xori,
    Lui,
    Lb,
    Lbu,
    Lh,
    Lhu,
    Lw,
    Lwu,
    Sb,
    Sh,
    Sw,
    Csrrw,
    Csrrs,
    Csrrc,
    Csrrwi,
    Csrrsi,
    Csrrci,
    Fence,
    Fencei,
    Jal,
    Jalr,
    Auipc,
    Beqz,
    Bnez,
    J,
    Jr,
    La,
    Li,
    Mv,
    Neg,
    Not,
    Seqz,
    Snez,
    Sltz,
    Sgez,
    Sgtz,
    B,
    Bltz,
    Bgez,
    Call,
    Bgt,
    Ble,
    Bgtu,
    Bleu,
    Bgtz,
    Blez,
    Csrc,
    Csrr,
    Csrs,
    Csrw,
    Csrci,
    Csrsi,
    Csrwi,
    Uret
}

impl Display for Inst {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Inst::Ret => write!(f, "ret"),
            Inst::Ebreak => write!(f, "ebreak"),
            Inst::Ecall => write!(f, "ecall"),
            Inst::Nop => write!(f, "nop"),
            Inst::Add => write!(f, "add"),
            Inst::Addw => write!(f, "addw"),
            Inst::And => write!(f, "and"),
            Inst::Or => write!(f, "or"),
            Inst::Sll => write!(f, "sll"),
            Inst::Sllw => write!(f, "sllw"),
            Inst::Slt => write!(f, "slt"),
            Inst::Sltu => write!(f, "sltu"),
            Inst::Sra => write!(f, "sra"),
            Inst::Sraw => write!(f, "sraw"),
            Inst::Srl => write!(f, "srl"),
            Inst::Srlw => write!(f, "srlw"),
            Inst::Sub => write!(f, "sub"),
            Inst::Xor => write!(f, "xor"),
            Inst::Mul => write!(f, "mul"),
            Inst::Mulh => write!(f, "mulh"),
            Inst::Mulhsu => write!(f, "mulhsu"),
            Inst::Mulhu => write!(f, "mulhu"),
            Inst::Div => write!(f, "div"),
            Inst::Divu => write!(f, "divu"),
            Inst::Divw => write!(f, "divw"),
            Inst::Rem => write!(f, "rem"),
            Inst::Remu => write!(f, "remu"),
            Inst::Remw => write!(f, "remw"),
            Inst::Remuw => write!(f, "remuw"),
            Inst::Beq => write!(f, "beq"),
            Inst::Bge => write!(f, "bge"),
            Inst::Bgeu => write!(f, "bgeu"),
            Inst::Blt => write!(f, "blt"),
            Inst::Bltu => write!(f, "bltu"),
            Inst::Bne => write!(f, "bne"),
            Inst::Addi => write!(f, "addi"),
            Inst::Addiw => write!(f, "addiw"),
            Inst::Andi => write!(f, "andi"),
            Inst::Ori => write!(f, "ori"),
            Inst::Slli => write!(f, "slli"),
            Inst::Slliw => write!(f, "slliw"),
            Inst::Slti => write!(f, "slti"),
            Inst::Sltiu => write!(f, "sltiu"),
            Inst::Srai => write!(f, "srai"),
            Inst::Sraiw => write!(f, "sraiw"),
            Inst::Srli => write!(f, "srli"),
            Inst::Srliw => write!(f, "srliw"),
            Inst::Xori => write!(f, "xori"),
            Inst::Lui => write!(f, "lui"),
            Inst::Lb => write!(f, "lb"),
            Inst::Lbu => write!(f, "lbu"),
            Inst::Lh => write!(f, "lh"),
            Inst::Lhu => write!(f, "lhu"),
            Inst::Lw => write!(f, "lw"),
            Inst::Lwu => write!(f, "lwu"),
            Inst::Sb => write!(f, "sb"),
            Inst::Sh => write!(f, "sh"),
            Inst::Sw => write!(f, "sw"),
            Inst::Csrrw => write!(f, "csrrw"),
            Inst::Csrrs => write!(f, "csrrs"),
            Inst::Csrrc => write!(f, "csrrc"),
            Inst::Csrrwi => write!(f, "csrrwi"),
            Inst::Csrrsi => write!(f, "csrrsi"),
            Inst::Csrrci => write!(f, "csrrci"),
            Inst::Fence => write!(f, "fence"),
            Inst::Fencei => write!(f, "fencei"),
            Inst::Jal => write!(f, "jal"),
            Inst::Jalr => write!(f, "jalr"),
            Inst::Auipc => write!(f, "auipc"),
            Inst::Beqz => write!(f, "beqz"),
            Inst::Bnez => write!(f, "bnez"),
            Inst::J => write!(f, "j"),
            Inst::Jr => write!(f, "jr"),
            Inst::La => write!(f, "la"),
            Inst::Li => write!(f, "li"),
            Inst::Mv => write!(f, "mv"),
            Inst::Neg => write!(f, "neg"),
            Inst::Not => write!(f, "not"),
            Inst::Seqz => write!(f, "seqz"),
            Inst::Snez => write!(f, "snez"),
            Inst::Sltz => write!(f, "sltz"),
            Inst::Sgez => write!(f, "sgez"),
            Inst::Sgtz => write!(f, "sgtz"),
            Inst::Bgez => write!(f, "bgez"),
            Inst::Bltz => write!(f, "bltz"),
            Inst::B => write!(f, "b"),
            Inst::Call => write!(f, "call"),
            Inst::Bgt => write!(f, "bgt"),
            Inst::Ble => write!(f, "ble"),
            Inst::Bgtu => write!(f, "bgtu"),
            Inst::Bleu => write!(f, "bleu"),
            Inst::Bgtz => write!(f, "bgtz"),
            Inst::Blez => write!(f, "blez"),
            Inst::Csrc => write!(f, "csrc"),
            Inst::Csrr => write!(f, "csrr"),
            Inst::Csrs => write!(f, "csrs"),
            Inst::Csrw => write!(f, "csrw"),
            Inst::Csrci => write!(f, "csrci"),
            Inst::Csrsi => write!(f, "csrsi"),
            Inst::Csrwi => write!(f, "csrwi"),
            Inst::Uret => write!(f, "uret"),
        }
    }
}

pub enum InstType {
    ArithType(ArithType),
    IArithType(IArithType),
    BasicType(BasicType),
    JumpLinkType(JumpLinkType),
    JumpLinkRType(JumpLinkRType),
    LoadType(LoadType),
    StoreType(StoreType),
    CSRType(CSRType),
    CSRIType(CSRIType),
    IgnoreType(IgnoreType),
    BranchType(BranchType),
    PseudoType(PseudoType),
    UpperArithType(UpperArithType),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum PseudoType {
    Beqz,
    Bnez,
    Bltz,
    Bgez,
    J,
    Jr,
    La,
    Li,
    Mv,
    Neg,
    Nop,
    Not,
    Ret,
    Seqz,
    Snez,
    Sgtz,
    Sltz,
    Sgez,
    B,
    Call,
    Bgt,
    Ble,
    Bgtu,
    Bleu,
    Bgtz,
    Blez,
    Csrc,
    Csrr,
    Csrs,
    Csrw,
    Csrci,
    Csrsi,
    Csrwi
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum UpperArithType {
    Lui
}

impl TryFrom<&SymbolData> for Inst {
    type Error = ();

    // TODO figure out how to ensure every Inst is covered
    fn try_from(value: &SymbolData) -> Result<Self, Self::Error> {
        match value.0.to_lowercase().as_str() {
            "ret" => Ok(Inst::Ret),
            "ebreak" => Ok(Inst::Ebreak),
            "ecall" => Ok(Inst::Ecall),
            "nop" => Ok(Inst::Nop),
            "add" => Ok(Inst::Add),
            "addw" => Ok(Inst::Addw),
            "and" => Ok(Inst::And),
            "or" => Ok(Inst::Or),
            "sll" => Ok(Inst::Sll),
            "sllw" => Ok(Inst::Sllw),
            "slt" => Ok(Inst::Slt),
            "sltu" => Ok(Inst::Sltu),
            "sra" => Ok(Inst::Sra),
            "sraw" => Ok(Inst::Sraw),
            "srl" => Ok(Inst::Srl),
            "srlw" => Ok(Inst::Srlw),
            "sub" => Ok(Inst::Sub),
            "xor" => Ok(Inst::Xor),
            "mul" => Ok(Inst::Mul),
            "mulh" => Ok(Inst::Mulh),
            "mulhsu" => Ok(Inst::Mulhsu),
            "mulhu" => Ok(Inst::Mulhu),
            "div" => Ok(Inst::Div),
            "divu" => Ok(Inst::Divu),
            "divw" => Ok(Inst::Divw),
            "rem" => Ok(Inst::Rem),
            "remu" => Ok(Inst::Remu),
            "remw" => Ok(Inst::Remw),
            "remuw" => Ok(Inst::Remuw),
            "beq" => Ok(Inst::Beq),
            "bge" => Ok(Inst::Bge),
            "bgeu" => Ok(Inst::Bgeu),
            "blt" => Ok(Inst::Blt),
            "bltu" => Ok(Inst::Bltu),
            "bne" => Ok(Inst::Bne),
            "addi" => Ok(Inst::Addi),
            "addiw" => Ok(Inst::Addiw),
            "andi" => Ok(Inst::Andi),
            "ori" => Ok(Inst::Ori),
            "slli" => Ok(Inst::Slli),
            "slliw" => Ok(Inst::Slliw),
            "slti" => Ok(Inst::Slti),
            "sltiu" => Ok(Inst::Sltiu),
            "srai" => Ok(Inst::Srai),
            "sraiw" => Ok(Inst::Sraiw),
            "srli" => Ok(Inst::Srli),
            "srliw" => Ok(Inst::Srliw),
            "xori" => Ok(Inst::Xori),
            "lui" => Ok(Inst::Lui),
            "lb" => Ok(Inst::Lb),
            "lbu" => Ok(Inst::Lbu),
            "lh" => Ok(Inst::Lh),
            "lhu" => Ok(Inst::Lhu),
            "lw" => Ok(Inst::Lw),
            "lwu" => Ok(Inst::Lwu),
            "sb" => Ok(Inst::Sb),
            "sh" => Ok(Inst::Sh),
            "sw" => Ok(Inst::Sw),
            "csrrw" => Ok(Inst::Csrrw),
            "csrrs" => Ok(Inst::Csrrs),
            "csrrc" => Ok(Inst::Csrrc),
            "csrrwi" => Ok(Inst::Csrrwi),
            "csrrsi" => Ok(Inst::Csrrsi),
            "csrrci" => Ok(Inst::Csrrci),
            "fence" => Ok(Inst::Fence),
            "fencei" => Ok(Inst::Fencei),
            "jal" => Ok(Inst::Jal),
            "jalr" => Ok(Inst::Jalr),
            "auipc" => Ok(Inst::Auipc),
            "beqz" => Ok(Inst::Beqz),
            "bnez" => Ok(Inst::Bnez),
            "j" => Ok(Inst::J),
            "jr" => Ok(Inst::Jr),
            "la" => Ok(Inst::La),
            "li" => Ok(Inst::Li),
            "mv" => Ok(Inst::Mv),
            "neg" => Ok(Inst::Neg),
            "not" => Ok(Inst::Not),
            "seqz" => Ok(Inst::Seqz),
            "snez" => Ok(Inst::Snez),
            "sgtz" => Ok(Inst::Sgtz),
            "sltz" => Ok(Inst::Sltz),    
            "b" => Ok(Inst::B),
            "bltz" => Ok(Inst::Bltz),
            "bgez" => Ok(Inst::Bgez),
            "call" => Ok(Inst::Call),
            "bgt" => Ok(Inst::Bgt),
            "ble" => Ok(Inst::Ble),
            "bgtu" => Ok(Inst::Bgtu),
            "bleu" => Ok(Inst::Bleu),
            "bgtz" => Ok(Inst::Bgtz),
            "blez" => Ok(Inst::Blez),
            "sgez" => Ok(Inst::Sgez),
            "csrc" => Ok(Inst::Csrc),
            "csrr" => Ok(Inst::Csrr),
            "csrs" => Ok(Inst::Csrs),
            "csrw" => Ok(Inst::Csrw),
            "csrci" => Ok(Inst::Csrci),
            "csrsi" => Ok(Inst::Csrsi),
            "csrwi" => Ok(Inst::Csrwi),
            "uret" => Ok(Inst::Uret),
            _ => Err(()),
        }
    }
}

// TODO uret
impl From<&Inst> for InstType {
    fn from(value: &Inst) -> Self {
        match value {
            Inst::Add => InstType::ArithType(ArithType::Add),
            Inst::Addw => InstType::ArithType(ArithType::Addw),
            Inst::And => InstType::ArithType(ArithType::And),
            Inst::Or => InstType::ArithType(ArithType::Or),
            Inst::Sll => InstType::ArithType(ArithType::Sll),
            Inst::Sllw => InstType::ArithType(ArithType::Sllw),
            Inst::Slt => InstType::ArithType(ArithType::Slt),
            Inst::Sltu => InstType::ArithType(ArithType::Sltu),
            Inst::Sra => InstType::ArithType(ArithType::Sra),
            Inst::Sraw => InstType::ArithType(ArithType::Sraw),
            Inst::Srl => InstType::ArithType(ArithType::Srl),
            Inst::Srlw => InstType::ArithType(ArithType::Srlw),
            Inst::Sub => InstType::ArithType(ArithType::Sub),
            Inst::Xor => InstType::ArithType(ArithType::Xor),
            Inst::Mul => InstType::ArithType(ArithType::Mul),
            Inst::Mulh => InstType::ArithType(ArithType::Mulh),
            Inst::Mulhsu => InstType::ArithType(ArithType::Mulhsu),
            Inst::Mulhu => InstType::ArithType(ArithType::Mulhu),
            Inst::Div => InstType::ArithType(ArithType::Div),
            Inst::Divu => InstType::ArithType(ArithType::Divu),
            Inst::Divw => InstType::ArithType(ArithType::Divw),
            Inst::Rem => InstType::ArithType(ArithType::Rem),
            Inst::Remu => InstType::ArithType(ArithType::Remu),
            Inst::Remw => InstType::ArithType(ArithType::Remw),
            Inst::Remuw => InstType::ArithType(ArithType::Remuw),
            Inst::Beq => InstType::BranchType(BranchType::Beq),
            Inst::Bge => InstType::BranchType(BranchType::Bge),
            Inst::Bgeu => InstType::BranchType(BranchType::Bgeu),
            Inst::Blt => InstType::BranchType(BranchType::Blt),
            Inst::Bltu => InstType::BranchType(BranchType::Bltu),
            Inst::Bne => InstType::BranchType(BranchType::Bne),
            Inst::Addi => InstType::IArithType(IArithType::Addi),
            Inst::Addiw => InstType::IArithType(IArithType::Addiw),
            Inst::Andi => InstType::IArithType(IArithType::Andi),
            Inst::Ori => InstType::IArithType(IArithType::Ori),
            Inst::Slli => InstType::IArithType(IArithType::Slli),
            Inst::Slliw => InstType::IArithType(IArithType::Slliw),
            Inst::Slti => InstType::IArithType(IArithType::Slti),
            Inst::Sltiu => InstType::IArithType(IArithType::Sltiu),
            Inst::Srai => InstType::IArithType(IArithType::Srai),
            Inst::Sraiw => InstType::IArithType(IArithType::Sraiw),
            Inst::Srli => InstType::IArithType(IArithType::Srli),
            Inst::Srliw => InstType::IArithType(IArithType::Srliw),
            Inst::Xori => InstType::IArithType(IArithType::Xori),
            Inst::Lui => InstType::UpperArithType(UpperArithType::Lui),
            Inst::Lb => InstType::LoadType(LoadType::Lb),
            Inst::Lbu => InstType::LoadType(LoadType::Lbu),
            Inst::Lh => InstType::LoadType(LoadType::Lh),
            Inst::Lhu => InstType::LoadType(LoadType::Lhu),
            Inst::Lw => InstType::LoadType(LoadType::Lw),
            Inst::Lwu => InstType::LoadType(LoadType::Lwu),
            Inst::Sb => InstType::StoreType(StoreType::Sb),
            Inst::Sh => InstType::StoreType(StoreType::Sh),
            Inst::Sw => InstType::StoreType(StoreType::Sw),
            Inst::Fence => InstType::IgnoreType(IgnoreType::Fence),
            Inst::Fencei => InstType::IgnoreType(IgnoreType::Fencei),
            Inst::Jal => InstType::JumpLinkType(JumpLinkType::Jal),
            Inst::Jalr => InstType::JumpLinkRType(JumpLinkRType::Jalr),
            Inst::Ecall => InstType::BasicType(BasicType::Ecall),
            Inst::Ebreak => InstType::BasicType(BasicType::Ebreak),
            Inst::Ret => InstType::PseudoType(PseudoType::Ret),
            Inst::Csrrw => InstType::CSRType(CSRType::Csrrw),
            Inst::Csrrs => InstType::CSRType(CSRType::Csrrs),
            Inst::Csrrc => InstType::CSRType(CSRType::Csrrc),
            Inst::Csrrwi => InstType::CSRIType(CSRIType::Csrrwi),
            Inst::Csrrsi => InstType::CSRIType(CSRIType::Csrrsi),
            Inst::Csrrci => InstType::CSRIType(CSRIType::Csrrci),
            Inst::Nop => InstType::PseudoType(PseudoType::Nop),
            Inst::Auipc => InstType::IArithType(IArithType::Auipc),
            Inst::Beqz => InstType::PseudoType(PseudoType::Beqz),
            Inst::Bnez => InstType::PseudoType(PseudoType::Bnez),
            Inst::J => InstType::PseudoType(PseudoType::J),
            Inst::Jr => InstType::PseudoType(PseudoType::Jr),
            Inst::Li => InstType::PseudoType(PseudoType::Li),
            Inst::La => InstType::PseudoType(PseudoType::La),
            Inst::Mv => InstType::PseudoType(PseudoType::Mv),
            Inst::Neg => InstType::PseudoType(PseudoType::Neg),
            Inst::Not => InstType::PseudoType(PseudoType::Not),
            Inst::Seqz => InstType::PseudoType(PseudoType::Seqz),
            Inst::Snez => InstType::PseudoType(PseudoType::Snez),
            Inst::Sltz => InstType::PseudoType(PseudoType::Sltz),
            Inst::Sgez => InstType::PseudoType(PseudoType::Sgez),
            Inst::Sgtz => InstType::PseudoType(PseudoType::Sgtz),
            Inst::B => InstType::PseudoType(PseudoType::B),
            Inst::Bltz => InstType::PseudoType(PseudoType::Bltz),
            Inst::Bgez => InstType::PseudoType(PseudoType::Bgez),
            Inst::Bgtz => InstType::PseudoType(PseudoType::Bgtz),
            Inst::Blez => InstType::PseudoType(PseudoType::Blez),
            Inst::Call => InstType::PseudoType(PseudoType::Call),
            Inst::Bgt => InstType::PseudoType(PseudoType::Bgt),
            Inst::Ble => InstType::PseudoType(PseudoType::Ble),
            Inst::Bgtu => InstType::PseudoType(PseudoType::Bgtu),
            Inst::Bleu => InstType::PseudoType(PseudoType::Bleu),
            Inst::Csrc => InstType::PseudoType(PseudoType::Csrc),
            Inst::Csrr => InstType::PseudoType(PseudoType::Csrr),
            Inst::Csrs => InstType::PseudoType(PseudoType::Csrs),
            Inst::Csrw => InstType::PseudoType(PseudoType::Csrw),
            Inst::Csrci => InstType::PseudoType(PseudoType::Csrci),
            Inst::Csrsi => InstType::PseudoType(PseudoType::Csrsi),
            Inst::Csrwi => InstType::PseudoType(PseudoType::Csrwi),
            Inst::Uret => InstType::BasicType(BasicType::Uret),
        }
    }
}

impl From<&ArithType> for Inst {
    fn from(value: &ArithType) -> Self {
        match value {
            ArithType::Add => Inst::Add,
            ArithType::Addw => Inst::Addw,
            ArithType::Sub => Inst::Sub,
            ArithType::Mul => Inst::Mul,
            ArithType::Div => Inst::Div,
            ArithType::Divu => Inst::Divu,
            ArithType::Divw => Inst::Divw,
            ArithType::Rem => Inst::Rem,
            ArithType::Remu => Inst::Remu,
            ArithType::Remw => Inst::Remw,
            ArithType::Remuw => Inst::Remuw,
            ArithType::And => Inst::And,
            ArithType::Or => Inst::Or,
            ArithType::Xor => Inst::Xor,
            ArithType::Sll => Inst::Sll,
            ArithType::Sllw => Inst::Sllw,
            ArithType::Slt => Inst::Slt,
            ArithType::Sltu => Inst::Sltu,
            ArithType::Sra => Inst::Sra,
            ArithType::Sraw => Inst::Sraw,
            ArithType::Srl => Inst::Srl,
            ArithType::Srlw => Inst::Srlw,
            ArithType::Mulh => Inst::Mulh,
            ArithType::Mulhsu => Inst::Mulhsu,
            ArithType::Mulhu => Inst::Mulhu,
        }
    }
}

impl From<&IArithType> for Inst {
    fn from(value: &IArithType) -> Self {
        match value {
            IArithType::Addi => Inst::Addi,
            IArithType::Addiw => Inst::Addiw,
            IArithType::Slliw => Inst::Slliw,
            IArithType::Srliw => Inst::Srliw,
            IArithType::Sraiw => Inst::Sraiw,
            IArithType::Slti => Inst::Slti,
            IArithType::Sltiu => Inst::Sltiu,
            IArithType::Xori => Inst::Xori,
            IArithType::Ori => Inst::Ori,
            IArithType::Andi => Inst::Andi,
            IArithType::Slli => Inst::Slli,
            IArithType::Srli => Inst::Srli,
            IArithType::Srai => Inst::Srai,
            IArithType::Auipc => Inst::Auipc,
        }
    }
}

impl From<&LoadType> for Inst {
    fn from(value: &LoadType) -> Self {
        match value {
            LoadType::Lb => Inst::Lb,
            LoadType::Lbu => Inst::Lbu,
            LoadType::Lh => Inst::Lh,
            LoadType::Lhu => Inst::Lhu,
            LoadType::Lw => Inst::Lw,
            LoadType::Lwu => Inst::Lwu,
        }
    }
}

impl From<&StoreType> for Inst {
    fn from(value: &StoreType) -> Self {
        match value {
            StoreType::Sb => Inst::Sb,
            StoreType::Sh => Inst::Sh,
            StoreType::Sw => Inst::Sw,
        }
    }
}

impl From<&JumpLinkType> for Inst {
    fn from(value: &JumpLinkType) -> Self {
        match value {
            JumpLinkType::Jal => Inst::Jal,
        }
    }
}

impl From<&JumpLinkRType> for Inst {
    fn from(value: &JumpLinkRType) -> Self {
        match value {
            JumpLinkRType::Jalr => Inst::Jalr,
        }
    }
}

impl From<&CSRType> for Inst {
    fn from(value: &CSRType) -> Self {
        match value {
            CSRType::Csrrw => Inst::Csrrw,
            CSRType::Csrrs => Inst::Csrrs,
            CSRType::Csrrc => Inst::Csrrc,
        }
    }
}

impl From<&CSRIType> for Inst {
    fn from(value: &CSRIType) -> Self {
        match value {
            CSRIType::Csrrwi => Inst::Csrrwi,
            CSRIType::Csrrsi => Inst::Csrrsi,
            CSRIType::Csrrci => Inst::Csrrci,
        }
    }
}

impl From<&BasicType> for Inst {
    fn from(value: &BasicType) -> Self {
        match value {
            BasicType::Ecall => Inst::Ecall,
            BasicType::Ebreak => Inst::Ebreak,
            BasicType::Uret => Inst::Uret,
        }
    }
}

impl From<&BranchType> for Inst {
    fn from(value: &BranchType) -> Self {
        match value {
            BranchType::Beq => Inst::Beq,
            BranchType::Bne => Inst::Bne,
            BranchType::Blt => Inst::Blt,
            BranchType::Bge => Inst::Bge,
            BranchType::Bltu => Inst::Bltu,
            BranchType::Bgeu => Inst::Bgeu,
        }
    }
}

impl From<&UpperArithType> for Inst {
    fn from(value: &UpperArithType) -> Self {
        match value {
            UpperArithType::Lui => Inst::Lui
        }
    }
}