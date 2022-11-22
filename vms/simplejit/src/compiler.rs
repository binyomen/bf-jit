use {
    crate::parser::{Instruction, Program},
    dynasmrt::{dynasm, DynasmApi},
    util::{
        asm::{
            call_read, call_write, epilogue, jump_begin, jump_end, prologue, Assembler,
            CompiledProgram, Runtime,
        },
        dasm, BfResult,
    },
};

pub fn compile(program: Program, runtime: &mut Runtime) -> BfResult<CompiledProgram> {
    let mut assembler = Assembler::new()?;
    let start = assembler.offset();

    prologue(&mut assembler, runtime);

    let mut open_bracket_stack = vec![];

    for (i, instruction) in program.instructions.into_iter().enumerate() {
        match instruction {
            Instruction::IncPtr => {
                dasm!(assembler
                    ; inc reg_data_ptr
                );
            }
            Instruction::DecPtr => {
                dasm!(assembler
                    ; dec reg_data_ptr
                );
            }
            Instruction::IncData => {
                dasm!(assembler
                    ; add BYTE [reg_data_ptr], 1
                );
            }
            Instruction::DecData => {
                dasm!(assembler
                    ; sub BYTE [reg_data_ptr], 1
                );
            }
            Instruction::Read => {
                call_read(&mut assembler, runtime);
            }
            Instruction::Write => {
                call_write(&mut assembler, runtime);
            }
            Instruction::JumpIfZero => {
                jump_begin(&mut assembler, &mut open_bracket_stack);
            }
            Instruction::JumpIfNotZero => {
                jump_end(&mut assembler, &mut open_bracket_stack, i)?;
            }
        }
    }

    epilogue(&mut assembler);

    Ok(CompiledProgram::new(assembler.finalize()?, start))
}
