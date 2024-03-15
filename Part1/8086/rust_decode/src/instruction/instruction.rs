use super::operand::{Operand, OperandTypeError};
use crate::assembled_instruction::*;

#[derive(Debug)]
pub enum DecodingError {
    InvalidBitUsageError(String),
    FieldAlreadyDecodedError,
    FieldNotYetDecodedError,
    InstructionNotRecognizedError(String),
    Error(String),
}

impl From<AssembledInstructionLookupError> for DecodingError {
    fn from(_e: AssembledInstructionLookupError) -> Self {
        DecodingError::InstructionNotRecognizedError("Could not find instruction".to_string())
    }
}

impl From<OperandTypeError> for DecodingError {
    fn from(_e: OperandTypeError) -> Self {
        DecodingError::Error("Operand type error".to_string())
    }
}

#[derive(Debug)]
pub struct Instruction {
    operation: Operation,
    reg: Option<Operand>,
    rm: Option<Operand>,
    flags: BitFlag,
    ass_instr: AssembledInstruction,
}

impl Instruction {
    pub fn set_flag(&mut self, flag: BitFlag, value: u8) -> Result<(), DecodingError> {
        if value != 0 {
            self.flags = self.flags | flag;
        }

        Ok(())
    }

    pub fn set_rm(&mut self, value: Option<u8>, mode: Option<u8>) -> Result<(), DecodingError> {
        match value {
            Some(val) => match &self.rm {
                Some(_) => Err(DecodingError::FieldAlreadyDecodedError),
                None => Ok(self.rm = Some(Operand::rm(val, mode, self.flags)?)),
            },
            None => Ok(()),
        }
    }

    pub fn set_reg(&mut self, value: Option<u8>) -> Result<(), DecodingError> {
        match value {
            Some(val) => match &self.reg {
                Some(_) => Err(DecodingError::FieldAlreadyDecodedError),
                None => Ok(self.reg = Some(Operand::reg(val, self.flags)?)),
            },
            None => Ok(()),
        }
    }

    pub fn new(byte: u8) -> Result<Self, DecodingError> {
        let ass_instr = get_assembled_instruction(byte)?;
        let first_byte: Byte = ass_instr.bytes[0].unwrap();

        let mut instr = Instruction {
            operation: ass_instr.operation,
            reg: None,
            rm: None,
            flags: BitFlag::NOTHING,
            ass_instr,
        };
        let mut reg: Option<u8> = None;

        // In First byte only flags, reg, and Literal bits are expeected
        for bits in first_byte.bits.iter().flatten() {
            let decoded_value: u8 = bits.decode_value(byte);

            match bits.usage {
                BitUsage::LITERAL => Ok(()),
                BitUsage::Flag(flag) => instr.set_flag(flag, decoded_value),
                BitUsage::REG => Ok(reg = Some(decoded_value)),
                u => Err(
                    DecodingError::InvalidBitUsageError(
                        format!(
                        "Invalid BitUsage {:?}\nOnly Literal, Flag and Reg Field are expected in first byte", u
                    )
                    )
                ),
            }?;
        }

        instr.set_reg(reg)?;

        Ok(instr)
    }

    pub fn is_finished(&self) -> bool {
        self.rm.is_some() & self.reg.is_some()
    }

    pub fn continue_disassembly(&mut self, byte: u8) -> Result<usize, DecodingError> {
        let second_byte: Byte = self.ass_instr.bytes[1]
            .ok_or(DecodingError::InvalidBitUsageError("Exp".to_string()))?;

        let mut reg: Option<u8> = None;
        let mut mode: Option<u8> = None;
        let mut rm: Option<u8> = None;

        // In First byte only flags, reg, and Literal bits are expeected
        for bits in second_byte.bits.iter().flatten() {
            let decoded_value: u8 = bits.decode_value(byte);

            match bits.usage {
                BitUsage::LITERAL => continue,
                BitUsage::Flag(flag) => self.set_flag(flag, decoded_value),
                BitUsage::REG => Ok(reg = Some(decoded_value)),
                BitUsage::RM => Ok(rm = Some(decoded_value)),
                BitUsage::MOD => Ok(mode = Some(decoded_value)),
                u => return Err(
                    DecodingError::InvalidBitUsageError(
                        format!(
                        "Invalid BitUsage {:?}\nOnly Literal, Flag and Reg Field are expected in first byte", u
                    )
                    )
                ),
            }?;
        }

        self.set_rm(rm, mode)?;
        self.set_reg(reg)?;

        Ok(0)
    }

    pub fn finalize_disassembly(&mut self, byte: Vec<u8>) -> Result<(), DecodingError> {
        Ok(())
    }
}
