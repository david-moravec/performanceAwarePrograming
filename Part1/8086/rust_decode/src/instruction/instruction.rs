use super::operand::{Displacement, Operand, OperandType, OperandTypeError, Size};
use crate::assembled_instruction::*;
use core::panic;
use std::{fmt, iter::IntoIterator};

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
                BitUsage::REG => instr.set_reg_operand(decoded_value),
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
                BitUsage::LITERAL => self.handle_literal(),
                BitUsage::Flag(flag) => self.set_flag(flag, decoded_value),
                BitUsage::REG => self.set_reg_operand(decoded_value),
                BitUsage::Data(_) => self.set_immediate_operand(Some(decoded_value.into())),
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

        self.set_rm_operand(rm, mode)?;

        Ok(self.additional_byte_count().into())
    }

    fn handle_literal(&mut self) -> Result<(), DecodingError> {
        self.set_immediate_operand(None)
    }

    fn should_process_bits(&self, bits: Bits) -> bool {
        match bits.usage {
            BitUsage::Data(bit_order) => match bit_order {
                BitOrder::LOW => true,
                BitOrder::HIGH => self.flags & BitFlag::W == BitFlag::W,
            },
            BitUsage::Disp(bit_order) => {
                let type_b = self
                    .operand_b
                    .as_ref()
                    .unwrap()
                    .operand_type
                    .as_ref()
                    .unwrap();

                match type_b {
                    OperandType::REGISTER(_) => false,
                    OperandType::MEMORY(Displacement::NO) => false,
                    OperandType::MEMORY(Displacement::YES(size)) => match bit_order {
                        BitOrder::LOW => true,
                        BitOrder::HIGH => matches!(size, Size::WORD),
                    },
                    OperandType::DIRECT_ACCESS(_) => true,
                    _ => panic!("Expected only memory Operand"),
                }
            }
            _ => true,
        }
    }

    fn bytes_to_finalize(&self) -> Vec<Byte> {
        let mut a: Vec<Byte> = vec![];

        for byte in self.ass_instr.bytes[2..].into_iter() {
            match byte {
                Some(b) => {
                    if self.should_process_bits(b.bits[0].unwrap()) {
                        a.push(*b);
                    }
                }
                None => (),
            }
        }

        a
    }

    pub fn finalize_disassembly(&mut self, bytes_given: Vec<u8>) -> Result<(), DecodingError> {
        let instruction_bytes = self.bytes_to_finalize();

        for (byte_given, byte_expected) in bytes_given.iter().zip(instruction_bytes) {
            for bits in byte_expected.bits.iter().flatten() {
                let decoded_value = bits.decode_value(*byte_given);

                if self.should_process_bits(*bits) {
                    match bits.usage {
                        BitUsage::Data(bit_order) => {
                            match self.set_immediate_operand(Some(decoded_value.into())) {
                                Ok(_) => Ok(()),
                                Err(DecodingError::FieldAlreadyDecodedError) => self.set_data(decoded_value, bit_order),
                                Err(e) => Err(e),
                            }
                        }
                        BitUsage::Disp(bit_order) => self.set_displacement(decoded_value, bit_order),
                        u => Err(
                            DecodingError::InvalidBitUsageError(
                                format!(
                                "Invalid BitUsage {:?}\nOnly Data or Displacement fields expctded on rest of bytes", u
                                )
                            )
                        ),
                    }?;
                }
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

    fn set_rm_operand(&mut self, rm: Option<u8>, mode: Option<u8>) -> Result<(), DecodingError> {
        match (rm, mode) {
            (None, None) => Ok(()),
            (Some(rm), Some(mode)) => match &self.operand_b {
                Some(_) => panic!(),
                None => Ok(self.operand_b = Some(Operand::rm(rm, mode, self.flags)?)),
            },
            _ => panic!("Both RM and mode needs to be specifed"),
        }
    }

    fn set_reg_operand(&mut self, reg: u8) -> Result<(), DecodingError> {
        match &self.operand_a {
            Some(_) => panic!(),
            None => Ok(self.operand_a = Some(Operand::reg(reg, self.flags)?)),
        }
    }

    fn additional_byte_count(&self) -> u8 {
        self.operand_a
            .as_ref()
            .map(|op| op.n_bytes_needed(self.flags, &self.ass_instr))
            .unwrap()
            + self
                .operand_b
                .as_ref()
                .map(|op| op.n_bytes_needed(self.flags, &self.ass_instr))
                .unwrap()
    }

    fn operands_sorted(&self) -> (&Operand, &Operand) {
        let (dst, src): (&Operand, &Operand);

        let op_a = self.operand_a.as_ref().unwrap();
        let op_b = self.operand_b.as_ref().unwrap();

        let b_type = op_b.operand_type.as_ref().unwrap();

        if self.flags.is_flag_toogled(BitFlag::D) {
            src = op_b;
            dst = op_a;
        } else if matches!(b_type, OperandType::IMMEDIATE(_)) {
            src = op_b;
            dst = op_a;
        } else {
            src = op_a;
            dst = op_b;
        };

        (dst, src)
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
            (OperandType::DIRECT_ACCESS(_), _) => {
                operand_b.set_displacement(displacement_decoded, bit_order)
            }
            (_, OperandType::DIRECT_ACCESS(_)) => {
                operand_a.set_displacement(displacement_decoded, bit_order)
            }
            (_, _) => panic!("No opearnds are memory cannot set displacement"),
        }
    }
    fn set_immediate_operand(&mut self, data: Option<u8>) -> Result<(), DecodingError> {
        let a = match (&self.operand_a, &self.operand_b) {
            (Some(_), None) => Ok(self.operand_b = Some(Operand::immediate(data, self.flags)?)),
            (None, Some(_)) => Ok(self.operand_a = Some(Operand::immediate(data, self.flags)?)),
            (None, None) => Ok(self.operand_a = Some(Operand::immediate(data, self.flags)?)),
            _ => Err(DecodingError::FieldAlreadyDecodedError),
        };

        a
    }

    fn set_data(&mut self, data: u8, bit_order: BitOrder) -> Result<(), DecodingError> {
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
        let (dst, src) = self.operands_sorted();

        let mut src_size: String = "".to_string();
        let dst_size: String = "".to_string();

        match (
            dst.operand_type.as_ref().unwrap(),
            src.operand_type.as_ref().unwrap(),
        ) {
            (OperandType::MEMORY(_), OperandType::IMMEDIATE(size)) => {
                src_size = format!("{:} ", size)
            }
            _ => (),
        };

        write!(
            f,
            "{} {}{}, {}{}",
            self.operation, dst_size, dst, src_size, src
        )
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
        assert!(instr.flags.is_flag_toogled(BitFlag::W));

        let instr = Instruction::new(TEST_INSTRUCTION2.to_be_bytes()[0]).unwrap();

        assert!(matches!(instr.operation, Operation::MOV));
        assert!(instr.flags.is_flag_toogled(BitFlag::W | BitFlag::D));
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
