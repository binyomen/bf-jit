use util::BfError;

#[derive(Debug, Eq, PartialEq)]
pub enum Instruction {
    IncPtr,
    DecPtr,
    IncData,
    DecData,
    Read,
    Write,
    JumpIfZero,
    JumpIfNotZero,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Program {
    pub instructions: Vec<Instruction>,
    pub jump_table: Vec<usize>,
}

pub fn parse(source_code: &str) -> Result<Program, BfError> {
    let mut instructions = vec![];

    for c in source_code.chars() {
        match c {
            '>' => instructions.push(Instruction::IncPtr),
            '<' => instructions.push(Instruction::DecPtr),
            '+' => instructions.push(Instruction::IncData),
            '-' => instructions.push(Instruction::DecData),
            ',' => instructions.push(Instruction::Read),
            '.' => instructions.push(Instruction::Write),
            '[' => instructions.push(Instruction::JumpIfZero),
            ']' => instructions.push(Instruction::JumpIfNotZero),
            _ => (),
        }
    }

    let jump_table = create_jump_table(&instructions)?;

    Ok(Program {
        instructions,
        jump_table,
    })
}

fn create_jump_table(instructions: &Vec<Instruction>) -> Result<Vec<usize>, BfError> {
    let mut pc = 0;
    let mut jump_table = vec![0; instructions.len()];

    while pc < instructions.len() {
        if instructions[pc] == Instruction::JumpIfZero {
            let mut bracket_nesting = 1;
            let mut seek = pc;

            while bracket_nesting != 0 && seek + 1 < instructions.len() {
                seek += 1;
                if instructions[seek] == Instruction::JumpIfNotZero {
                    bracket_nesting -= 1;
                } else if instructions[seek] == Instruction::JumpIfZero {
                    bracket_nesting += 1;
                }
            }

            if bracket_nesting == 0 {
                jump_table[pc] = seek;
                jump_table[seek] = pc;
            } else {
                return Err(BfError::Bf(format!("unmatched '[' at pc={pc}")));
            }
        }

        pc += 1;
    }

    Ok(jump_table)
}

#[cfg(test)]
mod tests {
    use super::{parse, Instruction, Program};

    #[test]
    fn parse_works() {
        let program = parse(">a<+bcde-,_.[]_1234567890か").unwrap();
        assert_eq!(
            program,
            Program {
                instructions: vec![
                    Instruction::IncPtr,
                    Instruction::DecPtr,
                    Instruction::IncData,
                    Instruction::DecData,
                    Instruction::Read,
                    Instruction::Write,
                    Instruction::JumpIfZero,
                    Instruction::JumpIfNotZero,
                ],
                jump_table: vec![0, 0, 0, 0, 0, 0, 7, 6],
            }
        );
    }
}
