use anyhow;
use anyhow::Context;
use clap::{Parser, Subcommand};
use flate2::read::ZlibDecoder;
use std::ffi::CStr;
use std::fs;
use std::io::{BufRead, BufReader, Read};

#[derive(Debug, Parser)]
#[command(name = "oxygit")]
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

#[derive(Debug)]
enum Kind {
    Blob,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    match args.command {
        Commands::Init => {
            fs::create_dir(".git").unwrap();
            fs::create_dir(".git/objects").unwrap();
            fs::create_dir(".git/refs").unwrap();
            fs::write(".git/HEAD", "ref: refs/heads/main\n").unwrap();
            println!("Initialized .git directory");
        }
        Commands::CatFile {
            pretty_print,
            object_hash,
        } => {
            anyhow::ensure!(pretty_print, "mode must be given (hint: use -p)");

            // open the file and decode contents
            let (prefix, rest) = object_hash.split_at(2);
            let f = fs::File::open(format!(".git/objects/{prefix}/{rest}"))
                .context("open in .git/objects")?;
            let z = ZlibDecoder::new(f);
            let mut z = BufReader::new(z);
            let mut buf = Vec::new();

            // read until (inclusive) the null byte then convert to &str
            z.read_until(0, &mut buf)
                .context("read header from .git/objects")?;
            let header = CStr::from_bytes_with_nul(&buf)
                .expect("know there is exactly one null, and it is at the end.");
            let header = header
                .to_str()
                .context(".git/objects file header isn't valid UTF-8")?;

            // extract the kind of file and the size of contents
            let Some((kind, size)) = header.split_once(' ') else {
                anyhow::bail!(
                    ".git/objects file header did not start with a known type: '{header}'"
                );
            };
            let kind = match kind {
                "blob" => Kind::Blob,
                _ => anyhow::bail!("we do not yet know how to print a '{kind}'"),
            };
            let size = size
                .parse::<u64>()
                .context(".git/objects file header has invalid size: {size}")?;

            // NOTE: won't error if decompressed file is too long, but will not spam stdout
            //   and be vulnerable to a zipbomb. 
            let mut z = z.take(size);
            match kind {
                Kind::Blob => {
                    let stdout = std::io::stdout();
                    let mut stdout = stdout.lock();
                    let n = std::io::copy(&mut z, &mut stdout)
                        .context("write .git/objects file into stdout")?;
                    anyhow::ensure!(
                        n == size, 
                        ".git/objects file was not the expected size (expected: {size}, actual: {n})"
                    );
                }
            }
        }
    }
    Ok(())
}
