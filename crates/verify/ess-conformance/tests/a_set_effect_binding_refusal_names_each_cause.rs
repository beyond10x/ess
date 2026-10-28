//! A binding into a set effect that publishes an event once per row it changes (`each: changed`),
//! whose filter key names a row of another entity (a downstream adopter's report of 2026-10-26).
//!
//! The flow, delivery and failure scenarios need one row the filter selects arranged before the
//! trigger. Four shapes each stopped that arrangement, and the refusal named none of them; its help
//! asked for two conditions that already held:
//!
//! | shape | what synthesis now arranges |
//! |---|---|
//! | the trigger addresses the row the key names (`updates:`/`instance:`) | the selected row created naming the row the trigger acts on |
//! | the creating command refuses a missing row behind the key (`exists: false`) | that row first, the selected row naming it, the trigger sent naming it |
//! | the creating command copies a field of that row (`{related:}`) | the same |
//! | the trigger publishes a literal for the value the effect writes | the bound input as that literal |
//!
//! Where the arrangement still cannot be made, the refusal names what stops it, and its help asks
//! only for what does not hold yet.

use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::synthesize::{synthesize, Synthesis};
use ess_conformance::{AdmittedSuite, Runner};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use serde_json::Value;

const SYSTEM: &str = include_str!("fixtures/set-effect-binding-causes/system.yaml");
const COMPONENTS: &str = include_str!("fixtures/set-effect-binding-causes/components.yaml");
const DOMAIN: &str = include_str!("fixtures/set-effect-binding-causes/c.yaml");

const BINDING: &str = "sync-on-flag-set";
const TRIGGER: &str = "demo.c.SetFlag";
const CREATOR: &str = "demo.c.OpenTicket";
const ASPECTS: [&str; 3] = ["flow", "delivery", "on-failure"];

/// One shape, as an edit of the fixture: each part it holds once, replaced.
#[derive(Clone, Copy, Debug)]
enum Cause {
    /// The trigger addresses the row the filter key names.
    Addresses,
    /// The creating command refuses a missing row behind the key.
    Guards,
    /// The creating command copies a field of that row.
    Copies,
    /// The trigger publishes a literal for the value the effect writes.
    Literal,
}

const CAUSES: [Cause; 4] = [
    Cause::Addresses,
    Cause::Guards,
    Cause::Copies,
    Cause::Literal,
];

impl Cause {
    fn edits(self) -> &'static [(&'static str, &'static str)] {
        match self {
            Self::Addresses => &[
                (
                    "      - name: set\n        emits: [demo.c.FlagSet]\n",
                    concat!(
                        "      - name: set\n",
                        "        updates: demo.c.Grant\n",
                        "        instance: grant_id\n",
                        "        sets: {flag: input.flag}\n",
                        "        emits: [demo.c.FlagSet]\n",
                    ),
                ),
                (
                    "  - name: demo.c.OpenTicket\n",
                    concat!(
                        "      - {name: not-found, unknown_instance: true, error: demo.c.Missing}\n",
                        "  - name: demo.c.OpenTicket\n",
                    ),
                ),
            ],
            Self::Guards => &[(
                "      - name: opened\n",
                concat!(
                    "      - {name: no-grant, when_related: {via: input.grant_id, exists: false}, ",
                    "error: demo.c.Missing}\n",
                    "      - name: opened\n",
                ),
            )],
            Self::Copies => &[(
                "sets: {grant_id: input.grant_id, flag: false}",
                "sets: {grant_id: input.grant_id, flag: {related: {via: input.grant_id, field: flag}}}",
            )],
            Self::Literal => &[(
                "{demo.c.FlagSet: {grant_id: input.grant_id, flag: input.flag}}",
                "{demo.c.FlagSet: {grant_id: input.grant_id, flag: true}}",
            )],
        }
    }
}

/// `text` with `part`, which it holds exactly once, replaced by `with`.
fn replaced(text: &str, part: &str, with: &str) -> String {
    assert_eq!(text.matches(part).count(), 1, "written once: {part}");
    text.replacen(part, with, 1)
}

