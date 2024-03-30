#[derive(Debug)]
enum Kind {
    Blob,
}

pub(crate) fn invoke(name_only: bool, tree_hash: &str) -> anyhow::Result<()> {
    // print all direct children of given tree object (one level)
    anyhow::ensure!(name_only, "only --name-only is supported for now");

    Ok(())
}
