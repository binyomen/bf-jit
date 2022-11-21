use {
    crate::{
        parser::{Instruction, Program},
        runtime::Runtime,
    },
    dynasmrt::{dynasm, AssemblyOffset, DynasmApi, DynasmLabelApi, ExecutableBuffer},
    util::{BfError, BfResult},
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
            ; .alias reg_temp1, r8
            ; .alias reg_temp1_low, r8b
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
            ; .alias reg_temp1, eax
            ; .alias reg_temp1_low, al
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
            ; .alias reg_temp1, r8
            ; .alias reg_temp1_low, r8b
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
            Instruction::IncPtr { count } => {
                dasm!(ops
                    // Reinterpret as i32, using the same bytes as before.
                    ; add reg_data_ptr, DWORD count as i32
                );
            }
            Instruction::DecPtr { count } => {
                dasm!(ops
                    // Reinterpret as i32, using the same bytes as before.
                    ; sub reg_data_ptr, DWORD count as i32
                );
            }
            Instruction::IncData { count } => {
                // Adding 256 is effectively a nop, since it will wrap around to
                // the original value. Mod out 256 to get a value between 0 and
                // 255.
                let wrapped_count = (count % 256) as u8;
                dasm!(ops
                    // Reinterpret as i8, using the same bytes as before.
                    ; add BYTE [reg_data_ptr], BYTE wrapped_count as i8
                );
            }
            Instruction::DecData { count } => {
                // Subtracting 256 is effectively a nop, since it will wrap
                // around to the original value. Mod out 256 to get a value
                // between 0 and 255.
                let wrapped_count = (count % 256) as u8;
                dasm!(ops
                    // Reinterpret as i8, using the same bytes as before.
                    ; sub BYTE [reg_data_ptr], BYTE wrapped_count as i8
                );
            }
            Instruction::Read { count } => {
                for _ in 0..count {
                    #[cfg(target_arch = "x86_64")]
                    dasm!(ops
                        // Reinterpret as i64, using the same bytes as before.
                        ; mov reg_arg1, QWORD runtime as *const Runtime as i64
                        ; mov reg_temp1, QWORD Runtime::read as *const () as i64
                    );
                    #[cfg(target_arch = "x86")]
                    dasm!(ops
                        // Reinterpret as i32, using the same bytes as before.
                        ; mov reg_arg1, DWORD runtime as *const Runtime as i32
                        ; mov reg_temp1, DWORD Runtime::read as *const () as i32
                    );

                    dasm!(ops
                        ; call reg_temp1
                        ; mov BYTE [reg_data_ptr], reg_return
                    );
                }
            }
            Instruction::Write { count } => {
                for _ in 0..count {
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
                        ; mov reg_temp1, QWORD Runtime::write as *const () as i64
                    );
                    #[cfg(target_arch = "x86")]
                    dasm!(ops
                        // Reinterpret as i32, using the same bytes as before.
                        ; mov reg_temp1, DWORD Runtime::write as *const () as i32
                    );

                    dasm!(ops
                        ; call reg_temp1
                    );
                }
            }
            Instruction::JumpBegin => {
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
            Instruction::JumpEnd => {
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
            Instruction::SetDataToZero => {
                dasm!(ops
                    ; mov BYTE [reg_data_ptr], 0
                );
            }
            Instruction::MovePtrUntilZero {
                count,
                forward,
                amount,
            } => {
                for _ in 0..count {
                    let begin_loop = ops.new_dynamic_label();
                    let end_loop = ops.new_dynamic_label();
                    dasm!(ops
                        ; =>begin_loop
                        ; cmp BYTE [reg_data_ptr], 0
                        ; jz =>end_loop
                    );

                    if forward {
                        dasm!(ops
                            // Reinterpret as i32, using the same bytes as before.
                            ; add reg_data_ptr, DWORD amount as i32
                        );
                    } else {
                        dasm!(ops
                            // Reinterpret as i32, using the same bytes as before.
                            ; sub reg_data_ptr, DWORD amount as i32
                        );
                    }

                    dasm!(ops
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
                    let skip_move = ops.new_dynamic_label();
                    dasm!(ops
                        ; cmp BYTE [reg_data_ptr], 0
                        ; jz =>skip_move
                        ; mov reg_temp1_low, BYTE [reg_data_ptr]
                    );

                    // Reinterpret as i32, using the same bytes as before.
                    let amount_i32 = amount as i32;
                    if forward {
                        dasm!(ops
                            ; add BYTE [reg_data_ptr + amount_i32], reg_temp1_low
                        );
                    } else {
                        dasm!(ops
                            ; add BYTE [reg_data_ptr - amount_i32], reg_temp1_low
                        );
                    }

                    dasm!(ops
                        ; mov BYTE [reg_data_ptr], 0
                        ; =>skip_move
                    );
                }
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

    Ok(CompiledProgram {
        buffer: ops.finalize()?,
        start,
    })
}
