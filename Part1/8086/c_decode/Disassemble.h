#include <stdio.h>

typedef unsigned char BYTE;
typedef unsigned short int BYTE_HI;
typedef unsigned char BINARY_INSTRUCTION[8];

void disassemble_binary_file(FILE* f);

//Tests
void test_disassemble_c();
