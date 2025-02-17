#pragma once

#include <cstdint>
#include <string>
#include <vector>
#include "llvm/Support/JSON.h"

class Operand {
public:
    virtual llvm::json::Value to_json() = 0;
};

class Register : public Operand {
public:
    Register(std::string value) : value(value) {};

    virtual llvm::json::Value to_json() override;
    std::string value;
};

class Integer : public Operand {
public:
    Integer(int64_t value) : value(value) {};

    virtual llvm::json::Value to_json() override;
    int64_t value;
};

class Label : public Operand {
public:
    Label(std::string value) : value(value) {};

    virtual llvm::json::Value to_json() override;
    std::string value;
};

class Instruction {
public:
    Instruction(std::string opcode, std::vector<Label> labels, std::vector<Operand*> operands)
        : opcode(opcode), labels(labels), operands(operands) {};

    std::string opcode;
    std::vector<Label> labels;
    std::vector<Operand*> operands;
    uint line = 0;
    uint column = 0;

    llvm::json::Value to_json();
    void set_location(uint line, uint column);
};

class InstructionStream {
public:
    void push(Instruction inst);
    llvm::json::Value to_json();

private:
    std::vector<Instruction> instructions;
};
