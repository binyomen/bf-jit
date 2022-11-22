use {
    crate::error::{BfError, BfResult},
    dynasmrt::{dynasm, AssemblyOffset, DynamicLabel, DynasmApi, DynasmLabelApi, ExecutableBuffer},
    std::{
        io::{Read, Write},
        mem,
    },
};

#[cfg(target_arch = "aarch64")]
pub use dynasmrt::aarch64::Assembler;
#[cfg(target_arch = "x86_64")]
pub use dynasmrt::x64::Assembler;
#[cfg(target_arch = "x86")]
pub use dynasmrt::x86::Assembler;

#[cfg(all(any(target_os = "linux", target_os = "macos"), target_arch = "x86_64"))]
#[macro_export]
macro_rules! dasm {
    ($assembler:ident $($t:tt)*) => {
        dynasm!($assembler
            ; .arch x64
            ; .alias reg_stack_ptr, rsp
            ; .alias reg_data_ptr, r13
            ; .alias reg_arg1, rdi
            ; .alias reg_arg2, rsi
            ; .alias reg_temp, r8
            ; .alias reg_temp_low, r8b
            ; .alias reg_return, al
            $($t)*
        )
    }
}
#[cfg(all(any(target_os = "linux", target_os = "macos"), target_arch = "x86_64"))]
const STACK_OFFSET: i32 = 0;

#[cfg(all(any(target_os = "linux", target_os = "macos"), target_arch = "x86"))]
#[macro_export]
macro_rules! dasm {
    ($assembler:ident $($t:tt)*) => {
        dynasm!($assembler
            ; .arch x86
            ; .alias reg_stack_ptr, esp
            ; .alias reg_data_ptr, ebx
            ; .alias reg_arg1, ecx
            ; .alias reg_arg2, edx
            ; .alias reg_temp, eax
            ; .alias reg_temp_low, al
            ; .alias reg_return, al
            $($t)*
        )
    }
}
// We should align the stack here to 16 bytes. However, considering we push the
// data pointer onto the stack and push the return address of functions when
// calling them, we don't need to do any additional work to align it.
#[cfg(all(any(target_os = "linux", target_os = "macos"), target_arch = "x86"))]
const STACK_OFFSET: i32 = 0;

#[cfg(all(any(target_os = "linux", target_os = "macos"), target_arch = "aarch64"))]
#[macro_export]
macro_rules! dasm {
    ($assembler:ident $($t:tt)*) => {
        dynasm!($assembler
            ; .arch aarch64
            ; .alias reg_stack_ptr, sp
            ; .alias reg_frame_ptr, x29 // The fp alias isn't supported by default.
            ; .alias reg_link, x30 // The lr alias isn't supported by default.
            ; .alias reg_data_ptr, x19
            ; .alias reg_arg1, x0
            ; .alias reg_arg2_low, w1
            ; .alias reg_temp, x9
            ; .alias reg_temp_low, w9
            ; .alias reg_temp2_low, w10
            ; .alias reg_temp3, x11
            ; .alias reg_return, w0
            $($t)*
        )
    }
}
// Offset the stack pointer by 32 bytes, which provides room for the contents of
// the frame pointer, link, and data pointer registers. The extra 8 bytes
// ensures the stack is properly aligned to 16 bytes.
#[cfg(all(any(target_os = "linux", target_os = "macos"), target_arch = "aarch64"))]
const STACK_OFFSET: u8 = 0x20;

#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
#[macro_export]
macro_rules! dasm {
    ($assembler:ident $($t:tt)*) => {
        dynasm!($assembler
            ; .arch x64
            ; .alias reg_stack_ptr, rsp
            ; .alias reg_data_ptr, r13
            ; .alias reg_arg1, rcx
            ; .alias reg_arg2, rdx
            ; .alias reg_temp, r8
            ; .alias reg_temp_low, r8b
            ; .alias reg_return, al
            $($t)*
        )
    }
}
// You need to allocate a shadow space on the stack for Windows function calls.
// The shadow space must be at least 32 bytes and aligned to 16 bytes, including
// the return address of any functions we call (8 bytes). Since we push
// reg_data_ptr onto the stack above, that means we're in alignment if we add
// reg_data_ptr (8 bytes) + return address (8 bytes) + shadow space (32 bytes) =
// 48 bytes.
#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
const STACK_OFFSET: i32 = 0x20;

#[cfg(target_arch = "aarch64")]
macro_rules! mov_u64 {
    ($assembler:ident, $destination:ident, $immediate:expr) => {
        let immediate: u64 = $immediate;
        let part1 = ((immediate & 0xffff) >> 0x00) as u32;
        let part2 = ((immediate & 0xffff0000) >> 0x10) as u32;
        let part3 = ((immediate & 0xffff00000000) >> 0x20) as u32;
        let part4 = ((immediate & 0xffff000000000000) >> 0x30) as u32;

        dasm!($assembler
            ; movz $destination, part1
            ; movk $destination, part2, lsl 16
            ; movk $destination, part3, lsl 32
            ; movk $destination, part4, lsl 48
        )
    };
}

