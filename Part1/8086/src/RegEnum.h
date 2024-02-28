#ifndef BYTEMASKS_H_INCLUDED
#define BYTEMASKS_H_INCLUDED

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

const char* Reg8Bits_to_str(enum Reg8Bits reg);
const char* Reg16Bits_to_str(enum Reg16Bits reg);

#endif
