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
            Instruction::IncPtr => data_pointer += 1,
            Instruction::DecPtr => data_pointer -= 1,
            Instruction::IncData => memory[data_pointer] = memory[data_pointer].wrapping_add(1),
            Instruction::DecData => memory[data_pointer] = memory[data_pointer].wrapping_sub(1),
            Instruction::Read => memory[data_pointer] = read(stdin)?,
            Instruction::Write => write(stdout, memory[data_pointer])?,
            Instruction::JumpIfZero => {
                pc = jump(true /*eq_zero*/, &program, &memory, data_pointer, &pc)
            }
            Instruction::JumpIfNotZero => {
                pc = jump(false /*eq_zero*/, &program, &memory, data_pointer, &pc)
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
    program: &Program,
    memory: &[u8; MEMORY_SIZE],
    data_pointer: usize,
    pc: &usize,
) -> usize {
    let cond = if eq_zero {
        |a, b| a == b
    } else {
        |a, b| a != b
    };

    if cond(memory[data_pointer], 0) {
        program.jump_table[*pc]
    } else {
        *pc
    }
}
