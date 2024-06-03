use std::collections::HashMap;
use std::fmt::{self, Display};
use std::fs::File;
use std::io::Write;
use std::u16;

use bitflags::{bitflags, parser::to_writer};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::assembled_instruction::Operation::*;
use crate::disassemble::{disassemble_next_instruction, DisassemblyResult};
use crate::InstructionBuffer;
use crate::MEMORY_SIZE;

#[derive(Debug, Clone, Copy)]
pub enum CpuOperand {
    Register(Reg),
    Memory(Access),
    Immediate(i16),
    Jump(i16),
    NotUsed,
}

#[derive(Debug, Clone, Copy)]
pub enum Access {
    Address(EffectiveAddress),
    Direct(usize),
}

#[derive(Debug, Clone, Copy)]
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
            1 => Self::C,
            2 => Self::D,
            3 => Self::B,
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

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    struct CpuFlags: u8 {
        const S = 0b001;
        const Z = 0b010;
        const ZERO =0b000;
    }
}

impl CpuFlags {
    pub fn is_flag_toogled(&self, flag: CpuFlags) -> bool {
        *self & flag == flag
    }
}

impl Display for CpuFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        to_writer(self, f)
    }
}

struct Registers {
    regs: HashMap<Reg, i16>,
}

impl Registers {
    pub fn mov(&mut self, reg: Reg, new: i16) -> () {
        let old = self.regs.insert(reg, new).unwrap_or(0);

        print!("{}:{:#x}->{:#x}", reg, old, new)
    }

    pub fn content_of(&self, reg: Reg) -> i16 {
        *self.regs.get(&reg).unwrap()
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
        let mut result = write!(f, "\n\nFinal Registers:\n");

        for reg in Reg::iter() {
            result = writeln!(f, "      {}", self.reg_to_str(reg));

            if result.is_err() {
                return result;
            }
        }

        result
    }
}

struct Memory {
    mem: Vec<u8>,
}

impl Memory {
    pub fn new() -> Self {
        Memory {
            mem: vec![0 as u8; MEMORY_SIZE],
        }
    }

    fn value_at(&self, index: usize) -> i16 {
        let low = self.mem[index];
        let high = self.mem[index + 1];

        ((high as i16) << 8) | low as i16
    }

    fn save_value_at(&mut self, index: usize, value: i16) -> () {
        self.mem[index] = value as u8;
        self.mem[index + 1] = (value >> 8) as u8;
    }
}

pub struct CPU {
    registers: Registers,
    flags: CpuFlags,
    buffer: InstructionBuffer,
    memory: Memory,
}

