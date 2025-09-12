pub mod commands;
pub mod io;
mod cli;
use cli::run_command;

fn main() {
    run_command()
}
