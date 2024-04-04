# OxyGit

A custom Git implementation in Rust.

Usage: `cargo run -- [COMMAND] ...`
- Use `cargo run -- --help` for more information.

Commands implemented so far:
- `init`
- `cat-file -p <SHA1_HASH>`
- `hash-object [-w] <PATH>`
- `ls-tree [--name-only] <SHA1_HASH>`
- `write-tree`

This project is an exercise in understanding the Git data model and maintaining a large Rust project.

# Resources
- Git Book: https://git-scm.com/book/en/v2/Git-Internals-Plumbing-and-Porcelain
- Codecrafters: https://app.codecrafters.io/courses/git/introduction
- Missing CS semester: https://www.youtube.com/watch?v=2sjqTHE0zok
