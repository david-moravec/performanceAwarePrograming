#include "RegEnum.h"

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

const char* reg_to_str(int reg, bool w) {
    return w ? Reg16Bits_to_str(reg) : Reg8Bits_to_str(reg);
}
