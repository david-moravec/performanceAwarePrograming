use crate::assembled_instruction::BitFlag;

#[derive(Debug)]
pub enum OperandType {
    REGISTER(Size),
    MEMORY(Displacement),
    IMMEDIATE(Size),
}

#[derive(Debug)]
enum Reg {
    A,
    B,
    C,
    D,
    SP,
    BP,
    SI,
    DI,
}

#[derive(Debug)]
pub enum Displacement {
    NO,
    YES(Size),
}

#[derive(Debug)]
pub enum Size {
    BYTE,
    WORD,
}

impl Size {
    pub fn new(flags: BitFlag) -> Self {
        if flags & BitFlag::W != BitFlag::NOTHING {
            Size::WORD
        } else {
            Size::BYTE
        }
    }
}

#[derive(Debug)]
pub enum OperandTypeError {
    UnknownModError,
    MissingSizeSpecifierError,
}

use OperandTypeError::*;

impl OperandType {
    fn try_from_mod(mode: u8, flags: BitFlag) -> Result<Self, OperandTypeError> {
        match mode {
            0b00 => Ok(OperandType::MEMORY(Displacement::NO)),
            0b01 => Ok(OperandType::MEMORY(Displacement::YES(Size::BYTE))),
            0b10 => Ok(OperandType::MEMORY(Displacement::YES(Size::WORD))),
            0b11 => Ok(OperandType::REGISTER(Size::new(flags))),
            _ => Err(UnknownModError),
        }
    }
}

#[derive(Debug)]
pub struct Operand {
    pub operand_type: Option<OperandType>,
    pub value: Option<u8>,
    pub displacement: Option<u16>,
    pub data: Option<u16>,
}

impl Operand {
    pub fn new(value: Option<u8>) -> Self {
        Operand {
            operand_type: None,
            value,
            displacement: None,
            data: None,
        }
    }

    pub fn rm(value: u8, mode: Option<u8>, flags: BitFlag) -> Result<Self, OperandTypeError> {
        let operand_type_opt: Option<OperandType>;

        match mode {
            Some(value) => operand_type_opt = Some(OperandType::try_from_mod(value, flags)?),
            None => operand_type_opt = None,
        }

        Ok(Operand {
            operand_type: operand_type_opt,
            value: Some(value),
            displacement: None,
            data: None,
        })
    }

    pub fn reg(value: u8, flags: BitFlag) -> Result<Self, OperandTypeError> {
        Ok(Operand {
            operand_type: Some(OperandType::REGISTER(Size::new(flags))),
            value: Some(value),
            displacement: None,
            data: None,
        })
    }
}
