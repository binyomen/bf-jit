use {
    crate::parser::{Instruction, Program},
    std::io::{Read, Write},
    util::{
        math::{unbalanced_wrapping_add, unbalanced_wrapping_sub},
        BfResult,
    },
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
                memory[data_pointer] = unbalanced_wrapping_add(memory[data_pointer], count);
            }
            Instruction::DecData { count } => {
                memory[data_pointer] = unbalanced_wrapping_sub(memory[data_pointer], count);
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
            Instruction::JumpBegin { destination } => {
                pc = jump(
                    true, /*eq_zero*/
                    &memory,
                    data_pointer,
                    destination,
                    pc,
                )
            }
            Instruction::JumpEnd { destination } => {
                pc = jump(
                    false, /*eq_zero*/
                    &memory,
                    data_pointer,
                    destination,
                    pc,
                )
            }
            Instruction::SetDataToZero => memory[data_pointer] = 0,
            Instruction::MovePtrUntilZero {
                count,
                forward,
                amount,
            } => {
                for _ in 0..count {
                    while memory[data_pointer] != 0 {
                        if forward {
                            data_pointer += amount
                        } else {
                            data_pointer -= amount
                        }
                    }
                }
            }
            Instruction::MoveData {
                count,
                forward,
                amount,
            } => {
                for _ in 0..count {
                    if memory[data_pointer] != 0 {
                        let move_to_ptr = if forward {
                            data_pointer + amount
                        } else {
                            data_pointer - amount
                        };

                        memory[move_to_ptr] =
                            memory[move_to_ptr].wrapping_add(memory[data_pointer]);
                        memory[data_pointer] = 0;
                    }
                }
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
