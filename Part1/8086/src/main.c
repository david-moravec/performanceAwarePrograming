#include <string.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>

#include "Disassemble.h"

void disassemble_binary_file(FILE* f) {
    BYTE buffer[8];

    while (true) {
        int succes = fread(&buffer, sizeof(BYTE), 2, f);

        if (!succes) {
            break;
        }

        printf("%s\n", disassemble_instruction(buffer));
    }
}

// TESTS //
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
