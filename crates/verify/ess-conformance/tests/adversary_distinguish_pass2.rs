//! Adversary pass 2 on beyond10x/ess#202, after the joined-source correction.
//!
//! The oracle is pass 1's: after a scenario sends `command` with literal inputs, every row it then
//! requires in the view is compared with what the *original* specification writes,
//! `target := input[original source]`. A required row that differs is one every implementation of
//! the original fails, so the mutant is killed.

use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::source::SourceMap;
use ess_conformance::mutate::{self, Document, MutantClass};
use ess_conformance::scenario::{ScenarioStep, ViewExpectation};
use ess_conformance::synthesize::{synthesize, Note, Synthesis};
use ess_domain::spec::RawSpecFile;
use ess_domain::system::Source;
use ess_primitives::node::Node;

fn documents(text: &str) -> (Vec<Document>, SourceMap) {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("well formed: {error}"));
    let mut texts = SourceMap::new();
    texts.insert("fixture.yaml".to_owned(), text.to_owned());
    (vec![(Source::new("fixture.yaml"), raw)], texts)
}

fn compiled((files, texts): &(Vec<Document>, SourceMap)) -> EssIr {
    mutate::compile(files.clone(), texts).unwrap_or_else(|stillborn| {
        panic!("the model compiles: {} {}", stillborn.code, stillborn.cause)
    })
}

fn retargets(model: &(Vec<Document>, SourceMap)) -> Vec<(String, EssIr)> {
    let (files, texts) = model;
    mutate::mutants(files, &[MutantClass::SetsRetarget])
        .into_iter()
        .map(|mutant| {
            let mutated = mutate::apply(files, &mutant.mutation).expect("the site exists");
            let ir = mutate::compile(mutated, texts).unwrap_or_else(|stillborn| {
                panic!(
                    "{} compiles: {} {}",
                    mutant.id, stillborn.code, stillborn.cause
                )
            });
            (mutant.id, ir)
        })
        .collect()
}

type Row = BTreeMap<String, Node>;

fn literals<'a>(
    values: impl Iterator<Item = (&'a String, &'a ess_conformance::scenario::ScenarioValue)>,
) -> Row {
    values
        .filter_map(|(field, value)| {
            value
                .as_literal()
                .cloned()
                .map(|node| (field.clone(), node))
        })
        .collect()
}

/// Every (literal input sent to `command`, row then required in `view`) pair in the suite.
fn sent_and_required(synthesis: &Synthesis, command: &str, view: &str) -> Vec<(Row, Row)> {
    let mut out = Vec::new();
    for scenario in synthesis.suite.scenarios.values() {
        let mut pending: Option<Row> = None;
        for step in &scenario.steps {
            match step {
                ScenarioStep::ExecuteCommand {
                    command: sent,
                    input,
                    ..
                } => {
                    pending = (sent.to_string() == command).then(|| literals(input.iter()));
                }
                ScenarioStep::ExpectView {
                    view: seen,
                    expectation: ViewExpectation::Contains { fields },
                }
                | ScenarioStep::EventuallyView {
                    view: seen,
                    expectation: ViewExpectation::Contains { fields },
                    ..
                } if seen.to_string() == view => {
                    if let Some(input) = &pending {
                        out.push((input.clone(), literals(fields.iter())));
                    }
                }
                _ => {}
            }
        }
    }
    out
}

/// The mutants (whose id contains `only`) whose suite holds no row the original contradicts.
fn survivors(
    text: &str,
    command: &str,
    view: &str,
    original: &[(&str, &str)],
    only: &str,
) -> Vec<String> {
    let model = documents(text);
    compiled(&model);
    let mutants: Vec<_> = retargets(&model)
        .into_iter()
        .filter(|(id, _)| id.contains(only))
        .collect();
    assert!(
        !mutants.is_empty(),
        "the audit finds retarget sites matching {only:?}"
    );
    let mut out = Vec::new();
    for (id, mutated) in mutants {
        let pairs = sent_and_required(&synthesize(&mutated), command, view);
        assert!(
            !pairs.is_empty(),
            "`{id}`: the suite sends `{command}` and then requires a `{view}` row"
        );
        let killed = pairs.iter().any(|(input, row)| {
            original.iter().any(
                |(target, source)| match (row.get(*target), input.get(*source)) {
                    (Some(held), Some(sent)) => held != sent,
                    _ => false,
                },
            )
        });
        if !killed {
            let shown: Vec<String> = pairs
                .iter()
                .map(|(input, row)| {
                    original
                        .iter()
                        .map(|(target, source)| {
                            format!(
                                "{target}<-{source}:{:?}/{:?}",
                                input.get(*source),
                                row.get(*target)
                            )
                        })
                        .collect::<Vec<_>>()
                        .join(" ")
                })
                .collect();
            out.push(format!(
                "{id}: {} rows [{}]",
                pairs.len(),
                shown.join(" | ")
            ));
        }
    }
    out
}

