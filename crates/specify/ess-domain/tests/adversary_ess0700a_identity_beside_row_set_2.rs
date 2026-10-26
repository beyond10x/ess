//! Adversary pass 2, ESS 0.70.0 unit A (beyond10x/ess#554): the refusal of a `when_subject` guard
//! beside a row set and an identity-addressed `when_related:`, at every format that reads row sets.

use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::error::{ValidationCode, ValidationErrors};

const REPRO: &str = include_str!("fixtures/identity-beside-row-set.yaml");

fn assemble(text: &str) -> Result<Specification, ValidationErrors> {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    Specification::assemble([(Source::new("wiring.yaml"), raw)])
}

fn listed(errors: &ValidationErrors) -> String {
    errors
        .as_slice()
        .iter()
        .map(|error| {
            format!(
                "{:?} {} {} [{}]",
                error.code,
                error.location,
                error.message,
                error.hint.as_deref().unwrap_or_default()
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn edited(text: &str, before: &str, after: &str) -> String {
    let changed = text.replacen(before, after, 1);
    assert_ne!(changed, text, "`{before}` is in the model");
    changed
}

const CONNECTED: &str = "      - name: connected\n        creates: wiring.board.Wire\n";

/// Below `ess/32` too, the refusal names the `when_subject` guard, which no format admits beside a
/// row set, and its hint does not send the author to `ess/32`, where the command stays refused.
#[test]
fn adversary_a2_when_subject_is_named_first_at_every_row_set_format() {
    let subject = "      - name: odd
        when_subject:
          predicate: from_port == input.from_port
        error: wiring.board.SourceTaken
";
    let text = edited(REPRO, CONNECTED, &format!("{subject}{CONNECTED}"));
    let mut report = Vec::new();
    for format in ["ess/22", "ess/26", "ess/27", "ess/30", "ess/31", "ess/32"] {
        let at = text.replacen("format: ess/32\n", &format!("format: {format}\n"), 1);
        let Err(errors) = assemble(&at) else {
            report.push(format!("{format}: validates"));
            continue;
        };
        let refused: Vec<_> = errors
            .as_slice()
            .iter()
            .filter(|error| error.code == ValidationCode::UnsupportedConstruct)
            .filter(|error| error.message.contains("selects on a row set"))
            .collect();
        let named = refused.len() == 1
            && refused[0].message.contains("a `when_subject` guard")
            && !refused[0].message.contains("identity-addressed")
            && !refused[0]
                .hint
                .as_deref()
                .unwrap_or_default()
                .contains("ess/32");
        if !named {
            report.push(format!("{format}:\n{}", listed(&errors)));
        }
    }
    assert!(report.is_empty(), "{}", report.join("\n"));
}
