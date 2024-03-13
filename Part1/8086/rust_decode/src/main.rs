#![allow(dead_code)]
use std::fs;
use std::io::Read;
use std::{env, io};

mod assembled_instruction;
mod disassemble;
mod instruction;

use disassemble::disassemble_8086;

const MEMORY_SIZE: usize = 1024 * 1024; //BYTES
const MEMORY_MASK: usize = MEMORY_SIZE - 1;

type Memory = Vec<u8>;

fn load_memory_from_file(file_name: &str, memory: &mut Memory) -> Result<usize, io::Error> {
    let mut f = fs::File::open(file_name)?;
    let file_size = fs::metadata(file_name)?.len();

    f.read(memory)?;

    Ok(file_size as usize & MEMORY_MASK)
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        println!("Please provide a binary file to disassemble");
        return;
    }

    let mut memory: Memory = vec![0; MEMORY_SIZE];

    let n_bytes_read = load_memory_from_file(&args[1], &mut memory)
        .expect("Error occured when trying to disassemble");

    disassemble_8086(&memory, n_bytes_read);
}
