use anyhow;
use anyhow::Context;
use clap::{Parser, Subcommand};
use flate2::read::ZlibDecoder;
use std::ffi::CStr;
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};

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

enum Kind {
    Blob,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    match args.command {
        Commands::Init => {
            fs::create_dir("ogit").unwrap();
            fs::create_dir("ogit/objects").unwrap();
            fs::create_dir("ogit/refs").unwrap();
            fs::write("ogit/HEAD", "ref: refs/heads/main\n").unwrap();
            println!("Initialized ogit directory");
        }
        Commands::CatFile {
            pretty_print,
            object_hash,
        } => {
            // open the file and decode contents
            let (prefix, rest) = object_hash.split_at(2);
            let f = fs::File::open(format!("ogit/objects/{prefix}/{rest}"))
                .context("open in ogit/objects")?;
            let z = ZlibDecoder::new(f);
            let mut z = BufReader::new(z);
            let mut buf = Vec::new();

            // read until (inclusive) the null byte then convert to &str
            z.read_until(0, &mut buf)
                .context("read header from ogit/objects")?;
            let header = CStr::from_bytes_with_nul(&buf)
                .expect("know there is exactly one null, and it is at the end.");
            let header = header
                .to_str()
                .context("ogit/objects file header isn't valid UTF-8")?;

            // extract the kind of file and the size of contents
            let Some((kind, size)) = header.split_once(' ') else {
                anyhow::bail!(
                    "ogit/objects file header did not start with a known type: '{header}'"
                );
            };
            let kind = match kind {
                "blob" => Kind::Blob,
                _ => anyhow::bail!("we do not yet know how to print a '{kind}'"),
            };
            let size = size
                .parse::<usize>()
                .context("ogit/objects file header has invalid size: {size}")?;

            // read `size` bytes into buffer, these bytes don't have to be UTF-8 (picture, etc.)
            buf.clear();
            buf.resize(size, 0); // allocate all zeros then overrite (MaybeUninit optimization?)
            z.read_exact(&mut buf[..])
                .context("read true contents of ogit/objects file")?;

            // the last
            let n = z
                .read(&mut [0])
                .context("validate EOF in ogit/objects file")?;
            anyhow::ensure!(n == 0, "ogit/objects file had {n} trailing byte(s) read");

            let stdout = std::io::stdout();
            let mut stdout = stdout.lock();

            match kind {
                // write bytes, no \n
                Kind::Blob => stdout
                    .write_all(&buf)
                    .context("write object contents to stdout")?,
            }
        }
    };
    Ok(())
}
