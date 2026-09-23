//! `ess skill`: the agent plugin's skills and agents, as this binary was built with them.
//!
//! The files live in `plugins/ess/` and are embedded by `build.rs`. Printing them from the binary
//! is what keeps guidance and commands at one version: an agent that never installs the plugin
//! still reads the skill that describes the `ess` it is running.

use std::process::ExitCode;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

include!(concat!(env!("OUT_DIR"), "/plugin_files.rs"));

/// The skill `ess skill` opens with when no path is named.
const FRONT_DOOR: &str = "skills/ess/SKILL.md";

#[derive(Debug, clap::Args)]
pub(crate) struct Input {
    /// A skill (`specify`), an agent (`agents/author`), or a file below a skill
    /// (`specify/references/<file>`). Omitted: the front door and an index of everything.
    path: Option<String>,
    /// Print the index as JSON instead of text.
    #[arg(long, conflicts_with = "path")]
    json: bool,
}

/// One embedded file, as the index describes it.
#[derive(Debug, Serialize)]
struct Entry {
    path: &'static str,
    kind: &'static str,
    name: Option<String>,
    description: Option<String>,
}

#[derive(Debug, Serialize)]
struct Index {
    version: &'static str,
    entries: Vec<Entry>,
}

#[derive(Debug, Deserialize)]
struct Frontmatter {
    name: String,
    description: String,
}

pub(crate) fn run(input: &Input) -> Result<ExitCode> {
    let Some(path) = &input.path else {
        return print_index(input.json);
    };
    let Some((resolved, text)) = resolve(path) else {
        eprintln!("error: `{path}` is not a skill, an agent or a file this binary carries");
        eprintln!("valid paths:");
        for (candidate, _) in FILES {
            eprintln!("  {}", spelling(candidate));
        }
        return Ok(ExitCode::from(2));
    };
    eprintln!("ess {}: {resolved}", env!("CARGO_PKG_VERSION"));
    print!("{text}");
    Ok(ExitCode::SUCCESS)
}

/// The embedded file a spelling names: a skill name, `agents/<name>`, or a path below `skills/`.
fn resolve(path: &str) -> Option<(&'static str, &'static str)> {
    let path = path.trim_matches('/');
    let candidates = [
        format!("skills/{path}/SKILL.md"),
        format!("{path}.md"),
        format!("skills/{path}"),
        path.to_owned(),
    ];
    candidates.iter().find_map(|candidate| {
        FILES
            .iter()
            .find(|(embedded, _)| embedded == candidate)
            .copied()
    })
}

/// The shortest spelling `resolve` accepts for an embedded file.
fn spelling(path: &str) -> String {
    if let Some(skill) = path
        .strip_prefix("skills/")
        .and_then(|rest| rest.strip_suffix("/SKILL.md"))
    {
        return skill.to_owned();
    }
    if let Some(agent) = path
        .strip_prefix("agents/")
        .and_then(|rest| rest.strip_suffix(".md"))
    {
        return format!("agents/{agent}");
    }
    path.strip_prefix("skills/").unwrap_or(path).to_owned()
}

fn frontmatter(text: &str) -> Option<Frontmatter> {
    let rest = text.strip_prefix("---\n")?;
    let (yaml, _) = rest.split_once("\n---")?;
    serde_yaml::from_str(yaml).ok()
}

fn index() -> Result<Index> {
    let entries = FILES
        .iter()
        .map(|(path, text)| {
            let kind = if path.ends_with("/SKILL.md") {
                "skill"
            } else if path.starts_with("agents/") {
                "agent"
            } else {
                "file"
            };
            let (name, description) = if kind == "file" {
                (None, None)
            } else {
                let parsed = frontmatter(text)
                    .with_context(|| format!("{path} has no name and description frontmatter"))?;
                (Some(parsed.name), Some(parsed.description))
            };
            Ok(Entry {
                path,
                kind,
                name,
                description,
            })
        })
        .collect::<Result<_>>()?;
    Ok(Index {
        version: env!("CARGO_PKG_VERSION"),
        entries,
    })
}

fn print_index(json: bool) -> Result<ExitCode> {
    let index = index()?;
    if json {
        println!("{}", serde_json::to_string_pretty(&index)?);
        return Ok(ExitCode::SUCCESS);
    }
    let front = FILES
        .iter()
        .find(|(path, _)| *path == FRONT_DOOR)
        .map(|(_, text)| *text)
        .context("the front-door skill is not embedded")?;
    print!("{front}");
    println!();
    println!("## Carried by this binary (ess {})", index.version);
    println!();
    println!("| `ess skill …` | kind | use |");
    println!("|---|---|---|");
    for entry in &index.entries {
        let summary = entry
            .description
            .as_deref()
            .map_or("", |text| text.split(". ").next().unwrap_or(text));
        println!(
            "| `{}` | {} | {} |",
            spelling(entry.path),
            entry.kind,
            summary.trim_end_matches('.')
        );
    }
    Ok(ExitCode::SUCCESS)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_embedded_file_is_reachable_by_its_short_spelling() {
        for (path, _) in FILES {
            let (resolved, _) = resolve(&spelling(path)).expect("the short spelling resolves");
            assert_eq!(resolved, *path);
        }
    }

    #[test]
    fn every_skill_and_agent_names_itself_after_its_path() {
        for entry in index().expect("the index builds").entries {
            if let Some(name) = entry.name {
                let short = spelling(entry.path);
                assert_eq!(
                    short.rsplit('/').next(),
                    Some(name.as_str()),
                    "{}",
                    entry.path
                );
            }
        }
    }

    /// A skill teaches the area spelling. The flat one still runs, which is why a document teaching
    /// it goes unnoticed by anything that executes a command.
    #[test]
    fn no_embedded_file_teaches_a_flat_spelling() {
        let first_level: Vec<&str> = crate::AREAS.iter().chain(crate::TOOLS).copied().collect();
        for (path, text) in FILES {
            for (number, line) in text.lines().enumerate() {
                let mut rest = line;
                while let Some(position) = rest.find("ess ") {
                    let before = rest[..position].chars().next_back();
                    let after = &rest[position + 4..];
                    rest = after;
                    if before.is_some_and(|c| c.is_alphanumeric() || c == '-' || c == '_') {
                        continue;
                    }
                    let word: String = after
                        .chars()
                        .take_while(|c| c.is_ascii_lowercase() || *c == '-')
                        .collect();
                    if word.is_empty() || word.starts_with('-') {
                        continue;
                    }
                    assert!(
                        first_level.contains(&word.as_str()),
                        "{path}:{}: `ess {word}` is not an area path",
                        number + 1
                    );
                }
            }
        }
    }

    #[test]
    fn the_front_door_is_embedded() {
        assert!(FILES.iter().any(|(path, _)| *path == FRONT_DOOR));
    }
}
