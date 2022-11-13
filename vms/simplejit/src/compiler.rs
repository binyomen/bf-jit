use {
    crate::{
        parser::{Instruction, Program},
        runtime::Runtime,
    },
    util::{BfError, BfResult},
};

struct CodeBuilder {
    machine_code: Vec<u8>,
}

impl CodeBuilder {
    fn new() -> Self {
        Self {
            machine_code: vec![],
        }
    }

    fn code(self) -> Vec<u8> {
        self.machine_code
    }

    fn len(&self) -> usize {
        self.machine_code.len()
    }

    fn emit_byte(&mut self, byte: u8) {
        self.machine_code.push(byte);
    }

    fn emit_bytes(&mut self, bytes: impl AsRef<[u8]>) {
        for byte in bytes.as_ref() {
            self.emit_byte(*byte);
        }
    }

    fn replace_byte_at_offset(&mut self, offset: usize, byte: u8) {
        self.machine_code[offset] = byte;
    }

    fn replace_u32_at_offset(&mut self, offset: usize, value: u32) {
        self.replace_byte_at_offset(offset, (value & 0xff).try_into().unwrap());
        self.replace_byte_at_offset(offset + 1, ((value >> 8) & 0xff).try_into().unwrap());
        self.replace_byte_at_offset(offset + 2, ((value >> 16) & 0xff).try_into().unwrap());
        self.replace_byte_at_offset(offset + 3, ((value >> 24) & 0xff).try_into().unwrap());
    }

    fn emit_u32(&mut self, value: u32) {
        self.emit_byte((value & 0xff).try_into().unwrap());
        self.emit_byte(((value >> 8) & 0xff).try_into().unwrap());
        self.emit_byte(((value >> 16) & 0xff).try_into().unwrap());
        self.emit_byte(((value >> 24) & 0xff).try_into().unwrap());
    }

    fn emit_u64(&mut self, value: u64) {
        self.emit_u32((value & 0xffffffff).try_into().unwrap());
        self.emit_u32(((value >> 32) & 0xffffffff).try_into().unwrap());
    }

    fn emit_pointer(&mut self, ptr: *const ()) {
        self.emit_u64((ptr as usize).try_into().unwrap());
    }
}

