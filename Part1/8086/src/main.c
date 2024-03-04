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

int main(int argc, char *argv[]) {
    test_disassemble_c();

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
