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
    operand_a: Option<Operand>,
    operand_b: Option<Operand>,
    flags: BitFlag,
    ass_instr: AssembledInstruction,
}

impl Instruction {
    pub fn new(byte: u8) -> Result<Self, DecodingError> {
        let ass_instr = get_assembled_instruction(byte)?;
        let first_byte: Byte = ass_instr.bytes[0].unwrap();

        let mut instr = Instruction {
            operation: ass_instr.operation,
            operand_a: None,
            operand_b: None,
            flags: BitFlag::NOTHING,
            ass_instr,
        };

        // In First byte only flags, reg, and Literal bits are expeected
        for bits in first_byte.bits.iter().flatten() {
            let decoded_value: u8 = bits.decode_value(byte);

            match bits.usage {
                BitUsage::LITERAL => Ok(()),
                BitUsage::Flag(flag) => instr.set_flag(flag, decoded_value),
                BitUsage::REG => Ok(instr.set_operand_a(decoded_value)?),
                u => Err(
                    DecodingError::InvalidBitUsageError(
                        format!(
                        "Invalid BitUsage {:?}\nOnly Literal, Flag and Reg Field are expected in first byte", u
                    )
                    )
                ),
            }?;
        }

        Ok(instr)
    }

    pub fn continue_disassembly(&mut self, byte: u8) -> Result<usize, DecodingError> {
        let second_byte: Byte = self.ass_instr.bytes[1]
            .ok_or(DecodingError::InvalidBitUsageError("Exp".to_string()))?;

        let mut mode: Option<u8> = None;
        let mut rm: Option<u8> = None;

        // In First byte only flags, reg, and Literal bits are expeected
        for bits in second_byte.bits.iter().flatten() {
            let decoded_value = bits.decode_value(byte);

            match bits.usage {
                BitUsage::LITERAL => continue,
                BitUsage::Flag(flag) => self.set_flag(flag, decoded_value),
                BitUsage::REG => Ok(self.set_operand_a(decoded_value)?),
                BitUsage::Data(bit_order) => Ok(self.set_data(decoded_value, bit_order).unwrap()),
                BitUsage::RM => Ok(rm = Some(decoded_value)),
                BitUsage::MOD => Ok(mode = Some(decoded_value)),
                u => return Err(
                    DecodingError::InvalidBitUsageError(
                        format!(
                        "Invalid BitUsage {:?}\nOnly Literal, Flag, Reg, Rm, Mod fields are expected in second byte", u
                    )
                    )
                ),
            }?;
        }

        self.set_operand_b(rm, mode)?;

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
                            "Invalid BitUsage {:?}\nOnly Data or Displacement fields expctded on rest of bytes", u
                            )
                        )
                    ),
                };
            }
        }
        Ok(())
    }

    fn set_flag(&mut self, flag: BitFlag, value: u8) -> Result<(), DecodingError> {
        if value == 1 {
            Ok(self.flags = self.flags | flag)
        } else if value == 0 {
            Ok(())
        } else {
            Err(DecodingError::UnexpectedDecodedValueError(value))
        }
    }

    fn set_operand_b(&mut self, rm: Option<u8>, mode: Option<u8>) -> Result<(), DecodingError> {
        match (rm, mode) {
            (None, None) => Ok(()),
            (Some(rm), Some(mode)) => match &self.operand_b {
                Some(_) => panic!(),
                None => Ok(self.operand_b = Some(Operand::rm(rm, mode, self.flags)?)),
            },
            _ => panic!("Both RM and mode needs to be specifed"),
        }
    }

    fn set_operand_a(&mut self, reg: u8) -> Result<(), DecodingError> {
        match &self.operand_a {
            Some(_) => panic!(),
            None => Ok(self.operand_a = Some(Operand::reg(reg, self.flags)?)),
        }
    }

    fn additional_byte_count(&self) -> u8 {
        self.operand_b
            .as_ref()
            .expect("Operand B must be set")
            .operand_type
            .as_ref()
            .unwrap()
            .additional_byte_count()
            + self
                .operand_a
                .as_ref()
                .expect("Operand must be set")
                .operand_type
                .as_ref()
                .unwrap()
                .additional_byte_count()
    }

    fn set_displacement(
        &mut self,
        displacement_decoded: u8,
        bit_order: BitOrder,
    ) -> Result<(), DecodingError> {
        let operand_b = self.operand_b.as_mut().expect("Operand must be set");
        let operand_a = self.operand_a.as_mut().expect("Operand Must be set");
        let operand_b_type = operand_b
            .operand_type
            .as_ref()
            .expect("operand type must be known");
        let operand_a_type = operand_a.operand_type.as_ref().unwrap();

        match (operand_b_type, operand_a_type) {
            (OperandType::MEMORY(_), OperandType::MEMORY(_)) => {
                panic!("Both operands are memory, aborting")
            }
            (OperandType::MEMORY(_), _) => {
                operand_b.set_displacement(displacement_decoded, bit_order)
            }
            (_, OperandType::MEMORY(_)) => {
                operand_a.set_displacement(displacement_decoded, bit_order)
            }
            (_, _) => panic!("No opearnds are memory cannot set displacement"),
        }
    }

    fn set_data(&mut self, data: u8, bit_order: BitOrder) -> Result<(), DecodingError> {
        match (&self.operand_a, &self.operand_b) {
            (Some(_), None) => self.operand_b = Some(Operand::immediate(None, self.flags)?),
            (None, Some(_)) => self.operand_a = Some(Operand::immediate(None, self.flags)?),
            _ => (),
        }

        let operand_b = self.operand_b.as_mut().expect("Operand must be set");
        let operand_a = self.operand_a.as_mut().expect("Operand Must be set");
        let operand_b_type = operand_b.operand_type.as_ref().unwrap();
        let operand_a_type = operand_a.operand_type.as_ref().unwrap();

        match (operand_a_type, operand_b_type) {
            (OperandType::IMMEDIATE(_), OperandType::IMMEDIATE(_)) => panic!(),
            (OperandType::IMMEDIATE(_), _) => Ok(operand_a.set_data(data, bit_order).unwrap()),
            (_, OperandType::IMMEDIATE(_)) => Ok(operand_b.set_data(data, bit_order).unwrap()),
            (_, _) => panic!("No opearnds are immediate cannot set data"),
        }
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let src: &Operand;
        let dst: &Operand;

        let rm = self.operand_b.as_ref().unwrap();
        let reg = self.operand_a.as_ref().unwrap();

        if self.flags & BitFlag::D == BitFlag::D {
            src = rm;
            dst = reg;
        } else if matches!(rm.operand_type.as_ref().unwrap(), OperandType::IMMEDIATE(_)) {
            src = rm;
            dst = reg;
        } else {
            src = reg;
            dst = rm;
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
            instr.operand_a.unwrap().operand_type.unwrap(),
            OperandType::REGISTER(Size::WORD)
        ));
        assert!(matches!(
            instr.operand_b.unwrap().operand_type.unwrap(),
            OperandType::REGISTER(Size::WORD)
        ));

        let mut instr = Instruction::new(TEST_INSTRUCTION2.to_be_bytes()[0]).unwrap();

        instr
            .continue_disassembly(TEST_INSTRUCTION2.to_be_bytes()[1])
            .unwrap();

        assert!(matches!(
            instr.operand_a.unwrap().operand_type.unwrap(),
            OperandType::REGISTER(Size::WORD)
        ));
        let rm_type = instr
            .operand_b
            .as_ref()
            .unwrap()
            .operand_type
            .as_ref()
            .unwrap();
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
