use {
    crate::parser::{Instruction, Program},
    dynasmrt::{dynasm, DynasmApi, DynasmLabelApi},
    util::{
        asm::{
            call_read, call_write, epilogue, jump_begin, jump_end, prologue, Assembler,
            CompiledProgram, Runtime,
        },
        dasm, BfResult,
    },
};

#[cfg(target_arch = "aarch64")]
use util::add_sub_u64;

pub fn compile(program: Program, runtime: &mut Runtime) -> BfResult<CompiledProgram> {
    let mut assembler = Assembler::new()?;
    let start = assembler.offset();

    prologue(&mut assembler, runtime);

    let mut open_bracket_stack = vec![];

    for (i, instruction) in program.instructions.into_iter().enumerate() {
        match instruction {
            Instruction::IncPtr { count } => {
                #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
                dasm!(assembler
                    // Reinterpret as i32, using the same bytes as before.
                    ; add reg_data_ptr, DWORD count as i32
                );
                #[cfg(target_arch = "aarch64")]
                add_sub_u64!(assembler, add, reg_data_ptr, reg_data_ptr, count.into());
            }
            Instruction::DecPtr { count } => {
                #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
                dasm!(assembler
                    // Reinterpret as i32, using the same bytes as before.
                    ; sub reg_data_ptr, DWORD count as i32
                );
                #[cfg(target_arch = "aarch64")]
                add_sub_u64!(assembler, sub, reg_data_ptr, reg_data_ptr, count.into());
            }
            Instruction::IncData { count } => {
                #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
                {
                    // Adding 256 is effectively a nop, since it will wrap around to
                    // the original value. Mod out 256 to get a value between 0 and
                    // 255.
                    let wrapped_count = (count % 256) as u8;
                    dasm!(assembler
                        // Reinterpret as i8, using the same bytes as before.
                        ; add BYTE [reg_data_ptr], BYTE wrapped_count as i8
                    );
                }
                #[cfg(target_arch = "aarch64")]
                dasm!(assembler
                    ; ldrb reg_temp_low, [reg_data_ptr]
                    ;; add_sub_u64!(assembler, add, reg_temp_low, reg_temp_low, count.into())
                    ; strb reg_temp_low, [reg_data_ptr]
                );
            }
            Instruction::DecData { count } => {
                #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
                {
                    // Subtracting 256 is effectively a nop, since it will wrap
                    // around to the original value. Mod out 256 to get a value
                    // between 0 and 255.
                    let wrapped_count = (count % 256) as u8;
                    dasm!(assembler
                        // Reinterpret as i8, using the same bytes as before.
                        ; sub BYTE [reg_data_ptr], BYTE wrapped_count as i8
                    );
                }
                #[cfg(target_arch = "aarch64")]
                dasm!(assembler
                    ; ldrb reg_temp_low, [reg_data_ptr]
                    ;; add_sub_u64!(assembler, sub, reg_temp_low, reg_temp_low, count.into())
                    ; strb reg_temp_low, [reg_data_ptr]
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
                #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
                dasm!(assembler
                    ; mov BYTE [reg_data_ptr], 0
                );
                #[cfg(target_arch = "aarch64")]
                dasm!(assembler
                    ; strb wzr, [reg_data_ptr]
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
                    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
                    dasm!(assembler
                        ; =>begin_loop
                        ; cmp BYTE [reg_data_ptr], 0
                        ; jz =>end_loop
                    );
                    #[cfg(target_arch = "aarch64")]
                    dasm!(assembler
                        ; =>begin_loop
                        ; ldrb reg_temp_low, [reg_data_ptr]
                        ; cmp reg_temp_low, 0
                        ; b.eq =>end_loop
                    );

                    if forward {
                        #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
                        dasm!(assembler
                            // Reinterpret as i32, using the same bytes as before.
                            ; add reg_data_ptr, DWORD amount as i32
                        );
                        #[cfg(target_arch = "aarch64")]
                        add_sub_u64!(assembler, add, reg_data_ptr, reg_data_ptr, amount.into());
                    } else {
                        #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
                        dasm!(assembler
                            // Reinterpret as i32, using the same bytes as before.
                            ; sub reg_data_ptr, DWORD amount as i32
                        );
                        #[cfg(target_arch = "aarch64")]
                        add_sub_u64!(assembler, sub, reg_data_ptr, reg_data_ptr, amount.into());
                    }

                    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
                    dasm!(assembler
                        ; jmp =>begin_loop
                        ; =>end_loop
                    );
                    #[cfg(target_arch = "aarch64")]
                    dasm!(assembler
                        ; b =>begin_loop
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

                    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
                    dasm!(assembler
                        ; cmp BYTE [reg_data_ptr], 0
                        ; jz =>skip_move
                        ; mov reg_temp_low, BYTE [reg_data_ptr]
                    );
                    #[cfg(target_arch = "aarch64")]
                    dasm!(assembler
                        ; ldrb reg_temp2_low, [reg_data_ptr]
                        ; cmp reg_temp2_low, 0
                        ; b.eq =>skip_move
                        ; ldrb reg_temp_low, [reg_data_ptr]
                    );

                    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
                    let amount_i32: i32 = amount.try_into()?;
                    if forward {
                        #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
                        dasm!(assembler
                            ; add BYTE [reg_data_ptr + amount_i32], reg_temp_low
                        );
                        #[cfg(target_arch = "aarch64")]
                        dasm!(assembler
                            // Put effective address in a temp register so we
                            // can actually have a u32 of offset.
                            ;; add_sub_u64!(assembler, add, reg_temp3, reg_data_ptr, amount.into())
                            ; ldrb reg_temp2_low, [reg_temp3]
                            ; add reg_temp2_low, reg_temp2_low, reg_temp_low
                            ; strb reg_temp2_low, [reg_temp3]
                        );
                    } else {
                        #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
                        dasm!(assembler
                            ; add BYTE [reg_data_ptr - amount_i32], reg_temp_low
                        );
                        #[cfg(target_arch = "aarch64")]
                        dasm!(assembler
                            // Put effective address in a temp register so we
                            // can actually have a u32 of offset.
                            ;; add_sub_u64!(assembler, sub, reg_temp3, reg_data_ptr, amount.into())
                            ; ldrb reg_temp2_low, [reg_temp3]
                            ; add reg_temp2_low, reg_temp2_low, reg_temp_low
                            ; strb reg_temp2_low, [reg_temp3]
                        );
                    }

                    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
                    dasm!(assembler
                        ; mov BYTE [reg_data_ptr], 0
                        ; =>skip_move
                    );
                    #[cfg(target_arch = "aarch64")]
                    dasm!(assembler
                        ; strb wzr, [reg_data_ptr]
                        ; =>skip_move
                    );
                }
            }
        }
    }

    epilogue(&mut assembler);

    Ok(CompiledProgram::new(assembler.finalize()?, start))
}
