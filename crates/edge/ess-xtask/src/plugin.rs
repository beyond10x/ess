//! `cargo xtask plugin check`: the agent plugin agrees with the workspace it ships from.
//!
//! One release tag carries the `ess` binary, the plugin and its skills. The binary embeds the
//! skills, so those cannot drift from it; the manifests and marketplaces can, and a plugin that
//! announces another version than the binary it describes is the drift this repository split the
//! plugin out of `agentplugins` to end.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

/// The marketplace identity and the one plugin it lists.
const IDENTITY: &str = "ess";
const PLUGIN_DIR: &str = "plugins/ess";

/// The two plugin manifests, one per host.
const MANIFESTS: [&str; 2] = [".claude-plugin/plugin.json", ".codex-plugin/plugin.json"];

/// The two marketplaces, one per host.
const MARKETPLACES: [&str; 2] = [
    ".claude-plugin/marketplace.json",
    ".agents/plugins/marketplace.json",
];

pub(crate) fn check(root: &Path, version: &str) -> Result<String, String> {
    let mut problems = Vec::new();
    for marketplace in MARKETPLACES {
        problems.extend(check_marketplace(root, marketplace));
    }
    let plugin = root.join(PLUGIN_DIR);
    for manifest in MANIFESTS {
        problems.extend(check_manifest(&plugin, manifest, version));
    }
    let skills = skill_names(&plugin)?;
    problems.extend(check_skills(&plugin, &skills));
    problems.extend(check_agents(&plugin, &skills));

    if problems.is_empty() {
        Ok(format!(
            "plugin {IDENTITY} {version}: {} skill(s), manifests and marketplaces agree\n",
            skills.len()
        ))
    } else {
        Err(format!("plugin check failed:\n  {}", problems.join("\n  ")))
    }
}

fn json(path: &Path) -> Result<serde_json::Value, String> {
    let text = fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))?;
    serde_json::from_str(&text).map_err(|error| format!("{}: {error}", path.display()))
}

fn check_marketplace(root: &Path, relative: &str) -> Vec<String> {
    let document = match json(&root.join(relative)) {
        Ok(document) => document,
        Err(problem) => return vec![problem],
    };
    let mut problems = Vec::new();
    if document["name"] != IDENTITY {
        problems.push(format!(
            "{relative} does not declare marketplace `{IDENTITY}`"
        ));
    }
    let listed = document["plugins"].as_array().is_some_and(|plugins| {
        plugins.iter().any(|plugin| {
            plugin["name"] == IDENTITY
                && (plugin["source"] == format!("./{PLUGIN_DIR}")
                    || plugin["source"]["path"] == format!("./{PLUGIN_DIR}"))
        })
    });
    if !listed {
        problems.push(format!(
            "{relative} does not list `{IDENTITY}` at `./{PLUGIN_DIR}`"
        ));
    }
    problems
}

fn check_manifest(plugin: &Path, relative: &str, version: &str) -> Vec<String> {
    let document = match json(&plugin.join(relative)) {
        Ok(document) => document,
        Err(problem) => return vec![problem],
    };
    let mut problems = Vec::new();
    if document["name"] != IDENTITY {
        problems.push(format!("{PLUGIN_DIR}/{relative} is not named `{IDENTITY}`"));
    }
    if document["version"] != version {
        problems.push(format!(
            "{PLUGIN_DIR}/{relative} carries version {}, the workspace is {version}",
            document["version"]
        ));
    }
    problems
}

fn skill_names(plugin: &Path) -> Result<BTreeSet<String>, String> {
    let directory = plugin.join("skills");
    let entries =
        fs::read_dir(&directory).map_err(|error| format!("{}: {error}", directory.display()))?;
    let mut names = BTreeSet::new();
    for entry in entries {
        let entry = entry.map_err(|error| format!("{}: {error}", directory.display()))?;
        if entry.path().is_dir() {
            names.insert(entry.file_name().to_string_lossy().into_owned());
        }
    }
    Ok(names)
}

/// The `name:` a markdown file's frontmatter declares.
fn frontmatter_name(text: &str) -> Option<String> {
    let rest = text.strip_prefix("---\n")?;
    let (yaml, _) = rest.split_once("\n---")?;
    let value: serde_yaml::Value = serde_yaml::from_str(yaml).ok()?;
    value["description"].as_str()?;
    value["name"].as_str().map(str::to_owned)
}

fn check_skills(plugin: &Path, skills: &BTreeSet<String>) -> Vec<String> {
    let mut problems = Vec::new();
    for skill in skills {
        let path = plugin.join("skills").join(skill).join("SKILL.md");
        match fs::read_to_string(&path) {
            Err(error) => problems.push(format!("{}: {error}", path.display())),
            Ok(text) => match frontmatter_name(&text) {
                Some(name) if &name == skill => {}
                Some(name) => problems.push(format!(
                    "skills/{skill}/SKILL.md declares name `{name}`, its folder is `{skill}`"
                )),
                None => problems.push(format!(
                    "skills/{skill}/SKILL.md has no `name` and `description` frontmatter"
                )),
            },
        }
    }
    problems
}

