use {std::error::Error, std::fs};

fn main() -> Result<(), Box<dyn Error>> {
    let args = std::env::args().collect::<Vec<String>>();
    let filepath = &args[0];
    let source_code = fs::read_to_string(filepath)?;

    simpleinterp::run(&source_code)?;

    Ok(())
}
