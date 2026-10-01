//! The one classification of a command's answer, held to recorded answers.
//!
//! `tests/vectors/answers.json` is data, not code: every case is a status and the body bytes a
//! served surface answered with, the [`Answer`] it must classify as, and where it came from — one
//! of the gatepass servers and the request that drew it, or `constructed` with the code that fixes
//! its shape. The TypeScript port of [`classify`] runs the same file, so the two cannot disagree
//! about a case without one of them failing it.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use ess_ui::binding::{classify, Answer};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct AnswerFile {
    format: String,
    /// How each recording server was started, by the name a vector's `source` gives it.
    servers: BTreeMap<String, String>,
    vectors: Vec<Vector>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Vector {
    /// What the case is.
    case: String,
    /// The HTTP status.
    status: u16,
    /// The body, byte for byte as answered.
    body: String,
    /// How it classifies.
    expected: Answer,
    /// `gatepass-rust`, `gatepass-go`, or `constructed`.
    source: String,
    /// The request that drew the answer (recorded) or the one it stands for (constructed).
    command: String,
    /// For a constructed case: the code whose output shape it copies, as `file:line-line`, or
    /// why no served answer has its shape.
    #[serde(default)]
    shape: Option<String>,
}

const RECORDED: [&str; 2] = ["gatepass-rust", "gatepass-go"];
const CONSTRUCTED: &str = "constructed";

fn vectors() -> AnswerFile {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/vectors/answers.json");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    let vectors: AnswerFile =
        serde_json::from_str(&text).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    assert_eq!(vectors.format, "ess-ui-answers/1");
    vectors
}

/// `(status, answer kind)` with, for an unfinished answer, whether it was committed.
fn kind(status: u16, answer: &Answer) -> String {
    let kind = match answer {
        Answer::Accepted => "accepted".to_owned(),
        Answer::Refused { .. } => "refused".to_owned(),
        Answer::NotGranted { .. } => "not_granted".to_owned(),
        Answer::Malformed { .. } => "malformed".to_owned(),
        Answer::Unfinished { committed } => format!("unfinished committed={committed}"),
        Answer::Transport => "transport".to_owned(),
    };
    format!("{status} {kind}")
}

#[test]
fn every_answer_vector_classifies_as_recorded() {
    let vectors = vectors();
    let mut wrong = Vec::new();
    for vector in &vectors.vectors {
        let answer = classify(vector.status, &vector.body);
        if answer != vector.expected {
            wrong.push(format!(
                "{} ({}): {} {:?} classified as {answer:?}, recorded {:?}",
                vector.case, vector.source, vector.status, vector.body, vector.expected
            ));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));

    // The file covers every answer the binding contract names; a case removed is a case the
    // TypeScript port stops being held to.
    let covered: BTreeSet<String> = vectors
        .vectors
        .iter()
        .map(|vector| kind(vector.status, &vector.expected))
        .collect();
    let mut missing = Vec::new();
    for required in [
        "202 accepted",
        "422 refused",
        "409 refused",
        "404 malformed",
        "403 not_granted",
        "403 refused",
        "400 refused",
        "400 malformed",
        "501 unfinished committed=true",
        "501 unfinished committed=false",
        "405 malformed",
    ] {
        if !covered.contains(required) {
            missing.push(required);
        }
    }
    assert!(
        missing.is_empty(),
        "no vector for {missing:?}; covered: {covered:?}"
    );
}

#[test]
fn every_vector_names_its_source_server_and_command() {
    let vectors = vectors();
    let mut problems = Vec::new();
    for vector in &vectors.vectors {
        let recorded = RECORDED.contains(&vector.source.as_str());
        if !recorded && vector.source != CONSTRUCTED {
            problems.push(format!(
                "{}: source `{}` is none of {RECORDED:?} or `{CONSTRUCTED}`",
                vector.case, vector.source
            ));
        }
        if recorded
            && vectors
                .servers
                .get(&vector.source)
                .is_none_or(|how| how.trim().is_empty())
        {
            problems.push(format!(
                "{}: `servers` does not say how `{}` was started",
                vector.case, vector.source
            ));
        }
        if vector.command.trim().is_empty() {
            problems.push(format!("{}: names no command", vector.case));
        }
        match (&vector.shape, recorded) {
            (Some(shape), false) if !shape.trim().is_empty() => {}
            (None, true) => {}
            (shape, _) => problems.push(format!(
                "{}: a recorded case names no shape and a constructed one names the code it \
                 copies; this one has {shape:?}",
                vector.case
            )),
        }
    }
    // A recorded case is recorded from both servers: they serve one contract.
    let recorded: BTreeSet<(&str, &str)> = vectors
        .vectors
        .iter()
        .filter(|vector| RECORDED.contains(&vector.source.as_str()))
        .map(|vector| (vector.case.as_str(), vector.source.as_str()))
        .collect();
    for (case, _) in &recorded {
        for server in RECORDED {
            if !recorded.contains(&(case, server)) {
                problems.push(format!("{case}: not recorded from {server}"));
            }
        }
    }
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}
