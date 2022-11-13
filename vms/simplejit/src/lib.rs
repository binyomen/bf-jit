use {
    runtime::Runtime,
    std::io::{Read, Write},
    util::BfResult,
};

mod compiler;
mod parser;
mod runtime;

#[cfg(not(target_arch = "x86_64"))]
compile_error!("Only x86-64 supported at the moment.");

pub fn run(source_code: &str, stdin: &mut dyn Read, stdout: &mut dyn Write) -> BfResult<()> {
    let program = parser::parse(source_code)?;
    let mut runtime = Runtime::new(stdin, stdout);

    let compiled_program = compiler::compile(program, &mut runtime)?;
    runtime.run(compiled_program)
}
