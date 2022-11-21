use {
    crate::error::BfResult,
    dynasmrt::{dynasm, AssemblyOffset, DynasmApi, ExecutableBuffer},
    std::{
        io::{Read, Write},
        mem,
    },
};

#[cfg(target_arch = "x86_64")]
pub use dynasmrt::x64::Assembler;
#[cfg(target_arch = "x86")]
pub use dynasmrt::x86::Assembler;

#[cfg(all(any(target_os = "linux", target_os = "macos"), target_arch = "x86_64"))]
#[macro_export]
macro_rules! dasm {
    ($ops:ident $($t:tt)*) => {
        dynasm!($ops
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
pub const REG_DATA_POINTER_NON_VOLATILE: bool = true;
#[cfg(all(any(target_os = "linux", target_os = "macos"), target_arch = "x86_64"))]
pub const STACK_OFFSET: i32 = 0;

#[cfg(all(any(target_os = "linux", target_os = "macos"), target_arch = "x86"))]
#[macro_export]
macro_rules! dasm {
    ($ops:ident $($t:tt)*) => {
        dynasm!($ops
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
#[cfg(all(any(target_os = "linux", target_os = "macos"), target_arch = "x86"))]
pub const REG_DATA_POINTER_NON_VOLATILE: bool = true;
#[cfg(all(any(target_os = "linux", target_os = "macos"), target_arch = "x86"))]
pub const STACK_OFFSET: i32 = 0;

#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
#[macro_export]
macro_rules! dasm {
    ($ops:ident $($t:tt)*) => {
        dynasm!($ops
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
#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
pub const REG_DATA_POINTER_NON_VOLATILE: bool = true;
// You need to allocate a shadow space on the stack for Windows function calls.
// The shadow space must be at least 32 bytes and aligned to 16 bytes, including
// the return address of any functions we call (8 bytes). Since we push
// reg_data_ptr onto the stack above, that means we're in alignment if we add
// reg_data_ptr (8 bytes) + return address (8 bytes) + shadow space (32 bytes) =
// 48 bytes.
#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
pub const STACK_OFFSET: i32 = 0x20;

pub fn set_data_pointer_initial_value(ops: &mut Assembler, runtime: &mut Runtime) {
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

#[cfg(target_arch = "x86_64")]
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

    #[cfg(target_arch = "x86_64")]
    pub extern "C" fn read(&mut self) -> u8 {
        self.read_inner()
    }
    #[cfg(target_arch = "x86_64")]
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
