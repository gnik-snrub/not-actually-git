use clap::{Parser, Subcommand};
use crate::commands::{
    init::init,
    add::add,
    status::status,
    commit::commit,
    checkout::checkout,
    branch::{ branch, branch_list },
    restore::restore,
};
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
    Checkout {
        branch: String,
    },
    Branch {
        branch_name: Option<String>,
        source_oid: Option<String>,
        #[arg(short = 'l', long = "list")]
        list: bool,
    },
    Restore {
        restore_path: String,
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
            status(true)?;
        },
        Cli { command: Some(Command::Commit { message })} => {
            commit(message)?;
        },
        Cli { command: Some(Command::Checkout { branch })} => {
            checkout(branch)?;
        },
        Cli { command: Some(Command::Branch { branch_name, source_oid, list })} => {
            if list {
                branch_list(true)?;
            } else {
                let b_name = branch_name.unwrap();
                branch(b_name, source_oid)?;
            }
        },
        Cli { command: Some(Command::Restore { restore_path })} => {
            restore(restore_path)?;
        },
        Cli { command: Some(Command::Test { })} => {
        },
        Cli { command: None } => {}
    }

    Ok(())
}
