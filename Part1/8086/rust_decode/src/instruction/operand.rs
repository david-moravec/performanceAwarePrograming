use std::{collections::HashMap, fmt};

use crate::assembled_instruction::{AssembledInstruction, BitFlag, BitOrder, S};
use crate::cpu::cpu::{CpuOperand, EffectiveAddress, Reg};
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
    Register(Size),
    Memory(Displacement),
    DirectAccess(Displacement),
    Immediate(Size),
    Jump,
    NotUsed,
}

impl OperandType {
    fn to_str(
        &self,
        value: Option<u8>,
        displacement_value: Option<i16>,
        data: Option<i16>,
    ) -> Result<String, OperandToStrError> {
        match self {
            Self::Register(size) => Ok(reg_encoding_table(*size)
                .get(&value.unwrap())
                .ok_or(OperandToStrError::RegisterValueError)?
                .to_string()),
            Self::Memory(displacement) => {
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
            Self::DirectAccess(_) => Ok(format!(
                "[{}]",
                displacement_value.expect("Disaplacment should have valeu")
            )),
            Self::Immediate(_) => {
                Ok(format!("{}", data.expect("Immediate must have data")).to_string())
            }
            Self::Jump => Ok(format!("{}", displacement_value.unwrap())),
            Self::NotUsed => Ok("".to_string()),
        }
    }

    pub fn total_bytes_required(&self, flags: BitFlag, ass_instr: &AssembledInstruction) -> u8 {
        match self {
            OperandType::Register(_) => 0,
            OperandType::Immediate(size) => {
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
            OperandType::Memory(displacement) => displacement.byte_count(),
            OperandType::DirectAccess(_) => 2,
            OperandType::Jump => 0,
            OperandType::NotUsed => 0,
        }
    }
}

impl OperandType {
    fn try_from_mod(rm: u8, mode: u8, flags: BitFlag) -> Result<Self, OperandTypeError> {
        match mode {
            0b00 => {
                if rm == 0b110 {
                    Ok(OperandType::DirectAccess(Displacement::YES(Size::WORD)))
                } else {
                    Ok(OperandType::Memory(Displacement::NO))
                }
            }
            0b01 => Ok(OperandType::Memory(Displacement::YES(Size::BYTE))),
            0b10 => Ok(OperandType::Memory(Displacement::YES(Size::WORD))),
            0b11 => Ok(OperandType::Register(Size::new(flags))),
            _ => Err(UnknownModError),
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

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
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
            operand_type: Some(OperandType::Immediate(Size::new(flags))),
            value: None,
            displacement: None,
            data: data.map(|d| d.into()),
        })
    }

    pub fn rm(rm: u8, mode: u8, flags: BitFlag) -> Result<Self, OperandTypeError> {
        Ok(Operand {
            operand_type: Some(OperandType::try_from_mod(rm, mode, flags)?),
            value: Some(rm),
            displacement: None,
            data: None,
        })
    }

    pub fn jump_operands(displacement: u8) -> (Operand, Operand) {
        let op_a = Operand {
            operand_type: Some(OperandType::Jump),
            value: None,
            displacement: Some(displacement.into()),
            data: None,
        };
        let op_b = Operand {
            operand_type: Some(OperandType::NotUsed),
            value: None,
            displacement: None,
            data: None,
        };

        (op_a, op_b)
    }

    pub fn reg(value: u8, flags: BitFlag) -> Result<Self, OperandTypeError> {
        Ok(Operand {
            operand_type: Some(OperandType::Register(Size::new(flags))),
            value: Some(value),
            displacement: None,
            data: None,
        })
    }

    pub fn signed_data(&self) -> Result<i16, DecodingError> {
        match self.operand_type {
            Some(OperandType::Immediate(size)) => match size {
                Size::BYTE => {
                    let u_data = self.data.ok_or(DecodingError::FieldNotYetDecodedError)?;

                    if u_data & 0x80 > 0 {
                        Ok((u_data | 0b11111111 << 8).try_into().unwrap())
                    } else {
                        Ok((u_data | 0x0000).try_into().unwrap())
                    }
                }
                Size::WORD => self.data.ok_or(DecodingError::FieldNotYetDecodedError),
            },
            Some(_) => Err(OperandTypeError::UncompatibleOperandTypeError.into()),
            None => Err(DecodingError::FieldNotYetDecodedError),
        }
    }

    pub fn signed_displacement(&self) -> Result<i16, DecodingError> {
        fn sign_extend(displacement: Option<i16>) -> Result<i16, DecodingError> {
            let u_disp = displacement.ok_or(DecodingError::FieldNotYetDecodedError)?;

            if u_disp & 0x80 > 0 {
                Ok((u_disp | 0b11111111 << 8).try_into().unwrap())
            } else {
                Ok((u_disp | 0x0000).try_into().unwrap())
            }
        }

        match self.operand_type {
            Some(OperandType::Memory(Displacement::YES(size))) => match size {
                Size::BYTE => sign_extend(self.displacement),
                Size::WORD => self
                    .displacement
                    .ok_or(DecodingError::FieldNotYetDecodedError),
            },
            Some(OperandType::DirectAccess(_)) => self
                .displacement
                .ok_or(DecodingError::FieldNotYetDecodedError),
            Some(OperandType::Jump) => sign_extend(self.displacement),
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
                    OperandType::Register(_) => bytes_required,
                    OperandType::Memory(Displacement::YES(_)) => match self.displacement {
                        Some(_) => bytes_required - 1,
                        None => bytes_required,
                    },
                    OperandType::Memory(Displacement::NO) => bytes_required,
                    OperandType::Immediate(_) => match self.data {
                        Some(_) => bytes_required - 1,
                        None => bytes_required,
                    },
                    OperandType::DirectAccess(Displacement::YES(_)) => match self.displacement {
                        Some(_) => bytes_required - 1,
                        None => bytes_required,
                    },
                    _ => bytes_required,
                }
            })
            .unwrap()
    }

    pub fn parse_for_cpu(&self) -> CpuOperand {
        match self
            .operand_type
            .as_ref()
            .expect("But, operand type must be known before parsing for CPU")
        {
            OperandType::Register(_) => CpuOperand::Register(Reg::new(self.value.unwrap())),
            OperandType::Memory(_) => CpuOperand::Memory(EffectiveAddress::new(
                self.value.unwrap(),
                self.displacement,
            )),
            OperandType::Immediate(_) => CpuOperand::Immediate(self.data.unwrap()),
            OperandType::DirectAccess(_) => CpuOperand::DirectAcces(self.displacement.unwrap()),
            OperandType::Jump => CpuOperand::Jump(self.signed_displacement().unwrap()),
            OperandType::NotUsed => CpuOperand::NotUsed,
        }
    }
}

impl fmt::Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let operand_str: String = self
            .operand_type
            .as_ref()
            .unwrap()
            .to_str(
                self.value,
                self.signed_displacement().ok(),
                self.signed_data().ok(),
            )
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
            operand_type: Some(OperandType::Memory(Displacement::YES(Size::BYTE))),
            data: None,
            displacement: Some(0x00db),
            value: None,
        };

        assert!(op.signed_displacement().unwrap() == -37);
    }
}
