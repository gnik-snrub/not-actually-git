use clap::{Parser, Subcommand};

use std::path::Path;
use std::fs::{create_dir, create_dir_all, write};

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
    }
}

pub fn run_command() {
    let tokens = Cli::parse();
    match tokens {
        Cli { command: Some(Command::Init { input_path })} => {
            let path = match &input_path {
                None => Path::new("./"),
                Some(input) => Path::new(input),
            };
            let canon_path = match path.canonicalize() {
                Err(_e) => {
                    println!("Error: Invalid directory");
                    return;
                },
                Ok(p) => {
                    p.join(".nag")
                }
            };

            let obj_path = canon_path.join("objects");
            let head_dir_path = canon_path.join("refs/heads");
            let main_bootstrap_path = head_dir_path.join("main");
            let head_file_path = canon_path.join("HEAD");

            if obj_path.exists() && head_dir_path.exists() &&
                main_bootstrap_path.exists() && head_file_path.exists() {
                    println!("Reinitialized existing NAG repository");
                    return;
            }

            let _ = create_dir(&canon_path);
            let _ = create_dir(&obj_path);
            let _ = create_dir_all(&head_dir_path);
            let _ = write(main_bootstrap_path, b"");
            let _ = write(head_file_path, b"ref: refs/heads/main\n");
        },
        Cli { command: None } => {}
    }
}
