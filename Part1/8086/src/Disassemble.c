#include <limits.h>
#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <stdbool.h>

#include "RegEnum.h"

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
    MOV           = 0b10001000,
    MOV_IMMEDIATE = 0b10110000,
};


static const char* opcode_to_str(enum Opcode instr) {
    switch (instr) {
        case MOV: return "mov";
        case MOV_IMMEDIATE: return "mov";
    }
    return "NON";
}

const bool opcode_is_valid(int opcode) {
    return strcmp(opcode_to_str(opcode), "NON");
}

typedef struct instruction {
    BYTE opcode;
    BYTE mod;
    BYTE reg;
    BYTE rm;

    short disp_lo;
    short disp_hi;
    short data_lo;
    short data_hi;

    bool s;
    bool w;
    bool d;
    bool v;
    bool z;

} DisassembledInstruction;

static const char* disassambled_instruction_to_str(const DisassembledInstruction* instruction) {
    char* instruction_str;
    instruction_str = (char *)malloc(50);

    BYTE reg = instruction->reg;
    BYTE rm = instruction->rm;
    BYTE mod = instruction->mod;
    short data_lo = instruction->data_lo;
    short data_hi = instruction->data_hi;
    short disp_lo = instruction->disp_lo;
    short disp_hi = instruction->disp_hi;
    bool w = instruction->w;
    bool d = instruction->d;

    const char* opcode = opcode_to_str(instruction->opcode);
    const char* source;
    const char* destination;

    switch (instruction->opcode) {
        case MOV:
            if (d) {
              destination = reg_to_str(reg, w);
              source = rm_to_str(rm, w, mod, disp_lo, disp_hi);
            } else {
              destination = rm_to_str(rm, w, mod, disp_lo, disp_hi);
              source = reg_to_str(reg, w);
            };
            break;
        case MOV_IMMEDIATE:
            destination = reg_to_str(reg, w);
            unsigned short int source = w ? data_lo + (data_hi) : data_lo;
            snprintf(instruction_str, 50,
                     "%s %s, %i",
                     opcode,
                     destination,
                     source
                     );

            return instruction_str;
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
    enum Byte0MaskOpcode4{
        OPCODE4 = 0b11110000,
        W4      = 0b00001000,
        REG4    = 0b00000111,
    };

    enum Byte0MaskOpcode6{
        OPCODE6 = 0b11111100,
        D6      = 0b00000010,
        W6      = 0b00000001,
    };


    int opcode = (enum Opcode) byte & OPCODE4;

    if (opcode_is_valid(opcode)) {
        dis_instr->opcode = opcode;
        dis_instr->reg = byte & REG4;
        dis_instr->w = byte & W4;
    } else {
        int opcode = (enum Opcode) byte & OPCODE6;
        dis_instr->opcode = opcode;
        dis_instr->d = byte & D6;
        dis_instr->w = byte & W6;
    }

}

static void disassemble_1_byte(const BYTE byte, DisassembledInstruction* dis_instr) {
    enum Byte1Mask {
        MOD = 0b11000000,
        REG = 0b00111000,
        RM  = 0b00000111,
    };

    if (dis_instr->opcode == MOV_IMMEDIATE){
        dis_instr->data_lo = byte;
        return;
    }

    dis_instr->mod = (byte & MOD) >> 6;
    dis_instr->reg = (byte & REG) >> 3;
    dis_instr->rm = byte & RM;
}

BYTE_HI high_byte(const BYTE byte) {
        return byte << 8;
}

void disassemble_rest_of_bytes(const BINARY_INSTRUCTION binary_instruction, DisassembledInstruction* dis_instr) {
    switch ((*dis_instr).opcode) {
        case MOV:
            dis_instr->disp_lo = binary_instruction[0];
            if (dis_instr->mod == 2) {
                dis_instr->disp_hi =high_byte(binary_instruction[1]);
            }
            break;
        case MOV_IMMEDIATE:
            dis_instr->data_lo = binary_instruction[0];
            if (dis_instr->w) {
                dis_instr->data_hi = high_byte(binary_instruction[1]);
            }
            break;
    }
}

void disassemble_binary_file(FILE* f) {
    BYTE buffer[8];

    while (true) {
        int succes = fread(&buffer, sizeof(BYTE), 1, f);

        if (!succes) {
            break;
        }

        DisassembledInstruction dis_instr = {0,0,0,0,0,0,0,0,0,0,0};

        // we need to disassemble first byte to know how many more bytes to disassemble
        disassemble_0_byte(buffer[0], &dis_instr);

        int bytes_to_read = 0;

        switch (dis_instr.opcode) {
            case MOV:
                int succes = fread(&buffer, sizeof(BYTE), 1, f);

                if (!succes) {
                    printf("\nError: Unexpected EOF\n");
                    return;
                }

                disassemble_1_byte(buffer[0], &dis_instr);

                switch (dis_instr.mod) {
                    case 1:
                        bytes_to_read = 1;
                        break;
                    case 2:
                        bytes_to_read = 2;
                        break;
                    case 3:
                        bytes_to_read = 0;
                        break;
                }
                break;
            case MOV_IMMEDIATE:
                bytes_to_read = dis_instr.w ? 2 : 1;
                break;
        }

        if (bytes_to_read) {
            succes = fread(&buffer, sizeof(BYTE), bytes_to_read, f);

            if (!succes) {
                printf("\nError: Unexpected EOF\n");
                return;
            }

            disassemble_rest_of_bytes(buffer, &dis_instr);
        }

        // if (opcode_is_valid(dis_instr.opcode)) {
        //     printf("%s\n", disassambled_instruction_to_str(&dis_instr));
        // }
        printf("%s\n", disassambled_instruction_to_str(&dis_instr));
    }
}


// Tests
#include <assert.h>

BYTE byte_reg_reg = 0b10001001;
BINARY_INSTRUCTION test_instr_reg_reg = {0b11011001, 0b000000000, 0b000000000, 0b00000000, 0b00000000};
BYTE byte_reg_imm = 0b10110011;
BINARY_INSTRUCTION test_instr_imm = {0b00000011, 0b000000001, 0b000000000, 0b00000000, 0b00000000};

void test_disassemble_0_byte() {
    DisassembledInstruction dis_instr;
    bool d;
    bool w;

    disassemble_0_byte(byte_reg_reg, &dis_instr);

    assert(dis_instr.opcode = MOV);
    assert(!dis_instr.d);
    assert(dis_instr.w);

    disassemble_0_byte(byte_reg_imm, &dis_instr);

    assert(dis_instr.opcode = MOV_IMMEDIATE);
    assert(dis_instr.reg = BL);
    assert(!dis_instr.w);
}

void test_disassemble_reg_reg() {
    DisassembledInstruction dis_instr_reg_reg;

    disassemble_0_byte(byte_reg_reg, &dis_instr_reg_reg);
    disassemble_1_byte(test_instr_reg_reg[0], &dis_instr_reg_reg);

    const char* dis_instr_str = disassambled_instruction_to_str(&dis_instr_reg_reg);

    assert(!strcmp(dis_instr_str, "mov cx, bx"));
}

void test_disassemble_reg_immediate() {
    DisassembledInstruction dis_instr_imm;

    disassemble_0_byte(byte_reg_imm, &dis_instr_imm);
    disassemble_rest_of_bytes((test_instr_imm), &dis_instr_imm);

    const char* dis_instr_imm_str = disassambled_instruction_to_str(&dis_instr_imm);

    assert(!strcmp(dis_instr_imm_str, "mov bl, 3"));

    dis_instr_imm.w = true;
    disassemble_rest_of_bytes((test_instr_imm), &dis_instr_imm);

    dis_instr_imm_str = disassambled_instruction_to_str(&dis_instr_imm);

    assert(!strcmp(dis_instr_imm_str, "mov bx, 259"));
}

void test_disassemle_signed_address_to_reg() {
    BINARY_INSTRUCTION instr = {0b10001001, 0b10001100, 0b11010100, 0b11111110};
    DisassembledInstruction dis_instr = {0,0,0,0,0,0,0,0,0,0,0};

    disassemble_0_byte(instr[0], &dis_instr);
    disassemble_1_byte(instr[1], &dis_instr);

    dis_instr.disp_lo = instr[2];
    dis_instr.disp_hi = instr[3] << 8;

    const char* to_test = disassambled_instruction_to_str(&dis_instr);

    assert(!strcmp(to_test, "mov [si - 300], cx"));
}

void test_disassemble_1_byte() {
    BYTE byte = test_instr_reg_reg[0];

    DisassembledInstruction dis_instr = {MOV, 0b11, 0,0,0,0,0,0,0,0,0};

    disassemble_1_byte(byte, &dis_instr);

    assert(dis_instr.reg == BL);
    assert(dis_instr.rm == CL);
}

void test_opcode_is_valid() {
    assert(opcode_is_valid(MOV));
    assert(!opcode_is_valid(254));
}

void test_disassemble_c() {
    // test_opcode_is_valid();
    // test_disassemble_reg_immediate();
    // test_disassemble_reg_reg();
    test_disassemle_signed_address_to_reg();
    // test_disassemble_0_byte();
    // test_disassemble_1_byte();
}
