use crate::instruction::instruction::Operation;

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone, Copy)]
pub struct Bits {
    usage: BitUsage,
    pub shift: u8,
    size: u8,
    value: Option<u8>,
}

impl Bits {
    pub const fn literal(value: u8, size: u8) -> Self {
        Bits {
            usage: BitUsage::LITERAL,
            value: Some(value),
            shift: 8 - size,
            size,
        }
    }
}

macro_rules! bits {
    ($val:expr; $size:expr) => {
        Bits::literal($val, $size)
    };
    ($usage:expr, $size:expr) => {
        Bits {
            usage: $usage,
            size: $size,
            shift: 0,
            value: None,
        }
    };
}

const MOD: Bits = bits!(BitUsage::MOD, 2);
const REG: Bits = bits!(BitUsage::REG, 3);
const RM: Bits = bits!(BitUsage::RM, 3);
const D: Bits = bits!(BitUsage::D, 1);
const W: Bits = bits!(BitUsage::W, 1);
const DATA_LO: Bits = bits!(BitUsage::DATALO, 8);
const DATA_HI: Bits = bits!(BitUsage::DATAHI, 8);
const DISP_LO: Bits = bits!(BitUsage::DISPLO, 8);
const DISP_HI: Bits = bits!(BitUsage::DISPHI, 8);

#[derive(Debug)]
pub struct AssembledInstruction {
    pub operation: Operation,
    pub bits: [Option<Bits>; 16],
}

impl AssembledInstruction {
    pub fn matches_byte(&self, byte: u8) -> bool {
        let literal = self
            .bits
            .into_iter()
            .filter(|bits| bits.map_or(false, |b| matches!(b.usage, BitUsage::LITERAL)))
            .next()
            .flatten();

        literal.map_or(false, |lit| {
            lit.value.map_or(false, |val| val == byte >> lit.shift)
        })
    }
}

macro_rules! INSTR {
    ($operation:expr, $($bits:expr),+) => {
        {
            let mut bits: [Option<Bits>; 16] = [None; 16];
            let mut i: usize = 0;
            let mut shift = 8;

            $(
                #[allow(unused_assignments)]
                {
                    if shift == 0 {shift = 8};
                    shift -= $bits.size;
                    $bits.shift = shift;
                    bits[i] = Some($bits);
                    i += 1;
                }
            )+

            AssembledInstruction {
                operation: $operation,
                bits: bits
            }
        }
    };
}

use crate::instruction::instruction::Operation::*;

const INSTRUCTION_TABLE: [AssembledInstruction; 3] = [
    INSTR!(
        MOV,
        bits!(0b100010; 6),
        D,
        W,
        MOD,
        REG,
        RM,
        DISP_LO,
        DISP_HI
    ),
    INSTR!(MOV, bits!(0b1011; 4), W, REG, DATA_LO, DATA_HI),
    INSTR!(
        ADD,
        bits!(0b000000; 6),
        D,
        W,
        MOD,
        REG,
        RM,
        DISP_LO,
        DISP_HI
    ),
];

#[derive(Debug)]
pub struct InstructionUndefinedError;

pub fn get_assembled_instruction(
    byte: u8,
) -> Result<AssembledInstruction, InstructionUndefinedError> {
    for instr in INSTRUCTION_TABLE {
        if instr.matches_byte(byte) {
            return Ok(instr);
        }
    }

    Err(InstructionUndefinedError)
}

#[cfg(test)]
mod tests {
    use super::get_assembled_instruction;

    #[test]
    fn test_get_assembled_instruction() {
        assert!(get_assembled_instruction(0b10001011).is_ok());
        assert!(get_assembled_instruction(0b10110000).is_ok());
        assert!(get_assembled_instruction(0b00000000).is_ok());
        assert!(get_assembled_instruction(0b10000000).is_err());
    }
}
