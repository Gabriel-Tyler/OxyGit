use anyhow::Context;
use core::fmt;
use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};
use sha1::{Digest, Sha1};
use std::{
    ffi::CStr,
    fs,
    io::{BufRead, BufReader, Read, Write},
    path::Path,
};

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    Blob,
    Tree,
    Commit,
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Kind::Blob => write!(f, "blob"),
            Kind::Tree => write!(f, "tree"),
            Kind::Commit => write!(f, "commit"),
        }
    }
}

pub(crate) struct Object<R> {
    pub(crate) kind: Kind,
    pub(crate) expected_size: u64,
    pub(crate) reader: R,
}

impl Object<()> {
    // may want to return a buffered reader
    pub(crate) fn blob_from_file(file: impl AsRef<Path>) -> anyhow::Result<Object<impl Read>> {
        let file = file.as_ref();
        let stat = fs::metadata(file).with_context(|| format!("stat {}", file.display()))?;

        // TODO: technically a race condition if file changes between stat and write
        let file = std::fs::File::open(file).with_context(|| format!("open {}", file.display()))?;
        Ok(Object {
            kind: Kind::Blob,
            expected_size: stat.len(),
            reader: file,
        })
    }
    pub(crate) fn read(hash: &str) -> anyhow::Result<Object<impl BufRead>> {
        let (prefix, rest) = hash.split_at(2);
        let f = fs::File::open(format!(".git/objects/{prefix}/{rest}"))
            .with_context(|| format!("open .git/objects/{prefix}/{rest}"))?;
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
            anyhow::bail!(".git/objects file header did not start with a known type: '{header}'");
        };
        let kind = match kind {
            "blob" => Kind::Blob,
            "tree" => Kind::Tree,
            "commit" => Kind::Commit,
            _ => anyhow::bail!("unknown kind '{kind}'"),
        };
        let size = size
            .parse::<u64>()
            .context(".git/objects file header has invalid size: {size}")?;

        // NOTE: won't error if decompressed file is too long, but will not spam stdout
        //   and be vulnerable to a zipbomb.
        let z = z.take(size);
        Ok(Object {
            kind,
            expected_size: size,
            reader: z,
        })
    }
}

impl<R> Object<R>
where
    R: Read,
{
    pub(crate) fn write(mut self, writer: impl Write) -> anyhow::Result<[u8; 20]> {
        // wrap writer in zlib encoder and hasher
        let writer = ZlibEncoder::new(writer, Compression::default());
        let mut writer = HashWriter {
            writer,
            hasher: Sha1::new(),
        };

        // write header and body
        write!(writer, "{} {}\0", self.kind, self.expected_size)?;
        std::io::copy(&mut self.reader, &mut writer).context("stream file into blob")?;

        // flush hash and compress
        writer.writer.finish()?;
        let hash = writer.hasher.finalize();

        Ok(hash.into())
    }

    pub(crate) fn write_to_objects(self) -> anyhow::Result<[u8; 20]> {
        let tmp = "temporary"; // ideally random name
        let hash = self
            .write(fs::File::create(tmp).context("construct temp file for tree")?)
            .context("stream tree object into tree object file")?;
        let hash_hex = hex::encode(hash);
        fs::create_dir_all(format!(".git/objects/{}/", &hash_hex[..2]))
            .context("create subdir of .git/objects")?;
        fs::rename(
            tmp,
            format!(".git/objects/{}/{}", &hash_hex[..2], &hash_hex[2..]),
        )
        .context("move tree file into .git/objects")?;
        Ok(hash)
    }
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
