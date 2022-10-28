use std::{error::Error, time::Instant};

fn main() -> Result<(), Box<dyn Error>> {
    let start = Instant::now();

    simpleinterp::run("")?;

    println!("{}", start.elapsed().as_nanos());

    Ok(())
}