fn unseparated(synthesis: &Synthesis) -> Vec<(String, String, String)> {
    synthesis
        .notes
        .iter()
        .filter_map(|note| match note {
            Note::UnseparatedSources {
                scenario,
                source,
                unread,
            } => Some((scenario.to_string(), source.clone(), unread.clone())),
            _ => None,
        })
        .collect()
}

const FLAG_MAP: [(&str, &str); 3] = [("paid", "paid"), ("gift", "gift"), ("rush", "rush")];

/// Three Booleans written to three fields, with `inputs` as the command's input list and `extra`
/// appended to the `placed` branch's payload.
fn order(inputs: &str, sets: &str, payload_extra: &str, event_extra: &str) -> String {
    format!(
        r"
format: ess/13
system: shop
version: v1
domain: shop.order

entities:
  - name: shop.order.Order
    identity: {{name: order_id, type: String}}
    fields:
      - {{name: name, type: String}}
      - {{name: display_name, type: String}}
      - {{name: paid, type: Boolean}}
      - {{name: gift, type: Boolean}}
      - {{name: rush, type: Boolean}}
    lifecycle:
      initial: Open
      states: [Open]
      terminal: [Open]

events:
  - name: shop.order.OrderPlaced
    fields:
      - {{name: order_id, type: String}}
{event_extra}
views:
  - name: shop.order.Orders
    source: shop.order.Order
    consistency: read_your_writes
    fields:
      - {{name: order_id, type: String}}
      - {{name: name, type: String}}
      - {{name: display_name, type: String}}
      - {{name: paid, type: Boolean}}
      - {{name: gift, type: Boolean}}
      - {{name: rush, type: Boolean}}

commands:
  - name: shop.order.PlaceOrder
    input:
{inputs}
    outcomes:
      - name: placed
        creates: shop.order.Order
        instance: order_id
        sets: {{{sets}}}
        emits: [shop.order.OrderPlaced]
        payload:
          shop.order.OrderPlaced: {{order_id: input.order_id{payload_extra}}}
"
    )
}

// ---- 1. a join the declared model makes takes the one pin ----------------------------------------

/// The declared model writes one title into two fields (`name` and `display_name`) beside the
/// identity, a String no target reads. `joined_pair` takes the first qualifying source in input
/// order, so the declared String join is the pair every Boolean mutant's synthesis separates and
/// pins, and the pair the Boolean retarget joins is left to the unpinned sibling loop.
#[test]
fn a_join_the_declared_model_makes_does_not_crowd_out_the_retargeted_pair() {
    let text = order(
        "      - {name: order_id, type: String}\n      - {name: title, type: String}\n      - {name: paid, type: Boolean}\n      - {name: gift, type: Boolean}\n      - {name: rush, type: Boolean}",
        "name: input.title, display_name: input.title, paid: input.paid, gift: input.gift, rush: input.rush",
        "",
        "",
    );
    let left = survivors(
        &text,
        "shop.order.PlaceOrder",
        "shop.order.Orders",
        &[
            ("name", "title"),
            ("display_name", "title"),
            ("paid", "paid"),
            ("gift", "gift"),
            ("rush", "rush"),
        ],
        "/placed/",
    );
    assert!(left.is_empty(), "retargets survive: {left:#?}");
}

// ---- 2. a retarget onto an input no target reads -------------------------------------------------

