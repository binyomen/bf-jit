use util::BfError;

mod parser;
mod vm;

pub fn run(source_code: &str) -> Result<(), BfError> {
    let program = parser::parse(source_code);
    vm::run(program)
}
