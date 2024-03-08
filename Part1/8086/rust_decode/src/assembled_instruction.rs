use crate::instruction::instruction::Operation;

type BitField = (u8, usize);

struct InstructionBits {
    opcode: BitField,
    reg: Option<BitField>,
    rm: Option<BitField>,
    mode: Option<BitField>,
    disp_lo: Option<BitField>,
    disp_hi: Option<BitField>,
    data_lo: Option<BitField>,
    data_hi: Option<BitField>,
    s: Option<BitField>,
    w: Option<BitField>,
    d: Option<BitField>,
    v: Option<BitField>,
    z: Option<BitField>,
}

struct AssembledInstruction {
    operation: Operation,
    bits: InstructionBits,
}
