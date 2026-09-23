//! `ess skill` carries exactly the plugin's skills and agents, and refuses what it does not carry.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .find(|directory| {
            std::fs::read_to_string(directory.join("Cargo.toml"))
                .is_ok_and(|manifest| manifest.starts_with("[workspace]"))
        })
        .expect("a member of this workspace lies under its root")
        .to_path_buf()
}

fn ess(arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ess"))
        .args(arguments)
        .output()
        .expect("the ess binary runs")
}

fn files(root: &Path, directory: &Path, found: &mut BTreeSet<String>) {
    for entry in std::fs::read_dir(directory).expect("a readable plugin directory") {
        let path = entry.expect("an entry").path();
        if path.is_dir() {
            files(root, &path, found);
        } else {
            let relative = path.strip_prefix(root).expect("under the plugin root");
            found.insert(relative.to_string_lossy().replace('\\', "/"));
        }
    }
}

#[test]
fn the_index_lists_every_skill_and_agent_file_in_the_plugin() {
    let plugin = workspace_root().join("plugins/ess");
    let mut expected = BTreeSet::new();
    files(&plugin, &plugin.join("skills"), &mut expected);
    files(&plugin, &plugin.join("agents"), &mut expected);

    let output = ess(&["skill", "--json"]);
    assert!(output.status.success());
    let index: serde_json::Value = serde_json::from_slice(&output.stdout).expect("JSON index");
    assert_eq!(index["version"], env!("CARGO_PKG_VERSION"));
    let listed: BTreeSet<String> = index["entries"]
        .as_array()
        .expect("entries")
        .iter()
        .map(|entry| entry["path"].as_str().expect("a path").to_owned())
        .collect();
    assert_eq!(listed, expected);
}

#[test]
fn a_skill_prints_the_file_it_names_byte_for_byte() {
    let path = workspace_root().join("plugins/ess/skills/specify/SKILL.md");
    let output = ess(&["skill", "specify"]);
    assert!(output.status.success());
    assert_eq!(output.stdout, std::fs::read(path).expect("the skill file"));
    let stderr = String::from_utf8(output.stderr).expect("UTF-8");
    assert_eq!(
        stderr,
        format!(
            "ess {}: skills/specify/SKILL.md\n",
            env!("CARGO_PKG_VERSION")
        )
    );
}

#[test]
fn an_unknown_path_exits_2_and_names_the_valid_ones() {
    let output = ess(&["skill", "no-such-skill"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).expect("UTF-8");
    assert!(
        stderr.contains("`no-such-skill` is not a skill"),
        "{stderr}"
    );
    assert!(stderr.contains("  agents/author\n"), "{stderr}");
    assert!(stderr.contains("  specify\n"), "{stderr}");
}
