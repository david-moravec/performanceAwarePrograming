use super::operand::Operand;
use super::singlebits::SingleBits;

//Instruction that has
pub struct Instruction {
    operation: Operation,
    operands: [Operand; 2],
    single_bits: SingleBits,
}

type InstructionAlt = Instruction;

//binary encodinig of opcode
pub enum Operation {
    MOV,
}
