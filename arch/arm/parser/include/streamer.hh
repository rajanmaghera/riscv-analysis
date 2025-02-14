#pragma once

#include "instruction.hh"
#include <llvm/MC/MCAsmInfo.h>
#include <llvm/MC/MCInstPrinter.h>
#include <llvm/MC/MCRegisterInfo.h>
#include <llvm/MC/TargetRegistry.h>
#include <vector>
#include "llvm/ADT/StringRef.h"
#include "llvm/MC/MCDirectives.h"
#include "llvm/MC/MCStreamer.h"
#include "llvm/Support/SMLoc.h"

class DumpStreamer : public llvm::MCStreamer {
public:
	DumpStreamer(llvm::MCContext &context,
				 llvm::MCInstPrinter &printer,
				 llvm::MCRegisterInfo &reg,
				 llvm::MCAsmInfo &mai);

	bool emitSymbolAttribute(llvm::MCSymbol *Symbol, llvm::MCSymbolAttr Attribute) override;
	void emitCommonSymbol(llvm::MCSymbol *Symbol, uint64_t Size, llvm::Align ByteAlignment) override;
	void emitZerofill(llvm::MCSection *Section, llvm::MCSymbol *Symbol = nullptr, uint64_t Size = 0, llvm::Align ByteAlignment = llvm::Align(1), llvm::SMLoc Loc = llvm::SMLoc()) override;
	void emitInstruction(const llvm::MCInst &Inst, const llvm::MCSubtargetInfo &STI) override;
	void emitLabel(llvm::MCSymbol *Symbol, llvm::SMLoc Loc = llvm::SMLoc()) override;

	std::string dump_instructions();

private:
	llvm::MCInstPrinter &printer;
	llvm::MCRegisterInfo &reg;
	llvm::MCAsmInfo &mai;

	InstructionStream instructions;
	std::vector<Label> current_labels;
};
