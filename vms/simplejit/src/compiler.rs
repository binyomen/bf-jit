use {
    crate::{
        parser::{Instruction, Program},
        runtime::Runtime,
    },
    dynasmrt::{
        dynasm, x64::Assembler, AssemblyOffset, DynasmApi, DynasmLabelApi, ExecutableBuffer,
    },
    util::{BfError, BfResult},
};

#[cfg(any(target_os = "linux", target_os = "macos"))]
macro_rules! dasm {
    ($ops:ident $($t:tt)*) => {
        dynasm!($ops
            ; .arch x64
            ; .alias reg_data_ptr, r13
            ; .alias reg_arg1, rdi
            ; .alias reg_arg2, rsi
            ; .alias reg_function_call, r8
            ; .alias reg_return, al
            $($t)*
        )
    }
}

#[cfg(target_os = "windows")]
macro_rules! dasm {
    ($ops:ident $($t:tt)*) => {
        dynasm!($ops
            ; .arch x64
            ; .alias reg_data_ptr, r13
            ; .alias reg_arg1, rcx
            ; .alias reg_arg2, rdx
            ; .alias reg_function_call, r8
            ; .alias reg_return, al
            $($t)*
        )
    }
}

pub struct CompiledProgram {
    buffer: ExecutableBuffer,
    start: AssemblyOffset,
}

impl CompiledProgram {
    pub fn function_ptr(&self) -> *const () {
        self.buffer.ptr(self.start) as *const ()
    }
}

struct LabelPair {
    begin_label: dynasmrt::DynamicLabel,
    end_label: dynasmrt::DynamicLabel,
}

pub fn compile(program: Program, runtime: &mut Runtime) -> BfResult<CompiledProgram> {
    let mut ops = Assembler::new()?;
    let start = ops.offset();

    // reg_data_ptr (r13) is non-volatile, so back it up for the caller.
    dasm!(ops
        ; push reg_data_ptr
    );

    // You need to allocate a shadow space on the stack for Windows function
    // calls. The shadow space must be at least 32 bytes and aligned to 16
    // bytes, including the return address of any functions we call (8 bytes).
    // Since we push reg_data_ptr onto the stack above, that means we're in
    // alignment if we add reg_data_ptr (8 bytes) + return address (8 bytes) +
    // shadow space (32 bytes) = 48 bytes.
    #[cfg(target_os = "windows")]
    {
        dasm!(ops
            ; sub rsp, 0x20
        );
    }

    // Set the initial value for the data pointer.
    dasm!(ops
        // Reinterpret as i64, using the same bytes as before.
        ; mov reg_data_ptr, QWORD runtime.memory_ptr() as i64
    );

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
                dasm!(ops
                    // Reinterpret as i64, using the same bytes as before.
                    ; mov reg_arg1, QWORD runtime as *const Runtime as i64
                    ; mov reg_function_call, QWORD Runtime::read as *const () as i64
                    ; call reg_function_call
                    ; mov BYTE [reg_data_ptr], reg_return
                );
            }
            Instruction::Write => {
                dasm!(ops
                    // Reinterpret as i64, using the same bytes as before.
                    ; mov reg_arg1, QWORD runtime as *const Runtime as i64
                    ; mov reg_arg2, [reg_data_ptr]
                    ; mov reg_function_call, QWORD Runtime::write as *const () as i64
                    ; call reg_function_call
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

    // Clear the shadow space on Windows.
    #[cfg(target_os = "windows")]
    {
        dasm!(ops
            ; add rsp, 0x20
        );
    }

    dasm!(ops
        ; pop reg_data_ptr
        ; ret
    );

    Ok(CompiledProgram {
        buffer: ops.finalize()?,
        start,
    })
}
