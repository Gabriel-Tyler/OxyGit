use anyhow::Context;
use std::{fs, io::Write, path::Path};

use crate::objects::{Kind, Object};

pub(crate) fn invoke(write: bool, path: &Path) -> anyhow::Result<()> {
    // by default, type of file is blob

    fn write_blob<W: Write>(path: &Path, writer: W) -> anyhow::Result<String> {
        let stat = fs::metadata(path).with_context(|| format!("stat {}", path.display()))?;

        // TODO: technically a race condition if file changes between stat and write
        let file = std::fs::File::open(path).with_context(|| format!("open {}", path.display()))?;
        let hash = Object {
            kind: Kind::Blob,
            expected_size: stat.len(),
            reader: file,
        }
        .write(writer)
        .context("stream file into blob")?;
        Ok(hex::encode(hash))
    }

    let hash = if write {
        let tmp = "temporary"; // ideally random name
        let hash = write_blob(
            path,
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
        write_blob(path, std::io::sink()).context("write out blob object")?
    };

    println!("{hash}");

    Ok(())
}
