use std::collections::HashMap;
use std::fmt::{self, write, Display};

use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::assembled_instruction::Operation::*;
use crate::instruction::instruction::Instruction;

#[derive(Debug)]
pub enum CpuOperand {
    Register(Reg),
    DirectAcces(i16),
    Memory(EffectiveAddress),
    Immediate(i16),
}

#[derive(Debug)]
pub enum EffectiveAddress {
    BxSi(i16),
    BxDi(i16),
    BpSi(i16),
    BpDi(i16),
    Si(i16),
    Di(i16),
    Bp(i16),
    Bx(i16),
}

impl EffectiveAddress {
    pub fn new(value: u8, displacement: Option<i16>) -> Self {
        let displacement = displacement.unwrap_or(0);

        match value {
            0 => Self::BxSi(displacement),
            1 => Self::BxDi(displacement),
            2 => Self::BpSi(displacement),
            3 => Self::BpDi(displacement),
            4 => Self::Si(displacement),
            5 => Self::Di(displacement),
            6 => Self::Bp(displacement),
            7 => Self::Bx(displacement),
            _ => panic!("Unknown effective addres value"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, EnumIter)]
pub enum Reg {
    A,
    B,
    C,
    D,
    Sp,
    Bp,
    Si,
    Di,
}

impl Reg {
    pub fn new(value: u8) -> Self {
        match value {
            0 => Self::A,
            1 => Self::B,
            2 => Self::C,
            3 => Self::D,
            4 => Self::Sp,
            5 => Self::Bp,
            6 => Self::Si,
            7 => Self::Di,
            _ => panic!("Unknown register value"),
        }
    }
}

impl fmt::Display for Reg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let to_write: &str;
        match self {
            Self::A => to_write = "ax",
            Self::B => to_write = "bx",
            Self::C => to_write = "cx",
            Self::D => to_write = "dx",
            Self::Sp => to_write = "sp",
            Self::Bp => to_write = "bp",
            Self::Si => to_write = "si",
            Self::Di => to_write = "di",
        }

        write!(f, "{}", to_write)
    }
}

struct Registers {
    regs: HashMap<Reg, i16>,
}

impl Registers {
    pub fn mov(&mut self, reg: Reg, new: i16) -> () {
        let old = self.regs.insert(reg, new).unwrap_or(0);

        println!("{}:{:#x}->{:#x}", reg, old, new)
    }

    fn reg_to_str(&self, reg: Reg) -> String {
        format!(
            "{}: {value:#x} ({value})",
            reg,
            value = self.regs.get(&reg).unwrap()
        )
    }
}

impl Display for Registers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut result = write!(f, "\nFinal Registers:\n");

        for reg in Reg::iter() {
            result = writeln!(f, "      {}", self.reg_to_str(reg));

            if result.is_err() {
                return result;
            }
        }

        result
    }
}

pub struct CPU {
    registers: Registers,
}

impl CPU {
    pub fn new() -> Self {
        let regs = HashMap::from([
            (Reg::A, 0),
            (Reg::B, 0),
            (Reg::C, 0),
            (Reg::D, 0),
            (Reg::Sp, 0),
            (Reg::Bp, 0),
            (Reg::Si, 0),
            (Reg::Di, 0),
        ]);

        CPU {
            registers: Registers { regs },
        }
    }
    pub fn execute_instruction(&mut self, instr: Instruction) -> () {
        let (dst, src) = match instr.operands_sorted() {
            (dst, src) => (dst.parse_for_cpu(), src.parse_for_cpu()),
        };

        print!("{} ; ", instr);

        match instr.operation() {
            MOV => self.execute_mov(dst, src),
            op => panic!("Unsupported operation {}", op),
        }
    }

    fn execute_mov(&mut self, destination: CpuOperand, source: CpuOperand) -> () {
        let value: i16;

        match source {
            CpuOperand::Immediate(val) => value = val,
            _other => panic!("Not yet supported"),
        };

        match destination {
            CpuOperand::Register(reg) => self.registers.mov(reg, value),
            other => panic!("mov not supported for {:?}", other),
        };
    }
}

impl Display for CPU {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.registers)
    }
}
