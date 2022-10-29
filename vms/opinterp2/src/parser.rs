use util::{BfError, BfResult};

#[derive(Eq, PartialEq)]
enum OpCode {
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
pub enum Instruction {
    IncPtr { count: usize },
    DecPtr { count: usize },
    IncData { count: usize },
    DecData { count: usize },
    Read { count: usize },
    Write { count: usize },
    JumpIfZero { destination: usize },
    JumpIfNotZero { destination: usize },
}

impl Instruction {
    fn set_destination(&mut self, value: usize) {
        match self {
            Instruction::JumpIfZero { destination } => *destination = value,
            Instruction::JumpIfNotZero { destination } => *destination = value,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Program {
    pub instructions: Vec<Instruction>,
}

pub fn parse(source_code: &str) -> BfResult<Program> {
    let opcodes = translate_to_opcodes(source_code);

    let mut instructions = vec![];

    let mut pc = 0;
    let mut open_bracket_stack = vec![];
    while pc < opcodes.len() {
        match &opcodes[pc] {
            OpCode::JumpIfZero => {
                open_bracket_stack.push(instructions.len());
                instructions.push(Instruction::JumpIfZero { destination: 0 });
                pc += 1;
            }
            OpCode::JumpIfNotZero => {
                if let Some(open_bracket_offset) = open_bracket_stack.pop() {
                    let len = instructions.len();
                    instructions[open_bracket_offset].set_destination(len);
                    instructions.push(Instruction::JumpIfNotZero {
                        destination: open_bracket_offset,
                    });

                    pc += 1;
                } else {
                    return Err(BfError::Bf(format!("unmatched ']' at pc={pc}")));
                }
            }
            opcode => {
                let start = pc;
                pc += 1;
                while pc < opcodes.len() && opcodes[pc] == *opcode {
                    pc += 1;
                }

                let count = pc - start;
                let instruction = match opcode {
                    OpCode::IncPtr => Instruction::IncPtr { count },
                    OpCode::DecPtr => Instruction::DecPtr { count },
                    OpCode::IncData => Instruction::IncData { count },
                    OpCode::DecData => Instruction::DecData { count },
                    OpCode::Read => Instruction::Read { count },
                    OpCode::Write => Instruction::Write { count },
                    _ => unreachable!(),
                };

                instructions.push(instruction);
            }
        }
    }

    if !open_bracket_stack.is_empty() {
        return Err(BfError::Bf("unmatched '['".to_owned()));
    }

    Ok(Program { instructions })
}

fn translate_to_opcodes(source_code: &str) -> Vec<OpCode> {
    let mut opcodes = vec![];

    for c in source_code.chars() {
        match c {
            '>' => opcodes.push(OpCode::IncPtr),
            '<' => opcodes.push(OpCode::DecPtr),
            '+' => opcodes.push(OpCode::IncData),
            '-' => opcodes.push(OpCode::DecData),
            ',' => opcodes.push(OpCode::Read),
            '.' => opcodes.push(OpCode::Write),
            '[' => opcodes.push(OpCode::JumpIfZero),
            ']' => opcodes.push(OpCode::JumpIfNotZero),
            _ => (),
        }
    }

    opcodes
}

#[cfg(test)]
mod tests {
    use {
        super::{parse, Instruction, Program},
        util::BfError,
    };

    #[test]
    fn parse_test() {
        let program = parse(">a<+bcde-,_.[]_1234567890か").unwrap();
        assert_eq!(
            program,
            Program {
                instructions: vec![
                    Instruction::IncPtr { count: 1 },
                    Instruction::DecPtr { count: 1 },
                    Instruction::IncData { count: 1 },
                    Instruction::DecData { count: 1 },
                    Instruction::Read { count: 1 },
                    Instruction::Write { count: 1 },
                    Instruction::JumpIfZero { destination: 7 },
                    Instruction::JumpIfNotZero { destination: 6 },
                ],
            }
        );
    }

    #[test]
    fn parse_count_test() {
        let program = parse(">>[>>><<+---,,],..").unwrap();
        assert_eq!(
            program,
            Program {
                instructions: vec![
                    Instruction::IncPtr { count: 2 },
                    Instruction::JumpIfZero { destination: 7 },
                    Instruction::IncPtr { count: 3 },
                    Instruction::DecPtr { count: 2 },
                    Instruction::IncData { count: 1 },
                    Instruction::DecData { count: 3 },
                    Instruction::Read { count: 2 },
                    Instruction::JumpIfNotZero { destination: 1 },
                    Instruction::Read { count: 1 },
                    Instruction::Write { count: 2 },
                ],
            }
        );
    }

    #[test]
    fn parse_error_test() {
        let err = parse("..[...").unwrap_err();
        assert_eq!(err, BfError::Bf("unmatched '['".to_owned()));

        let err = parse("..]...").unwrap_err();
        assert_eq!(err, BfError::Bf("unmatched ']' at pc=2".to_owned()));
    }
}
