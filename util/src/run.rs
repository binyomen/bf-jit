use {
    crate::error::BfError,
    std::{env, fs},
};

pub trait RunFunction: Fn(&str) -> Result<(), BfError> {}
impl<T> RunFunction for T where T: Fn(&str) -> Result<(), BfError> {}

pub fn run_main(run_function: impl RunFunction) -> Result<(), BfError> {
    let args = env::args().collect::<Vec<String>>();
    let filepath = &args[0];
    let source_code = fs::read_to_string(filepath)?;

    run_function(&source_code)
}
