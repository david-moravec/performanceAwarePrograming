#include <assert.h>
#include <string.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <limits.h>

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

const char* Reg8Bits_to_str(enum Reg8Bits reg) {
    switch (reg) {
        case AL: return "al";
        case CL: return "cl";
        case DL: return "dl";
        case BL: return "bl";
        case AH: return "ah";
        case CH: return "ch";
        case DF: return "df";
        case HB: return "hb";
    }
    return "NO";
}

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

const char* Reg16Bits_to_str(enum Reg16Bits reg) {
    switch (reg) {
        case AX: return "ax";
        case CX: return "cx";
        case DX: return "dx";
        case BX: return "bx";
        case SP: return "sp";
        case BP: return "bp";
        case SI: return "si";
        case DI: return "di";
    }
    return "NO";
}

enum Byte5Mask {
    OPCODE = 0b11111100,
    D      = 0b00000010,
    W      = 0b00000001,
};

enum Byte4Mask {
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
    char opcode[4];
    char dst_reg[3];
    char src_reg[3];
} DisassembledInstruction;

const DisassembledInstruction* construct_instruction(const char opcode[4], const char dst_reg[3], const char src_reg[3]) {
     DisassembledInstruction* in =(DisassembledInstruction*) malloc(sizeof(DisassembledInstruction));

    strcpy_s(in->opcode, sizeof(in->opcode) + 1, opcode);
    strcpy_s(in->dst_reg, sizeof(in->dst_reg) + 1, dst_reg);
    strcpy_s(in->src_reg, sizeof(in->src_reg) + 1, src_reg);

    return in;
}

const char* disassamled_instruction_to_str(const DisassembledInstruction* instruction) {
    char* instruction_str;
    instruction_str = (char *)malloc(50);
    snprintf(instruction_str, 50, "%s %s, %s", instruction->opcode, instruction->dst_reg, instruction->src_reg);
    return instruction_str;
}

const BYTE nth_byte(unsigned long long value, unsigned char n) {
    unsigned int shift_by = n * 8;
    return (value >> shift_by) & 0b11111111;
}

void disassemble_0_byte(const BYTE byte, char* opcode, bool* d, bool* w) {
    enum Instruction instr = (enum Instruction) byte & OPCODE;
    strcpy_s(opcode, sizeof(opcode) + 1, instruction_to_str(instr));
    *d = byte & D;
    *w = byte & W;
}

void disassemble_1_byte(const BYTE byte, char* dst_reg, char* src_reg, bool* d, bool* w) {
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

    //print_binary_instruction(binary_instruction);

    disassemble_0_byte(binary_instruction[0], dis_instr.opcode, &d, &w);
    disassemble_1_byte(binary_instruction[1], dis_instr.dst_reg, dis_instr.src_reg, &d, &w);
    

    return disassamled_instruction_to_str(&dis_instr);
}


const char* disassemble_binary_file(FILE* f) {
    BYTE buffer[8];

    while (true) {
        int succes = fread(&buffer, sizeof(BYTE), 2, f);

        if (!succes) {
            break;
        }


        printf("%s\n", disassemble_instruction(buffer));
    }



    return "lol";
}

// TESTS //
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

void test_disassembled_instruction_to_str() {
    const char* dis_i = disassamled_instruction_to_str(construct_instruction("mov", "cx", "bx"));
    assert(!strcmp(dis_i, "mov cx, bx"));
}

void test_nth_byte() {
    unsigned short test = 0b1000100111001011;
    assert(0b10001001 == nth_byte(test, 1));
    assert(0b11001011 == nth_byte(test, 0));
}

int main(int argc, char *argv[]) {
    test_nth_byte();
    test_disassembled_instruction_to_str();
    test_disassemble_0_byte();
    test_disassemble_1_byte();
    test_disassemble_instruction();

    if (argc == 1) {
        printf("Please provide assembled binary file");
        return 1;
    }

    FILE* f;
    f = fopen(argv[1], "rb");

    if (f!=NULL) {
        printf("bits 16\n\n\n\n");
        disassemble_binary_file(f);
    } else {
        printf("Cannot read %s", argv[1]);
    }

    return 0;
}
