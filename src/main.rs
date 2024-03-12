use std::fs;
use std::io;
use std::io::Read;
use flate2::read::ZlibDecoder;
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "mygit")]
#[command(about = "My custom Git CLI", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    // Clone
    // Push
    // Add
    Init,
    CatFile {
        #[arg(short = 'p')]
        pretty_print: bool,
        object_hash: String,
    },
}

fn main() {
    let args = Args::parse();
    match args.command {
        Commands::Init => {
            fs::create_dir("ogit").unwrap();
            fs::create_dir("ogit/objects").unwrap();
            fs::create_dir("ogit/refs").unwrap();
            fs::write("ogit/HEAD", "ref: refs/heads/main\n").unwrap();
            println!("Initialized ogit directory");
        },
        Commands::CatFile { pretty_print, object_hash } => {
            if pretty_print {
                // read contents of blob obj file from .git/objects
                let (prefix, rest) = object_hash.split_at(2);
                /* 
                let encoded = fs::read_to_string(format!("ogit/objects/{prefix}/{rest}")).unwrap();
                // decompress using Zlib and flate2
                let decoded = decode_reader(encoded.into_bytes()).unwrap();
                // extract relevent content and print (no \n)
                print!("{decoded}");
                */
            }
        }
    }
}

fn decode_reader(bytes: Vec<u8>) -> io::Result<String> {
    let mut z = ZlibDecoder::new(&bytes[..]);
    let mut s = String::new();
    z.read_to_string(&mut s)?;
    Ok(s)
}