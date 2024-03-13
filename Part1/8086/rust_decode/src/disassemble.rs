use crate::assembled_instruction::{get_assembled_instruction, AssembledInstruction};
use crate::instruction::instruction::Instruction;

use super::Memory;

#[derive(Debug)]
pub struct DisassemblyError;

pub fn disassemble_8086(memory: &Memory, n_bytes_read: usize) -> Result<(), DisassemblyError> {
    let mut assembled_instr_opt: Option<AssembledInstruction> = None;
    let mut _instruction: Instruction;

    for i in 0..n_bytes_read {
        let byte = memory[i];

        if let Some(instr) = assembled_instr_opt {
            println!("{:?}", instr.operation)
        }

        assembled_instr_opt = get_assembled_instruction(byte).ok();
    }

    Ok(())
}
