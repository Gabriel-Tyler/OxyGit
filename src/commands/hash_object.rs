use anyhow::Context;
use flate2::{write::ZlibEncoder, Compression};
use sha1::{Digest, Sha1};
use std::{fs, io::Write, path::Path};

pub fn invoke(write: bool, path: &Path) -> anyhow::Result<()> {
    // by default, type of file is blob

    fn write_blob<W: Write>(path: &Path, writer: W) -> anyhow::Result<String> {
        let stat = fs::metadata(path).with_context(|| format!("stat {}", path.display()))?;
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

        // flush hash and compress
        writer.writer.finish()?;
        let hash = writer.hasher.finalize();

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
