use {
    crate::parser::{Instruction, Program},
    dynasmrt::{dynasm, DynasmApi, DynasmLabelApi},
    util::{
        dasm,
        jit::{
            set_data_pointer_initial_value, Assembler, CompiledProgram, Runtime,
            REG_DATA_POINTER_NON_VOLATILE, STACK_OFFSET,
        },
        BfError, BfResult,
    },
};

struct LabelPair {
    begin_label: dynasmrt::DynamicLabel,
    end_label: dynasmrt::DynamicLabel,
}

pub fn compile(program: Program, runtime: &mut Runtime) -> BfResult<CompiledProgram> {
    let mut assembler = Assembler::new()?;
    let start = assembler.offset();

    // Back up non-volatile registers for the caller.
    if REG_DATA_POINTER_NON_VOLATILE {
        dasm!(assembler
            ; push reg_data_ptr
        );
    }

    if STACK_OFFSET > 0 {
        dasm!(assembler
            ; sub reg_stack_ptr, STACK_OFFSET
        );
    }

    set_data_pointer_initial_value(&mut assembler, runtime);

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
                    #[cfg(target_arch = "x86_64")]
                    dasm!(assembler
                        // Reinterpret as i64, using the same bytes as before.
                        ; mov reg_arg1, QWORD runtime as *const Runtime as i64
                        ; mov reg_temp, QWORD Runtime::read as *const () as i64
                    );
                    #[cfg(target_arch = "x86")]
                    dasm!(assembler
                        // Reinterpret as i32, using the same bytes as before.
                        ; mov reg_arg1, DWORD runtime as *const Runtime as i32
                        ; mov reg_temp, DWORD Runtime::read as *const () as i32
                    );

                    dasm!(assembler
                        ; call reg_temp
                        ; mov BYTE [reg_data_ptr], reg_return
                    );
                }
            }
            Instruction::Write { count } => {
                for _ in 0..count {
                    #[cfg(target_arch = "x86_64")]
                    dasm!(assembler
                        // Reinterpret as i64, using the same bytes as before.
                        ; mov reg_arg1, QWORD runtime as *const Runtime as i64
                    );
                    #[cfg(target_arch = "x86")]
                    dasm!(assembler
                        // Reinterpret as i32, using the same bytes as before.
                        ; mov reg_arg1, DWORD runtime as *const Runtime as i32
                    );

                    dasm!(assembler
                        ; mov reg_arg2, [reg_data_ptr]
                    );

                    #[cfg(target_arch = "x86_64")]
                    dasm!(assembler
                        // Reinterpret as i64, using the same bytes as before.
                        ; mov reg_temp, QWORD Runtime::write as *const () as i64
                    );
                    #[cfg(target_arch = "x86")]
                    dasm!(assembler
                        // Reinterpret as i32, using the same bytes as before.
                        ; mov reg_temp, DWORD Runtime::write as *const () as i32
                    );

                    dasm!(assembler
                        ; call reg_temp
                    );
                }
            }
            Instruction::JumpBegin => {
                let begin_label = assembler.new_dynamic_label();
                let end_label = assembler.new_dynamic_label();
                open_bracket_stack.push(LabelPair {
                    begin_label,
                    end_label,
                });

                dasm!(assembler
                    ; cmp BYTE [reg_data_ptr], 0
                    ; jz =>end_label
                    ; =>begin_label
                );
            }
            Instruction::JumpEnd => {
                let LabelPair {
                    begin_label,
                    end_label,
                } = open_bracket_stack.pop().ok_or_else(|| {
                    BfError::Bf(format!("Unmatched closing ']' at position {i}."))
                })?;

                dasm!(assembler
                    ; cmp BYTE [reg_data_ptr], 0
                    ; jnz =>begin_label
                    ; =>end_label
                );
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

    if STACK_OFFSET > 0 {
        dasm!(assembler
            ; add reg_stack_ptr, STACK_OFFSET
        );
    }

    if REG_DATA_POINTER_NON_VOLATILE {
        dasm!(assembler
            ; pop reg_data_ptr
        );
    }

    dasm!(assembler
        ; ret
    );

    Ok(CompiledProgram::new(assembler.finalize()?, start))
}
