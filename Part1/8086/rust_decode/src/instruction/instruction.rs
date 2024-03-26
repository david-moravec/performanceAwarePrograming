use super::operand::{Operand, OperandError, OperandType, OperandTypeError};
use crate::assembled_instruction::*;
use core::panic;
use std::fmt;

#[derive(Debug)]
pub enum DecodingError {
    InvalidBitUsageError(String),
    FieldAlreadyDecodedError,
    FieldNotYetDecodedError,
    InstructionNotRecognizedError(String),
    UnexpectedDecodedValueError(u8),
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
        if value == 1 {
            Ok(self.flags = self.flags | flag)
        } else if value == 0 {
            Ok(())
        } else {
            Err(DecodingError::UnexpectedDecodedValueError(value))
        }
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

    pub fn additional_byte_count(&self) -> u8 {
        self.rm
            .as_ref()
            .unwrap()
            .operand_type
            .as_ref()
            .unwrap()
            .additional_byte_count()
            + self
                .reg
                .as_ref()
                .unwrap()
                .operand_type
                .as_ref()
                .unwrap()
                .additional_byte_count()
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

        Ok(self.additional_byte_count().into())
    }

    pub fn finalize_disassembly(&mut self, bytes_given: Vec<u8>) -> Result<(), DecodingError> {
        let instruction_bytes = self.ass_instr.bytes.clone();

        for (byte_given, byte_expected_opt) in bytes_given.iter().zip(instruction_bytes[2..].iter())
        {
            for bits in byte_expected_opt
                .expect("More bytes given than expected")
                .bits
                .iter()
                .flatten()
            {
                let decoded_value = bits.decode_value(*byte_given);

                match bits.usage {
                    BitUsage::Data(bit_order) => self.set_data(decoded_value, bit_order)?,
                    BitUsage::Disp(bit_order) => self.set_displacement(decoded_value, bit_order)?,
                    u => return Err(
                        DecodingError::InvalidBitUsageError(
                            format!(
                            "Invalid BitUsage {:?}\nOnly data or displacement expctded on rest of bytes", u
                            )
                        )
                    ),
                };
            }
        }
        Ok(())
    }

    fn set_displacement(
        &mut self,
        displacement_decoded: u8,
        bit_order: BitOrder,
    ) -> Result<(), DecodingError> {
        let rm = self
            .rm
            .as_mut()
            .ok_or(DecodingError::FieldNotYetDecodedError)?;
        let reg = self
            .reg
            .as_mut()
            .ok_or(DecodingError::FieldNotYetDecodedError)?;

        let rm_type = rm
            .operand_type
            .as_ref()
            .ok_or(DecodingError::FieldNotYetDecodedError)?;
        let reg_type = reg
            .operand_type
            .as_ref()
            .ok_or(DecodingError::FieldNotYetDecodedError)?;

        match (rm_type, reg_type) {
            (OperandType::MEMORY(_), OperandType::MEMORY(_)) => {
                panic!("Both operands are memory, aborting")
            }
            (OperandType::MEMORY(_), _) => rm.set_displacement(displacement_decoded, bit_order),
            (_, OperandType::MEMORY(_)) => reg.set_displacement(displacement_decoded, bit_order),
            (_, _) => panic!("No opearnds are memory cannot set displacement"),
        }
    }

    fn set_data(&mut self, data: u8, bit_order: BitOrder) -> Result<(), DecodingError> {
        let rm = self
            .rm
            .as_mut()
            .ok_or(DecodingError::FieldNotYetDecodedError)?;
        let reg = self
            .reg
            .as_mut()
            .ok_or(DecodingError::FieldNotYetDecodedError)?;

        let rm_type = rm
            .operand_type
            .as_ref()
            .ok_or(DecodingError::FieldNotYetDecodedError)?;
        let reg_type = reg
            .operand_type
            .as_ref()
            .ok_or(DecodingError::FieldNotYetDecodedError)?;

        match (rm_type, reg_type) {
            (OperandType::IMMEDIATE(_), OperandType::IMMEDIATE(_)) => {
                panic!("Both operands are immediate, aborting")
            }
            (OperandType::IMMEDIATE(_), _) => rm.set_data(data, bit_order),
            (_, OperandType::IMMEDIATE(_)) => reg.set_data(data, bit_order),
            (_, _) => panic!("No opearnds are immediate cannot set data"),
        }
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let src: &Operand;
        let dst: &Operand;

        if self.flags & BitFlag::D == BitFlag::D {
            src = self.rm.as_ref().unwrap();
            dst = self.reg.as_ref().unwrap();
        } else {
            src = self.reg.as_ref().unwrap();
            dst = self.rm.as_ref().unwrap();
        };

        write!(f, "{} {}, {}", self.operation, dst, src)
    }
}

#[cfg(test)]
mod test {
    use crate::instruction::operand::{Displacement, OperandType, Size};

    use super::*;

    static TEST_INSTRUCTION: u16 = 0b1000101111011001;
    static TEST_INSTRUCTION2: u16 = 0b1000101110011001;

    #[test]
    fn test_instruction_new() {
        let instr = Instruction::new(TEST_INSTRUCTION.to_be_bytes()[0]).unwrap();

        assert!(matches!(instr.operation, Operation::MOV));
        assert!(instr.flags & BitFlag::W == BitFlag::W);

        let instr = Instruction::new(TEST_INSTRUCTION2.to_be_bytes()[0]).unwrap();

        assert!(matches!(instr.operation, Operation::MOV));
        assert!(instr.flags == BitFlag::W | BitFlag::D);
    }

    #[test]
    fn test_instruction_continue_disassembly() {
        let mut instr = Instruction::new(TEST_INSTRUCTION.to_be_bytes()[0]).unwrap();

        instr
            .continue_disassembly(TEST_INSTRUCTION.to_be_bytes()[1])
            .unwrap();

        assert!(matches!(
            instr.reg.unwrap().operand_type.unwrap(),
            OperandType::REGISTER(Size::WORD)
        ));
        assert!(matches!(
            instr.rm.unwrap().operand_type.unwrap(),
            OperandType::REGISTER(Size::WORD)
        ));

        let mut instr = Instruction::new(TEST_INSTRUCTION2.to_be_bytes()[0]).unwrap();

        instr
            .continue_disassembly(TEST_INSTRUCTION2.to_be_bytes()[1])
            .unwrap();

        assert!(matches!(
            instr.reg.unwrap().operand_type.unwrap(),
            OperandType::REGISTER(Size::WORD)
        ));
        let rm_type = instr.rm.as_ref().unwrap().operand_type.as_ref().unwrap();
        assert!(matches!(
            rm_type,
            OperandType::MEMORY(Displacement::YES(Size::WORD))
        ));
    }

    #[test]
    fn test_instruction_display() {
        let mut instr = Instruction::new(TEST_INSTRUCTION.to_be_bytes()[0]).unwrap();
        instr
            .continue_disassembly(TEST_INSTRUCTION.to_be_bytes()[1])
            .unwrap();

        assert_eq!(format!("{}", instr), "mov bx, cx")
    }
}