#[cfg(target_arch = "aarch64")]
#[macro_export]
macro_rules! add_sub_u64 {
    ($assembler:ident, $operation:ident, $destination:ident, $addend:ident, $immediate:expr) => {
        // add and sub can only accept a 12-bit immediate. Create multiple
        // adds/subs if necessary.
        const MAX_VALUE: u32 = 2_u32.pow(12);
        let mut new_immediate: u64 = $immediate;
        while new_immediate > MAX_VALUE.into() {
            dasm!($assembler
                ; $operation $destination, $addend, MAX_VALUE
            );
            new_immediate -= Into::<u64>::into(MAX_VALUE);
        }

        dasm!($assembler
            // At this point new_immediate is definitely less than u32::MAX.
            ; $operation $destination, $addend, new_immediate as u32
        );
    };
}

pub struct LabelPair {
    begin_label: DynamicLabel,
    end_label: DynamicLabel,
}

pub fn prologue(assembler: &mut Assembler, runtime: &mut Runtime) {
    if STACK_OFFSET > 0 {
        #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
        dasm!(assembler
            ; sub reg_stack_ptr, STACK_OFFSET
        );
        #[cfg(target_arch = "aarch64")]
        dasm!(assembler
            ; sub reg_stack_ptr, reg_stack_ptr, Into::<u32>::into(STACK_OFFSET)
        );
    }

    // Back up the frame pointer and link register to create a frame record (see
    // https://github.com/ARM-software/abi-aa/blob/main/aapcs64/aapcs64.rst#the-frame-pointer).
    #[cfg(target_arch = "aarch64")]
    dasm!(assembler
        ; stp reg_frame_ptr, reg_link, [reg_stack_ptr, 0x10]
        ; add reg_frame_ptr, reg_stack_ptr, 0x10
    );

    // Back up non-volatile registers for the caller.
    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    dasm!(assembler
        ; push reg_data_ptr
    );
    #[cfg(target_arch = "aarch64")]
    dasm!(assembler
        ; str reg_data_ptr, [reg_stack_ptr, 0x8]
    );

    #[cfg(target_arch = "x86_64")]
    dasm!(assembler
        // Reinterpret as i64, using the same bytes as before.
        ; mov reg_data_ptr, QWORD runtime.memory_ptr() as i64
    );
    #[cfg(target_arch = "x86")]
    dasm!(assembler
        // Reinterpret as i32, using the same bytes as before.
        ; mov reg_data_ptr, DWORD runtime.memory_ptr() as i32
    );
    #[cfg(target_arch = "aarch64")]
    dasm!(assembler
        // Reinterpret as u64, using the same bytes as before.
        ;; mov_u64!(assembler, reg_data_ptr, runtime.memory_ptr() as u64)
    );
}

pub fn epilogue(assembler: &mut Assembler) {
    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    dasm!(assembler
        ; pop reg_data_ptr
    );
    #[cfg(target_arch = "aarch64")]
    dasm!(assembler
        ; ldr reg_data_ptr, [reg_stack_ptr, 0x8]
    );

    // Restore the frame pointer and link register from the frame record.
    #[cfg(target_arch = "aarch64")]
    dasm!(assembler
        ; ldp reg_frame_ptr, reg_link, [reg_stack_ptr, 0x10]
    );

    if STACK_OFFSET > 0 {
        #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
        dasm!(assembler
            ; add reg_stack_ptr, STACK_OFFSET
        );
        #[cfg(target_arch = "aarch64")]
        dasm!(assembler
            ; add reg_stack_ptr, reg_stack_ptr, Into::<u32>::into(STACK_OFFSET)
        );
    }

    dasm!(assembler
        ; ret
    );
}

pub fn call_read(assembler: &mut Assembler, runtime: &mut Runtime) {
    #[cfg(target_arch = "x86_64")]
    dasm!(assembler
        // Reinterpret as i64, using the same bytes as before.
        ; mov reg_arg1, QWORD runtime as *const Runtime as i64
        ; mov reg_temp, QWORD Runtime::read as *const () as i64
        ; call reg_temp
        ; mov BYTE [reg_data_ptr], reg_return
    );
    #[cfg(target_arch = "x86")]
    dasm!(assembler
        // Reinterpret as i32, using the same bytes as before.
        ; mov reg_arg1, DWORD runtime as *const Runtime as i32
        ; mov reg_temp, DWORD Runtime::read as *const () as i32
        ; call reg_temp
        ; mov BYTE [reg_data_ptr], reg_return
    );
    #[cfg(target_arch = "aarch64")]
    dasm!(assembler
        // Reinterpret as u64, using the same bytes as before.
        ;; mov_u64!(assembler, reg_arg1, runtime as *const Runtime as u64)
        ;; mov_u64!(assembler, reg_temp, Runtime::read as *const () as u64)
        ; blr reg_temp
        ; strb reg_return, [reg_data_ptr]
    );
}

