//! Adversary pass 2 for story:subject-guard-input-and-case-folding (beyond10x/ess#140): what the
//! suite actually sends for a fold guard, not what the candidate search tries. A candidate that no
//! scenario carries distinguishes no target.

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::{synthesize, ScenarioStep};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::node::Node;

fn compiled(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|e| panic!("parses: {e}\n{text}"));
    let specification = Specification::assemble([(Source::new("adv2.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("validates:\n{errors}\n{text}"));
    compile(&specification, &SourceMap::new()).unwrap_or_else(|d| panic!("resolves:\n{d}"))
}

fn lookup(guard: &str) -> String {
    format!(
        r"format: ess/15
system: demo
version: v1
domain: demo.orders
errors:
  - name: demo.orders.UnknownSource
    fields: []
events:
  - name: demo.orders.Looked
    fields:
      - {{name: channel, type: String}}
commands:
  - name: demo.orders.Lookup
    input:
      - {{name: channel, type: String}}
    outcomes:
      - name: matched
        when: {guard}
        error: demo.orders.UnknownSource
      - name: found
        emits: [demo.orders.Looked]
        payload:
          demo.orders.Looked:
            channel: input.channel
"
    )
}

/// Every `channel` any scenario sends to `demo.orders.Lookup`, with the scenario id.
fn sent(text: &str) -> (Vec<(String, String)>, Vec<String>) {
    let synthesis = synthesize(&compiled(text));
    let mut found = Vec::new();
    for (id, scenario) in &synthesis.suite.scenarios {
        for step in &scenario.steps {
            if let ScenarioStep::ExecuteCommand { command, input, .. } = step {
                if command.to_string() == "demo.orders.Lookup" {
                    if let Some(Node::Text(value)) =
                        input.get("channel").and_then(|value| value.as_literal())
                    {
                        found.push((id.to_string(), value.clone()));
                    }
                }
            }
        }
    }
    let refusals = synthesis.refusals.iter().map(ToString::to_string).collect();
    (found, refusals)
}

fn unicode_folds(left: &str, right: &str) -> bool {
    left.to_lowercase() == right.to_lowercase() || left.to_uppercase() == right.to_uppercase()
}

/// Round 1 added `unicode_refuting` so that a Unicode-folding target (`strings.EqualFold`,
/// `toLowerCase`) fails the suite. It only does if some scenario sends that text: the default branch
/// has one witness, and it is the first refuting candidate — the one-character change.
#[test]
fn adv2_some_scenario_sends_a_text_only_unicode_folding_equates_with_the_literal() {
    for literal in ["café", "Kelp", "sku", "Σοφία"] {
        let (sent, refusals) = sent(&lookup(&format!(
            "{{channel: {{equals_ignore_case: \"{literal}\"}}}}"
        )));
        assert!(
            sent.iter().any(
                |(_, text)| !text.eq_ignore_ascii_case(literal) && unicode_folds(text, literal)
            ),
            "{literal}: no scenario sends a text that ASCII folding refutes and Unicode folding \
             accepts, so a Unicode-folding target passes this suite; sent {sent:?}; refusals \
             {refusals:?}"
        );
    }
}

/// `fold_refuting` turns each member into another member here (web→xeb→yeb→xeb), so the
/// one-character change refutes nothing; the default branch must still be witnessed.
#[test]
fn adv2_the_default_is_witnessed_when_every_one_character_change_is_another_member() {
    let (sent, refusals) = sent(&lookup("{channel: {in_ignore_case: [web, xeb, yeb]}}"));
    let members = ["web", "xeb", "yeb"];
    assert!(
        sent.iter().any(|(id, text)| id.ends_with("/outcome/found")
            && !members.iter().any(|m| text.eq_ignore_ascii_case(m))),
        "sent {sent:?}; refusals {refusals:?}"
    );
    assert!(
        sent.iter()
            .any(|(id, text)| id.ends_with("/outcome/matched")
                && members.iter().any(|m| text.eq_ignore_ascii_case(m))
                && !members.contains(&text.as_str())),
        "the matched branch is witnessed on a case-changed member: sent {sent:?}"
    );
}

/// A literal with no ASCII letter is its own case change: both branches are still witnessed.
#[test]
fn adv2_a_literal_without_letters_is_witnessed_both_ways() {
    for literal in ["123", "-", "😀"] {
        let (sent, refusals) = sent(&lookup(&format!(
            "{{channel: {{equals_ignore_case: \"{literal}\"}}}}"
        )));
        assert!(
            sent.iter()
                .any(|(id, text)| id.ends_with("/outcome/matched") && text == literal),
            "{literal}: sent {sent:?}; refusals {refusals:?}"
        );
        assert!(
            sent.iter()
                .any(|(id, text)| id.ends_with("/outcome/found") && text != literal),
            "{literal}: sent {sent:?}; refusals {refusals:?}"
        );
    }
}

/// E7's acceptance: synthesis witnesses both sides — a case-changed literal matches, and a
/// one-character change does not. The refuting side must be that one-character change, or a target
/// that compares only lengths (or only the first byte folded) passes the default scenario.
#[test]
fn adv2_the_default_is_witnessed_on_a_one_character_change_of_the_literal() {
    let (sent, refusals) = sent(&lookup("{channel: {equals_ignore_case: web}}"));
    let one_changed = |text: &str| {
        text.chars().count() == 3
            && text
                .chars()
                .zip("web".chars())
                .filter(|(a, b)| !a.eq_ignore_ascii_case(b))
                .count()
                == 1
    };
    assert!(
        sent.iter()
            .any(|(id, text)| id.ends_with("/outcome/found") && one_changed(text)),
        "the default branch is not witnessed on a one-character change of `web`: sent {sent:?}; \
         refusals {refusals:?}"
    );
}
