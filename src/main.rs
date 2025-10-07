pub mod tests;

pub mod commands;
pub mod core;
mod cli;
use cli::run_command;

fn main() {
    if let Err(e) = run_command() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
