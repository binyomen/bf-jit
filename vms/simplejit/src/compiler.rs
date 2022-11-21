use {
    crate::parser::{Instruction, Program},
    dynasmrt::{dynasm, DynasmApi, DynasmLabelApi},
    util::{
        jit::{CompiledProgram, Runtime},
        BfError, BfResult,
    },
};

#[cfg(target_arch = "x86_64")]
use dynasmrt::x64::Assembler;
#[cfg(target_arch = "x86")]
use dynasmrt::x86::Assembler;

#[cfg(all(any(target_os = "linux", target_os = "macos"), target_arch = "x86_64"))]
macro_rules! dasm {
    ($ops:ident $($t:tt)*) => {
        dynasm!($ops
            ; .arch x64
            ; .alias reg_stack_ptr, rsp
            ; .alias reg_data_ptr, r13
            ; .alias reg_arg1, rdi
            ; .alias reg_arg2, rsi
            ; .alias reg_function_call, r8
            ; .alias reg_return, al
            $($t)*
        )
    }
}
#[cfg(all(any(target_os = "linux", target_os = "macos"), target_arch = "x86_64"))]
const REG_DATA_POINTER_NON_VOLATILE: bool = true;
#[cfg(all(any(target_os = "linux", target_os = "macos"), target_arch = "x86_64"))]
const STACK_OFFSET: i32 = 0;

#[cfg(all(any(target_os = "linux", target_os = "macos"), target_arch = "x86"))]
macro_rules! dasm {
    ($ops:ident $($t:tt)*) => {
        dynasm!($ops
            ; .arch x86
            ; .alias reg_stack_ptr, esp
            ; .alias reg_data_ptr, ebx
            ; .alias reg_arg1, ecx
            ; .alias reg_arg2, edx
            ; .alias reg_function_call, eax
            ; .alias reg_return, al
            $($t)*
        )
    }
}
#[cfg(all(any(target_os = "linux", target_os = "macos"), target_arch = "x86"))]
const REG_DATA_POINTER_NON_VOLATILE: bool = true;
#[cfg(all(any(target_os = "linux", target_os = "macos"), target_arch = "x86"))]
const STACK_OFFSET: i32 = 0;

#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
macro_rules! dasm {
    ($ops:ident $($t:tt)*) => {
        dynasm!($ops
            ; .arch x64
            ; .alias reg_stack_ptr, rsp
            ; .alias reg_data_ptr, r13
            ; .alias reg_arg1, rcx
            ; .alias reg_arg2, rdx
            ; .alias reg_function_call, r8
            ; .alias reg_return, al
            $($t)*
        )
    }
}
#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
const REG_DATA_POINTER_NON_VOLATILE: bool = true;
// You need to allocate a shadow space on the stack for Windows function calls.
// The shadow space must be at least 32 bytes and aligned to 16 bytes, including
// the return address of any functions we call (8 bytes). Since we push
// reg_data_ptr onto the stack above, that means we're in alignment if we add
// reg_data_ptr (8 bytes) + return address (8 bytes) + shadow space (32 bytes) =
// 48 bytes.
#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
const STACK_OFFSET: i32 = 0x20;

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

    // Set the initial value for the data pointer.
    #[cfg(target_arch = "x86_64")]
    dasm!(ops
        // Reinterpret as i64, using the same bytes as before.
        ; mov reg_data_ptr, QWORD runtime.memory_ptr() as i64
    );
    #[cfg(target_arch = "x86")]
    dasm!(ops
        // Reinterpret as i32, using the same bytes as before.
        ; mov reg_data_ptr, DWORD runtime.memory_ptr() as i32
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
                #[cfg(target_arch = "x86_64")]
                dasm!(ops
                    // Reinterpret as i64, using the same bytes as before.
                    ; mov reg_arg1, QWORD runtime as *const Runtime as i64
                    ; mov reg_function_call, QWORD Runtime::read as *const () as i64
                );
                #[cfg(target_arch = "x86")]
                dasm!(ops
                    // Reinterpret as i32, using the same bytes as before.
                    ; mov reg_arg1, DWORD runtime as *const Runtime as i32
                    ; mov reg_function_call, DWORD Runtime::read as *const () as i32
                );

                dasm!(ops
                    ; call reg_function_call
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
                    ; mov reg_function_call, QWORD Runtime::write as *const () as i64
                );
                #[cfg(target_arch = "x86")]
                dasm!(ops
                    // Reinterpret as i32, using the same bytes as before.
                    ; mov reg_function_call, DWORD Runtime::write as *const () as i32
                );

                dasm!(ops
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
