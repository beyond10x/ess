//! Bind the compiled schema provider to the complete source bytes used for this build.
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fs, path::Path, process::Command};
fn hash(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .flat_map(|b| {
            [
                char::from(b"0123456789abcdef"[usize::from(b >> 4)]),
                char::from(b"0123456789abcdef"[usize::from(b & 15)]),
            ]
        })
        .collect()
}
fn main() {
    let manifest = std::env::var("CARGO_MANIFEST_DIR").expect("Cargo manifest");
    let root = Path::new(&manifest)
        .ancestors()
        .nth(3)
        .expect("workspace root");
    let output = Command::new("git")
        .args([
            "ls-files",
            "-z",
            "--cached",
            "--others",
            "--exclude-standard",
        ])
        .current_dir(root)
        .output()
        .expect("Git source authority");
    assert!(output.status.success(), "Git source authority refused");
    let mut files = BTreeMap::new();
    for path in output.stdout.split(|b| *b == 0).filter(|p| !p.is_empty()) {
        let path = std::str::from_utf8(path).expect("source authority has non-UTF8 name");
        let bytes = fs::read(root.join(path)).expect("source authority file");
        files.insert(
            path.to_owned(),
            Sha256::digest(bytes)
                .iter()
                .flat_map(|b| {
                    [
                        char::from(b"0123456789abcdef"[usize::from(b >> 4)]),
                        char::from(b"0123456789abcdef"[usize::from(b & 15)]),
                    ]
                })
                .collect::<String>(),
        );
        println!("cargo:rerun-if-changed={}", root.join(path).display());
    }
    // Directory changes detect new untracked package/model/test inputs as well as edits.
    println!("cargo:rerun-if-changed={}", root.join("crates").display());
    println!("cargo:rerun-if-changed={}", root.join("examples").display());
    let out = std::env::var("OUT_DIR").expect("Cargo output directory");
    fs::write(
        Path::new(&out).join("consumer-source.json"),
        serde_json::to_vec(&files).expect("source serialization"),
    )
    .expect("source stamp write");
    let environment = std::env::vars()
        .filter(|(k, _)| {
            k.starts_with("CARGO_CFG_")
                || k.starts_with("CARGO_FEATURE_")
                || matches!(
                    k.as_str(),
                    "HOST"
                        | "TARGET"
                        | "PROFILE"
                        | "DEBUG"
                        | "OPT_LEVEL"
                        | "NUM_JOBS"
                        | "CARGO_ENCODED_RUSTFLAGS"
                        | "RUSTC"
                        | "RUSTDOC"
                        | "RUSTC_WRAPPER"
                        | "RUSTC_WORKSPACE_WRAPPER"
                        | "CARGO_BUILD_RUSTC_WRAPPER"
                        | "CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER"
                )
        })
        .collect::<BTreeMap<_, _>>();
    let mut tools = BTreeMap::new();
    for (key, args) in [("RUSTC", vec!["-vV"]), ("CARGO", vec!["-Vv"])] {
        let path = std::env::var(key).expect("Cargo tool identity");
        let identity = Command::new(&path)
            .args(args)
            .output()
            .expect("tool version");
        assert!(identity.status.success(), "tool version refused");
        tools.insert(key,serde_json::json!({"path":path,"sha256":hash(&fs::read(&path).expect("tool bytes")),"version":String::from_utf8(identity.stdout).expect("tool version UTF8")}));
    }
    let mut configuration = BTreeMap::new();
    let cargo_home = std::env::var_os("CARGO_HOME").map_or_else(
        || {
            std::path::PathBuf::from(std::env::var_os("HOME").expect("normal Cargo home"))
                .join(".cargo")
        },
        std::path::PathBuf::from,
    );
    let dirs = root
        .ancestors()
        .map(|p| p.join(".cargo"))
        .chain(std::iter::once(cargo_home));
    for dir in dirs {
        for name in ["config", "config.toml"] {
            let path = dir.join(name);
            if path.is_file() {
                configuration.insert(
                    path.to_string_lossy().into_owned(),
                    hash(&fs::read(&path).expect("Cargo configuration")),
                );
                println!("cargo:rerun-if-changed={}", path.display());
            }
        }
    }
    fs::write(Path::new(&out).join("consumer-build.json"),serde_json::to_vec(&serde_json::json!({"environment":environment,"tools":tools,"cargo_configuration":configuration})).expect("build profile serialization")).expect("build profile write");
}
