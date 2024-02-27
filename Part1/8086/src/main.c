#include <assert.h>
#include <string.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <limits.h>

typedef unsigned char BYTE;

// https://stackoverflow.com/questions/35926722/what-is-the-format-specifier-for-binary-in-c
void print_byte(BYTE byte)
{
    int i = CHAR_BIT; /* however many bits are in a byte on your platform */
    while(i--) {
        putchar('0' + ((byte >> i) & 1)); /* loop through and print the bits */
    }
}

enum Reg8Bits {
    AL = 0,
    CL = 1,
    DL = 2,
    BL = 3,
    AH = 4,
    CH = 5,
    DF = 6,
    HB = 7,
};

enum Reg16Bits {
    AX = 0,
    CX = 1,
    DX = 2,
    BX = 3,
    SP = 4,
    BP = 5,
    SI = 6,
    DI = 7,
};

enum Byte1Mask {
    OPCODE = 0b11111100,
    D      = 0b00000010,
    W      = 0b00000001,
};

enum Byte2Mask {
    MOD = 0b11000000,
    REG = 0b00111000,
    RM  = 0b00000111,
};

enum Instruction {
    MOV = 0b10001000,
};

const char* instruction_to_str(enum Instruction instr) {
    switch (instr) {
        case MOV: return "mov";
    }
    return "NON";
}

typedef struct instruction {
    char opcode[9];
    char fst_reg[3];
    char snd_reg[3];
} DisassembledInstruction;

const DisassembledInstruction* construct_instruction(const char opcode[9], const char fst_reg[3], const char snd_reg[3]) {
     DisassembledInstruction* in =(DisassembledInstruction*) malloc(sizeof(DisassembledInstruction));

    strcpy_s(in->opcode, sizeof(in->opcode) + 1, opcode);
    strcpy_s(in->fst_reg, sizeof(in->fst_reg) + 1, fst_reg);
    strcpy_s(in->snd_reg, sizeof(in->snd_reg) + 1, snd_reg);

    return in;
}

const char* disassamled_instruction_to_str(const DisassembledInstruction* instruction) {
    char* instruction_str;
    instruction_str = (char *)malloc(50);
    snprintf(instruction_str, 50, "%s %s, %s", instruction->opcode, instruction->fst_reg, instruction->snd_reg);
    return instruction_str;
}

const BYTE nth_byte(unsigned short value, unsigned char n) {
    unsigned int shift_by = n * 8;
    return (value >> shift_by) & 0b11111111;
}

void disassemble_0_byte(const BYTE byte, char* opcode, bool* d, bool* w) {
    enum Instruction instr = (enum Instruction) byte & OPCODE;
    strcpy_s(opcode, sizeof(opcode) + 1, instruction_to_str(instr));
    *d = byte & D;
    *w = byte & W;
}

void disassemble_1_byte(const BYTE byte, char* fst_reg, char* snd_reg, bool* d, bool* w) {
    //char* opcode = byte_to_opcode(byte & OPCODE);
    //*d = byte & D;
    //*w = byte & w;
}

const char* disassemble(unsigned short binary_instruction) {
    //assumed to be 2 bytes long
    //
    char opcode[9];
    char fst_reg[3];
    char snd_reg[3];
    bool d = false;
    bool w = false;

    disassemble_0_byte(nth_byte(binary_instruction, 0), opcode, &d, &w);
    //disassemble_1_byte(nth_byte(binary_instruction, 1), fst_reg, snd_reg, &d, &w);
    

    return "mov cx, bx";
}

// TESTS //

void test_disassemble() {
    bool are_different = strcmp(disassemble(0x89D9), "mov cx, bx");
    assert(!are_different);
}

void test_disassemble_0_byte() {
    BYTE byte = 0b10001010;

    DisassembledInstruction dis_instr;
    bool d;
    bool w;

    disassemble_0_byte(byte, dis_instr.opcode, &d, &w);

    assert(!strcmp(dis_instr.opcode, "mov"));
    assert(d);
    assert(!w);
}

void test_disassembled_instruction_to_str() {
    const char* dis_i = disassamled_instruction_to_str(construct_instruction("mov", "cx", "bx"));
    assert(!strcmp(dis_i, "mov cx, bx"));
}

void test_nth_byte() {
    assert(0b11001100 == nth_byte(0b1100110011010101, 1));
    assert(0b11010101 == nth_byte(0b1100110011010101, 0));
}

int main(int argc, char *argv[]) {
    test_disassemble();
    test_nth_byte();
    test_disassembled_instruction_to_str();
    test_disassemble_0_byte();
    return 0;
}
