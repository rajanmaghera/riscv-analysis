#include "llvm/Support/InitLLVM.h"
#include "llvm/Support/TargetSelect.h"
#include "llvm/MC/TargetRegistry.h"

#include <iostream>
#include <llvm/ADT/StringRef.h>
#include <llvm/MC/MCAsmMacro.h>
#include <llvm/MC/MCInst.h>
#include <llvm/MC/MCSection.h>
#include <llvm/Support/SMLoc.h>
#include <llvm/Support/raw_ostream.h>
#include <llvm/TargetParser/SubtargetFeature.h>
#include "llvm/Support/CommandLine.h"
#include "llvm/MC/MCTargetOptionsCommandFlags.h"
#include "llvm/MC/MCTargetOptions.h"
#include "llvm/MC/MCAsmBackend.h"
#include "llvm/MC/MCAsmInfo.h"
#include "llvm/MC/MCCodeEmitter.h"
#include "llvm/MC/MCContext.h"
#include "llvm/MC/MCInstPrinter.h"
#include "llvm/MC/MCInstrInfo.h"
#include "llvm/MC/MCObjectFileInfo.h"
#include "llvm/MC/MCObjectWriter.h"
#include "llvm/MC/MCParser/AsmLexer.h"
#include "llvm/MC/MCParser/MCTargetAsmParser.h"
#include "llvm/MC/MCRegisterInfo.h"
#include "llvm/MC/MCStreamer.h"
#include "llvm/MC/MCSubtargetInfo.h"
#include "llvm/MC/MCTargetOptionsCommandFlags.h"
#include "llvm/MC/TargetRegistry.h"
#include "llvm/Support/CommandLine.h"
#include "llvm/Support/Compression.h"
#include "llvm/Support/FileUtilities.h"
#include "llvm/Support/FormattedStream.h"
#include "llvm/Support/InitLLVM.h"
#include "llvm/Support/MemoryBuffer.h"
#include "llvm/Support/SourceMgr.h"
#include "llvm/Support/TargetSelect.h"
#include "llvm/Support/ToolOutputFile.h"
#include "llvm/Support/WithColor.h"
#include "llvm/TargetParser/Host.h"

static llvm::cl::OptionCategory MCCategory("MC Options");
static llvm::cl::opt<std::string> InputFilename(llvm::cl::Positional,
                                                llvm::cl::desc("<input file>"),
                                                llvm::cl::init("-"),
                                                llvm::cl::cat(MCCategory));

int main(int argc, char **argv) {
    std::cout << "This is a test\n";

    llvm::InitLLVM X(argc, argv);

    // Initialize targets and assembly printers/parsers.
    llvm::InitializeAllTargetInfos();
    llvm::InitializeAllTargetMCs();
    llvm::InitializeAllAsmParsers();
    llvm::InitializeAllDisassemblers();

    llvm::ErrorOr<std::unique_ptr<llvm::MemoryBuffer>> buffer_ptr =
      llvm::MemoryBuffer::getFileOrSTDIN(InputFilename, /*IsText=*/true);
    llvm::MemoryBuffer *buffer = buffer_ptr->get();

    // Add the source
    llvm::SourceMgr src_mgr;
    src_mgr.AddNewSourceBuffer(std::move(*buffer_ptr), llvm::SMLoc());

    // Setup target info
    llvm::Triple target_triple;
    std::string triple_name = "aarch64-unknown-linux";
    std::string error;
    llvm::StringRef arch_name = "aarch64";
    const llvm::Target *target
        = llvm::TargetRegistry::lookupTarget(arch_name, target_triple, error);

    llvm::MCTargetOptions MCOptions = llvm::mc::InitMCTargetOptionsFromFlags();
    MCOptions.AsmVerbose = true;

    llvm::MCRegisterInfo *MRI = target->createMCRegInfo(triple_name);
    assert(MRI && "Unable to create target register info");

    llvm::MCAsmInfo *MAI = target->createMCAsmInfo(*MRI, triple_name, MCOptions);
    assert(MAI && "Unable to create target asm info");
    MAI->setPreserveAsmComments(true);

    llvm::MCSubtargetInfo *STI = target->createMCSubtargetInfo(triple_name, "", "");
    assert(STI && "Unable to create subtarget info!");

    llvm::MCContext Ctx(target_triple, MAI, MRI, STI, &src_mgr,
                  &MCOptions);

    std::unique_ptr<llvm::MCInstrInfo> MCII(target->createMCInstrInfo());

    std::unique_ptr<llvm::MCStreamer> Str;
    llvm::MCInstPrinter *IP = target->createMCInstPrinter(target_triple, 0, *MAI, *MCII, *MRI);
    llvm::raw_ostream *output = &llvm::outs();

    // Setup the streamer
    std::unique_ptr<llvm::MCCodeEmitter> CE;
    CE.reset(target->createMCCodeEmitter(*MCII, Ctx));
    std::unique_ptr<llvm::MCAsmBackend> MAB(
        target->createMCAsmBackend(*STI, *MRI, MCOptions));
    auto FOut = std::make_unique<llvm::formatted_raw_ostream>(*output);
    Str.reset(target->createAsmStreamer(Ctx, std::move(FOut), IP,
                                           std::move(CE), std::move(MAB)));

    // Assemble the input
    std::unique_ptr<llvm::MCAsmParser> parser(llvm::createMCAsmParser(src_mgr, Ctx, *Str, *MAI));
    std::unique_ptr<llvm::MCTargetAsmParser> TAP(
        target->createMCAsmParser(*STI, *parser, *MCII, MCOptions));

    if (!TAP) {
        llvm::errs() << "This target doesn't support assembly parsing\n";
        return 1;
    }

    parser->setTargetParser(*TAP);
    bool result = parser->Run(true);

    std::cout << "What could go wrong\n";
    return 0;
}
