use anyhow::Context;
use std::{fs, io::Write, path::Path};

use crate::objects::Object;

pub(crate) fn invoke(write: bool, file: &Path) -> anyhow::Result<()> {
    // by default, type of file is blob

    fn write_blob<W: Write>(file: &Path, writer: W) -> anyhow::Result<String> {
        let hash = Object::blob_from_file(file)
            .context("open blob input file")?
            .write(writer)
            .context("stream file into blob")?;
        Ok(hex::encode(hash))
    }

    let hash = if write {
        let tmp = "temporary"; // ideally random name
        let hash = write_blob(
            file,
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
        write_blob(file, std::io::sink()).context("write out blob object")?
    };

    println!("{hash}");

    Ok(())
}
