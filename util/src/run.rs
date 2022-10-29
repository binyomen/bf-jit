use {
    crate::error::BfResult,
    std::{
        env, fs,
        io::{self, Read, Write},
    },
};

pub trait RunFunction: Fn(&str, &mut dyn Read, &mut dyn Write) -> BfResult<()> {}
impl<T> RunFunction for T where T: Fn(&str, &mut dyn Read, &mut dyn Write) -> BfResult<()> {}

pub fn run_main(run_function: impl RunFunction) -> BfResult<()> {
    let args = env::args().collect::<Vec<String>>();
    let filepath = &args[1];
    let source_code = fs::read_to_string(filepath)?;

    run_function(&source_code, &mut io::stdin(), &mut io::stdout())
}
