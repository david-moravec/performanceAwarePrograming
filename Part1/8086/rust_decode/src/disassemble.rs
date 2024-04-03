use crate::assembled_instruction::AssembledInstructionLookupError;
use crate::instruction::instruction::{DecodingError, Instruction};
use crate::{BufferEndReachedError, InstructionBuffer, MEMORY_SIZE};

#[derive(Debug)]
pub enum DisassemblyError {
    LookupError(AssembledInstructionLookupError),
    BufferError(BufferEndReachedError),
    DecodeError(DecodingError),
}

impl From<AssembledInstructionLookupError> for DisassemblyError {
    fn from(e: AssembledInstructionLookupError) -> Self {
        DisassemblyError::LookupError(e)
    }
}

impl From<DecodingError> for DisassemblyError {
    fn from(e: DecodingError) -> Self {
        DisassemblyError::DecodeError(e)
    }
}

impl From<BufferEndReachedError> for DisassemblyError {
    fn from(e: BufferEndReachedError) -> Self {
        DisassemblyError::BufferError(e)
    }
}

type DisassemblyResult<T> = Result<T, DisassemblyError>;

pub fn disassemble_instruction(
    buffer: &mut InstructionBuffer,
) -> DisassemblyResult<(Instruction, usize)> {
    let mut instr = Instruction::new(buffer.next_byte()?)?;
    let mut bytes_processed: usize = 1;

    let n_of_bytes_needed = instr.continue_disassembly(buffer.next_byte()?)?;
    bytes_processed += 1;

    if n_of_bytes_needed == 0 {
        return Ok((instr, bytes_processed));
    }

    instr.finalize_disassembly(buffer.next_n_bytes(n_of_bytes_needed.into())?)?;
    bytes_processed += n_of_bytes_needed;

    Ok((instr, bytes_processed))
}

pub fn disassemble_bytes_in(mut buffer: InstructionBuffer) -> DisassemblyResult<Vec<Instruction>> {
    let mut current_byte = 0;
    let mut instructions: Vec<Instruction> = Vec::with_capacity(MEMORY_SIZE / 2);

    while current_byte < buffer.bytes_loaded {
        let (instr, bytes_processed) = disassemble_instruction(&mut buffer)?;
        instructions.push(instr);
        current_byte += bytes_processed;
    }

    Ok(instructions)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_memory_mode_instruction() {
        let instr: u16 = 0x8A00;

        let mut buffer = InstructionBuffer {
            buf: instr.to_be_bytes().to_vec(),
            last_read: 0,
            bytes_loaded: 2,
        };

        let (instruction, bytes_processed) = disassemble_instruction(&mut buffer).unwrap();

        assert_eq!(format!("{}", instruction), "mov al, [bx + si]");
        assert_eq!(bytes_processed, 2);
    }

    #[test]
    fn test_memory_mode_instruction_displacement() {
        let instr: u32 = 0x8A808713;

        let mut buffer = InstructionBuffer {
            buf: instr.to_be_bytes().to_vec(),
            last_read: 0,
            bytes_loaded: 4,
        };

        let (instruction, bytes_processed) = disassemble_instruction(&mut buffer).unwrap();

        assert_eq!(format!("{}", instruction), "mov al, [bx + si+4999]");
        assert_eq!(bytes_processed, 4);
    }

    #[test]
    fn test_immediate_to_register() {
        let mut buffer = InstructionBuffer {
            buf: [0xBA, 0x6C, 0x0F].to_vec(),
            last_read: 0,
            bytes_loaded: 3,
        };

        let (instruction, bytes_processed) = disassemble_instruction(&mut buffer).unwrap();

        println!("{:?}", instruction);

        assert_eq!(format!("{}", instruction), "mov dx, 3948");
        assert_eq!(bytes_processed, 3);
    }

    #[test]
    fn test_immediate_to_memory() {
        let mut buffer = InstructionBuffer {
            buf: [0xC6, 0x03, 0x07].to_vec(),
            last_read: 0,
            bytes_loaded: 3,
        };

        let (instruction, bytes_processed) = disassemble_instruction(&mut buffer).unwrap();

        println!("bytes processed {:}", bytes_processed);
        println!("{:?}", instruction);

        assert_eq!(format!("{}", instruction), "mov [bp + di], byte 7");
        assert_eq!(bytes_processed, 3);
    }
}
