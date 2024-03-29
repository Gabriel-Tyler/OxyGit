use anyhow::Context;
use std::fs;

pub(crate) fn invoke() -> anyhow::Result<()> {
    fs::create_dir(".git").context("create .git dir")?;
    fs::create_dir(".git/objects").context("create .git/objects dir")?;
    fs::create_dir(".git/refs").context("create .git/refs dir")?;
    fs::write(".git/HEAD", "ref: refs/heads/main\n").context("write to .git/HEAD")?;
    println!("Initialized .git directory");
    Ok(())
}
