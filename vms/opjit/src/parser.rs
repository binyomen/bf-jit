use {
    std::{iter::Peekable, slice, str::CharIndices},
    util::{BfError, BfResult},
};

#[derive(Debug, Eq, PartialEq)]
enum AstNode {
    IncPtr,
    DecPtr,
    IncData,
    DecData,
    Read,
    Write,
    Loop { seq: AstSeq },
    SetDataToZero,
    MovePtrUntilZero { forward: bool, amount: u32 },
    MoveData { forward: bool, amount: u32 },
}

#[derive(Debug, Eq, PartialEq)]
struct AstSeq {
    children: Vec<AstNode>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum Instruction {
    IncPtr {
        count: u32,
    },
    DecPtr {
        count: u32,
    },
    IncData {
        count: u32,
    },
    DecData {
        count: u32,
    },
    Read {
        count: u32,
    },
    Write {
        count: u32,
    },
    JumpBegin,
    JumpEnd,
    SetDataToZero,
    MovePtrUntilZero {
        count: u32,
        forward: bool,
        amount: u32,
    },
    MoveData {
        count: u32,
        forward: bool,
        amount: u32,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub struct Program {
    pub instructions: Vec<Instruction>,
}

pub fn parse(source_code: &str) -> BfResult<Program> {
    let ast = create_ast(source_code)?;
    let ast = optimize_loops(ast);

    let instructions = compile(ast);

    Ok(Program { instructions })
}

fn create_ast(source_code: &str) -> BfResult<AstSeq> {
    fn peek_meaningful_char(chars: &mut Peekable<CharIndices>) -> Option<(usize, char)> {
        while let Some((_i, c)) = chars.peek() {
            match c {
                '>' | '<' | '+' | '-' | ',' | '.' | '[' | ']' => break,
                _ => chars.next(),
            };
        }

        Some(*chars.peek()?)
    }

    fn parse_seq(chars: &mut Peekable<CharIndices>, is_in_loop: bool) -> BfResult<AstSeq> {
        let mut children = vec![];

        while let Some((i, c)) = peek_meaningful_char(chars) {
            // Special handling to break without consuming the character.
            if c == ']' && is_in_loop {
                break;
            }

            chars.next();

            match c {
                '>' => children.push(AstNode::IncPtr),
                '<' => children.push(AstNode::DecPtr),
                '+' => children.push(AstNode::IncData),
                '-' => children.push(AstNode::DecData),
                ',' => children.push(AstNode::Read),
                '.' => children.push(AstNode::Write),
                '[' => {
                    let seq = parse_seq(chars, true /*is_in_loop*/)?;
                    if let Some((_, ']')) = chars.peek() {
                        // Consume the "]".
                        chars.next();
                    } else {
                        return Err(BfError::Bf(format!("Unmatched '[' at index {i}.")));
                    }

                    children.push(AstNode::Loop { seq });
                }
                ']' => return Err(BfError::Bf(format!("Unmatched ']' at index {i}."))),
                _ => unreachable!(),
            }
        }

        Ok(AstSeq { children })
    }

    parse_seq(
        &mut source_code.char_indices().peekable(),
        false, /*is_in_loop*/
    )
}

fn optimize_loops(seq: AstSeq) -> AstSeq {
    fn check_next_node<'a>(
        iter: &'a mut Peekable<slice::Iter<AstNode>>,
        expected_node: &AstNode,
    ) -> Option<&'a AstNode> {
        let node = iter.next()?;
        if node == expected_node {
            Some(node)
        } else {
            None
        }
    }

    fn check_move_ptr_until_zero_pattern(nodes: &[AstNode]) -> Option<(bool, u32)> {
        if nodes.is_empty() {
            None
        } else if nodes.iter().all(|node| node == &AstNode::IncPtr) {
            Some((true /*forward*/, nodes.len().try_into().unwrap()))
        } else if nodes.iter().all(|node| node == &AstNode::DecPtr) {
            Some((false /*forward*/, nodes.len().try_into().unwrap()))
        } else {
            None
        }
    }

    fn check_set_data_to_zero_pattern(nodes: &[AstNode]) -> bool {
        !nodes.is_empty()
            && (nodes.iter().all(|node| node == &AstNode::IncData)
                || nodes.iter().all(|node| node == &AstNode::DecData))
    }

    fn check_move_data_pattern(nodes: &[AstNode]) -> Option<(bool, u32)> {
        let mut iter = nodes.iter().peekable();

        check_next_node(&mut iter, &AstNode::DecData)?;

        let first_direction_node = match iter.next()? {
            AstNode::IncPtr => Some(AstNode::IncPtr),
            AstNode::DecPtr => Some(AstNode::DecPtr),
            _ => None,
        }?;

        let mut first_direction_count = 1;
        while iter.peek()? == &&first_direction_node {
            first_direction_count += 1;
            debug_assert_eq!(iter.peek(), Some(&&first_direction_node));
            iter.next();
        }

        check_next_node(&mut iter, &AstNode::IncData)?;

        let second_direction_node = if first_direction_node == AstNode::IncPtr {
            AstNode::DecPtr
        } else {
            AstNode::IncPtr
        };
        check_next_node(&mut iter, &second_direction_node);

        let mut second_direction_count = 1;
        while iter.peek() == Some(&&second_direction_node) {
            second_direction_count += 1;
            debug_assert_eq!(iter.peek(), Some(&&second_direction_node));
            iter.next();
        }

        if second_direction_count != first_direction_count || iter.next().is_some() {
            None
        } else {
            let forward = first_direction_node == AstNode::IncPtr;
            Some((forward, first_direction_count))
        }
    }

    fn optimize_node(node: AstNode) -> AstNode {
        if let AstNode::Loop { seq } = node {
            if let Some((forward, amount)) = check_move_ptr_until_zero_pattern(&seq.children) {
                AstNode::MovePtrUntilZero { forward, amount }
            } else if check_set_data_to_zero_pattern(&seq.children) {
                AstNode::SetDataToZero
            } else if let Some((forward, amount)) = check_move_data_pattern(&seq.children) {
                AstNode::MoveData { forward, amount }
            } else {
                AstNode::Loop {
                    seq: optimize_seq(seq),
                }
            }
        } else {
            node
        }
    }

    fn optimize_seq(seq: AstSeq) -> AstSeq {
        AstSeq {
            children: seq.children.into_iter().map(optimize_node).collect(),
        }
    }

    optimize_seq(seq)
}

fn compile(seq: AstSeq) -> Vec<Instruction> {
    fn compile_loop(seq: AstSeq, position: usize) -> Vec<Instruction> {
        let children = compile_seq(seq, position + 1);

        let jump_begin = Instruction::JumpBegin;
        let jump_end = Instruction::JumpEnd;

        let mut instructions = vec![jump_begin];
        instructions.extend(children);
        instructions.push(jump_end);

        instructions
    }

    fn compile_node(node: AstNode, count: u32) -> Instruction {
        match node {
            AstNode::IncPtr => Instruction::IncPtr { count },
            AstNode::DecPtr => Instruction::DecPtr { count },
            AstNode::IncData => Instruction::IncData { count },
            AstNode::DecData => Instruction::DecData { count },
            AstNode::Read => Instruction::Read { count },
            AstNode::Write => Instruction::Write { count },
            AstNode::SetDataToZero => Instruction::SetDataToZero,
            AstNode::MovePtrUntilZero { forward, amount } => Instruction::MovePtrUntilZero {
                count,
                forward,
                amount,
            },
            AstNode::MoveData { forward, amount } => Instruction::MoveData {
                count,
                forward,
                amount,
            },
            _ => unreachable!(),
        }
    }

    fn compile_seq(seq: AstSeq, position: usize) -> Vec<Instruction> {
        let mut instructions = vec![];

        let mut iter = seq.children.into_iter().peekable();
        while let Some(node) = iter.next() {
            match node {
                AstNode::Loop { seq } => {
                    instructions.extend(compile_loop(seq, position + instructions.len()))
                }
                _ => {
                    let mut count = 1;
                    while iter.peek() == Some(&node) {
                        count += 1;
                        debug_assert_eq!(iter.peek(), Some(&node));
                        iter.next();
                    }

                    instructions.push(compile_node(node, count));
                }
            }
        }

        instructions
    }

    compile_seq(seq, 0 /*position*/)
}

#[cfg(test)]
mod tests {
    use {
        super::{parse, Instruction, Program},
        util::BfError,
    };

