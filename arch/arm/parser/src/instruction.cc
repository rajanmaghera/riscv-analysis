#include "instruction.hh"
#include <llvm/Support/JSON.h>

void InstructionStream::push(Instruction inst) {
    instructions.push_back(inst);
}

llvm::json::Value InstructionStream::to_json() {
    // Collect all instructions
    llvm::json::Value insts = llvm::json::Array();
    llvm::json::Array *acc = insts.getAsArray();
    for (auto i : instructions) {
        acc->push_back(i.to_json());
    }

    // Create the output object
    auto value = llvm::json::Object();
    value["instructions"] = insts;
    return value;
}

llvm::json::Value Register::to_json() {
    llvm::json::Object obj = llvm::json::Object();
    obj["type"] = "register";
    obj["value"] = value;
    return obj;
}

llvm::json::Value Integer::to_json() {
    llvm::json::Object obj = llvm::json::Object();
    obj["type"] = "integer";
    obj["value"] = value;
    return obj;
}

llvm::json::Value Label::to_json() {
    llvm::json::Object obj = llvm::json::Object();
    obj["type"] = "label";
    obj["value"] = value;
    return obj;
}

llvm::json::Value Instruction::to_json() {
    llvm::json::Object value = llvm::json::Object();
    value["opcode"] = opcode;

    // Collect the labels
    llvm::json::Value labels = llvm::json::Array();
    llvm::json::Array *l_acc = labels.getAsArray();
    for (auto i : this->labels) {
        l_acc->push_back(i.value);
    }
    value["labels"] = labels;

    // Collect the operands
    llvm::json::Value operands = llvm::json::Array();
    llvm::json::Array *o_acc = operands.getAsArray();
    for (auto i : this->operands) {
        o_acc->push_back(i->to_json());
    }
    value["operands"] = operands;

    value["line"] = line - 1;
    value["column"] = column;
    return value;
}

void Instruction::set_location(uint line, uint column) {
    this->line = line;
    this->column = column;
}
