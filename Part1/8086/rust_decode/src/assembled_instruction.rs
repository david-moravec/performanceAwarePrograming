use crate::instruction::instruction::Operation;

#[derive(Clone, Copy)]
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

#[derive(Clone, Copy)]
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

macro_rules! bits {
    ($val:expr) => {
        Bits::literal($val)
    };
    ($usage:expr, $size:expr) => {
        Bits {
            usage: $usage,
            size: $size,
            value: None,
        }
    };
}

const MOD: Bits = bits!(BitUsage::MOD, 2);
const REG: Bits = bits!(BitUsage::REG, 3);
const RM: Bits = bits!(BitUsage::RM, 3);
const D: Bits = bits!(BitUsage::D, 1);
const W: Bits = bits!(BitUsage::W, 1);

pub struct AssembledInstruction {
    operation: Operation,
    bits: [Option<Bits>; 16],
}

macro_rules! INSTR {
    ($operation:expr, $($bits:expr),+) => {
        {
            let mut bits: [Option<Bits>; 16] = [None; 16];
            let mut i: usize = 0;

            $(
                bits[i] = Some($bits);
                i += 1;
            )+

            AssembledInstruction {
                operation: $operation,
                bits: bits
            }
        }
    };
}

use crate::instruction::instruction::Operation::*;

pub const INSTRUCTION_TABLE: [AssembledInstruction; 1] =
    [INSTR!(MOV, bits!(0b100010), D, W, MOD, REG, RM)];
