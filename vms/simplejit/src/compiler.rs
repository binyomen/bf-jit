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
    let mut ops = Assembler::new()?;
    let start = ops.offset();

    // Back up non-volatile registers for the caller.
    if REG_DATA_POINTER_NON_VOLATILE {
        dasm!(ops
            ; push reg_data_ptr
        );
    }

    if STACK_OFFSET > 0 {
        dasm!(ops
            ; sub reg_stack_ptr, STACK_OFFSET
        );
    }

    set_data_pointer_initial_value(&mut ops, runtime);

    let mut open_bracket_stack = vec![];

    for (i, instruction) in program.instructions.into_iter().enumerate() {
        match instruction {
            Instruction::IncPtr => {
                dasm!(ops
                    ; inc reg_data_ptr
                );
            }
            Instruction::DecPtr => {
                dasm!(ops
                    ; dec reg_data_ptr
                );
            }
            Instruction::IncData => {
                dasm!(ops
                    ; add BYTE [reg_data_ptr], 1
                );
            }
            Instruction::DecData => {
                dasm!(ops
                    ; sub BYTE [reg_data_ptr], 1
                );
            }
            Instruction::Read => {
                #[cfg(target_arch = "x86_64")]
                dasm!(ops
                    // Reinterpret as i64, using the same bytes as before.
                    ; mov reg_arg1, QWORD runtime as *const Runtime as i64
                    ; mov reg_temp, QWORD Runtime::read as *const () as i64
                );
                #[cfg(target_arch = "x86")]
                dasm!(ops
                    // Reinterpret as i32, using the same bytes as before.
                    ; mov reg_arg1, DWORD runtime as *const Runtime as i32
                    ; mov reg_temp, DWORD Runtime::read as *const () as i32
                );

                dasm!(ops
                    ; call reg_temp
                    ; mov BYTE [reg_data_ptr], reg_return
                );
            }
            Instruction::Write => {
                #[cfg(target_arch = "x86_64")]
                dasm!(ops
                    // Reinterpret as i64, using the same bytes as before.
                    ; mov reg_arg1, QWORD runtime as *const Runtime as i64
                );
                #[cfg(target_arch = "x86")]
                dasm!(ops
                    // Reinterpret as i32, using the same bytes as before.
                    ; mov reg_arg1, DWORD runtime as *const Runtime as i32
                );

                dasm!(ops
                    ; mov reg_arg2, [reg_data_ptr]
                );

                #[cfg(target_arch = "x86_64")]
                dasm!(ops
                    // Reinterpret as i64, using the same bytes as before.
                    ; mov reg_temp, QWORD Runtime::write as *const () as i64
                );
                #[cfg(target_arch = "x86")]
                dasm!(ops
                    // Reinterpret as i32, using the same bytes as before.
                    ; mov reg_temp, DWORD Runtime::write as *const () as i32
                );

                dasm!(ops
                    ; call reg_temp
                );
            }
            Instruction::JumpIfZero => {
                let begin_label = ops.new_dynamic_label();
                let end_label = ops.new_dynamic_label();
                open_bracket_stack.push(LabelPair {
                    begin_label,
                    end_label,
                });

                dasm!(ops
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

                dasm!(ops
                    ; cmp BYTE [reg_data_ptr], 0
                    ; jnz =>begin_label
                    ; =>end_label
                );
            }
        }
    }

    if STACK_OFFSET > 0 {
        dasm!(ops
            ; add reg_stack_ptr, STACK_OFFSET
        );
    }

    if REG_DATA_POINTER_NON_VOLATILE {
        dasm!(ops
            ; pop reg_data_ptr
        );
    }

    dasm!(ops
        ; ret
    );

    Ok(CompiledProgram::new(ops.finalize()?, start))
}
