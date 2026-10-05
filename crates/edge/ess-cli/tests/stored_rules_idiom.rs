//! Rules held as data are the system's to evaluate (beyond10x/ess#451).
//!
//! ESS does not evaluate a rule the system stores at run time. The boundary it recommends: the
//! rule's parts are typed rows, whether a stored rule holds is an `external:` outcome a suite
//! injects, and an ALL/ANY fold over stored per-condition results is a `when_related` row-set
//! guard. `fixtures/stored-rules/` holds both models; `docs/design/stored-rules-boundary.md` and
//! the guide section state the boundary, and these cases read them.

use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

const NOTE: &str = "docs/design/stored-rules-boundary.md";
const GUIDE: &str = "website/docs/guides/specify/guards-and-predicates.md";
const GUIDE_HEADING: &str = "## A rule stored as data is evaluated by the system";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/stored-rules")
        .join(name)
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
}

struct Run {
    code: Option<i32>,
    output: String,
}

fn ess(arguments: &[&str]) -> Run {
    let output = Command::new(env!("CARGO_BIN_EXE_ess"))
        .args(arguments)
        .output()
        .expect("run the built ess binary");
    Run {
        code: output.status.code(),
        output: format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ),
    }
}

/// Validates `model` and synthesizes its suite, which synthesis refuses nothing of.
fn synthesized(model: &Path) -> serde_json::Value {
    let path = model.to_str().expect("a UTF-8 path");
    let validate = ess(&["specify", "validate", "--path", path]);
    assert_eq!(validate.code, Some(0), "validates:\n{}", validate.output);
    let directory = tempfile::tempdir().expect("a temporary directory");
    let out = directory.path().join("suite.json");
    let synthesize = ess(&[
        "verify",
        "conform",
        "synthesize",
        "--path",
        path,
        "--out",
        out.to_str().expect("a UTF-8 path"),
    ]);
    assert_eq!(
        synthesize.code,
        Some(0),
        "synthesizes:\n{}",
        synthesize.output
    );
    assert!(
        synthesize.output.contains(" 0 refusal(s)"),
        "synthesis refuses nothing:\n{}",
        synthesize.output
    );
    serde_json::from_str(&read(&out)).expect("the suite is JSON")
}

fn ids(suite: &serde_json::Value) -> Vec<String> {
    suite["scenarios"]
        .as_object()
        .expect("the suite has scenarios")
        .keys()
        .cloned()
        .collect()
}

/// The text from `heading` to the next heading of the same level or above.
fn section<'t>(text: &'t str, heading: &str) -> Option<&'t str> {
    let level = heading.split(' ').next().unwrap_or_default().len();
    let start = text.find(&format!("\n{heading}\n"))? + 1;
    let body = &text[start + heading.len()..];
    let end = body
        .lines()
        .scan(0, |offset, line| {
            let at = *offset;
            *offset += line.len() + 1;
            Some((at, line))
        })
        .skip(1)
        .find(|(_, line)| {
            let hashes = line.chars().take_while(|c| *c == '#').count();
            hashes > 0 && hashes <= level && line[hashes..].starts_with(' ')
        })
        .map_or(body.len(), |(at, _)| at);
    Some(&text[start..start + heading.len() + end])
}

/// Every fenced YAML block's body in `text`, in order.
fn yaml_blocks(text: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut open: Option<String> = None;
    for line in text.lines() {
        match &mut open {
            None if line.trim_start().starts_with("```yaml") => open = Some(String::new()),
            Some(body) if line.trim_start() == "```" => {
                blocks.push(std::mem::take(body));
                open = None;
            }
            Some(body) => {
                body.push_str(line);
                body.push('\n');
            }
            None => {}
        }
    }
    blocks
}

/// The targets of every Markdown link in `text`.
fn link_targets(text: &str) -> Vec<&str> {
    text.match_indices("](")
        .filter_map(|(at, _)| {
            let rest = &text[at + 2..];
            rest.find(')').map(|end| &rest[..end])
        })
        .collect()
}

#[test]
fn stored_rules_idiom_validates_and_injects_verdict() {
    let suite = synthesized(&fixture("model.yaml"));
    let ids = ids(&suite);
    for id in [
        "demo.rules.TestRule/outcome/not-met",
        "demo.rules.TestRule/outcome/met",
    ] {
        assert!(ids.iter().any(|it| it == id), "{id} is written: {ids:#?}");
    }
    let steps = suite["scenarios"]["demo.rules.TestRule/outcome/not-met"]["steps"]
        .as_array()
        .expect("the not-met scenario has steps");
    assert_eq!(
        steps.first().and_then(|step| step["step"].as_str()),
        Some("configure_external_outcome"),
        "the evaluator's verdict is injected before the command: {steps:#?}"
    );
}

#[test]
fn stored_rules_fold_as_row_set_guard() {
    let model = read(&fixture("fold.yaml"));
    assert!(
        model.starts_with("format: ess/22\n"),
        "the fold model is ess/22"
    );
    for shape in ["forall: holds == true", "exists: true", "when_related:"] {
        assert!(model.contains(shape), "the fold model uses `{shape}`");
    }
    let ids = ids(&synthesized(&fixture("fold.yaml")));
    for id in [
        "demo.rules.TestAll/outcome/met",
        "demo.rules.TestAll/outcome/not-met",
        "demo.rules.TestAny/outcome/met",
        "demo.rules.TestAny/outcome/not-met",
    ] {
        assert!(ids.iter().any(|it| it == id), "{id} is written: {ids:#?}");
    }
}

#[test]
fn stored_rules_note_states_the_boundary() {
    let note = read(&root().join(NOTE));
    let sections: [(&str, &[&str]); 4] = [
        (
            "## A stored rule's parts are typed rows",
            &["typed rows", "enum"],
        ),
        (
            "## Whether a stored rule holds is the evaluator's",
            &["`external:`", "injected"],
        ),
        (
            "## A fold over stored results is a row-set guard",
            &["`when_related`", "`forall`", "`exists`"],
        ),
        (
            "## Patterns held as data",
            &[
                "regular expression",
                "stored",
                "#449",
                "no pattern predicate",
            ],
        ),
    ];
    let mut last = 0;
    for (heading, phrases) in sections {
        let text =
            section(&note, heading).unwrap_or_else(|| panic!("{NOTE}: no heading `{heading}`"));
        let at = note.find(text).expect("the section is in the note");
        assert!(
            at > last,
            "{NOTE}: `{heading}` follows the sections before it"
        );
        last = at;
        for phrase in phrases {
            assert!(
                text.contains(phrase),
                "{NOTE}: `{heading}` says {phrase:?}:\n{text}"
            );
        }
    }
}

#[test]
fn stored_rules_guide_section_states_the_boundary() {
    let guide = read(&root().join(GUIDE));
    let text = section(&guide, GUIDE_HEADING)
        .unwrap_or_else(|| panic!("{GUIDE}: no heading `{GUIDE_HEADING}`"));
    for phrase in ["`external:`", "`when_related`"] {
        assert!(
            text.contains(phrase),
            "{GUIDE}: the section says {phrase:?}:\n{text}"
        );
    }
    assert!(
        link_targets(text)
            .iter()
            .any(|target| target.ends_with(NOTE)),
        "{GUIDE}: the section links {NOTE}:\n{text}"
    );
    let model = read(&fixture("model.yaml"));
    assert!(
        yaml_blocks(text).contains(&model),
        "{GUIDE}: the section shows `tests/fixtures/stored-rules/model.yaml`, byte for byte:\n{text}"
    );
}
