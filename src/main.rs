use anyhow::Context;
use clap::{Parser, Subcommand};
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Digest, Sha1};
use std::ffi::CStr;
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};

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
            // Output contents of file based on type of file.
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
        Commands::HashObject { write, path } => {
            // by default, type of file is blob

            fn write_blob<W: Write>(path: &Path, writer: W) -> anyhow::Result<String> {
                let stat =
                    fs::metadata(path).with_context(|| format!("stat {}", path.display()))?;
                let size = stat.len();

                let writer = ZlibEncoder::new(writer, Compression::default());
                let mut writer = HashWriter {
                    writer,
                    hasher: Sha1::new(),
                };

                // write header
                write!(writer, "blob {size}\0")?;

                // write body
                let mut file = std::fs::File::open(&path)?;
                std::io::copy(&mut file, &mut writer).context("stream file into blob")?;

                // flush compress and hash
                let hash = writer.hasher.finalize();
                writer.writer.finish()?;

                Ok(hex::encode(hash))
            }

            let hash = if write {
                let tmp = "temporary"; // ideally random name
                let hash = write_blob(
                    &path,
                    fs::File::create(tmp).context("construct temp file for blob")?,
                )
                .context("write blob object to temp file")?;
                fs::create_dir_all(format!(".git/objects/{}/", &hash[..2]))
                    .context("create subdir of .git/objects")?;
                fs::rename(tmp, format!(".git/objects/{}/{}", &hash[..2], &hash[2..]))
                    .context("move blob file into .git/objects")?;
                hash
            } else {
                // sink consume into the void
                write_blob(&path, std::io::sink()).context("write out blob object")?
            };

            println!("{hash}");
        }
    }
    Ok(())
}

struct HashWriter<W> {
    writer: W,
    hasher: Sha1,
}

impl<W> Write for HashWriter<W>
where
    W: Write,
{
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let n = self.writer.write(buf)?;
        self.hasher.update(&buf[..n]);
        Ok(n)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}
