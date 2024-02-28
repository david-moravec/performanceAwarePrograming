typedef unsigned char BYTE;
typedef unsigned char BINARY_INSTRUCTION[8];

void print_byte(BYTE byte);
const char* disassemble_instruction(const BINARY_INSTRUCTION binary_instruction);

//Tests
void test_disassemble_instruction();
void test_disassemble_0_byte();
void test_disassemble_1_byte();
void test_disassembled_instruction_to_str();
void test_nth_byte();

