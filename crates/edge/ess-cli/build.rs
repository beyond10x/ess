//! Embeds the agent plugin's skills and agents into the `ess` binary.
//!
//! `ess skill` prints the guidance the binary was built with, so an agent that runs a binary reads
//! the instructions for that binary's commands and not for whichever plugin release it happened to
//! install. The files stay where the plugin marketplace serves them from, `plugins/ess/`; this
//! script only lists them. The workspace is `publish = false`, so a path outside the crate is
//! legitimate here.

use std::env;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

/// The plugin subdirectories an agent reads. Manifests stay out: they describe the package, not
/// the work.
const EMBEDDED: [&str; 2] = ["skills", "agents"];

fn main() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("cargo sets the manifest"));
    let plugin = manifest.join("../../../plugins/ess");
    println!("cargo:rerun-if-changed={}", plugin.display());

    let mut files = Vec::new();
    for directory in EMBEDDED {
        collect(&plugin, &plugin.join(directory), &mut files);
    }
    files.sort();

    let mut source = String::from("pub(crate) static FILES: &[(&str, &str)] = &[\n");
    for (relative, absolute) in &files {
        writeln!(source, "    ({relative:?}, include_str!({absolute:?})),").expect("in memory");
    }
    source.push_str("];\n");

    let out = PathBuf::from(env::var("OUT_DIR").expect("cargo sets OUT_DIR"));
    fs::write(out.join("plugin_files.rs"), source).expect("OUT_DIR is writable");
}

fn collect(root: &Path, directory: &Path, files: &mut Vec<(String, String)>) {
    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };
    for entry in entries {
        let path = entry.expect("a readable plugin directory entry").path();
        println!("cargo:rerun-if-changed={}", path.display());
        if path.is_dir() {
            collect(root, &path, files);
        } else {
            let relative = path
                .strip_prefix(root)
                .expect("collected under the plugin root")
                .components()
                .map(|part| part.as_os_str().to_string_lossy().into_owned())
                .collect::<Vec<_>>()
                .join("/");
            let absolute = path
                .canonicalize()
                .expect("a plugin file resolves")
                .to_string_lossy()
                .into_owned();
            files.push((relative, absolute));
        }
    }
}