/// The fixture with every cause in `causes` written in.
fn with(causes: &[Cause]) -> String {
    causes
        .iter()
        .flat_map(|cause| cause.edits())
        .fold(DOMAIN.to_owned(), |text, (part, edit)| {
            replaced(&text, part, edit)
        })
}

fn model(domain: &str) -> EssIr {
    let files = [
        ("system.yaml", SYSTEM),
        ("components.yaml", COMPONENTS),
        ("c.yaml", domain),
    ];
    let mut sources = Vec::new();
    for (label, text) in files {
        let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{label}: {error}"));
        sources.push((Source::new(label), raw));
    }
    let spec = Specification::assemble(sources)
        .unwrap_or_else(|errors| panic!("the fixture validates:\n{errors}\n{domain}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}\n{domain}"))
}

fn refusals(synthesis: &Synthesis) -> Vec<String> {
    synthesis.refusals.iter().map(ToString::to_string).collect()
}

fn refused_as(synthesis: &Synthesis, id: &str) -> Vec<String> {
    refusals(synthesis)
        .into_iter()
        .filter(|refusal| refusal.contains(&format!("`{id}`")))
        .collect()
}

fn steps_of(synthesis: &Synthesis, id: &str) -> Vec<Value> {
    let document: Value =
        serde_json::from_str(&synthesis.suite.to_canonical_json().expect("serialises"))
            .expect("JSON");
    document["scenarios"][id]["steps"]
        .as_array()
        .unwrap_or_else(|| panic!("`{id}` is synthesized: {:#?}", refusals(synthesis)))
        .clone()
}

/// Every scenario the interpreter of the same model does not pass, with its diagnostics.
fn failing(model: &EssIr, synthesis: &Synthesis) -> BTreeMap<String, String> {
    let admitted =
        AdmittedSuite::from_suite(&synthesis.suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(model.clone()))
        .into_report()
        .scenarios
        .into_iter()
        .filter(|result| result.status != Status::Passed)
        .map(|result| {
            let diagnostics: Vec<String> = result.diagnostics().map(|d| format!("{d:?}")).collect();
            (
                result.scenario.to_string(),
                format!("{:?}: {diagnostics:#?}", result.status),
            )
        })
        .collect()
}

/// The `help:` line of `refusal`.
fn help(refusal: &str) -> &str {
    refusal
        .lines()
        .find_map(|line| line.trim().strip_prefix("help: "))
        .unwrap_or_else(|| panic!("a help line: {refusal}"))
}

/// The binding's flow, delivery and failure scenarios exist, each creating a ticket before the
/// trigger under the grant the trigger is sent, and every scenario passes against the interpreter.
fn arranged(causes: &[Cause]) {
    let text = with(causes);
    let ir = model(&text);
    let synthesis = synthesize(&ir);
    assert_eq!(refusals(&synthesis), Vec::<String>::new(), "{causes:?}");
    for aspect in ASPECTS {
        let id = format!("{BINDING}/binding/{aspect}");
        let steps = steps_of(&synthesis, &id);
        let sent = |command: &str| -> Vec<&Value> {
            steps
                .iter()
                .filter(|step| step["step"] == "execute_command" && step["command"] == command)
                .collect()
        };
        let trigger = *sent(TRIGGER).last().expect("the trigger is sent");
        let at = |wanted: &Value| {
            steps
                .iter()
                .position(|step| std::ptr::eq(step, wanted))
                .expect("a step of the scenario")
        };
        let grant = &trigger["input"]["grant_id"];
        assert!(
            sent(CREATOR)
                .iter()
                .any(|open| open["input"]["grant_id"] == *grant && at(open) < at(trigger)),
            "{causes:?} {id}: a ticket under {grant} is opened before the trigger: {steps:#?}"
        );
    }
    assert_eq!(failing(&ir, &synthesis), BTreeMap::new(), "{causes:?}");
}

