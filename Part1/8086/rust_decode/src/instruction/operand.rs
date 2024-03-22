use std::{collections::HashMap, fmt};

use crate::assembled_instruction::BitFlag;

#[derive(Debug)]
pub enum OperandToStrError {
    WrongRegisterValueError,
}

#[derive(Debug)]
pub enum OperandError {
    TypeDecodingError(OperandTypeError),
    ToStrError(OperandToStrError),
}

impl From<OperandTypeError> for OperandError {
    fn from(e: OperandTypeError) -> Self {
        Self::TypeDecodingError(e)
    }
}

impl From<OperandToStrError> for OperandError {
    fn from(e: OperandToStrError) -> Self {
        Self::ToStrError(e)
    }
}

#[derive(Debug)]
pub enum OperandType {
    REGISTER(Size),
    MEMORY(Displacement),
    IMMEDIATE(Size),
}

impl OperandType {
    fn to_str(&self, value: u8) -> Result<String, OperandToStrError> {
        match self {
            Self::REGISTER(size) => Ok(reg_encoding_table(*size)
                .get(&value)
                .ok_or(OperandToStrError::WrongRegisterValueError)?
                .to_string()),
            Self::MEMORY(_) => Ok("mem".to_string()),
            Self::IMMEDIATE(_) => Ok("mem".to_string()),
        }
    }
}

lazy_static! {
    static ref REG_WORD: HashMap<u8, &'static str> = HashMap::from([
        (0b000, "ax"),
        (0b001, "cx"),
        (0b010, "dx"),
        (0b011, "bx"),
        (0b100, "sp"),
        (0b101, "bp"),
        (0b110, "si"),
        (0b111, "di"),
    ]);
    static ref REG_BYTE: HashMap<u8, &'static str> = HashMap::from([
        (0b000, "al"),
        (0b001, "cl"),
        (0b010, "dl"),
        (0b011, "bl"),
        (0b100, "ah"),
        (0b101, "ch"),
        (0b110, "dh"),
        (0b111, "bh"),
    ]);
}

fn reg_encoding_table(size: Size) -> &'static HashMap<u8, &'static str> {
    match size {
        Size::WORD => &REG_WORD,
        Size::BYTE => &REG_BYTE,
    }
}

#[derive(Debug)]
pub enum Displacement {
    NO,
    YES(Size),
}

#[derive(Debug, Clone, Copy)]
pub enum Size {
    BYTE,
    WORD,
}

impl Size {
    pub fn new(flags: BitFlag) -> Self {
        if flags & BitFlag::W != BitFlag::NOTHING {
            Self::WORD
        } else {
            Self::BYTE
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

impl fmt::Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            self.operand_type
                .as_ref()
                .unwrap()
                .to_str(self.value.unwrap())
                .unwrap()
        )
    }
}
