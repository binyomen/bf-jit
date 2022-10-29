mod error;
mod run;

pub use {
    error::{BfError, BfResult},
    run::{run_main, RunFunction},
};
