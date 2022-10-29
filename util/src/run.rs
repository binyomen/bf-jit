use {
    crate::error::BfError,
    std::{
        env, fs,
        io::{self, Read, Write},
    },
};

pub trait RunFunction: Fn(&str, &mut dyn Read, &mut dyn Write) -> Result<(), BfError> {}
impl<T> RunFunction for T where T: Fn(&str, &mut dyn Read, &mut dyn Write) -> Result<(), BfError> {}

pub fn run_main(run_function: impl RunFunction) -> Result<(), BfError> {
    let args = env::args().collect::<Vec<String>>();
    let filepath = &args[1];
    let source_code = fs::read_to_string(filepath)?;

    run_function(&source_code, &mut io::stdin(), &mut io::stdout())
}
