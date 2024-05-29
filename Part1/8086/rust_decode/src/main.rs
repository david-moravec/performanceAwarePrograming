#![allow(dead_code)]

#[macro_use]
extern crate lazy_static;
extern crate bitflags;

use std::fs;
use std::io::{self, Read};

use clap::Parser;

mod assembled_instruction;
mod cpu;
mod disassemble;
mod instruction;

use cpu::cpu::CPU;
use disassemble::disassemble_bytes_in;

const MEMORY_SIZE: usize = 1024 * 1024; //BYTES
const MEMORY_MASK: usize = MEMORY_SIZE - 1;

pub struct InstructionBuffer {
    buf: Vec<u8>,
    last_read: usize,
    bytes_loaded: usize,
}

#[derive(Debug)]
pub struct BufferEndReachedError;

impl InstructionBuffer {
    pub fn new(file_name: &str) -> Result<Self, io::Error> {
        let mut f = fs::File::open(file_name)?;
        let file_size = fs::metadata(file_name)?.len();

        let mut buf: Vec<u8> = vec![0; MEMORY_SIZE];

        f.read(&mut buf)?;

        Ok(InstructionBuffer {
            buf,
            last_read: 0,
            bytes_loaded: file_size as usize & MEMORY_MASK,
        })
    }

    pub fn next_n_bytes(&mut self, n: usize) -> Result<Vec<u8>, BufferEndReachedError> {
        let last_read = self.last_read;
        let read_until = last_read + n;

        if read_until > self.bytes_loaded {
            return Err(BufferEndReachedError);
        }

        self.last_read = read_until;
        Ok(self.buf[last_read..read_until]
            .iter()
            .map(|bit| *bit)
            .collect::<Vec<u8>>())
    }

    pub fn next_byte(&mut self) -> Result<u8, BufferEndReachedError> {
        Ok(self.next_n_bytes(1)?[0])
    }

    pub fn jump_by(&mut self, n: i16) -> () {
        let new = self.last_read as i64 + n as i64;

        if new < 0 {
            panic!("Instruction Buffer overflow")
        } else {
            self.last_read = new as usize
        }
    }

    pub fn is_at_the_end(&self) -> bool {
        self.last_read >= self.bytes_loaded
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
struct Args {
    path: String,

    #[arg(short, long)]
    exec: bool,
}

fn main() {
    let args = Args::parse();

    let buffer = InstructionBuffer::new(&args.path).expect("Loading instruction to buffer failed");

    if args.exec {
        let mut cpu = CPU::new(buffer);

        cpu.execute_instructions().unwrap();

        println!("{}", cpu);
    } else {
        println!("bits 16");

        for instruction in disassemble_bytes_in(buffer).expect("Disassembly of Instructions failed")
        {
            println!("{}", instruction);
        }
    }
}
