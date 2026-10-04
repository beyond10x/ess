//! Adversary pass 1 on beyond10x/ess#290: where a type's use is not one the classifier reads.
//!
//! The classifier answers a type change from the roles it finds the type in. A role it does not
//! recognise contributes nothing, and a type with no recognised role is answered `compatible`
//! everywhere — a guess the module's own rule (design §30: no model answer means `Unknown`) forbids.
//! Each red case below narrows a type whose only use is a value somebody outside the system
//! supplies — a deployed component setting, an external channel's delivery context, a periodic
//! host's context — and asserts the narrowing is not called compatible.
//!
//! The green cases probe wrappers and a type used both ways round, which the unit claims to handle.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_diff::compatibility::{ChangeCompatibility, Compatibility, TypeUse};
use ess_diff::EssDelta;
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

fn model(text: &str) -> EssIr {
    let specification =
        Specification::assemble([(Source::new("spec.yaml"), RawSpecFile::parse(text).unwrap())])
            .unwrap_or_else(|errors| panic!("{errors}"));
    let mut sources = SourceMap::new();
    sources.insert("spec.yaml", text);
    compile(&specification, &sources).unwrap_or_else(|errors| panic!("{errors}"))
}

/// The one change `id`, and its classification.
fn the(delta: &EssDelta, id: &str) -> ChangeCompatibility {
    let ids: Vec<String> = delta.changes().iter().map(|c| c.id().to_string()).collect();
    assert_eq!(
        ids,
        vec![id.to_owned()],
        "the delta holds exactly the one change"
    );
    delta
        .compatibility_of(&delta.changes()[0].id())
        .expect("classified")
        .clone()
}

// ---- a type held only by a component setting ------------------------------------------------

fn with_setting(variants: &str) -> String {
    format!(
        "format: ess/5
system: demo
version: v1
domain: demo.calls
types:
  - name: demo.calls.Channel
    kind: enum
    variants: [{variants}]
components:
  - component: calls-service
    owns:
      domains: [demo.calls]
    settings:
      - name: default-channel
        type: demo.calls.Channel
        required: true
"
    )
}

/// A deployment that configured `default-channel: Chat` against `before` is refused by `after`.
/// The classifier finds no use (component settings are not graph edges) and says `compatible`.
#[test]
fn a_variant_removed_from_a_type_only_a_component_setting_holds_is_not_compatible() {
    let delta = ess_diff::classified(
        &model(&with_setting("Voice, Video, Chat")),
        &model(&with_setting("Voice, Video")),
    )
    .unwrap();
    let compatibility = the(&delta, "type/demo.calls.Channel/variant-removed/Chat");
    assert_ne!(
        compatibility.verdict(),
        Compatibility::Compatible,
        "a setting value deployed against `before` is refused by `after`; uses = {:?}",
        compatibility.uses()
    );
}

// ---- a type held only by an external channel's delivery context -----------------------------

const INBOX: &str = include_str!("../../ess-conformance/tests/fixtures/delivery-context.yaml");

fn with_context_region(variants: &str) -> String {
    let typed = INBOX.replacen(
        "  - {name: demo.inbox.AccountId, kind: newtype, of: String}\n",
        &format!(
            "  - {{name: demo.inbox.AccountId, kind: newtype, of: String}}\n  - {{name: demo.inbox.Region, kind: enum, variants: [{variants}]}}\n"
        ),
        1,
    );
    let text = typed.replacen(
        "        - {name: account_id, type: demo.inbox.AccountId}\n",
        "        - {name: account_id, type: demo.inbox.AccountId}\n        - {name: region, type: demo.inbox.Region}\n",
        1,
    );
    assert_ne!(text, typed, "the fixture still holds the context field");
    text
}

/// The channel delivers `region: Apac` with every message; `after` no longer admits it.
#[test]
fn a_variant_removed_from_a_type_only_a_delivery_context_holds_is_not_compatible() {
    let delta = ess_diff::classified(
        &model(&with_context_region("Eu, Us, Apac")),
        &model(&with_context_region("Eu, Us")),
    )
    .unwrap();
    let compatibility = the(&delta, "type/demo.inbox.Region/variant-removed/Apac");
    assert_ne!(
        compatibility.verdict(),
        Compatibility::Compatible,
        "a context value the channel already sends is refused by `after`; uses = {:?}",
        compatibility.uses()
    );
}

// ---- a type held only by a periodic host's context -------------------------------------------

const PERIODIC: &str = include_str!("../../../specify/ess-domain/tests/fixtures/periodic.yaml");

fn with_host_region(variants: &str) -> String {
    let typed = PERIODIC.replacen(
        "domain: example.poll\n",
        &format!(
            "domain: example.poll\ntypes:\n  - name: example.poll.Region\n    kind: enum\n    variants: [{variants}]\n"
        ),
        1,
    );
    let text = typed.replacen(
        "          context_fields:\n            - name: agent_id\n              type: String\n",
        "          context_fields:\n            - name: agent_id\n              type: String\n            - name: region\n              type: example.poll.Region\n",
        1,
    );
    assert_ne!(text, typed, "the fixture still holds the host context");
    text
}

