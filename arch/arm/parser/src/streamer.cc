#include "streamer.hh"
#include <iostream>
#include <llvm/MC/MCContext.h>
#include <llvm/MC/MCInst.h>

DumpStreamer::DumpStreamer(llvm::MCContext &context) : MCStreamer(context) {
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
    std::cout << "Instruction:\n";
    std::cout << Inst.getOpcode() << "\n";

    int n_args = Inst.getNumOperands();
    for (int i = 0; i < n_args; i++) {
        llvm::MCOperand operand = Inst.getOperand(i);
        if (operand.isImm()) {
            std::cout << "  " << i << " imm: " << operand.getImm() << "\n";
        }

        if (operand.isReg()) {
            std::cout << "  " << i << " reg: " << operand.getReg() << "\n";
        }


        // FIXME: Labels are missing?
    }
}