/// An input `notify` goes only into the event payload and is declared before the flags. The
/// generator retargets each flag to the *first* same-typed input, which is `notify`, so every
/// mutant writes `X: input.notify`: no input feeds two targets, `joined_pair` finds nothing, and
/// the joined-source rule never fires for the mutant the audit actually makes.
#[test]
fn a_retarget_onto_an_input_only_the_payload_reads_is_killed() {
    let text = order(
        "      - {name: order_id, type: String}\n      - {name: notify, type: Boolean}\n      - {name: paid, type: Boolean}\n      - {name: gift, type: Boolean}\n      - {name: rush, type: Boolean}",
        "paid: input.paid, gift: input.gift, rush: input.rush",
        ", notify: input.notify",
        "      - {name: notify, type: Boolean}\n",
    );
    let left = survivors(
        &text,
        "shop.order.PlaceOrder",
        "shop.order.Orders",
        &FLAG_MAP,
        "/placed/",
    );
    assert!(left.is_empty(), "retargets survive: {left:#?}");
}

// ---- 3. the name preference among several unread inputs ------------------------------------------

/// Inputs in order `paid, notify, gift, rush`, `notify` read only by the payload. The `gift`
/// retarget is `gift: input.paid` (paid is the first Boolean other than gift), so `paid` feeds two
/// targets and both `notify` and `gift` are unread. Only the target-name preference in
/// `joined_pair` picks `gift`; the first declared unread input is `notify`. This case pins that line.
#[test]
fn the_unread_input_named_like_a_target_is_the_one_separated() {
    let text = order(
        "      - {name: order_id, type: String}\n      - {name: paid, type: Boolean}\n      - {name: notify, type: Boolean}\n      - {name: gift, type: Boolean}\n      - {name: rush, type: Boolean}",
        "paid: input.paid, gift: input.gift, rush: input.rush",
        ", notify: input.notify",
        "      - {name: notify, type: Boolean}\n",
    );
    let left = survivors(
        &text,
        "shop.order.PlaceOrder",
        "shop.order.Orders",
        &FLAG_MAP,
        "/placed/gift",
    );
    assert!(left.is_empty(), "retargets survive: {left:#?}");
}

/// The declared model of case 3 feeds each input into at most one target, so it joins nothing:
/// even with the guard holding `paid` and the payload-only `notify` equal, it carries no
/// `UnseparatedSources` note. A threshold of one target instead of two would name (paid, notify).
#[test]
fn a_declared_model_joining_nothing_carries_no_unseparated_note_even_when_guards_hold_inputs_equal()
{
    let text = order(
        "      - {name: order_id, type: String}\n      - {name: paid, type: Boolean}\n      - {name: notify, type: Boolean}\n      - {name: gift, type: Boolean}\n      - {name: rush, type: Boolean}",
        "paid: input.paid, gift: input.gift, rush: input.rush",
        ", notify: input.notify",
        "      - {name: notify, type: Boolean}\n",
    )
    .replace(
        "      - name: placed\n",
        "      - name: placed\n        when:\n          all: [paid == true, notify == true]\n",
    ) + r"
      - name: refused
        error: shop.order.NotPlaced

errors:
  - name: shop.order.NotPlaced
    summary: Only paid orders with notice are placed here.
    fields: []
";
    let model = documents(&text);
    assert_eq!(
        unseparated(&synthesize(&compiled(&model))),
        Vec::<(String, String, String)>::new()
    );
}

// ---- 4. two outcomes of one command --------------------------------------------------------------

/// Two branches of one command each write the three flags; each branch's mutants are killed on
/// that branch's own scenario.
#[test]
fn retargets_are_killed_on_each_of_two_outcomes_of_one_command() {
    let text = order(
        "      - {name: order_id, type: String}\n      - {name: paid, type: Boolean}\n      - {name: gift, type: Boolean}\n      - {name: rush, type: Boolean}\n      - {name: tier, type: String}",
        "paid: input.paid, gift: input.gift, rush: input.rush",
        "",
        "",
    )
    .replace(
        "      - name: placed\n",
        "      - name: express\n        when: tier == 'express'\n        creates: shop.order.Order\n        instance: order_id\n        sets: {paid: input.paid, gift: input.gift, rush: input.rush}\n        emits: [shop.order.OrderPlaced]\n        payload:\n          shop.order.OrderPlaced: {order_id: input.order_id}\n      - name: placed\n",
    );
    for branch in ["/express/", "/placed/"] {
        let left = survivors(
            &text,
            "shop.order.PlaceOrder",
            "shop.order.Orders",
            &FLAG_MAP,
            branch,
        );
        assert!(left.is_empty(), "retargets on {branch} survive: {left:#?}");
    }
}