/// Every Claude agent wrapper is named after its file and runs a skill that exists.
fn check_agents(plugin: &Path, skills: &BTreeSet<String>) -> Vec<String> {
    let directory = plugin.join("agents");
    let Ok(entries) = fs::read_dir(&directory) else {
        return Vec::new();
    };
    let mut problems = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(stem) = path
            .file_stem()
            .map(|stem| stem.to_string_lossy().into_owned())
        else {
            continue;
        };
        let Ok(text) = fs::read_to_string(&path) else {
            problems.push(format!("agents/{stem}.md is unreadable"));
            continue;
        };
        match frontmatter_name(&text) {
            Some(name) if name == stem => {}
            _ => problems.push(format!(
                "agents/{stem}.md does not declare `name: {stem}` with a description"
            )),
        }
        let backed = skills
            .iter()
            .any(|skill| text.contains(&format!("`ess skill {skill}`")));
        if !backed {
            problems.push(format!(
                "agents/{stem}.md names no existing skill as `ess skill <name>`"
            ));
        }
    }
    problems
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(root: &Path, relative: &str, text: &str) {
        let path = root.join(relative);
        fs::create_dir_all(path.parent().expect("a parent")).expect("mkdir");
        fs::write(path, text).expect("write");
    }

    fn fixture(root: &Path, version: &str) {
        write(
            root,
            ".claude-plugin/marketplace.json",
            r#"{"name":"ess","plugins":[{"name":"ess","source":"./plugins/ess"}]}"#,
        );
        write(
            root,
            ".agents/plugins/marketplace.json",
            r#"{"name":"ess","plugins":[{"name":"ess","source":{"source":"local","path":"./plugins/ess"}}]}"#,
        );
        for manifest in MANIFESTS {
            write(
                root,
                &format!("plugins/ess/{manifest}"),
                &format!(r#"{{"name":"ess","version":"{version}"}}"#),
            );
        }
        write(
            root,
            "plugins/ess/skills/specify/SKILL.md",
            "---\nname: specify\ndescription: d\n---\n",
        );
        write(
            root,
            "plugins/ess/agents/author.md",
            "---\nname: author\ndescription: d\n---\nRun `ess skill specify`.\n",
        );
    }

    fn scratch(name: &str) -> std::path::PathBuf {
        let root =
            std::env::temp_dir().join(format!("ess-xtask-plugin-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        root
    }

    #[test]
    fn the_committed_plugin_agrees_with_the_workspace() {
        let root = crate::workspace_root().expect("the workspace root");
        let (version, _) = crate::release_inputs(&root).expect("the release inputs");
        check(&root, &version).expect("the committed plugin agrees");
    }

    #[test]
    fn a_consistent_fixture_passes() {
        let root = scratch("ok");
        fixture(&root, "1.2.3");
        check(&root, "1.2.3").expect("the fixture agrees");
        fs::remove_dir_all(&root).expect("cleanup");
    }

    #[test]
    fn a_manifest_behind_the_workspace_is_refused() {
        let root = scratch("version");
        fixture(&root, "1.2.3");
        let error = check(&root, "1.3.0").expect_err("a stale manifest must fail");
        assert!(
            error.contains("carries version \"1.2.3\", the workspace is 1.3.0"),
            "{error}"
        );
        fs::remove_dir_all(&root).expect("cleanup");
    }

    #[test]
    fn a_skill_named_unlike_its_folder_is_refused() {
        let root = scratch("name");
        fixture(&root, "1.2.3");
        write(
            &root,
            "plugins/ess/skills/specify/SKILL.md",
            "---\nname: other\ndescription: d\n---\n",
        );
        let error = check(&root, "1.2.3").expect_err("a misnamed skill must fail");
        assert!(error.contains("declares name `other`"), "{error}");
        fs::remove_dir_all(&root).expect("cleanup");
    }

    #[test]
    fn an_agent_backed_by_no_skill_is_refused() {
        let root = scratch("agent");
        fixture(&root, "1.2.3");
        write(
            &root,
            "plugins/ess/agents/author.md",
            "---\nname: author\ndescription: d\n---\nRun `ess skill gone`.\n",
        );
        let error = check(&root, "1.2.3").expect_err("an orphan agent must fail");
        assert!(error.contains("names no existing skill"), "{error}");
        fs::remove_dir_all(&root).expect("cleanup");
    }

    #[test]
    fn a_marketplace_under_another_identity_is_refused() {
        let root = scratch("identity");
        fixture(&root, "1.2.3");
        write(
            &root,
            ".claude-plugin/marketplace.json",
            r#"{"name":"beyond10x","plugins":[{"name":"ess","source":"./plugins/ess"}]}"#,
        );
        let error = check(&root, "1.2.3").expect_err("a foreign identity must fail");
        assert!(
            error.contains("does not declare marketplace `ess`"),
            "{error}"
        );
        fs::remove_dir_all(&root).expect("cleanup");
    }
}
