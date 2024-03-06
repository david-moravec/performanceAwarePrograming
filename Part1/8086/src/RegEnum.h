#ifndef BYTEMASKS_H_INCLUDED
#define BYTEMASKS_H_INCLUDED

#include <stdbool.h>

#include "Disassemble.h"

enum Reg8Bits {
    AL = 0,
    CL = 1,
    DL = 2,
    BL = 3,
    AH = 4,
    CH = 5,
    DH = 6,
    BH = 7,
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

const char* reg_to_str(BYTE reg, bool w);
const char* rm_to_str(BYTE rm, bool w, BYTE mod, BYTE data_lo, BYTE_HI data_hi);

void test_regenum_c();

#endif
