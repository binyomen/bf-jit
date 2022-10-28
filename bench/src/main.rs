use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    bench::graph_results()?;
    Ok(())
}
