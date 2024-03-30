use std::{
    ffi::CStr,
    io::{BufRead, Read, Write},
};

use anyhow::Context;

use crate::objects::{Kind, Object};

pub(crate) fn invoke(name_only: bool, tree_hash: &str) -> anyhow::Result<()> {
    // print all direct children of given tree object (one level)
    anyhow::ensure!(name_only, "only --name-only is supported for now");

    let mut object = Object::read(tree_hash).context("parse out tree object file")?;
    match object.kind {
        Kind::Tree => {
            let mut buf = Vec::new();
            let mut hashbuf = [0; 20];
            let stdout = std::io::stdout();
            let mut stdout = stdout.lock();
            loop {
                buf.clear();
                let n = object
                    .reader
                    .read_until(0, &mut buf)
                    .context("read mode and name of next tree object entry")?;
                if n == 0 {
                    break; // reached end of file
                }
                object
                    .reader
                    .read_exact(&mut hashbuf[..])
                    .context("read 20 byte tree entry hash")?;

                let mut mode_and_name = CStr::from_bytes_with_nul(&buf)
                    .context("convert to CStr")?
                    .to_bytes()
                    .splitn(2, |&b| b == b' ');
                let mode = mode_and_name.next().expect("tree entry has no mode");
                let name = mode_and_name
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("tree entry has no file name"))?;

                if name_only {
                    stdout
                        .write_all(name)
                        .context("write name of tree entry ot stdout")?;
                } else {
                    stdout
                        .write_all(mode)
                        .context("write mode of tree entry ot stdout")?;
                    let kind = "tree";
                    let hash = hex::encode(&hashbuf);
                    write!(stdout, " {kind} {hash} ")
                        .context("tree entry kind and hash to stdout")?;
                    stdout
                        .write_all(name)
                        .context("write mode of tree entry ot stdout")?;
                }
                writeln!(stdout, "").context("write newline")?;
            }
        }
        _ => anyhow::bail!("ls_tree can only ls trees, given '{}'", object.kind),
    }

    Ok(())
}
