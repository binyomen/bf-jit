use {
    crate::parser::{Instruction, Program},
    dynasmrt::{dynasm, DynasmApi, DynasmLabelApi},
    util::{
        dasm,
        jit::{
            call_read, call_write, epilogue, jump_begin, jump_end, prologue, Assembler,
            CompiledProgram, Runtime,
        },
        BfResult,
    },
};

pub fn compile(program: Program, runtime: &mut Runtime) -> BfResult<CompiledProgram> {
    let mut assembler = Assembler::new()?;
    let start = assembler.offset();

    prologue(&mut assembler, runtime);

    let mut open_bracket_stack = vec![];

    for (i, instruction) in program.instructions.into_iter().enumerate() {
        match instruction {
            Instruction::IncPtr { count } => {
                dasm!(assembler
                    // Reinterpret as i32, using the same bytes as before.
                    ; add reg_data_ptr, DWORD count as i32
                );
            }
            Instruction::DecPtr { count } => {
                dasm!(assembler
                    // Reinterpret as i32, using the same bytes as before.
                    ; sub reg_data_ptr, DWORD count as i32
                );
            }
            Instruction::IncData { count } => {
                // Adding 256 is effectively a nop, since it will wrap around to
                // the original value. Mod out 256 to get a value between 0 and
                // 255.
                let wrapped_count = (count % 256) as u8;
                dasm!(assembler
                    // Reinterpret as i8, using the same bytes as before.
                    ; add BYTE [reg_data_ptr], BYTE wrapped_count as i8
                );
            }
            Instruction::DecData { count } => {
                // Subtracting 256 is effectively a nop, since it will wrap
                // around to the original value. Mod out 256 to get a value
                // between 0 and 255.
                let wrapped_count = (count % 256) as u8;
                dasm!(assembler
                    // Reinterpret as i8, using the same bytes as before.
                    ; sub BYTE [reg_data_ptr], BYTE wrapped_count as i8
                );
            }
            Instruction::Read { count } => {
                for _ in 0..count {
                    call_read(&mut assembler, runtime);
                }
            }
            Instruction::Write { count } => {
                for _ in 0..count {
                    call_write(&mut assembler, runtime);
                }
            }
            Instruction::JumpBegin => {
                jump_begin(&mut assembler, &mut open_bracket_stack);
            }
            Instruction::JumpEnd => {
                jump_end(&mut assembler, &mut open_bracket_stack, i)?;
            }
            Instruction::SetDataToZero => {
                dasm!(assembler
                    ; mov BYTE [reg_data_ptr], 0
                );
            }
            Instruction::MovePtrUntilZero {
                count,
                forward,
                amount,
            } => {
                for _ in 0..count {
                    let begin_loop = assembler.new_dynamic_label();
                    let end_loop = assembler.new_dynamic_label();
                    dasm!(assembler
                        ; =>begin_loop
                        ; cmp BYTE [reg_data_ptr], 0
                        ; jz =>end_loop
                    );

                    if forward {
                        dasm!(assembler
                            // Reinterpret as i32, using the same bytes as before.
                            ; add reg_data_ptr, DWORD amount as i32
                        );
                    } else {
                        dasm!(assembler
                            // Reinterpret as i32, using the same bytes as before.
                            ; sub reg_data_ptr, DWORD amount as i32
                        );
                    }

                    dasm!(assembler
                        ; jmp =>begin_loop
                        ; =>end_loop
                    );
                }
            }
            Instruction::MoveData {
                count,
                forward,
                amount,
            } => {
                for _ in 0..count {
                    let skip_move = assembler.new_dynamic_label();
                    dasm!(assembler
                        ; cmp BYTE [reg_data_ptr], 0
                        ; jz =>skip_move
                        ; mov reg_temp_low, BYTE [reg_data_ptr]
                    );

                    // Reinterpret as i32, using the same bytes as before.
                    let amount_i32 = amount as i32;
                    if forward {
                        dasm!(assembler
                            ; add BYTE [reg_data_ptr + amount_i32], reg_temp_low
                        );
                    } else {
                        dasm!(assembler
                            ; add BYTE [reg_data_ptr - amount_i32], reg_temp_low
                        );
                    }

                    dasm!(assembler
                        ; mov BYTE [reg_data_ptr], 0
                        ; =>skip_move
                    );
                }
            }
        }
    }

    epilogue(&mut assembler);

    Ok(CompiledProgram::new(assembler.finalize()?, start))
}