/// The host supplies `region` for the binding's lifetime: a binding edge (`maps a value of type`),
/// which the classifier's match drops.
#[test]
fn a_variant_removed_from_a_type_only_a_periodic_host_supplies_is_not_compatible() {
    let delta = ess_diff::classified(
        &model(&with_host_region("Eu, Us, Apac")),
        &model(&with_host_region("Eu, Us")),
    )
    .unwrap();
    let compatibility = the(&delta, "type/example.poll.Region/variant-removed/Apac");
    assert_ne!(
        compatibility.verdict(),
        Compatibility::Compatible,
        "a host value already supplied is refused by `after`; uses = {:?}",
        compatibility.uses()
    );
}

// ---- wrappers, and one type used both ways ---------------------------------------------------

fn command_with(variants: &str, input: &str, response: &str) -> String {
    format!(
        "format: ess/5
system: demo
version: v1
domain: demo.calls
types:
  - name: demo.calls.Channel
    kind: enum
    variants: [{variants}]
events:
  - name: demo.calls.Pinged
    fields:
      - {{name: note, type: String}}
commands:
  - name: demo.calls.Ping
    input:
      - {{name: note, type: String}}
{input}{response}    outcomes:
      - name: pinged
        emits: [demo.calls.Pinged]
        payload:
          demo.calls.Pinged: {{note: input.note}}
"
    )
}

#[test]
fn a_type_wrapped_in_optional_list_or_map_on_a_command_input_is_a_caller_input() {
    for wrapped in [
        "Optional<demo.calls.Channel>",
        "List<demo.calls.Channel>",
        "'Map<String, demo.calls.Channel>'",
    ] {
        let input = format!("      - {{name: channel, type: {wrapped}}}\n");
        let delta = ess_diff::classified(
            &model(&command_with("Voice, Video, Chat", &input, "")),
            &model(&command_with("Voice, Video", &input, "")),
        )
        .unwrap();
        let compatibility = the(&delta, "type/demo.calls.Channel/variant-removed/Chat");
        assert_eq!(
            compatibility.uses(),
            Some(&[TypeUse::Input].into_iter().collect()),
            "{wrapped}"
        );
        assert_eq!(
            compatibility.callers(),
            Compatibility::Breaking,
            "{wrapped}"
        );
    }
}

#[test]
fn a_type_both_sent_and_returned_breaks_callers_one_way_and_readers_the_other() {
    let input = "      - {name: channel, type: demo.calls.Channel}\n";
    let response = "    response:\n      - {name: channel, type: demo.calls.Channel}\n";
    let three = command_with("Voice, Video, Chat", input, response);
    let two = command_with("Voice, Video", input, response);

    let narrowed = ess_diff::classified(&model(&three), &model(&two)).unwrap();
    let narrowed = the(&narrowed, "type/demo.calls.Channel/variant-removed/Chat");
    assert_eq!(
        narrowed.uses(),
        Some(&[TypeUse::Input, TypeUse::Output].into_iter().collect())
    );
    assert_eq!(narrowed.callers(), Compatibility::Breaking);
    assert_eq!(narrowed.readers(), Compatibility::Compatible);
    assert_eq!(narrowed.history(), Compatibility::Compatible);

    let widened = ess_diff::classified(&model(&two), &model(&three)).unwrap();
    let widened = the(&widened, "type/demo.calls.Channel/variant-added/Chat");
    assert_eq!(widened.callers(), Compatibility::Compatible);
    assert_eq!(widened.readers(), Compatibility::Breaking);
    assert_eq!(widened.history(), Compatibility::Compatible);
}

/// The same narrowing written as a type swap — the input moves from `Channel` to a smaller
/// `Narrow` — is not a variant removal, so it must at least not be called compatible.
#[test]
fn a_narrowing_written_as_a_type_swap_on_a_command_input_is_not_compatible() {
    let extra = "  - name: demo.calls.Narrow\n    kind: enum\n    variants: [Voice, Video]\n";
    let before = command_with(
        "Voice, Video, Chat",
        "      - {name: channel, type: demo.calls.Channel}\n",
        "",
    )
    .replacen("events:\n", &format!("{extra}events:\n"), 1);
    let after = before.replacen(
        "      - {name: channel, type: demo.calls.Channel}\n",
        "      - {name: channel, type: demo.calls.Narrow}\n",
        1,
    );
    assert_ne!(before, after);
    let delta = ess_diff::classified(&model(&before), &model(&after)).unwrap();
    let verdicts: Vec<(String, Compatibility)> = delta
        .changes()
        .iter()
        .map(|change| {
            (
                change.id().to_string(),
                delta.compatibility_of(&change.id()).unwrap().verdict(),
            )
        })
        .collect();
    assert_ne!(
        verdicts.len(),
        0,
        "the delta classifies at least one change"
    );
    assert!(
        verdicts
            .iter()
            .any(|(_, verdict)| *verdict != Compatibility::Compatible),
        "{verdicts:#?}"
    );
}
