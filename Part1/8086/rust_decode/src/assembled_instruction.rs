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

    PLACEHOLDER,
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

#[derive(Debug, Clone, Copy)]
pub struct Byte {
    bits: [Option<Bits>; 8],
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
    pub bytes: [Option<Byte>; 6],
}

#[derive(Debug)]
pub enum DisassemblyError {
    IncompleteInstructionDefinitionError,
    LiteralNotFoundError,
    InstructionUndefinedError,
}

type DisassemblyResult<T> = Result<T, DisassemblyError>;

impl AssembledInstruction {
    pub fn literal_in(&self, byte: u8) -> DisassemblyResult<bool> {
        let literal = self.bytes[0]
            .ok_or(DisassemblyError::IncompleteInstructionDefinitionError)?
            .bits[0]
            .ok_or(DisassemblyError::IncompleteInstructionDefinitionError)?;

        Ok(literal.value.expect("Literal has to have a value") == byte >> literal.shift)
    }
}

macro_rules! INSTR {
    ($operation:expr, $($bytes:expr),+) => {
        {

            let mut bytes: [Option<Byte>; 6] = [None; 6];
            let mut i: usize = 0;


            $(
                #[allow(unused_assignments)]
                {
                    bytes[i] = INSTR!(@explode_byte $bytes);
                    i += 1;
                }
            )+

            AssembledInstruction {
                operation: $operation,
                bytes: bytes
            }
        }
    };
    (@explode_byte $byte:expr) => {
        {
            let mut bits: [Option<Bits>; 8] = [None; 8];
            let mut i: usize = 0;

            let n_bits = $byte.len();

            while i < n_bits {
                bits[i] = Some($byte[i]);
                i += 1;
            }

            Some(Byte {bits})
        }
    }
}

use crate::instruction::instruction::Operation::*;

const INSTRUCTION_TABLE: [AssembledInstruction; 3] = [
    INSTR!(MOV, [Bits::literal(0b100010, 6), D, W], [MOD, REG, RM]),
    INSTR!(
        MOV,
        [Bits::literal(0b1011, 4), W, REG],
        [DATA_LO],
        [DATA_HI]
    ),
    INSTR!(
        ADD,
        [Bits::literal(0b000000, 6), D, W],
        [MOD, REG, RM],
        [DISP_LO],
        [DISP_HI]
    ),
];

pub fn get_assembled_instruction(byte: u8) -> DisassemblyResult<AssembledInstruction> {
    for instr in INSTRUCTION_TABLE {
        if instr.literal_in(byte)? {
            return Ok(instr);
        }
    }

    Err(DisassemblyError::InstructionUndefinedError)
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
