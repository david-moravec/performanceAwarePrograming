enum OperandType {
    REGISTER,
    MEMORY,
    IMMEDIATE,
}

pub struct Operand {
    operand_type: OperandType,
    value: u8,
}

enum Register {
    A,
    B,
    C,
    D,
    SP,
    BP,
    SI,
    DI,
}