/// The refusals of the binding's flow, delivery and failure scenarios in the fixture with `causes`
/// and `extra` written in: one each, all alike.
fn refused(causes: &[Cause], extra: (&str, &str)) -> String {
    let text = replaced(&with(causes), extra.0, extra.1);
    let synthesis = synthesize(&model(&text));
    let mut found = Vec::new();
    for aspect in ASPECTS {
        let refused = refused_as(&synthesis, &format!("{BINDING}/binding/{aspect}"));
        assert_eq!(refused.len(), 1, "{causes:?} {aspect}: {refused:#?}");
        found.push(refused[0].replace(&format!("binding/{aspect}"), "binding/<aspect>"));
    }
    assert!(
        found.windows(2).all(|pair| pair[0] == pair[1]),
        "{found:#?}"
    );
    found.swap_remove(0)
}

/// The help asks for neither condition the old help named where it already holds: the trigger
/// copies the key from an input of its own, and the creating command writes it from an input.
fn asks_for_nothing_that_holds(help: &str) {
    assert!(
        !help.contains("copy every input the filter reads from an input of its own"),
        "the trigger already copies the key from its own input: {help}"
    );
    assert!(
        !help.contains("write each field the filter reads from an input"),
        "the creating command already writes the key from its input: {help}"
    );
}

#[test]
fn a_set_effect_binding_refusal_names_each_cause() {
    // Without any of the four shapes the row was arranged already, and still is.
    arranged(&[]);
    // Each shape on its own is arranged, and so are all four together.
    for cause in CAUSES {
        arranged(&[cause]);
    }
    arranged(&CAUSES);

    // The trigger addresses the grant, and the creating command refuses a grant missing or holding
    // the flag unset, as every grant does before the trigger: no ticket can be created naming it.
    let named = refused(
        &[Cause::Addresses, Cause::Guards],
        (
            "      - name: opened\n",
            concat!(
                "      - {name: unset, when_related: {via: input.grant_id, predicate: flag == false}, ",
                "error: demo.c.Missing}\n",
                "      - name: opened\n",
            ),
        ),
    );
    assert!(
        named.contains("the trigger acts on the `demo.c.Grant` row the key names")
            && named.contains("no command creating `demo.c.Ticket` can be sent naming it"),
        "{named}"
    );
    asks_for_nothing_that_holds(help(&named));

    // The creating command refuses a missing grant, and the trigger refuses a grant that exists:
    // sent naming a grant arranged for the ticket, it would not publish.
    let guarded = refused(
        &[Cause::Guards],
        (
            "      - name: set\n",
            concat!(
                "      - {name: known, when_related: {entity: demo.c.Grant, ",
                "where: grant_id == input.grant_id, exists: true}, error: demo.c.Missing}\n",
                "      - name: set\n",
            ),
        ),
    );
    assert!(
        guarded.contains("`demo.c.OpenTicket` reads the `demo.c.Grant` row `input.grant_id` names")
            && guarded.contains("a guard of `demo.c.SetFlag` reads its input `grant_id`"),
        "{guarded}"
    );
    asks_for_nothing_that_holds(help(&guarded));

    // The creating command refuses a missing grant and one holding the flag unset, and nothing
    // sets a grant's flag: no grant arranged first is one a ticket can be created under.
    let related = refused(
        &[Cause::Guards],
        (
            "      - name: opened\n",
            concat!(
                "      - {name: unset, when_related: {via: input.grant_id, predicate: flag == false}, ",
                "error: demo.c.Missing}\n",
                "      - name: opened\n",
            ),
        ),
    );
    assert!(
        related.contains("`demo.c.OpenTicket` reads the `demo.c.Grant` row `input.grant_id` names")
            && related.contains(
                "no `demo.c.Ticket` can be created naming a `demo.c.Grant` row arranged before the \
                 trigger"
            ),
        "{related}"
    );
    asks_for_nothing_that_holds(help(&related));

    // The trigger mints the key: the event fills it from no value a scenario chooses.
    let minted = refused(
        &[],
        (
            "{demo.c.FlagSet: {grant_id: input.grant_id, flag: input.flag}}",
            "{demo.c.FlagSet: {grant_id: {generated: true}, flag: input.flag}}",
        ),
    );
    assert!(
        minted.contains("the event fills `grant_id` from a value no scenario chooses"),
        "{minted}"
    );
    assert!(
        !help(&minted).contains("write each field the filter reads from an input"),
        "the creating command already writes the key from its input: {minted}"
    );
}
