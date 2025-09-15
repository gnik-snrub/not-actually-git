pub mod tests;

pub mod commands;
pub mod io;
pub mod repo;
mod cli;
use cli::run_command;

fn main() {
    run_command()
}
