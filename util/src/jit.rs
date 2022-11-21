use {
    crate::error::{BfError, BfResult},
    dynasmrt::{dynasm, AssemblyOffset, DynamicLabel, DynasmApi, DynasmLabelApi, ExecutableBuffer},
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
const REG_DATA_POINTER_NON_VOLATILE: bool = true;
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
#[cfg(all(any(target_os = "linux", target_os = "macos"), target_arch = "x86"))]
const REG_DATA_POINTER_NON_VOLATILE: bool = true;
#[cfg(all(any(target_os = "linux", target_os = "macos"), target_arch = "x86"))]
const STACK_OFFSET: i32 = 0;

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

pub struct LabelPair {
    begin_label: DynamicLabel,
    end_label: DynamicLabel,
}

pub fn prologue(assembler: &mut Assembler, runtime: &mut Runtime) {
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
}

pub fn epilogue(assembler: &mut Assembler) {
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
}

pub fn call_read(assembler: &mut Assembler, runtime: &mut Runtime) {
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

pub fn call_write(assembler: &mut Assembler, runtime: &mut Runtime) {
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

pub fn jump_begin(assembler: &mut Assembler, open_bracket_stack: &mut Vec<LabelPair>) {
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

    dasm!(assembler
        ; cmp BYTE [reg_data_ptr], 0
        ; jnz =>begin_label
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