// ---- 5. the stored-row path, where the move would leave the branch -------------------------------

/// An `updates` branch the stored-row search arranges (a `when_subject` refusal beside it) whose
/// own guard holds `paid` and `gift` true. The `gift` retarget joins exactly that pair, so no move
/// keeps the branch, and the note must name the scenario on this path too.
#[test]
fn the_stored_row_path_names_a_pair_its_guards_hold_equal() {
    let text = r"
format: ess/13
system: shop
version: v1
domain: shop.order

entities:
  - name: shop.order.Order
    identity: {name: order_id, type: String}
    fields:
      - {name: locked, type: Boolean}
      - {name: paid, type: Boolean}
      - {name: gift, type: Boolean}
      - {name: rush, type: Boolean}
    lifecycle:
      initial: Open
      states: [Open]
      terminal: [Open]

errors:
  - name: shop.order.Locked
    fields: []
  - name: shop.order.NotFlagged
    fields: []

events:
  - name: shop.order.OrderChanged
    fields:
      - {name: order_id, type: String}

views:
  - name: shop.order.Orders
    source: shop.order.Order
    consistency: read_your_writes
    fields:
      - {name: order_id, type: String}
      - {name: state, type: shop.order.Order.State}
      - {name: locked, type: Boolean}
      - {name: paid, type: Boolean}
      - {name: gift, type: Boolean}
      - {name: rush, type: Boolean}

commands:
  - name: shop.order.PlaceOrder
    input:
      - {name: order_id, type: String}
      - {name: locked, type: Boolean}
    outcomes:
      - name: placed
        creates: shop.order.Order
        instance: order_id
        sets: {locked: input.locked, paid: 'false', gift: 'false', rush: 'false'}
        emits: [shop.order.OrderChanged]
        payload:
          shop.order.OrderChanged: {order_id: input.order_id}
  - name: shop.order.SetFlags
    input:
      - {name: order_id, type: String}
      - {name: paid, type: Boolean}
      - {name: gift, type: Boolean}
      - {name: rush, type: Boolean}
    outcomes:
      - name: locked
        when_subject:
          predicate: locked == true
        error: shop.order.Locked
      - name: flagged
        when:
          all: [paid == true, gift == true]
        updates: shop.order.Order
        instance: order_id
        sets: {paid: input.paid, gift: input.gift, rush: input.rush}
        emits: [shop.order.OrderChanged]
        payload:
          shop.order.OrderChanged: {order_id: input.order_id}
      - name: unflagged
        error: shop.order.NotFlagged
";
    let model = documents(text);
    let (id, mutated) = retargets(&model)
        .into_iter()
        .find(|(id, _)| id.ends_with("/flagged/gift"))
        .expect("the audit retargets `gift`");
    let synthesis = synthesize(&mutated);
    assert!(
        !sent_and_required(&synthesis, "shop.order.SetFlags", "shop.order.Orders").is_empty(),
        "`{id}` requires an `Orders` row after `SetFlags`"
    );
    assert_eq!(
        unseparated(&synthesis),
        vec![(
            "shop.order.SetFlags/outcome/flagged".to_owned(),
            "paid".to_owned(),
            "gift".to_owned()
        )],
        "`{id}` joins `paid` and `gift`, which the guard holds equal"
    );
}

// ---- 6. a note about a scenario the suite does not hold ------------------------------------------