pub fn call_write(assembler: &mut Assembler, runtime: &mut Runtime) {
    #[cfg(target_arch = "x86_64")]
    dasm!(assembler
        // Reinterpret as i64, using the same bytes as before.
        ; mov reg_arg1, QWORD runtime as *const Runtime as i64
        ; mov reg_arg2, [reg_data_ptr]
        ; mov reg_temp, QWORD Runtime::write as *const () as i64
        ; call reg_temp
    );
    #[cfg(target_arch = "x86")]
    dasm!(assembler
        // Reinterpret as i32, using the same bytes as before.
        ; mov reg_arg1, DWORD runtime as *const Runtime as i32
        ; mov reg_arg2, [reg_data_ptr]
        ; mov reg_temp, DWORD Runtime::write as *const () as i32
        ; call reg_temp
    );
    #[cfg(target_arch = "aarch64")]
    dasm!(assembler
        // Reinterpret as u64, using the same bytes as before.
        ;; mov_u64!(assembler, reg_arg1, runtime as *const Runtime as u64)
        ; ldrb reg_arg2_low, [reg_data_ptr]
        ;; mov_u64!(assembler, reg_temp, Runtime::write as *const () as u64)
        ; blr reg_temp
    );
}

pub fn jump_begin(assembler: &mut Assembler, open_bracket_stack: &mut Vec<LabelPair>) {
    let begin_label = assembler.new_dynamic_label();
    let end_label = assembler.new_dynamic_label();
    open_bracket_stack.push(LabelPair {
        begin_label,
        end_label,
    });

    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    dasm!(assembler
        ; cmp BYTE [reg_data_ptr], 0
        ; jz =>end_label
        ; =>begin_label
    );
    #[cfg(target_arch = "aarch64")]
    dasm!(assembler
        ; ldrb reg_temp_low, [reg_data_ptr]
        ; cmp reg_temp_low, 0
        ; b.eq =>end_label
        ; =>begin_label
    );
}

pub fn jump_end(
    assembler: &mut Assembler,
    open_bracket_stack: &mut Vec<LabelPair>,
    instruction_index: usize,
) -> BfResult<()> {
    let LabelPair {
        begin_label,
        end_label,
    } = open_bracket_stack.pop().ok_or_else(|| {
        BfError::Bf(format!(
            "Unmatched closing ']' at position {instruction_index}."
        ))
    })?;

    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    dasm!(assembler
        ; cmp BYTE [reg_data_ptr], 0
        ; jnz =>begin_label
        ; =>end_label
    );
    #[cfg(target_arch = "aarch64")]
    dasm!(assembler
        ; ldrb reg_temp_low, [reg_data_ptr]
        ; cmp reg_temp_low, 0
        ; b.ne =>begin_label
        ; =>end_label
    );

    Ok(())
}

pub struct CompiledProgram {
    buffer: ExecutableBuffer,
    start: AssemblyOffset,
}

impl CompiledProgram {
    pub fn new(buffer: ExecutableBuffer, start: AssemblyOffset) -> Self {
        CompiledProgram { buffer, start }
    }

    pub fn function_ptr(&self) -> *const () {
        self.buffer.ptr(self.start) as *const ()
    }
}

#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
type AsmEntryPoint = extern "C" fn();
#[cfg(target_arch = "x86")]
type AsmEntryPoint = extern "fastcall" fn();

const MEMORY_SIZE: usize = 30000;

pub struct Runtime<'a> {
    memory: [u8; MEMORY_SIZE],
    stdin: &'a mut dyn Read,
    stdout: &'a mut dyn Write,
}

impl<'a> Runtime<'a> {
    pub fn new(stdin: &'a mut dyn Read, stdout: &'a mut dyn Write) -> Self {
        Self {
            memory: [0; MEMORY_SIZE],
            stdin,
            stdout,
        }
    }

    pub fn memory_ptr(&mut self) -> *mut u8 {
        self.memory.as_mut_ptr()
    }

    pub fn run(&self, compiled_program: CompiledProgram) -> BfResult<()> {
        let entry_point_pointer = compiled_program.function_ptr();
        let entry_point =
            unsafe { mem::transmute::<*const (), AsmEntryPoint>(entry_point_pointer) };
        entry_point();

        Ok(())
    }

    fn read_inner(&mut self) -> u8 {
        let mut c = [0; 1];
        self.stdin.read_exact(&mut c).unwrap();

        c[0]
    }

    fn write_inner(&mut self, byte: u8) {
        self.stdout.write_all(&[byte]).unwrap();
        self.stdout.flush().unwrap();
    }

    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    pub extern "C" fn read(&mut self) -> u8 {
        self.read_inner()
    }
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    pub extern "C" fn write(&mut self, byte: u8) {
        self.write_inner(byte)
    }

    #[cfg(target_arch = "x86")]
    pub extern "fastcall" fn read(&mut self) -> u8 {
        self.read_inner()
    }
    #[cfg(target_arch = "x86")]
    pub extern "fastcall" fn write(&mut self, byte: u8) {
        self.write_inner(byte)
    }
}
