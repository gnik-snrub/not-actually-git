use clap::{Parser, Subcommand};
use crate::commands::{init::init, hash::hash};
use crate::io::read_file;

#[derive(Parser, Debug)]
#[command(name = "Not Actually Git")]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Command>
}

#[derive(Subcommand, Debug)]
enum Command {
    Init {
        input_path: Option<String>
    },
    Hash {
        file_path: String
    }
}

pub fn run_command() {
    let tokens = Cli::parse();
    match tokens {
        Cli { command: Some(Command::Init { input_path })} => {
            init(input_path);
        },
        Cli { command: Some(Command::Hash { file_path })} => {
            let file = read_file(&file_path);
            hash(&file);
        }
        Cli { command: None } => {}
    }
}
