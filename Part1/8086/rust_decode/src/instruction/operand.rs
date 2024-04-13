use std::{collections::HashMap, fmt};

use crate::assembled_instruction::{AssembledInstruction, BitFlag, BitOrder, S};
use crate::instruction::instruction::DecodingError;

#[derive(Debug)]
pub enum OperandToStrError {
    RegisterValueError,
    EffectiveAddrValueError,
    UnexpctedOperandError,
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
    fn to_str(
        &self,
        value: Option<u8>,
        displacement_value: Option<i16>,
        data: Option<i16>,
    ) -> Result<String, OperandToStrError> {
        match self {
            Self::REGISTER(size) => Ok(reg_encoding_table(*size)
                .get(&value.unwrap())
                .ok_or(OperandToStrError::RegisterValueError)?
                .to_string()),
            Self::MEMORY(displacement) => {
                let eff_addr = EFFECTIVE_ADDR
                    .get(&value.unwrap())
                    .ok_or(OperandToStrError::EffectiveAddrValueError)?
                    .to_string();

                match displacement {
                    Displacement::NO => Ok(format!("[{}]", eff_addr)),
                    Displacement::YES(_) => {
                        let disp_val = displacement_value.expect("Displacement should have value");

                        if disp_val == 0 {
                            Ok(format!("[{}]", eff_addr))
                        } else {
                            Ok(format!("[{}{:+}]", eff_addr, disp_val))
                        }
                    }
                }
            }
            Self::IMMEDIATE(_) => {
                Ok(format!("{}", data.expect("Immediate must have data")).to_string())
            }
        }
    }

    pub fn total_bytes_required(&self, flags: BitFlag, ass_instr: &AssembledInstruction) -> u8 {
        match self {
            OperandType::REGISTER(_) => 0,
            OperandType::IMMEDIATE(size) => {
                if ass_instr.includes_bits(S) {
                    if flags.is_flag_toogled(BitFlag::S) {
                        size.byte_count() - 1
                    } else {
                        size.byte_count()
                    }
                } else {
                    size.byte_count()
                }
            }
            OperandType::MEMORY(displacement) => displacement.byte_count(),
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
    static ref EFFECTIVE_ADDR: HashMap<u8, &'static str> = HashMap::from([
        (0b000, "bx + si"),
        (0b001, "bx + di"),
        (0b010, "bp + si"),
        (0b011, "bp + di"),
        (0b100, "si"),
        (0b101, "di"),
        (0b110, "bp"),
        (0b111, "bx"),
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

impl Displacement {
    pub fn byte_count(&self) -> u8 {
        match self {
            Self::NO => 0,
            Self::YES(size) => size.byte_count(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Size {
    BYTE,
    WORD,
}

impl Size {
    pub fn new(flags: BitFlag) -> Self {
        if flags.is_flag_toogled(BitFlag::W) {
            Self::WORD
        } else {
            Self::BYTE
        }
    }

    pub fn byte_count(&self) -> u8 {
        match self {
            Self::BYTE => 1,
            Self::WORD => 2,
        }
    }
}

impl fmt::Display for Size {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s: String;

        match self {
            Self::WORD => s = "word".to_string(),
            Self::BYTE => s = "byte".to_string(),
        }

        write!(f, "{}", s)
    }
}

#[derive(Debug)]
pub enum OperandTypeError {
    UnknownModError,
    MissingSizeSpecifierError,
    UncompatibleOperandTypeError,
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
    pub displacement: Option<i16>,
    pub data: Option<i16>,
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

    pub fn immediate(data: Option<u8>, flags: BitFlag) -> Result<Self, OperandTypeError> {
        Ok(Operand {
            operand_type: Some(OperandType::IMMEDIATE(Size::new(flags))),
            value: None,
            displacement: None,
            data: data.map(|d| d.into()),
        })
    }

    pub fn rm(value: u8, mode: u8, flags: BitFlag) -> Result<Self, OperandTypeError> {
        Ok(Operand {
            operand_type: Some(OperandType::try_from_mod(mode, flags)?),
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

    pub fn signed_displacement(&self) -> Result<i16, DecodingError> {
        match self.operand_type {
            Some(OperandType::MEMORY(Displacement::YES(size))) => match size {
                Size::BYTE => {
                    let u_disp = self
                        .displacement
                        .ok_or(DecodingError::FieldNotYetDecodedError)?;

                    if u_disp & 0x80 > 0 {
                        Ok((u_disp | 0b11111111 << 8).try_into().unwrap())
                    } else {
                        Ok((u_disp | 0x0000).try_into().unwrap())
                    }
                }
                Size::WORD => self
                    .displacement
                    .ok_or(DecodingError::FieldNotYetDecodedError),
            },
            Some(_) => Err(OperandTypeError::UncompatibleOperandTypeError.into()),
            None => Err(DecodingError::FieldNotYetDecodedError),
        }
    }

    pub fn set_data(&mut self, data_decoded: u8, bit_order: BitOrder) -> Result<(), DecodingError> {
        match bit_order {
            BitOrder::LOW => match self.data {
                Some(_) => Err(DecodingError::FieldAlreadyDecodedError),
                None => Ok(self.data = Some(data_decoded.into())),
            },
            BitOrder::HIGH => match self.data {
                None => Err(DecodingError::FieldNotYetDecodedError),
                Some(data) => Ok(self.data = Some(data | (data_decoded as i16) << 8)),
            },
        }
    }

    pub fn set_displacement(
        &mut self,
        displacement_decoded: u8,
        bit_order: BitOrder,
    ) -> Result<(), DecodingError> {
        match bit_order {
            BitOrder::LOW => match self.displacement {
                Some(_) => Err(DecodingError::FieldAlreadyDecodedError),
                None => Ok(self.displacement = Some(displacement_decoded.into())),
            },
            BitOrder::HIGH => match self.displacement {
                None => Err(DecodingError::FieldNotYetDecodedError),
                Some(data) => {
                    Ok(self.displacement = Some(data | (displacement_decoded as i16) << 8))
                }
            },
        }
    }

    pub fn n_bytes_needed(&self, flags: BitFlag, ass_instr: &AssembledInstruction) -> u8 {
        self.operand_type
            .as_ref()
            .map(|op_type| {
                let bytes_required = op_type.total_bytes_required(flags, ass_instr);

                match op_type {
                    OperandType::REGISTER(_) => bytes_required,
                    OperandType::MEMORY(Displacement::YES(_)) => match self.displacement {
                        Some(_) => bytes_required - 1,
                        None => bytes_required,
                    },
                    OperandType::MEMORY(Displacement::NO) => bytes_required,
                    OperandType::IMMEDIATE(_) => match self.data {
                        Some(_) => bytes_required - 1,
                        None => bytes_required,
                    },
                }
            })
            .unwrap()
    }
}

impl fmt::Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let operand_str: String = self
            .operand_type
            .as_ref()
            .unwrap()
            .to_str(self.value, self.signed_displacement().ok(), self.data)
            .unwrap();

        write!(f, "{}", operand_str)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_signed_displacement() {
        let op = Operand {
            operand_type: Some(OperandType::MEMORY(Displacement::YES(Size::BYTE))),
            data: None,
            displacement: Some(0x00db),
            value: None,
        };

        let signed_displacement = op.signed_displacement();

        assert!(op.signed_displacement().unwrap() == -37);
    }
}
