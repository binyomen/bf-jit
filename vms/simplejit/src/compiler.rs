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
            Instruction::Write => {
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
            Instruction::JumpIfZero => {
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
            Instruction::JumpIfNotZero => {
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
