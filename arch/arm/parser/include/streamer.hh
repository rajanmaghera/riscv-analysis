#pragma once

#include <llvm/MC/TargetRegistry.h>
#include "llvm/ADT/StringRef.h"
#include "llvm/MC/MCDirectives.h"
#include "llvm/MC/MCStreamer.h"
#include "llvm/Support/SMLoc.h"

class DumpStreamer : public llvm::MCStreamer {
public:
	DumpStreamer(llvm::MCContext &context);

	bool emitSymbolAttribute(llvm::MCSymbol *Symbol, llvm::MCSymbolAttr Attribute) override;
	void emitCommonSymbol(llvm::MCSymbol *Symbol, uint64_t Size, llvm::Align ByteAlignment) override;
	void emitZerofill(llvm::MCSection *Section, llvm::MCSymbol *Symbol = nullptr, uint64_t Size = 0, llvm::Align ByteAlignment = llvm::Align(1), llvm::SMLoc Loc = llvm::SMLoc()) override;
	void emitInstruction(const llvm::MCInst &Inst, const llvm::MCSubtargetInfo &STI) override;
};
