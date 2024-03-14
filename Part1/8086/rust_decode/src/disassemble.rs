use crate::assembled_instruction::{get_assembled_instruction, AssembledInstruction};
use crate::instruction::instruction::Instruction;

use super::Memory;

#[derive(Debug)]
pub struct DisassemblyError;

pub fn disassemble_8086(memory: &Memory, n_bytes_read: usize) -> Result<(), DisassemblyError> {
    let mut i = 0;

    while i < n_bytes_read {
        let pending_byte = memory[i];

        let assembled_instr_opt = get_assembled_instruction(pending_byte).ok();

        if let Some(ass_instr) = assembled_instr_opt {
            let operation = &ass_instr.operation;

            for test_bits in &ass_instr.bytes {}
        }
        i += 1;
    }

    Ok(())
}
