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
    let mut bytes_processed = 1;

    if instr.is_finished() {
        return Ok((instr, bytes_processed));
    }

    let n_of_bytes_needed = instr.continue_disassembly(buffer.next_byte()?)?;
    bytes_processed += 1;

    if instr.is_finished() {
        return Ok((instr, bytes_processed));
    }

    instr.finalize_disassembly(buffer.next_n_bytes(n_of_bytes_needed)?)?;
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
