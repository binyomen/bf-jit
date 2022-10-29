use {
    crate::parser::{Instruction, Program},
    std::io::{Read, Write},
    util::BfResult,
};

const MEMORY_SIZE: usize = 30000;

pub fn run(program: Program, stdin: &mut dyn Read, stdout: &mut dyn Write) -> BfResult<()> {
    let mut memory: [u8; MEMORY_SIZE] = [0; MEMORY_SIZE];
    let mut pc = 0;
    let mut data_pointer = 0;

    while pc < program.instructions.len() {
        match program.instructions[pc] {
            Instruction::IncPtr { count } => data_pointer += count,
            Instruction::DecPtr { count } => data_pointer -= count,
            Instruction::IncData { count } => {
                let mut val = memory[data_pointer];
                for _ in 0..count {
                    val = val.wrapping_add(1)
                }
                memory[data_pointer] = val;
            }
            Instruction::DecData { count } => {
                let mut val = memory[data_pointer];
                for _ in 0..count {
                    val = val.wrapping_sub(1)
                }
                memory[data_pointer] = val;
            }
            Instruction::Read { count } => {
                for _ in 0..count {
                    memory[data_pointer] = read(stdin)?
                }
            }
            Instruction::Write { count } => {
                for _ in 0..count {
                    write(stdout, memory[data_pointer])?
                }
            }
            Instruction::JumpIfZero { destination } => {
                pc = jump(
                    true, /*eq_zero*/
                    &memory,
                    data_pointer,
                    destination,
                    pc,
                )
            }
            Instruction::JumpIfNotZero { destination } => {
                pc = jump(
                    false, /*eq_zero*/
                    &memory,
                    data_pointer,
                    destination,
                    pc,
                )
            }
        }

        pc += 1;
    }

    Ok(())
}

fn read(stdin: &mut dyn Read) -> BfResult<u8> {
    let mut c = [0; 1];
    stdin.read_exact(&mut c)?;

    Ok(c[0])
}

fn write(stdout: &mut dyn Write, byte: u8) -> BfResult<()> {
    stdout.write_all(&[byte])?;
    stdout.flush()?;

    Ok(())
}

fn jump(
    eq_zero: bool,
    memory: &[u8; MEMORY_SIZE],
    data_pointer: usize,
    destination: usize,
    pc: usize,
) -> usize {
    let cond = if eq_zero {
        |a, b| a == b
    } else {
        |a, b| a != b
    };

    if cond(memory[data_pointer], 0) {
        destination
    } else {
        pc
    }
}
