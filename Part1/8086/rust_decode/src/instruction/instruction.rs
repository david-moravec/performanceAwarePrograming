use super::operand::Operand;
use super::singlebits::SingleBits;

pub struct Instruction {
    pub operation: Operation,
    pub operands: [Operand; 2],
    pub single_bits: SingleBits,
}

#[derive(Debug)]
pub enum Operation {
    MOV,
    ADD,
}
