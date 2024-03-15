use std::fmt::Display;

use bitflags::bitflags;

#[derive(Debug, Clone, Copy)]
pub enum BitUsage {
    LITERAL,
    MOD,
    REG,
    RM,
    Flag(BitFlag),
    Data(BitOrder),
    Disp(BitOrder),

    PLACEHOLDER,
}

#[derive(Debug, Clone, Copy)]
pub enum BitOrder {
    LOW,
    HIGH,
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct BitFlag: u8 {
        const W = 0b00001;
        const S = 0b00010;
        const D = 0b00100;
        const V = 0b01000;
        const Z = 0b10000;
        const NOTHING = 0b00000;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Bits {
    pub usage: BitUsage,
    size: u8,
    value: Option<u8>,
    pub shift: Option<u8>,
}

impl Bits {
    pub const fn literal(value: u8, size: u8) -> Self {
        Bits {
            usage: BitUsage::LITERAL,
            value: Some(value),
            shift: Some(8 - size),
            size,
        }
    }

    pub fn decode_value(&self, byte: u8) -> u8 {
        byte >> self.shift.expect("Every bits need shift specified")
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Byte {
    pub bits: [Option<Bits>; 8],
}

macro_rules! bits {
    ($val:expr; $size:expr) => {
        Bits::literal($val, $size)
    };
    (-f $bit_flag:expr) => {
        bits!(BitUsage::Flag($bit_flag), 1)
    };
    (-data $bit_order:expr) => {
        bits!(BitUsage::Data($bit_order), 8)
    };
    (-disp $bit_order:expr) => {
        bits!(BitUsage::Disp($bit_order), 8)
    };
    ($usage:expr, $size:expr) => {
        Bits {
            usage: $usage,
            size: $size,
            shift: None,
            value: None,
        }
    };
}

const MOD: Bits = bits!(BitUsage::MOD, 2);
const REG: Bits = bits!(BitUsage::REG, 3);
const RM: Bits = bits!(BitUsage::RM, 3);
const D: Bits = bits!(-f BitFlag::D);
const W: Bits = bits!(-f BitFlag::W);
const DATA_LO: Bits = bits!(-data BitOrder::LOW);
const DATA_HI: Bits = bits!(-data BitOrder::HIGH);
const DISP_LO: Bits = bits!(-disp BitOrder::LOW);
const DISP_HI: Bits = bits!(-disp BitOrder::HIGH);

#[derive(Debug)]
pub enum AssembledInstructionLookupError {
    IncompleteDefinitionError,
    LiteralMissingError,
    InstructionUndefinedError,
}

type InstuctionLookupResult<T> = Result<T, AssembledInstructionLookupError>;

#[derive(Debug, Clone, Copy)]
pub struct AssembledInstruction {
    pub operation: Operation,
    pub bytes: [Option<Byte>; 6],
}

impl AssembledInstruction {
    pub fn literal_in(&self, byte: u8) -> InstuctionLookupResult<bool> {
        let literal = self.bytes[0]
            .ok_or(AssembledInstructionLookupError::IncompleteDefinitionError)?
            .bits[0]
            .ok_or(AssembledInstructionLookupError::IncompleteDefinitionError)?;

        let _ = matches!(literal.usage, BitUsage::LITERAL)
            .then(|| ())
            .ok_or(AssembledInstructionLookupError::LiteralMissingError)?;

        Ok(literal.value.expect("Literal has to have a value")
            == byte >> literal.shift.expect("Should not Fail"))
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
            let mut shift = 8;

            while i < $byte.len() {
                let mut bits_cp: Bits = $byte[i].clone();
                shift -= bits_cp.size;
                if let None = bits_cp.shift {
                    bits_cp.shift = Some(shift);
                }

                bits[i] = Some(bits_cp);
                i += 1;
            }

            Some(Byte {bits})
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Operation {
    MOV,
    ADD,
}

use std::fmt;

impl fmt::Display for Operation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let str_repr = match self {
            MOV => "mov",
            ADD => "add",
        };

        write!(f, "{}", str_repr)
    }
}

use Operation::*;

lazy_static! {
    static ref INSTRUCTION_TABLE: [AssembledInstruction; 3] = [
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
}

pub fn get_assembled_instruction(byte: u8) -> InstuctionLookupResult<AssembledInstruction> {
    for instr in INSTRUCTION_TABLE.iter() {
        if instr.literal_in(byte)? {
            return Ok(instr.clone());
        }
    }

    Err(AssembledInstructionLookupError::InstructionUndefinedError)
}

#[cfg(test)]
mod tests {
    use super::{get_assembled_instruction, INSTRUCTION_TABLE};

    #[test]
    fn test_get_assembled_instruction() {
        assert!(get_assembled_instruction(0b10001011).is_ok());
        assert!(get_assembled_instruction(0b10110000).is_ok());
        assert!(get_assembled_instruction(0b00000000).is_ok());
        assert!(get_assembled_instruction(0b10000000).is_err());
    }

    #[test]
    fn test_shift() {
        assert_eq!(
            INSTRUCTION_TABLE[0].bytes[0].unwrap().bits[1]
                .unwrap()
                .shift,
            Some(1)
        );
        assert_eq!(
            INSTRUCTION_TABLE[0].bytes[1].unwrap().bits[0]
                .unwrap()
                .shift,
            Some(6)
        );
        assert_eq!(
            INSTRUCTION_TABLE[1].bytes[0].unwrap().bits[1]
                .unwrap()
                .shift,
            Some(3)
        );
    }
}
