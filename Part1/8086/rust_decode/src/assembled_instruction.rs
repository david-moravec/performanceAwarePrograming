use crate::instruction::instruction::Operation;

enum BitUsage {
    LITERAL,
    MOD,
    REG,
    RM,
    W,
    S,
    D,
    V,
    Z,
    DATALO,
    DATAHI,
    DISPLO,
    DISPHI,
}

struct Bits {
    usage: BitUsage,
    size: u8,
    value: Option<u8>,
}

impl Bits {
    pub const fn literal(value: u8) -> Self {
        Bits {
            usage: BitUsage::LITERAL,
            value: Some(value),
            size: 8 - value.leading_zeros() as u8,
        }
    }
}

const MOD: Bits = Bits {
    usage: BitUsage::MOD,
    size: 2,
    value: None,
};

const REG: Bits = Bits {
    usage: BitUsage::REG,
    size: 3,
    value: None,
};

const RM: Bits = Bits {
    usage: BitUsage::RM,
    size: 3,
    value: None,
};

const D: Bits = Bits {
    usage: BitUsage::D,
    size: 1,
    value: None,
};

const W: Bits = Bits {
    usage: BitUsage::W,
    size: 1,
    value: None,
};

pub struct AssembledInstruction {
    operation: Operation,
    bits: [Bits; 6],
}

pub const INSTRUCTION_TABLE: [AssembledInstruction; 1] = [AssembledInstruction {
    operation: Operation::MOV,
    bits: [Bits::literal(0b100010), D, W, MOD, REG, RM],
}];
