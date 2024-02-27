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

void test_disassembled_instruction_to_str() {
    const char* dis_i = disassamled_instruction_to_str(construct_instruction("mov", "cx", "bx"));
    assert(!strcmp(dis_i, "mov cx, bx"));
}

const char* disassemble_1_byte(const BYTE byte, bool* d, bool* w) {
    //char* opcode = byte_to_opcode(byte & OPCODE);
    //*d = byte & D;
    //*w = byte & w;

    return "lol";
}

enum Byte2Mask {
    MOD = 0b11000000,
    REG = 0b00111000,
    RM  = 0b00000111,
};

enum Instructions {
    MOV = 0b100010,
};

const BYTE nth_byte(unsigned short value, unsigned char n) {
    unsigned int shift_by = n * 8;
    return (value >> shift_by) & 0b11111111;
}

void test_nth_byte() {
    assert(0b11001100 == nth_byte(0b1100110011010101, 1));
    assert(0b11010101 == nth_byte(0b1100110011010101, 0));
}


const char* disassemble(unsigned short binary_instruction) {
    return "mov cx, bx";
}

int main(int argc, char *argv[]) {
    bool are_different = strcmp(disassemble(0x89D9), "mov cx, bx");
    assert(!are_different);
    test_nth_byte();
    test_disassembled_instruction_to_str();
    return 0;
}
