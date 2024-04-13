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

    fn test_instruction(bytes: Vec<u8>, instruction_str: &str) -> () {
        let mut buffer = InstructionBuffer {
            buf: bytes.clone(),
            last_read: 0,
            bytes_loaded: bytes.len(),
        };

        let (instruction, bytes_processed) = disassemble_instruction(&mut buffer).unwrap();

        println!("{:?}", instruction);

        assert_eq!(bytes_processed, bytes.len());
        assert_eq!(format!("{}", instruction), instruction_str);
    }

    #[test]
    fn test_memory_mode_instruction() {
        test_instruction(vec![0x8A, 0x00], "mov al, [bx + si]")
    }

    #[test]
    fn test_memory_mode_instruction_displacement() {
        test_instruction(vec![0x8A, 0x80, 0x87, 0x13], "mov al, [bx + si+4999]")
    }

    #[test]
    fn test_mov_a() {
        test_instruction(vec![0x8b, 0x41, 0xdb], "mov ax, [bx + di-37]")
    }

    #[test]
    fn test_immediate_to_register() {
        test_instruction(vec![0xBA, 0x6C, 0x0F], "mov dx, 3948")
    }

    #[test]
    fn test_immediate_to_memory() {
        test_instruction(vec![0xC6, 0x03, 0x07], "mov [bp + di], byte 7")
    }

    #[test]
    fn test_immediate_add() {
        test_instruction(vec![0x83, 0xc6, 0x02], "add si, 2")
    }

    #[test]
    fn test_immediate_add_a() {
        test_instruction(vec![0x03, 0x18], "add bx, [bx + si]")
    }

    #[test]
    fn test_immediate_add_b() {
        test_instruction(vec![0x03, 0x18], "add bx, [bx + si]")
    }

    #[test]
    fn test_immediate_add_c() {
        test_instruction(vec![0x02, 0x7A, 0x04], "add bh, [bp + si+4]")
    }
}