pub fn compile(program: Program, runtime: &mut Runtime) -> BfResult<Vec<u8>> {
    let mut builder = CodeBuilder::new();

    // pushq %r13
    builder.emit_bytes([0x41, 0x55]);

    #[cfg(target_os = "windows")]
    {
        // subq $0x20, %rsp
        builder.emit_bytes([0x48, 0x83, 0xec, 0x20]);
    }

    // movabs <address of memory>, %r13
    builder.emit_bytes([0x49, 0xbd]);
    builder.emit_pointer(runtime.memory_ptr() as *const ());

    let mut open_bracket_stack = vec![];

    for (i, instruction) in program.instructions.into_iter().enumerate() {
        match instruction {
            Instruction::IncPtr => {
                // inc %r13
                builder.emit_bytes([0x49, 0xff, 0xc5]);
            }
            Instruction::DecPtr => {
                // dec %r13
                builder.emit_bytes([0x49, 0xff, 0xcd]);
            }
            Instruction::IncData => {
                // addb $1, 0(%r13)
                builder.emit_bytes([0x41, 0x80, 0x45, 0x00, 0x01]);
            }
            Instruction::DecData => {
                // subb $1, 0(%r13)
                builder.emit_bytes([0x41, 0x80, 0x6d, 0x00, 0x01]);
            }
            Instruction::Read => {
                #[cfg(any(target_os = "linux", target_os = "macos"))]
                {
                    // movabs <address of runtime>, %rdi
                    builder.emit_bytes([0x48, 0xbf]);
                    builder.emit_pointer(runtime as *const Runtime as *const ());
                }
                #[cfg(target_os = "windows")]
                {
                    // movabs <address of runtime>, %rcx
                    builder.emit_bytes([0x48, 0xb9]);
                    builder.emit_pointer(runtime as *const Runtime as *const ());
                }

                // movabs <address of read function>, %rax
                builder.emit_bytes([0x48, 0xb8]);
                builder.emit_pointer(Runtime::read as *const ());
                // call *%rax
                builder.emit_bytes([0xff, 0xd0]);
                // movq %rax, 0(%r13)
                builder.emit_bytes([0x49, 0x89, 0x45, 0x00]);
            }
            Instruction::Write => {
                #[cfg(any(target_os = "linux", target_os = "macos"))]
                {
                    // movabs <address of runtime>, %rdi
                    builder.emit_bytes([0x48, 0xbf]);
                    builder.emit_pointer(runtime as *const Runtime as *const ());
                    // movq 0(%r13), %rsi
                    builder.emit_bytes([0x49, 0x8b, 0x75, 0x00]);
                }
                #[cfg(target_os = "windows")]
                {
                    // movabs <address of runtime>, %rcx
                    builder.emit_bytes([0x48, 0xb9]);
                    builder.emit_pointer(runtime as *const Runtime as *const ());
                    // movq 0(%r13), %rdx
                    builder.emit_bytes([0x49, 0x8b, 0x55, 0x00]);
                }

                // movabs <address of write function>, %rax
                builder.emit_bytes([0x48, 0xb8]);
                builder.emit_pointer(Runtime::write as *const ());
                // call *%rax
                builder.emit_bytes([0xff, 0xd0]);
            }
            Instruction::JumpIfZero => {
                // cmpb $0, 0(%r13)
                builder.emit_bytes([0x41, 0x80, 0x7d, 0x00, 0x00]);

                // Save the location in the stack, and emit JZ (with 32-bit relative offset) with 4
                // placeholder zeroes that will be fixed up later.
                open_bracket_stack.push(builder.len());
                builder.emit_bytes([0x0f, 0x84]);
                builder.emit_u32(0);
            }
            Instruction::JumpIfNotZero => {
                let open_bracket_offset = open_bracket_stack.pop().ok_or_else(|| {
                    BfError::Bf(format!("Unmatched closing ']' at position {i}."))
                })?;

                // cmpb $0, 0(%r13)
                builder.emit_bytes([0x41, 0x80, 0x7d, 0x00, 0x00]);

                let jump_back_from = builder.len() + 6;
                let jump_back_to = open_bracket_offset + 6;
                let pcrel_offset_back = compute_relative_32bit_offset(jump_back_from, jump_back_to);

                // jnz <open_bracket_location>
                builder.emit_bytes([0x0f, 0x85]);
                builder.emit_u32(pcrel_offset_back);

                let jump_forward_from = open_bracket_offset + 6;
                let jump_forward_to = builder.len();
                let pcrel_offset_forward =
                    compute_relative_32bit_offset(jump_forward_from, jump_forward_to);
                builder.replace_u32_at_offset(open_bracket_offset + 2, pcrel_offset_forward);
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        // addq $0x20, %rsp
        builder.emit_bytes([0x48, 0x83, 0xc4, 0x20]);
    }
    // popq %r13
    builder.emit_bytes([0x41, 0x5d]);

    // ret
    builder.emit_byte(0xc3);

    Ok(builder.code())
}

fn compute_relative_32bit_offset(jump_from: usize, jump_to: usize) -> u32 {
    if jump_to >= jump_from {
        let diff: u64 = (jump_to - jump_from).try_into().unwrap();
        debug_assert!(diff < (1u64 << 31));
        diff.try_into().unwrap()
    } else {
        // Here the diff is negative, so we need to encode it as 2s complement.
        let diff: u64 = (jump_from - jump_to).try_into().unwrap();
        debug_assert!(diff - 1 < (1u64 << 31));
        let diff_u32: u32 = diff.try_into().unwrap();
        !diff_u32 + 1
    }
}
