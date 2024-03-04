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
    BYTE opcode;
    BYTE mod;
    BYTE reg;
    BYTE rm;

    BYTE disp_lo;
    BYTE disp_hi;
    BYTE data_lo;
    BYTE data_hi;

    bool s;
    bool w;
    bool d;
    bool v;
    bool z;

} DisassembledInstruction;

static const char* disassambled_instruction_to_str(const DisassembledInstruction* instruction) {
    char* instruction_str;
    instruction_str = (char *)malloc(50);

    const char* opcode = opcode_to_str(instruction->opcode);
    const char* source;
    const char* destination;

    if (instruction->d) {
        destination = reg_to_str(instruction->reg, instruction->w);
        source = reg_to_str(instruction->rm, instruction->w);
    } else {
        destination = reg_to_str(instruction->rm, instruction->w);
        source = reg_to_str(instruction->reg, instruction->w);

    }

    snprintf(instruction_str, 50,
             "%s %s, %s",
             opcode,
             destination,
             source
             );

    return instruction_str;
}


static void disassemble_0_byte(const BYTE byte, DisassembledInstruction* dis_instr) {
    enum Byte0Mask {
        OPCODE = 0b11111100,
        D      = 0b00000010,
        W      = 0b00000001,
    };

    dis_instr->opcode = (enum Opcode) byte & OPCODE;
    dis_instr->d = byte & D;
    dis_instr->w = byte & W;
}

static void disassemble_1_byte(const BYTE byte, DisassembledInstruction* dis_instr) {
    enum Byte1Mask {
        MOD = 0b11000000,
        REG = 0b00111000,
        RM  = 0b00000111,
    };

    dis_instr->reg = (byte & REG) >> 3;
    dis_instr->rm = byte & RM;
}

const char* disassemble_instruction(const BINARY_INSTRUCTION binary_instruction) {
    DisassembledInstruction dis_instr;
    bool d = false;
    bool w = false;

    disassemble_0_byte(binary_instruction[0], &dis_instr);
    disassemble_1_byte(binary_instruction[1], &dis_instr);

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

    disassemble_0_byte(byte, &dis_instr);

    assert(dis_instr.opcode = MOV);
    assert(!dis_instr.d);
    assert(dis_instr.w);
}

void test_disassemble_1_byte() {
    BYTE byte = test_instr[1];

    DisassembledInstruction dis_instr;
    bool d = false;
    bool w = false;

    disassemble_1_byte(byte, &dis_instr);

    assert(dis_instr.reg == BL);
    assert(dis_instr.rm == CL);
}

void test_disassemble_c() {
    test_disassemble_instruction();
    test_disassemble_0_byte();
    test_disassemble_1_byte();
}
