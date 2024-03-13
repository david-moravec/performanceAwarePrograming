use super::operand::Operand;
use super::singlebits::SingleBits;

pub struct Instruction {
    operation: Operation,
    operands: [Operand; 2],
    single_bits: SingleBits,
}

pub enum Operation {
    MOV,
}