    #[test]
    fn parse_test() {
        let program = parse(">a<+bcde-,_.[]._1[.]234567890か").unwrap();
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
                    Instruction::JumpBegin,
                    Instruction::JumpEnd,
                    Instruction::Write { count: 1 },
                    Instruction::JumpBegin,
                    Instruction::Write { count: 1 },
                    Instruction::JumpEnd,
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
                    Instruction::JumpBegin,
                    Instruction::IncPtr { count: 3 },
                    Instruction::DecPtr { count: 2 },
                    Instruction::IncData { count: 1 },
                    Instruction::DecData { count: 3 },
                    Instruction::Read { count: 2 },
                    Instruction::JumpEnd,
                    Instruction::Read { count: 1 },
                    Instruction::Write { count: 2 },
                ],
            }
        );
    }

    #[test]
    fn nested_loops_test() {
        let program = parse(".[..[.....]...]..").unwrap();
        assert_eq!(
            program,
            Program {
                instructions: vec![
                    Instruction::Write { count: 1 },
                    Instruction::JumpBegin,
                    Instruction::Write { count: 2 },
                    Instruction::JumpBegin,
                    Instruction::Write { count: 5 },
                    Instruction::JumpEnd,
                    Instruction::Write { count: 3 },
                    Instruction::JumpEnd,
                    Instruction::Write { count: 2 },
                ],
            }
        );
    }

    #[test]
    fn optimized_loops_test() {
        assert_eq!(
            parse("[>]").unwrap(),
            Program {
                instructions: vec![Instruction::MovePtrUntilZero {
                    count: 1,
                    forward: true,
                    amount: 1,
                },],
            }
        );
        assert_eq!(
            parse("[<]").unwrap(),
            Program {
                instructions: vec![Instruction::MovePtrUntilZero {
                    count: 1,
                    forward: false,
                    amount: 1,
                },],
            }
        );
        assert_eq!(
            parse("[>>>]").unwrap(),
            Program {
                instructions: vec![Instruction::MovePtrUntilZero {
                    count: 1,
                    forward: true,
                    amount: 3,
                },],
            }
        );
        assert_eq!(
            parse("[<<<]").unwrap(),
            Program {
                instructions: vec![Instruction::MovePtrUntilZero {
                    count: 1,
                    forward: false,
                    amount: 3,
                },],
            }
        );

        assert_eq!(
            parse("[+]").unwrap(),
            Program {
                instructions: vec![Instruction::SetDataToZero,],
            }
        );
        assert_eq!(
            parse("[-]").unwrap(),
            Program {
                instructions: vec![Instruction::SetDataToZero,],
            }
        );
        assert_eq!(
            parse("[++++++]").unwrap(),
            Program {
                instructions: vec![Instruction::SetDataToZero,],
            }
        );
        assert_eq!(
            parse("[-----]").unwrap(),
            Program {
                instructions: vec![Instruction::SetDataToZero,],
            }
        );

        assert_eq!(
            parse("[->+<]").unwrap(),
            Program {
                instructions: vec![Instruction::MoveData {
                    count: 1,
                    forward: true,
                    amount: 1
                },],
            }
        );
        assert_eq!(
            parse("[-<+>]").unwrap(),
            Program {
                instructions: vec![Instruction::MoveData {
                    count: 1,
                    forward: false,
                    amount: 1
                },],
            }
        );
        assert_eq!(
            parse("[->>>>+<<<<]").unwrap(),
            Program {
                instructions: vec![Instruction::MoveData {
                    count: 1,
                    forward: true,
                    amount: 4
                },],
            }
        );
        assert_eq!(
            parse("[-<<<<+>>>>]").unwrap(),
            Program {
                instructions: vec![Instruction::MoveData {
                    count: 1,
                    forward: false,
                    amount: 4
                },],
            }
        );

        assert_eq!(
            parse("[>][>]").unwrap(),
            Program {
                instructions: vec![Instruction::MovePtrUntilZero {
                    count: 2,
                    forward: true,
                    amount: 1,
                },],
            }
        );
        assert_eq!(
            parse("[<][<]").unwrap(),
            Program {
                instructions: vec![Instruction::MovePtrUntilZero {
                    count: 2,
                    forward: false,
                    amount: 1,
                },],
            }
        );
        assert_eq!(
            parse("[+][+]").unwrap(),
            Program {
                instructions: vec![Instruction::SetDataToZero,],
            }
        );
        assert_eq!(
            parse("[-][-]").unwrap(),
            Program {
                instructions: vec![Instruction::SetDataToZero,],
            }
        );
        assert_eq!(
            parse("[->+<][->+<]").unwrap(),
            Program {
                instructions: vec![Instruction::MoveData {
                    count: 2,
                    forward: true,
                    amount: 1,
                },],
            }
        );
        assert_eq!(
            parse("[-<+>][-<+>]").unwrap(),
            Program {
                instructions: vec![Instruction::MoveData {
                    count: 2,
                    forward: false,
                    amount: 1,
                },],
            }
        );

        assert_eq!(
            parse("[<<<>]").unwrap(),
            Program {
                instructions: vec![
                    Instruction::JumpBegin,
                    Instruction::DecPtr { count: 3 },
                    Instruction::IncPtr { count: 1 },
                    Instruction::JumpEnd,
                ],
            }
        );
        assert_eq!(
            parse("[+++-]").unwrap(),
            Program {
                instructions: vec![
                    Instruction::JumpBegin,
                    Instruction::IncData { count: 3 },
                    Instruction::DecData { count: 1 },
                    Instruction::JumpEnd,
                ],
            }
        );
        assert_eq!(
            parse("[->>+<]").unwrap(),
            Program {
                instructions: vec![
                    Instruction::JumpBegin,
                    Instruction::DecData { count: 1 },
                    Instruction::IncPtr { count: 2 },
                    Instruction::IncData { count: 1 },
                    Instruction::DecPtr { count: 1 },
                    Instruction::JumpEnd,
                ],
            }
        );
        assert_eq!(
            parse("[->+<<]").unwrap(),
            Program {
                instructions: vec![
                    Instruction::JumpBegin,
                    Instruction::DecData { count: 1 },
                    Instruction::IncPtr { count: 1 },
                    Instruction::IncData { count: 1 },
                    Instruction::DecPtr { count: 2 },
                    Instruction::JumpEnd,
                ],
            }
        );
        assert_eq!(
            parse("[->>>+<<<<]").unwrap(),
            Program {
                instructions: vec![
                    Instruction::JumpBegin,
                    Instruction::DecData { count: 1 },
                    Instruction::IncPtr { count: 3 },
                    Instruction::IncData { count: 1 },
                    Instruction::DecPtr { count: 4 },
                    Instruction::JumpEnd,
                ],
            }
        );
        assert_eq!(
            parse("[->+<.]").unwrap(),
            Program {
                instructions: vec![
                    Instruction::JumpBegin,
                    Instruction::DecData { count: 1 },
                    Instruction::IncPtr { count: 1 },
                    Instruction::IncData { count: 1 },
                    Instruction::DecPtr { count: 1 },
                    Instruction::Write { count: 1 },
                    Instruction::JumpEnd,
                ],
            }
        );
    }

    #[test]
    fn nested_optimized_loops_test() {
        assert_eq!(
            parse("..[...[++]..].").unwrap(),
            Program {
                instructions: vec![
                    Instruction::Write { count: 2 },
                    Instruction::JumpBegin,
                    Instruction::Write { count: 3 },
                    Instruction::SetDataToZero,
                    Instruction::Write { count: 2 },
                    Instruction::JumpEnd,
                    Instruction::Write { count: 1 },
                ],
            }
        );
    }

    #[test]
    fn parse_error_test() {
        let err = parse("..[...").unwrap_err();
        assert_eq!(err, BfError::Bf("Unmatched '[' at index 2.".to_owned()));

        let err = parse("...]...").unwrap_err();
        assert_eq!(err, BfError::Bf("Unmatched ']' at index 3.".to_owned()));
    }
}
