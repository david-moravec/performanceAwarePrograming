#include "RegEnum.h"
#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

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
        case 0b010: return "bp + di";
        case 0b011: return "bp + di";
        case 0b100: return "si";
        case 0b101: return "di";
        case 0b110: return "DIRECT ADDRES";
        case 0b111: return "bx";
    }

    return "NON";
}

const char* displacement_effective_address(const char* eff_addr, unsigned short int displacement) {
    char* buffer;
    buffer = (char *)malloc(20);

    if (displacement) {
        snprintf(buffer, 20, "[%s + %u]", eff_addr, displacement);
    } else {
        snprintf(buffer, 20, "[%s]", eff_addr);
    }

    return buffer;
}


const char* rm_to_str(const BYTE rm, const bool w, const BYTE mod, const BYTE data_lo, const BYTE data_hi) {
    char* buffer;
    buffer = (char *)malloc(20);

    const char* eff_addr = effective_address(rm);

    switch (mod) {
        case 0:
            return displacement_effective_address(eff_addr, 0);
        case 1:
            return displacement_effective_address(eff_addr, data_lo);
        case 2:
            return displacement_effective_address(eff_addr, data_lo + data_hi);
        case 3:
            return reg_to_str(rm, w);

        return buffer;
    }

    return "NON";
}

const char* reg_to_str(BYTE reg, bool w) {
    return w ? Reg16Bits_to_str(reg) : Reg8Bits_to_str(reg);
}

void test_displacement_effective_address() {
    const char* eff_addr = effective_address(0b011); // bp + di
    
    const char* to_test = displacement_effective_address(eff_addr, 0);
    assert(!strcmp(to_test, "[bp + di]"));

    to_test = displacement_effective_address(eff_addr, 4);
    assert(!strcmp(to_test, "[bp + di + 4]"));

    to_test = displacement_effective_address(eff_addr, 763);
    assert(!strcmp(to_test, "[bp + di + 763]"));
}

void test_rm_to_str() {
    BYTE rm = 0b101;
    bool w = false;
    BYTE data_lo = 7;
    unsigned short int data_hi = 255;

    BYTE mod = 0b00;
    const char* to_test = rm_to_str(rm, w, mod, data_lo, data_hi);
    assert(!strcmp(to_test, "[di]"));

    mod = 0b01;
    to_test = rm_to_str(rm, w, mod, data_lo, data_hi);
    assert(!strcmp(to_test, "[di + 7]"));


    mod = 0b10;
    to_test = rm_to_str(rm, w, mod, data_lo, data_hi);
    assert(!strcmp(to_test, "[di + 262]"));

    mod = 0b11;
    to_test = rm_to_str(rm, w, mod, data_lo, data_hi);
    assert(!strcmp(to_test, "ch"));
}

void test_regenum_c() {
    test_displacement_effective_address();
    test_rm_to_str();
}
