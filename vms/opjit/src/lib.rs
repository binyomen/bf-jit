use {
    std::io::{Read, Write},
    util::{asm::Runtime, BfResult},
};

mod compiler;
mod parser;

#[cfg(not(any(
    all(target_os = "linux", target_arch = "x86_64"),
    all(target_os = "linux", target_arch = "x86"),
    all(target_os = "linux", target_arch = "aarch64"),
    all(target_os = "windows", target_arch = "x86_64"),
    all(target_os = "macos", target_arch = "x86_64"),
)))]
compile_error!("Unsupported system.");

pub fn run(source_code: &str, stdin: &mut dyn Read, stdout: &mut dyn Write) -> BfResult<()> {
    let program = parser::parse(source_code)?;
    let mut runtime = Runtime::new(stdin, stdout);

    let compiled_program = compiler::compile(program, &mut runtime)?;
    runtime.run(compiled_program)
}