impl CPU {
    pub fn new(buffer: InstructionBuffer) -> Self {
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
            flags: CpuFlags::ZERO,
            buffer,
            memory: Memory {
                mem: vec![0 as u8; MEMORY_SIZE],
            },
        }
    }

    pub fn execute_instructions(&mut self) -> DisassemblyResult<()> {
        while !self.buffer.is_at_the_end() {
            self.execute_next_instruction()?;
        }

        Ok(())
    }

    fn execute_next_instruction(&mut self) -> DisassemblyResult<()> {
        let old_ip = self.buffer.last_read;

        let instr = disassemble_next_instruction(&mut self.buffer)?;

        let (dst, src) = match instr.operands_sorted() {
            (dst, src) => (dst.parse_for_cpu(), src.parse_for_cpu()),
        };

        print!(
            "\n{} ; ip:{:#x}->{:#x} ",
            instr, old_ip, self.buffer.last_read
        );

        match instr.operation() {
            MOV => Ok(self.execute_mov(dst, src)),
            ADD => Ok(self.execute_add(dst, src)),
            SUB => Ok(self.execute_sub(dst, src)),
            CMP => Ok(self.execute_cmp(dst, src)),
            JNZ => Ok(self.execute_jnz(src)),
            _ => todo!(),
        }
    }

    pub fn dump_memory(&self) -> std::io::Result<()> {
        let mut file = File::create("memory_dump.data")?;
        file.write_all(&self.memory.mem)?;
        Ok(())
    }

    fn value(&self, source: CpuOperand) -> i16 {
        match source {
            CpuOperand::Immediate(val) => val,
            CpuOperand::Register(reg) => self.registers.content_of(reg),
            CpuOperand::Memory(access) => self.memory.value_at(self.access_to_index(access)),
            _ => todo!(),
        }
    }

    fn access_to_index(&self, access: Access) -> usize {
        match access {
            Access::Direct(index) => index,
            Access::Address(addr) => self.address_to_index(addr),
        }
    }

    fn address_to_index(&self, address: EffectiveAddress) -> usize {
        let result = match address {
            EffectiveAddress::Si(displacement) => self.registers.content_of(Reg::Si) + displacement,
            EffectiveAddress::Di(displacement) => self.registers.content_of(Reg::Di) + displacement,
            EffectiveAddress::Bp(displacement) => self.registers.content_of(Reg::Bp) + displacement,
            EffectiveAddress::Bx(displacement) => self.registers.content_of(Reg::B) + displacement,
            EffectiveAddress::BpSi(displacement) => {
                self.registers.content_of(Reg::Bp)
                    + self.registers.content_of(Reg::Si)
                    + displacement
            }
            EffectiveAddress::BxSi(displacement) => {
                self.registers.content_of(Reg::B)
                    + self.registers.content_of(Reg::Si)
                    + displacement
            }
            EffectiveAddress::BxDi(displacement) => {
                self.registers.content_of(Reg::B)
                    + self.registers.content_of(Reg::Di)
                    + displacement
            }
            EffectiveAddress::BpDi(displacement) => {
                self.registers.content_of(Reg::Bp)
                    + self.registers.content_of(Reg::Di)
                    + displacement
            }
        };

        result.try_into().unwrap()
    }

    fn put_value_in_destination(&mut self, destination: CpuOperand, value: i16) -> () {
        match destination {
            CpuOperand::Register(reg) => self.registers.mov(reg, value),
            CpuOperand::Memory(access) => self.save_in_mem(access, value),
            _ => todo!(),
        };
    }

    fn save_in_mem(&mut self, access: Access, value: i16) -> () {
        self.memory
            .save_value_at(self.access_to_index(access), value)
    }

    fn flip_flags(&mut self, value: i16) -> () {
        let flags_before = self.flags.clone();

        if value == 0 {
            self.flip_flag(CpuFlags::Z)
        } else if (value as u16) & 0x8000 != 0 {
            self.flip_flag(CpuFlags::S)
        }

        if value != 0 {
            self.unflip_flag(CpuFlags::Z)
        }

        if value as u16 & 0x8000 == 0 {
            self.unflip_flag(CpuFlags::S)
        }

        if flags_before != self.flags {
            print!("   flags:{} -> {}", flags_before, self.flags)
        }
    }

    fn flip_flag(&mut self, flag: CpuFlags) -> () {
        self.flags = self.flags | flag;
    }

    fn unflip_flag(&mut self, flag: CpuFlags) -> () {
        self.flags = self.flags & !flag
    }

    fn execute_mov(&mut self, destination: CpuOperand, source: CpuOperand) -> () {
        self.execute(destination, source, |_d, s| s, true, false)
    }

    fn execute_add(&mut self, destination: CpuOperand, source: CpuOperand) -> () {
        self.execute(destination, source, |d, s| d + s, true, true)
    }

    fn execute_sub(&mut self, destination: CpuOperand, source: CpuOperand) -> () {
        self.execute(destination, source, |d, s| d - s, true, true)
    }

    fn execute_cmp(&mut self, destination: CpuOperand, source: CpuOperand) -> () {
        self.execute(destination, source, |d, s| d - s, false, true)
    }

    fn execute_jnz(&mut self, jump_operand: CpuOperand) -> () {
        match jump_operand {
            CpuOperand::Jump(jmp) => {
                if !self.flags.is_flag_toogled(CpuFlags::Z) {
                    self.buffer.jump_by(jmp);
                }
            }
            _ => panic!("Not a jump instruction"),
        }
    }

    fn execute<F>(
        &mut self,
        destination: CpuOperand,
        source: CpuOperand,
        operation: F,
        save_to_dest: bool,
        check_flags: bool,
    ) -> ()
    where
        F: Fn(i16, i16) -> i16,
    {
        let value = operation(self.value(destination), self.value(source));

        if save_to_dest {
            self.put_value_in_destination(destination, value);
        }

        if check_flags {
            self.flip_flags(value)
        }
    }
}

impl Display for CPU {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}      ip: {:#x} ({})\n   flags:{}",
            self.registers, self.buffer.last_read, self.buffer.last_read, self.flags
        )
    }
}