/// The branch's guard holds `paid` and `gift` true, so the `gift` retarget's scenario is noted as
/// unseparated. The command also orders `starts_at` against `now` and against a fixed instant after
/// the synthesis reference, which `now_offset::install` refuses by removing every scenario sending
/// it, after the note was recorded. A note naming a scenario the suite does not hold points the
/// reader at nothing.
#[test]
fn every_unseparated_note_names_a_scenario_the_suite_holds() {
    let text = r"
format: ess/16
system: demo
version: v1
domain: demo.jobs
types:
  - {name: demo.jobs.JobId, kind: newtype, of: Uuid}
entities:
  - name: demo.jobs.Job
    identity: {name: job_id, type: demo.jobs.JobId}
    fields:
      - {name: starts_at, type: Timestamp}
      - {name: paid, type: Boolean}
      - {name: gift, type: Boolean}
      - {name: rush, type: Boolean}
    lifecycle: {initial: Scheduled, states: [Scheduled], terminal: [Scheduled], transitions: []}
actors:
  - {name: demo.jobs.Clerk, may: [demo.jobs.ScheduleJob]}
errors:
  - name: demo.jobs.TooEarly
    summary: The start is before the programme opened.
    fields: []
  - name: demo.jobs.StartInPast
    summary: The start time is more than 60 seconds in the past.
    fields: []
  - name: demo.jobs.NotPaid
    summary: Only paid gifts are scheduled.
    fields: []
commands:
  - name: demo.jobs.ScheduleJob
    input:
      - {name: starts_at, type: Timestamp}
      - {name: paid, type: Boolean}
      - {name: gift, type: Boolean}
      - {name: rush, type: Boolean}
    outcomes:
      - name: too-early
        when: starts_at < '2025-01-01T00:00:00Z'
        error: demo.jobs.TooEarly
      - name: start-in-past
        when: starts_at < now - 60s
        error: demo.jobs.StartInPast
      - name: scheduled
        when:
          all: [paid == true, gift == true]
        creates: demo.jobs.Job
        instance: job_id
        sets: {starts_at: input.starts_at, paid: input.paid, gift: input.gift, rush: input.rush}
        emits: [demo.jobs.JobScheduled]
        payload:
          demo.jobs.JobScheduled: {job_id: {generated: true}}
      - name: unpaid
        error: demo.jobs.NotPaid
events:
  - name: demo.jobs.JobScheduled
    fields:
      - {name: job_id, type: demo.jobs.JobId}
views:
  - name: demo.jobs.JobDetails
    source: demo.jobs.Job
    consistency: read_your_writes
    fields:
      - {name: job_id, type: demo.jobs.JobId}
      - {name: starts_at, type: Timestamp}
      - {name: paid, type: Boolean}
      - {name: gift, type: Boolean}
      - {name: rush, type: Boolean}
";
    let model = documents(text);
    let (id, mutated) = retargets(&model)
        .into_iter()
        .find(|(id, _)| id.ends_with("/scheduled/gift"))
        .expect("the audit retargets `gift`");
    let synthesis = synthesize(&mutated);
    let dangling: Vec<_> = unseparated(&synthesis)
        .into_iter()
        .filter(|(scenario, _, _)| {
            !synthesis
                .suite
                .scenarios
                .keys()
                .any(|held| held.to_string() == *scenario)
        })
        .collect();
    assert!(
        dangling.is_empty(),
        "`{id}`: notes name scenarios the suite does not hold: {dangling:?}; held: {:?}",
        synthesis.suite.scenarios.keys().collect::<Vec<_>>()
    );
}

// ---- 7. determinism of the notes -----------------------------------------------------------------

#[test]
fn the_unseparated_notes_are_the_same_on_every_synthesis() {
    let text = order(
        "      - {name: order_id, type: String}\n      - {name: paid, type: Boolean}\n      - {name: gift, type: Boolean}\n      - {name: rush, type: Boolean}",
        "paid: input.paid, gift: input.gift, rush: input.rush",
        "",
        "",
    )
    .replace(
        "      - name: placed\n",
        "      - name: placed\n        when:\n          all: [paid == true, gift == true]\n",
    ) + r"
      - name: refused
        error: shop.order.NotPlaced

errors:
  - name: shop.order.NotPlaced
    summary: Only paid gifts are placed here.
    fields: []
";
    for (id, mutated) in retargets(&documents(&text)) {
        let first = synthesize(&mutated);
        let second = synthesize(&mutated);
        assert_eq!(first.notes, second.notes, "`{id}`");
        assert_eq!(
            first.suite.to_canonical_json().unwrap(),
            second.suite.to_canonical_json().unwrap(),
            "`{id}`"
        );
    }
}
