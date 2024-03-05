#include "RegEnum.h"
#include <stdlib.h>

#include "Disassemble.h"

const char* Reg8Bits_to_str(enum Reg8Bits reg) {
    switch (reg) {
        case AL: return "al";
        case CL: return "cl";
        case DL: return "dl";
        case BL: return "bl";
        case AH: return "ah";
        case CH: return "ch";
        case DH: return "dh";
        case BH: return "bh";
    }
    return "NO";
}

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

const char* effective_address(int rm) {
    switch (rm) {
        case 0b000: return "bx + si";
        case 0b001: return "bx + di";
        case 0b010: return "bx + di";
        case 0b011: return "bx + di";
        case 0b100: return "si";
        case 0b101: return "di";
        case 0b110: return "DIRECT ADDRES";
        case 0b111: return "bx";
    }

    return "NON";
}


const char* rm_to_str(int rm, bool w, int mod, BYTE data_lo, BYTE data_hi) {
    char* buffer;
    buffer = (char *)malloc(20);

    const char* eff_addr = effective_address(rm);

    switch (mod) {
        case 0:
            snprintf(buffer, 20, "[%s]", eff_addr);
            break;
        case 1:
            if (data_lo) {
                snprintf(buffer, 20, "[%s + %u]", eff_addr, data_lo);
            }
            break;
        case 3:
            unsigned short int data = data_lo + data_hi;
            if (data) {
                snprintf(buffer, 20, "[%s + %u]", eff_addr, data);
            }
            break;
        case 4:
            return reg_to_str(rm, w);

        return buffer;
    }

    return "NON";
}

const char* reg_to_str(int reg, bool w) {
    return w ? Reg16Bits_to_str(reg) : Reg8Bits_to_str(reg);
}
