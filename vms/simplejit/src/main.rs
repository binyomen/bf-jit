use util::{run::run_main, BfResult};

fn main() -> BfResult<()> {
    run_main(simplejit::run)
}
