#include "streamer.hh"

#include <iostream>
#include <llvm/ADT/StringRef.h>
#include <llvm/MC/MCContext.h>
#include <llvm/MC/MCExpr.h>
#include <llvm/MC/MCInst.h>
#include <llvm/MC/MCRegisterInfo.h>
#include <llvm/MC/MCSymbol.h>
#include <llvm/Support/Casting.h>
#include <llvm/Support/raw_ostream.h>
#include <sstream>

DumpStreamer::DumpStreamer(llvm::MCContext &context,
                           llvm::MCInstPrinter &printer,
                           llvm::MCRegisterInfo &reg,
                           llvm::MCAsmInfo &mai)
    : MCStreamer(context), printer(printer), reg(reg), mai(mai) {
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
    std::cout << opcode;

    for (int i = 0; i < Inst.getNumOperands(); i++) {
        llvm::MCOperand operand = Inst.getOperand(i);
        if (operand.isImm()) {
            std::cout << " " << operand.getImm();
        }

        if (operand.isReg()) {
            std::string out = reg.getName(operand.getReg());
            std::cout << " " << out;
        }

        if (operand.isExpr() && operand.getExpr()->getKind() == llvm::MCExpr::SymbolRef) {
            const llvm::MCSymbolRefExpr *sre = llvm::cast<llvm::MCSymbolRefExpr>(operand.getExpr());
            const llvm::MCSymbol &sym = sre->getSymbol();

            std::string out;
            llvm::raw_string_ostream os(out);
            sym.print(os, &mai);

            std::cout << " " << out;
        }
    }

    std::cout << std::endl;
}

void DumpStreamer::emitLabel(llvm::MCSymbol *Symbol, llvm::SMLoc Loc) {
    std::string out;
    llvm::raw_string_ostream os(out);
    Symbol->print(os, &mai);

    std::cout << ";; label: " << out << "\n";
}
