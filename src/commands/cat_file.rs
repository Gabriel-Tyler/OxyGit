use anyhow::Context;

use crate::objects::{Kind, Object};

pub(crate) fn invoke(pretty_print: bool, object_hash: &str) -> anyhow::Result<()> {
    anyhow::ensure!(pretty_print, "mode must be given (hint: use -p)");

    let mut object = Object::read(object_hash).context("parse out object file")?;
    // Output contents of file based on type of file.
    match object.kind {
        Kind::Blob => {
            let stdout = std::io::stdout();
            let mut stdout = stdout.lock();
            let n = std::io::copy(&mut object.reader, &mut stdout)
                .context("write .git/objects file into stdout")?;
            anyhow::ensure!(
                n == object.expected_size,
                ".git/objects file was not the expected size (expected: {}, actual: {n})",
                object.expected_size
            );
        }
        _ => anyhow::bail!("don't yet know how to print'{}'", object.kind),
    }
    Ok(())
}
