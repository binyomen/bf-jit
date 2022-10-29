use {
    std::io::{Read, Write},
    util::BfResult,
};

mod parser;
mod vm;

pub fn run(source_code: &str, stdin: &mut dyn Read, stdout: &mut dyn Write) -> BfResult<()> {
    let program = parser::parse(source_code)?;
    vm::run(program, stdin, stdout)
}
