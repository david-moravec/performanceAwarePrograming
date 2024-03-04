#include <limits.h>
#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <stdbool.h>

#include "RegEnum.h"

typedef unsigned char BYTE;
typedef unsigned char BINARY_INSTRUCTION[8];

// https://stackoverflow.com/questions/35926722/what-is-the-format-specifier-for-binary-in-c
void print_byte(BYTE byte)
{
    int i = CHAR_BIT; /* however many bits are in a byte on your platform */
    while(i--) {
        putchar('0' + ((byte >> i) & 1)); /* loop through and print the bits */
    }
}

enum Opcode {
    MOV = 0b10001000,
};

static const char* opcode_to_str(enum Opcode instr) {
    switch (instr) {
        case MOV: return "mov";
    }
    return "NON";
}

typedef struct instruction {
    char opcode[4];
    char dst_reg[3];
    char src_reg[3];
} DisassembledInstruction;

static const DisassembledInstruction* construct_instruction(
    const char opcode[4],
    const char dst_reg[3],
    const char src_reg[3]
) {
    DisassembledInstruction* in =(DisassembledInstruction*) malloc(sizeof(DisassembledInstruction));

    strcpy_s(in->opcode, sizeof(in->opcode) + 1, opcode);
    strcpy_s(in->dst_reg, sizeof(in->dst_reg) + 1, dst_reg);
    strcpy_s(in->src_reg, sizeof(in->src_reg) + 1, src_reg);

    return in;
}

static const char* disassambled_instruction_to_str(const DisassembledInstruction* instruction) {
    char* instruction_str;
    instruction_str = (char *)malloc(50);
    snprintf(instruction_str, 50,
             "%s %s, %s",
             instruction->opcode,
             instruction->dst_reg,
             instruction->src_reg
             );

    return instruction_str;
}


static void disassemble_0_byte(const BYTE byte, char* opcode, bool* d, bool* w) {
    enum Byte0Mask {
        OPCODE = 0b11111100,
        D      = 0b00000010,
        W      = 0b00000001,
    };

    enum Opcode instr = (enum Opcode) byte & OPCODE;
    strcpy_s(opcode, sizeof(opcode) + 1, opcode_to_str(instr));
    *d = byte & D;
    *w = byte & W;
}

static void disassemble_1_byte(const BYTE byte, char* dst_reg, char* src_reg, bool* d, bool* w) {
    enum Byte1Mask {
        MOD = 0b11000000,
        REG = 0b00111000,
        RM  = 0b00000111,
    };

    int reg, rm;
    reg = (byte & REG) >> 3;
    rm = byte & RM;

    const char* fst;
    const char* snd;

    if (*w) {
        fst = Reg16Bits_to_str(rm);
        snd = Reg16Bits_to_str(reg);
    } else {
        fst = Reg8Bits_to_str(rm);
        snd = Reg8Bits_to_str(reg);
    }

    strcpy_s(dst_reg, sizeof(dst_reg), fst);
    strcpy_s(src_reg, sizeof(src_reg), snd);
}

const char* disassemble_instruction(const BINARY_INSTRUCTION binary_instruction) {
    DisassembledInstruction dis_instr;
    bool d = false;
    bool w = false;

    disassemble_0_byte(binary_instruction[0], dis_instr.opcode, &d, &w);
    disassemble_1_byte(binary_instruction[1], dis_instr.dst_reg, dis_instr.src_reg, &d, &w);

    return disassambled_instruction_to_str(&dis_instr);
}

// Tests
#include <assert.h>

BINARY_INSTRUCTION test_instr = {0b10001001, 0b11011001, 0b000000000, 0b000000000, 0b00000000, 0b00000000};

void test_disassemble_instruction() {
    const char* dis_instr = disassemble_instruction(test_instr);
    bool are_different = strcmp(dis_instr, "mov cx, bx");
    assert(!are_different);
}

void test_disassemble_0_byte() {
    BYTE byte = test_instr[0];

    DisassembledInstruction dis_instr;
    bool d;
    bool w;

    disassemble_0_byte(byte, dis_instr.opcode, &d, &w);

    assert(!strcmp(dis_instr.opcode, "mov"));
    assert(!d);
    assert(w);
}

void test_disassemble_1_byte() {
    BYTE byte = test_instr[1];

    DisassembledInstruction dis_instr;
    bool d = false;
    bool w = false;

    disassemble_1_byte(byte, dis_instr.dst_reg, dis_instr.src_reg, &d, &w);

    assert(!strcmp(dis_instr.dst_reg, "cl"));
    assert(!strcmp(dis_instr.src_reg, "bl"));
}

void test_disassemble_c() {
    test_disassemble_instruction();
    test_disassemble_0_byte();
    test_disassemble_1_byte();
}
