use anyhow::Context;
use std::path::Path;

use crate::objects::Object;

pub(crate) fn invoke(write: bool, file: &Path) -> anyhow::Result<()> {
    // by default, type of file is blob
    let object = Object::blob_from_file(file).context("open blob input file")?;
    let hash = if write {
        object
            .write_to_objects()
            .context("stream object into blob object file")?
    } else {
        // sink consume into the void
        object
            .write(std::io::sink())
            .context("stream object into void")?
    };

    println!("{}", hex::encode(hash));

    Ok(())
}
