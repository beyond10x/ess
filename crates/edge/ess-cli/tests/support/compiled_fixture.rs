//! One compilation of a standalone Rust fixture program, shared by every test process.
//!
//! Nextest runs each test in its own process, so a fixture program compiled behind a
//! per-process `OnceLock` was compiled once per test. This compiles it once per source bytes,
//! compiler and flags into `CARGO_TARGET_TMPDIR`, which every test binary of the build shares,
//! and hands each caller the same executable to copy or run.
use std::{
    collections::hash_map::DefaultHasher,
    ffi::OsString,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

/// The executable `rustc` builds from `source` with `flags`, compiled on first use.
///
/// The key is the source's bytes, the compiler and the flags, so an edited fixture is a
/// different program and never reuses a stale executable. Concurrent first uses each compile
/// into a private name and rename it into place: the rename is atomic and every candidate is
/// the same program, so a reader never sees a partial file.
pub fn compiled(source: &Path, compiler: &OsString, flags: &[&str]) -> PathBuf {
    let bytes = std::fs::read(source)
        .unwrap_or_else(|error| panic!("reading fixture source {}: {error}", source.display()));
    let mut hasher = DefaultHasher::new();
    bytes.hash(&mut hasher);
    compiler.hash(&mut hasher);
    flags.hash(&mut hasher);
    let stem = source.file_stem().unwrap().to_string_lossy();
    let directory = Path::new(env!("CARGO_TARGET_TMPDIR")).join("compiled-fixtures");
    let program = directory.join(format!(
        "{stem}-{:016x}{}",
        hasher.finish(),
        std::env::consts::EXE_SUFFIX
    ));
    if program.is_file() {
        return program;
    }
    std::fs::create_dir_all(&directory).unwrap();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let staging = directory.join(format!(
        "{stem}-{}-{nanos}.partial{}",
        std::process::id(),
        std::env::consts::EXE_SUFFIX
    ));
    let output = Command::new(compiler)
        .args(flags)
        .arg(source)
        .arg("-o")
        .arg(&staging)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "compiling fixture {}: {}",
        source.display(),
        String::from_utf8_lossy(&output.stderr)
    );
    std::fs::rename(&staging, &program).unwrap();
    program
}
