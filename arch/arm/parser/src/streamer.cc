#include "streamer.hh"
#include "instruction.hh"

#include <iostream>
#include <llvm/ADT/StringRef.h>
#include <llvm/MC/MCContext.h>
#include <llvm/MC/MCExpr.h>
#include <llvm/MC/MCInst.h>
#include <llvm/MC/MCRegisterInfo.h>
#include <llvm/MC/MCSymbol.h>
#include <llvm/Support/Casting.h>
#include <llvm/Support/FormatVariadic.h>
#include <llvm/Support/raw_ostream.h>
#include <llvm/Support/SourceMgr.h>
#include <ostream>
#include <sstream>
#include <vector>

DumpStreamer::DumpStreamer(llvm::MCContext &context,
                           llvm::MCInstPrinter &printer,
                           llvm::MCRegisterInfo &reg,
                           llvm::MCAsmInfo &mai,
                           llvm::SourceMgr &src_mgr)
    : MCStreamer(context), printer(printer), reg(reg), mai(mai),
      src_mgr(src_mgr) {
};

bool DumpStreamer::emitSymbolAttribute(llvm::MCSymbol *Symbol, llvm::MCSymbolAttr Attribute) {
    // Pretend that everything succeeds
    return true;
}

void DumpStreamer::emitCommonSymbol(llvm::MCSymbol *Symbol, uint64_t Size, llvm::Align ByteAlignment) {
    // Do nothing
}

void DumpStreamer::emitZerofill(llvm::MCSection *Section, llvm::MCSymbol *Symbol, uint64_t Size, llvm::Align ByteAlignment, llvm::SMLoc Loc) {
    // Do nothing
}

void DumpStreamer::emitInstruction(const llvm::MCInst &Inst, const llvm::MCSubtargetInfo &STI) {
    std::string opcode = printer.getOpcodeName(Inst.getOpcode()).str();
    std::cerr << opcode;

    // Collect operands
    std::vector<Operand*> operands;
    for (int i = 0; i < Inst.getNumOperands(); i++) {
        llvm::MCOperand operand = Inst.getOperand(i);
        if (operand.isImm()) {
            std::cerr << " " << operand.getImm();
            operands.push_back(new Integer(operand.getImm()));
        }

        if (operand.isReg()) {
            std::string out = reg.getName(operand.getReg());
            std::cerr << " " << out;
            operands.push_back(new Register(out));
        }

        if (operand.isExpr() && operand.getExpr()->getKind() == llvm::MCExpr::SymbolRef) {
            const llvm::MCSymbolRefExpr *sre = llvm::cast<llvm::MCSymbolRefExpr>(operand.getExpr());
            const llvm::MCSymbol &sym = sre->getSymbol();

            std::string out;
            llvm::raw_string_ostream os(out);
            sym.print(os, &mai);

            std::cerr << " " << out;
            operands.push_back(new Label(out));
        }
    }
    std::cerr << std::endl;

    // Get the source location
    auto lc = src_mgr.getLineAndColumn(Inst.getLoc());
    uint line = lc.first;
    uint column = lc.second;

    // Add all labels
    Instruction inst = Instruction(opcode, current_labels, operands);
    inst.set_location(line, column);
    instructions.push(inst);
    current_labels.clear();
}

void DumpStreamer::emitLabel(llvm::MCSymbol *Symbol, llvm::SMLoc Loc) {
    std::string out;
    llvm::raw_string_ostream os(out);
    Symbol->print(os, &mai);

    std::cerr << ";; label: " << out << "\n";
    current_labels.push_back(Label(out));
}

std::string DumpStreamer::dump_instructions() {
    std::string out;
    llvm::raw_string_ostream os(out);
    // os << llvm::formatv("{0}", instructions.to_json());
    os << llvm::formatv("{0:2}", instructions.to_json());
    return out;
}
