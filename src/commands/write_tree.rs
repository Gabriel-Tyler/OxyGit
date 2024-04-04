use anyhow::Context;
use std::io::Cursor;
use std::os::unix::fs::PermissionsExt;
use std::{fs, os::unix::fs::MetadataExt, path::Path};

use crate::objects::{Kind, Object};

fn write_tree_for(path: &Path) -> anyhow::Result<Option<[u8; 20]>> {
    let mut dir =
        fs::read_dir(path).with_context(|| format!("open directory {}", path.display()))?;
    let mut tree_object = Vec::new();
    while let Some(entry) = dir.next() {
        let entry = entry.with_context(|| format!("bad directory entry in {}", path.display()))?;
        let file_name = entry.file_name();
        let meta = entry.metadata().context("metadata for directory entry")?;
        let mode = if meta.is_dir() {
            "40000"
        } else if meta.is_symlink() {
            "120000"
        } else if (meta.permissions().mode() & 0o111) != 0 {
            // has at least one executable bit
            "100755"
        } else {
            "100644"
        };
        let path = entry.path();
        let hash = if meta.is_dir() {
            let Some(hash) = write_tree_for(&path)? else {
                // empty directory, don't include in parent
                continue;
            };
            hash
        } else {
            let tmp = "temporary"; // ideally random name
            let hash = Object::blob_from_file(&path)
                .context("open blob input file")?
                .write(fs::File::create(tmp).context("construct temp file for blob")?)
                .context("stream file into blob")?;
            let hash_hex = hex::encode(hash);
            fs::create_dir_all(format!(".git/objects/{}/", &hash_hex[..2]))
                .context("create subdir of .git/objects")?;
            fs::rename(
                tmp,
                format!(".git/objects/{}/{}", &hash_hex[..2], &hash_hex[2..]),
            )
            .context("move blob file into .git/objects")?;
            hash
        };
        tree_object.extend(mode.as_bytes());
        tree_object.push(b' ');
        tree_object.extend(file_name.as_encoded_bytes());
        tree_object.push(b'\0');
        tree_object.extend(hash);
    }
    if tree_object.is_empty() {
        Ok(None)
    } else {
        Ok(Some(
            Object {
                kind: Kind::Tree,
                expected_size: tree_object.len() as u64,
                reader: Cursor::new(tree_object),
            }
            .write_to_objects()
            .context("write tree object")?,
        ))
    }
}

pub(crate) fn invoke() -> anyhow::Result<()> {
    Ok(())
}
