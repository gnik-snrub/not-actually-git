use clap::{Parser, Subcommand};
use crate::commands::{init::init, add::add, status::status, commit::commit};
use crate::core::io::read_file;
use crate::core::hash::hash;

use std::path::Path;

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
    },
    Add {
        path_str: String
    },
    Status {

    },
    Commit {
        message: String,
    },
    Test {

    }
}

pub fn run_command() -> std::io::Result<()> {
    let tokens = Cli::parse();
    match tokens {
        Cli { command: Some(Command::Init { input_path })} => {
            init(input_path);
        },
        Cli { command: Some(Command::Hash { file_path })} => {
            let file = read_file(&file_path);
            hash(&file);
        },
        Cli { command: Some(Command::Add { path_str })} => {
            let path = Path::new(&path_str);
            add(&path)?;
        },
        Cli { command: Some(Command::Status { })} => {
            status()?;
        },
        Cli { command: Some(Command::Commit { message })} => {
            commit(message)?;
        },
        Cli { command: Some(Command::Test { })} => {

        },
        Cli { command: None } => {}
    }

    Ok(())
}
