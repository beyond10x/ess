//! Witnesses of a `String` newtype with `prefix:` start with it (beyond10x/ess#146).
//!
//! The base text of a leaf is its fact path (`channel`), which a type declaring `prefix: "/"`
//! refuses; the witness puts the prefix in front, so a generated success scenario sends a value a
//! correct implementation accepts. Under an alphabet the prefix is kept and the rest is mapped, and
//! a count guard resizes after the prefix, never through it.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::{synthesize::Synthesis, ScenarioStep, ScenarioValue};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

const MODEL: &str = "format: ess/15
system: demo
version: v1
domain: demo.msgs
types:
  - name: demo.msgs.Channel
    kind: newtype
    of: String
    prefix: \"/\"
errors:
  - name: demo.msgs.Refused
    fields: []
events:
  - name: demo.msgs.Posted
    fields:
      - {name: channel, type: demo.msgs.Channel}
      - {name: reply_to, type: demo.msgs.Channel}
actors:
  - {name: demo.msgs.Poster, may: [demo.msgs.Post]}
commands:
  - name: demo.msgs.Post
    input:
      - {name: channel, type: demo.msgs.Channel}
      - {name: reply_to, type: demo.msgs.Channel}
    outcomes:
      - name: posted
        emits: [demo.msgs.Posted]
        payload:
          demo.msgs.Posted: {channel: input.channel, reply_to: input.reply_to}
";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("msgs.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn synthesis(text: &str) -> Synthesis {
    ess_conformance::synthesize::synthesize(&ir(text))
}

/// Every text sent at `field` by any command in the suite.
fn sent(result: &Synthesis, field: &str) -> Vec<String> {
    result
        .suite
        .scenarios
        .values()
        .flat_map(|scenario| &scenario.steps)
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand { input, .. } => match input.get(field) {
                Some(ScenarioValue::Literal {
                    value: Node::Text(text),
                }) => Some(text.clone()),
                _ => None,
            },
            _ => None,
        })
        .collect()
}

#[test]
fn issue_146_every_witness_starts_with_the_prefix() {
    let result = synthesis(MODEL);
    assert!(result.refusals.is_empty(), "{:#?}", result.refusals);
    let channels = sent(&result, "channel");
    let replies = sent(&result, "reply_to");
    assert!(!channels.is_empty(), "a scenario sends a channel");
    for text in channels.iter().chain(&replies) {
        assert!(text.starts_with('/'), "{text:?} does not start with `/`");
    }
    // Rule 2 still holds under the prefix: two same-typed fields are never interchangeable.
    assert!(
        channels.iter().all(|channel| !replies.contains(channel)),
        "{channels:?} and {replies:?} must differ"
    );
}

#[test]
fn guard_alternatives_start_with_the_prefix_too() {
    let text = MODEL.replace(
        "      - name: posted\n",
        "      - name: refused\n        when: channel == \"/ops\"\n        error: demo.msgs.Refused\n      - name: posted\n",
    );
    let result = synthesis(&text);
    assert!(result.refusals.is_empty(), "{:#?}", result.refusals);
    let channels = sent(&result, "channel");
    assert!(
        channels.iter().any(|channel| channel == "/ops"),
        "{channels:?}"
    );
    assert!(
        channels.iter().any(|channel| channel != "/ops"),
        "{channels:?}"
    );
    for text in &channels {
        assert!(text.starts_with('/'), "{text:?} does not start with `/`");
    }
}

#[test]
fn under_an_alphabet_the_prefix_is_kept_and_the_rest_is_mapped() {
    let text = MODEL.replace(
        "    prefix: \"/\"\n",
        "    prefix: \"/\"\n    alphabet: \"/abc\"\n",
    );
    let result = synthesis(&text);
    assert!(result.refusals.is_empty(), "{:#?}", result.refusals);
    for text in sent(&result, "channel")
        .iter()
        .chain(&sent(&result, "reply_to"))
    {
        assert!(text.starts_with('/'), "{text:?}");
        assert!(
            text.chars().all(|character| "/abc".contains(character)),
            "{text:?} leaves the alphabet"
        );
    }
}

#[test]
fn a_count_guard_resizes_after_the_prefix() {
    let text = MODEL.replace(
        "      - name: posted\n",
        "      - name: refused\n        when: channel.count > 12\n        error: demo.msgs.Refused\n      - name: posted\n",
    );
    let result = synthesis(&text);
    assert!(result.refusals.is_empty(), "{:#?}", result.refusals);
    let channels = sent(&result, "channel");
    assert!(
        channels.iter().any(|channel| channel.chars().count() == 13),
        "{channels:?}"
    );
    for text in &channels {
        assert!(text.starts_with('/'), "{text:?}");
    }
}

#[test]
fn a_prefix_longer_than_a_count_bound_is_still_a_witness_only_where_it_fits() {
    // `channel.count > 0` with prefix `/`: the length-0 row cannot start with `/`, so the only
    // rows sent are ones that do.
    let text = MODEL.replace(
        "      - name: posted\n",
        "      - name: refused\n        when: channel.count < 1\n        error: demo.msgs.Refused\n      - name: posted\n",
    );
    let result = synthesis(&text);
    for text in sent(&result, "channel") {
        assert!(text.starts_with('/'), "{text:?}");
    }
}
