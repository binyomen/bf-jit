use util::{run_main, BfResult};

fn main() -> BfResult<()> {
    run_main(opinterp::run)
}
