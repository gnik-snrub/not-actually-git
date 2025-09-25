use clap::{Parser, Subcommand};
use crate::commands::{init::init, hash::hash, add::add, status::status};
use crate::core::io::read_file;
use crate::core::repo::find_repo_root;
use crate::core::tree::write_tree;

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
            add(&path);
        },
        Cli { command: Some(Command::Status { })} => {
            status();
        },
        Cli { command: Some(Command::Test { })} => {
            let found_repo = find_repo_root();
            match found_repo {
                Ok(root) => {
                    let ass = write_tree(&root);
                    println!("{}", ass.unwrap());
                },
                Err(e) => {
                    println!("Error: {:?}", e);
                }
            }
        },
        Cli { command: None } => {}
    }
}
