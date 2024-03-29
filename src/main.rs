use clap::{Parser, Subcommand};
use std::path::PathBuf;

pub(crate) mod commands;

#[derive(Debug, Parser)]
#[command(name = "oxygit")]
#[command(about = "My custom Git CLI", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Init,
    CatFile {
        #[arg(short = 'p')]
        pretty_print: bool,
        object_hash: String,
    },
    HashObject {
        #[arg(short = 'w')]
        write: bool,
        path: PathBuf,
        // TODO: -t <type>, e.g, -t commit
        //   by default, type = blob
    },
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    match args.command {
        Commands::Init => {
            commands::init::invoke()?;
        }
        Commands::CatFile {
            pretty_print,
            object_hash,
        } => {
            commands::cat_file::invoke(pretty_print, &object_hash)?;
        }
        Commands::HashObject { write, path } => {
            commands::hash_object::invoke(write, &path)?;
        }
    }
    Ok(())
}
