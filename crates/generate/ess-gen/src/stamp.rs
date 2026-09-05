//! Admission of the writer's stamp envelopes; never a substring search through model content.

use super::ArtifactDigests;

const LICENSE_SUFFIX: &str =
    "\n--\nThe licence of `assets/mermaid.min.js`, redistributed with this site.\n";

#[derive(PartialEq, Eq)]
struct Envelope {
    system: String,
    specification_version: String,
    digests: ArtifactDigests,
}

struct Comment {
    envelope: Envelope,
    regenerate: String,
}

pub(super) fn read(text: &str) -> Option<ArtifactDigests> {
    let text = text.trim_start();
    let text = text.strip_prefix("<!DOCTYPE html>\n").unwrap_or(text);
    if is_comment(text) {
        let (stamp, rest) = comment(text)?;
        if is_comment(rest.trim_start()) {
            return None;
        }
        // Cargo manifests carry only the comment. OpenAPI and AsyncAPI emit both copies;
        // a missing or malformed YAML copy cannot become a comment-only artifact.
        if text.starts_with("# ")
            && !rest.trim().is_empty()
            && !manifest_body(rest, &stamp.regenerate)
            && structured(rest)? != stamp.envelope
        {
            return None;
        }
        return Some(stamp.envelope.digests);
    }
    let license = include_str!("../assets/mermaid.LICENSE");
    if let Some(rest) = text.strip_prefix(license) {
        let rest = if license.ends_with('\n') {
            rest
        } else {
            rest.strip_prefix('\n')?
        };
        return lines(rest.strip_prefix(LICENSE_SUFFIX)?.lines())
            .map(|stamp| stamp.envelope.digests);
    }
    structured(text).map(|stamp| stamp.digests)
}

fn manifest_body(text: &str, regenerate: &str) -> bool {
    // These are the existing synthesis writers' frames, not a TOML validity check. The
    // projection command never emits a manifest and therefore always owes paired YAML.
    let synthesis = regenerate == "ess synthesize"
        || regenerate.starts_with("ess synthesize --")
        || regenerate == "cargo xtask synth --target clap";
    synthesis
        && matches!(
            text.trim_start().lines().next(),
            Some("[workspace]" | "[package]")
        )
}

fn is_comment(text: &str) -> bool {
    text.starts_with("# generated from ")
        || text.starts_with("// generated from ")
        || text
            .strip_prefix("<!--\n")
            .is_some_and(|rest| rest.trim_start().starts_with("generated from "))
        || text
            .strip_prefix("/*\n")
            .is_some_and(|rest| rest.trim_start().starts_with("* generated from "))
}

fn comment(text: &str) -> Option<(Comment, &str)> {
    for prefix in ["# ", "// "] {
        if text.starts_with(prefix) {
            let mut rest = text;
            let mut block = Vec::new();
            for _ in 0..4 {
                let (line, tail) = rest.split_once('\n')?;
                block.push(line.strip_prefix(prefix)?);
                rest = tail;
            }
            return Some((lines(block.into_iter())?, rest));
        }
    }
    for (open, close, prefix) in [("<!--\n", "-->\n", ""), ("/*\n", " */\n", "*")] {
        if let Some(content) = text.strip_prefix(open) {
            let (block, rest) = content.split_once(close)?;
            let normalized: Option<Vec<_>> = block
                .lines()
                .map(|line| line.trim().strip_prefix(prefix).map(str::trim_start))
                .collect();
            return Some((lines(normalized?.into_iter())?, rest));
        }
    }
    None
}

fn lines<'a>(mut lines: impl Iterator<Item = &'a str>) -> Option<Comment> {
    let origin = lines.next()?.strip_prefix("generated from ")?;
    let (system, specification_version) = origin.split_once(' ')?;
    if system.is_empty() || specification_version.trim().is_empty() {
        return None;
    }
    let source = lines.next()?.strip_prefix("model digest ")?;
    let contract = lines.next()?.strip_prefix("contract digest ")?;
    let regenerate = lines
        .next()?
        .strip_prefix("do not edit: regenerate with `")?;
    if !regenerate.ends_with('`') || regenerate.len() < 2 || lines.next().is_some() {
        return None;
    }
    Some(Comment {
        envelope: Envelope {
            system: system.to_owned(),
            specification_version: specification_version.to_owned(),
            digests: digests(source, contract)?,
        },
        regenerate: regenerate.strip_suffix('`')?.to_owned(),
    })
}

fn hash(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn digests(source: &str, contract: &str) -> Option<ArtifactDigests> {
    let contract_hash = contract.strip_prefix("slice-sha256/2:").unwrap_or(contract);
    (hash(source) && hash(contract_hash)).then(|| ArtifactDigests {
        source_digest: source.to_owned(),
        contract_digest: contract.to_owned(),
    })
}

/// `serde_yaml`'s mapping visitor rejects duplicate keys, including in JSON input. JSON's generic
/// Value visitor overwrites them, so it cannot establish this envelope's uniqueness guarantee.
fn structured(text: &str) -> Option<Envelope> {
    let document: serde_yaml::Value = serde_yaml::from_str(text).ok()?;
    let root = document.as_mapping()?;
    // A docs document has per-page stamps and no single artifact stamp. Do not pick its first page.
    if root.get("format").and_then(serde_yaml::Value::as_str) == Some("ess-docs/1") {
        return None;
    }
    let mut candidates = Vec::new();
    if ["source_digest", "spec_digest", "contract_digest"]
        .iter()
        .any(|key| root.contains_key(*key))
    {
        candidates.push((&document, false));
    }
    for key in ["provenance", "x-ess-provenance"] {
        if let Some(value) = root.get(key) {
            candidates.push((value, key == "x-ess-provenance"));
        }
    }
    if let Some(value) = root
        .get("info")
        .and_then(serde_yaml::Value::as_mapping)
        .and_then(|info| info.get("x-ess-provenance"))
    {
        candidates.push((value, false));
    }
    // One structured location per document. Unlike the paired YAML comment, two locations are
    // ambiguous even if they currently agree; no writer emits that shape.
    if candidates.len() != 1 {
        return None;
    }
    let (value, schema_attribution) = candidates[0];
    let stamp = value.as_mapping()?;
    // Schema Attribution adds regenerate; plain Provenance (including the info extension
    // and a synthesis plan) does not. Require the fields each actual writer promises.
    if schema_attribution {
        required_text(stamp, "regenerate")?;
    }
    let source = match (stamp.get("source_digest"), stamp.get("spec_digest")) {
        (Some(value), None) | (None, Some(value)) => value.as_str()?,
        _ => return None,
    };
    Some(Envelope {
        system: required_text(stamp, "system")?.to_owned(),
        specification_version: required_text(stamp, "specification_version")?.to_owned(),
        digests: digests(source, stamp.get("contract_digest")?.as_str()?)?,
    })
}

fn required_text<'a>(stamp: &'a serde_yaml::Mapping, key: &str) -> Option<&'a str> {
    let value = stamp.get(key)?.as_str()?;
    (!value.trim().is_empty()).then_some(value)
}
