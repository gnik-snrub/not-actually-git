pub mod tests;

pub mod commands;
pub mod core;
mod cli;
use cli::run_command;

fn main() {
    run_command()
}
