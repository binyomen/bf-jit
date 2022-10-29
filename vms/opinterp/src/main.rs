use util::{run_main, BfError};

fn main() -> Result<(), BfError> {
    run_main(opinterp::run)
}
