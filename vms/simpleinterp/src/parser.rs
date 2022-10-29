#[derive(Debug, Eq, PartialEq)]
pub enum Instruction {
    IncPtr,
    DecPtr,
    IncData,
    DecData,
    Write,
    Read,
    JumpIfZero,
    JumpIfNotZero,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Program {
    pub instructions: Vec<Instruction>,
}

pub fn parse(source_code: &str) -> Program {
    let mut instructions = vec![];

    for c in source_code.chars() {
        match c {
            '>' => instructions.push(Instruction::IncPtr),
            '<' => instructions.push(Instruction::DecPtr),
            '+' => instructions.push(Instruction::IncData),
            '-' => instructions.push(Instruction::DecData),
            '.' => instructions.push(Instruction::Write),
            ',' => instructions.push(Instruction::Read),
            '[' => instructions.push(Instruction::JumpIfZero),
            ']' => instructions.push(Instruction::JumpIfNotZero),
            _ => (),
        }
    }

    Program { instructions }
}

#[cfg(test)]
mod tests {
    use super::{parse, Instruction, Program};

    #[test]
    fn parse_works() {
        let program = parse(">a<+bcde-._,[]_1234567890か");
        assert_eq!(
            program,
            Program {
                instructions: vec![
                    Instruction::IncPtr,
                    Instruction::DecPtr,
                    Instruction::IncData,
                    Instruction::DecData,
                    Instruction::Write,
                    Instruction::Read,
                    Instruction::JumpIfZero,
                    Instruction::JumpIfNotZero,
                ],
            }
        );
    }
}
