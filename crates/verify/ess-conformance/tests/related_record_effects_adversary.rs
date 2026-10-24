//! Adversary cases for `story:a-cascade-onto-rows-of-a-guarded-creator-is-witnessed`: a moving
//! `affects:` entry whose rows are created by a branch below guarded siblings. Each case states what
//! the story or the CHANGELOG line promises and drives synthesis, the interpreter and a mutated
//! target against it.

use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::report::Status;
use ess_conformance::{AdmittedSuite, ConformanceScenario, ConformanceSuite, Runner, ScenarioStep};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

const FEED: &str = r"format: ess/25
system: cascade
version: v1
domain: cascade.feed
types:
  - {name: cascade.feed.FeedId, kind: newtype, of: Uuid}
  - {name: cascade.feed.ItemId, kind: newtype, of: Uuid}
entities:
  - name: cascade.feed.Feed
    identity: {name: feed_id, type: cascade.feed.FeedId}
    fields: [{name: label, type: String}]
    lifecycle:
      initial: Open
      states: [Open, Cancelled]
      terminal: [Cancelled]
      transitions: [{name: cancel, from: [Open], to: Cancelled}]
  - name: cascade.feed.Item
    identity: {name: item_id, type: cascade.feed.ItemId}
    fields: [{name: feed_id, type: cascade.feed.FeedId}]
    relations:
      - {name: feed, kind: references, target: cascade.feed.Feed, cardinality: one, via: feed_id}
    lifecycle:
      initial: Pending
      states: [Pending, Discarded]
      terminal: [Discarded]
      transitions: [{name: discard, from: [Pending], to: Discarded}]
errors:
  - {name: cascade.feed.FeedNotFound, fields: []}
  - {name: cascade.feed.WrongState, fields: []}
  - {name: cascade.feed.Blank, fields: []}
events:
  - name: cascade.feed.FeedOpened
    fields: [{name: feed_id, type: cascade.feed.FeedId}]
  - name: cascade.feed.ItemPosted
    fields: [{name: item_id, type: cascade.feed.ItemId}]
  - name: cascade.feed.FeedCancelled
    fields: [{name: feed_id, type: cascade.feed.FeedId}]
actors:
  - name: cascade.feed.Operator
    may: [cascade.feed.Open, cascade.feed.Post, cascade.feed.Cancel]
commands:
  - name: cascade.feed.Open
    input: [{name: label, type: String}]
    outcomes:
      - name: opened
        creates: cascade.feed.Feed
        instance: feed_id
        emits: [cascade.feed.FeedOpened]
        payload: {cascade.feed.FeedOpened: {feed_id: {generated: true}}}
        sets: {label: input.label}
  - name: cascade.feed.Post
    input:
      - {name: feed_id, type: cascade.feed.FeedId}
      - {name: urgent, type: Boolean}
    outcomes:
      - name: posted
        creates: cascade.feed.Item
        instance: item_id
        emits: [cascade.feed.ItemPosted]
        payload: {cascade.feed.ItemPosted: {item_id: {generated: true}}}
        sets: {feed_id: input.feed_id}
  - name: cascade.feed.Cancel
    input: [{name: feed_id, type: cascade.feed.FeedId}]
    outcomes:
      - name: cancelled
        moves: cascade.feed.Feed.cancel
        instance: feed_id
        emits: [cascade.feed.FeedCancelled]
        payload: {cascade.feed.FeedCancelled: {feed_id: input.feed_id}}
        affects:
          - entity: cascade.feed.Item
            where: feed_id == input.feed_id
            moves: cascade.feed.Item.discard
      - {name: wrong-state, wrong_state: true, error: cascade.feed.WrongState}
views:
  - name: cascade.feed.Items
    source: cascade.feed.Item
    consistency: read_your_writes
    fields:
      - {name: item_id, type: cascade.feed.ItemId}
      - {name: feed_id, type: cascade.feed.FeedId}
      - {name: state, type: cascade.feed.Item.State}
  - name: cascade.feed.Feeds
    source: cascade.feed.Feed
    consistency: read_your_writes
    fields:
      - {name: feed_id, type: cascade.feed.FeedId}
      - {name: state, type: cascade.feed.Feed.State}
";

const CANCELLED: &str = "cascade.feed.Cancel/outcome/cancelled";
const POSTED: &str = "cascade.feed.Post/outcome/posted";

const UNKNOWN_FEED: &str = "      - name: unknown-feed
        when_related: {via: input.feed_id, exists: false}
        error: cascade.feed.FeedNotFound
";
const BLANK: &str = "      - {name: blank, when: urgent == true, error: cascade.feed.Blank}
";

const ENTRY: &str = "        affects:
          - entity: cascade.feed.Item
            where: feed_id == input.feed_id
            moves: cascade.feed.Item.discard
";

/// `model` with `extra` input fields on `Post` and `refusals` above its creating branch.
fn post_with(model: &str, extra: &str, refusals: &[&str]) -> String {
    let input = "      - {name: urgent, type: Boolean}\n";
    let at = "      - name: posted\n";
    assert!(model.contains(input) && model.contains(at));
    model
        .replacen(input, &format!("{input}{extra}"), 1)
        .replacen(at, &format!("{}{at}", refusals.concat()), 1)
}

fn ir_of(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("feed.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model is admitted: {errors}\n{text}"));
    compile(&spec, &SourceMap::new()).expect("the model compiles")
}

fn find<'a>(suite: &'a ConformanceSuite, id: &str) -> Option<&'a ConformanceScenario> {
    suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map(|(_, scenario)| scenario)
}

fn run(suite: &ConformanceSuite, ir: EssIr) -> BTreeMap<String, Status> {
    let target = ess_conformance::interpret::Interpreted::for_model(ir);
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &target)
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| (result.scenario.to_string(), result.status))
        .collect()
}

fn sends(scenario: &ConformanceScenario, command: &str) -> usize {
    scenario
        .steps
        .iter()
        .filter(|step| {
            matches!(step, ScenarioStep::ExecuteCommand { command: sent, .. }
                if sent.to_string() == command)
        })
        .count()
}

fn causes(synthesis: &ess_conformance::synthesize::Synthesis) -> Vec<String> {
    synthesis
        .refusals
        .iter()
        .map(|refusal| format!("{refusal}: {}", refusal.cause))
        .collect()
}

/// A target that skips the move: the entry left out, and a command no scenario sends taking the
/// transition so the model stays admitted.
fn skipping(model: &str) -> String {
    let may = "    may: [";
    let views = "views:\n";
    assert!(model.contains(ENTRY) && model.contains(may) && model.contains(views));
    model
        .replacen(ENTRY, "", 1)
        .replacen(may, "    may: [cascade.feed.Discard, ", 1)
        .replacen(
            views,
            "  - name: cascade.feed.Discard
    input: [{name: item_id, type: cascade.feed.ItemId}]
    outcomes:
      - name: discarded
        moves: cascade.feed.Item.discard
        instance: item_id
        emits: [cascade.feed.ItemPosted]
        payload: {cascade.feed.ItemPosted: {item_id: input.item_id}}
      - {name: not-pending, wrong_state: true, error: cascade.feed.WrongState}
views:
",
            1,
        )
}

/// A target that discards every pending item, whatever feed it belongs to.
fn ignoring_filter(model: &str) -> String {
    let filter = "            where: feed_id == input.feed_id\n";
    assert!(model.contains(filter));
    model.replacen(
        filter,
        "            where: {any: [feed_id == input.feed_id, feed_id != input.feed_id]}\n",
        1,
    )
}

/// The cancellation of `model` is synthesized, passes the interpreter, and fails a target that
/// skips the move and one that ignores the filter.
fn witnessed(model: &str) {
    let ir = ir_of(model);
    let synthesis = ess_conformance::synthesize::synthesize(&ir);
    let cancel: Vec<String> = causes(&synthesis)
        .into_iter()
        .filter(|cause| cause.contains("Cancel"))
        .collect();
    assert_eq!(cancel, Vec::<String>::new(), "{model}");
    let cancelled = find(&synthesis.suite, CANCELLED)
        .unwrap_or_else(|| panic!("no {CANCELLED}: {:#?}", causes(&synthesis)));
    assert!(
        sends(cancelled, "cascade.feed.Post") >= 3,
        "{:#?}",
        cancelled.steps
    );
    let statuses = run(&synthesis.suite, ir);
    assert_eq!(
        statuses.get(CANCELLED),
        Some(&Status::Passed),
        "{statuses:#?}"
    );
    let skipped = run(&synthesis.suite, ir_of(&skipping(model)));
    assert_eq!(
        skipped.get(CANCELLED),
        Some(&Status::Failed),
        "{skipped:#?}"
    );
    let wide = run(&synthesis.suite, ir_of(&ignoring_filter(model)));
    assert_eq!(
        wide.get(CANCELLED),
        Some(&Status::Failed),
        "a target discarding another feed's items passes:\n{:#?}",
        cancelled.steps
    );
}

// ---- several guarded siblings -------------------------------------------------------------------

#[test]
fn adv_two_boolean_guarded_siblings_are_refuted_together() {
    witnessed(&post_with(
        FEED,
        "      - {name: hidden, type: Boolean}\n",
        &[
            UNKNOWN_FEED,
            BLANK,
            "      - {name: concealed, when: hidden == true, error: cascade.feed.Blank}\n",
        ],
    ));
}

#[test]
fn adv_four_integer_guarded_siblings_are_refuted_together() {
    let model = post_with(
        FEED,
        "      - {name: a, type: Integer}
      - {name: b, type: Integer}
      - {name: c, type: Integer}
      - {name: d, type: Integer}
",
        &[
            UNKNOWN_FEED,
            "      - {name: big-a, when: a > 10, error: cascade.feed.Blank}\n",
            "      - {name: big-b, when: b > 10, error: cascade.feed.Blank}\n",
            "      - {name: big-c, when: c > 10, error: cascade.feed.Blank}\n",
            "      - {name: big-d, when: d > 10, error: cascade.feed.Blank}\n",
        ],
    );
    // Control: the creating branch is reached on its own.
    let synthesis = ess_conformance::synthesize::synthesize(&ir_of(&model));
    assert!(
        find(&synthesis.suite, POSTED).is_some(),
        "{:#?}",
        causes(&synthesis)
    );
    witnessed(&model);
}

#[test]
fn adv_a_disjunctive_guarded_sibling_is_refuted() {
    witnessed(&post_with(
        FEED,
        "      - {name: a, type: Integer}\n",
        &[
            UNKNOWN_FEED,
            "      - {name: odd, when: {any: [urgent == true, a > 10]}, error: cascade.feed.Blank}\n",
        ],
    ));
}

#[test]
fn adv_a_guarded_creator_cascade_fails_a_target_ignoring_the_filter() {
    witnessed(&post_with(FEED, "", &[UNKNOWN_FEED, BLANK]));
}

// ---- the related row's state ----------------------------------------------------------------------

/// [`FEED`] where a feed is created `Draft` and opened by `Activate`, and `Post` refuses a feed in
/// a state `refused` names.
fn activated(refused: &str) -> String {
    let lifecycle = "      initial: Open
      states: [Open, Cancelled]
      terminal: [Cancelled]
      transitions: [{name: cancel, from: [Open], to: Cancelled}]
";
    let may = "cascade.feed.Cancel]\n";
    let views = "views:\n";
    assert!(FEED.contains(lifecycle) && FEED.contains(may));
    let model = FEED
        .replacen(
            lifecycle,
            "      initial: Draft
      states: [Draft, Open, Cancelled]
      terminal: [Cancelled]
      transitions:
        - {name: activate, from: [Draft], to: Open}
        - {name: cancel, from: [Open], to: Cancelled}
",
            1,
        )
        .replacen(may, "cascade.feed.Cancel, cascade.feed.Activate]\n", 1)
        .replacen(
            views,
            "  - name: cascade.feed.Activate
    input: [{name: feed_id, type: cascade.feed.FeedId}]
    outcomes:
      - name: activated
        moves: cascade.feed.Feed.activate
        instance: feed_id
        emits: [cascade.feed.FeedOpened]
        payload: {cascade.feed.FeedOpened: {feed_id: input.feed_id}}
      - {name: not-draft, wrong_state: true, error: cascade.feed.WrongState}
views:
",
            1,
        );
    let stale = format!(
        "      - name: stale-feed
        when_related: {{via: input.feed_id, predicate: {refused}}}
        error: cascade.feed.WrongState
"
    );
    if refused.is_empty() {
        post_with(&model, "", &[UNKNOWN_FEED])
    } else {
        post_with(&model, "", &[UNKNOWN_FEED, &stale])
    }
}

#[test]
fn adv_control_a_creator_refusing_only_an_unknown_activated_feed_is_witnessed() {
    // Green control for the case below: the activation alone is not what blocks the cascade.
    witnessed(&activated(""));
}

#[test]
fn adv_a_creator_refusing_a_feed_not_yet_activated_is_witnessed() {
    // The story's added acceptance: `state != Active`, the subject reached through a move.
    let model = activated("state != Open");
    // Control: the creating branch, below both refusals, is reached on its own.
    let synthesis = ess_conformance::synthesize::synthesize(&ir_of(&model));
    assert!(
        find(&synthesis.suite, POSTED).is_some(),
        "{:#?}",
        causes(&synthesis)
    );
    witnessed(&model);
}

#[test]
fn adv_a_creator_taken_only_before_the_subject_moves_is_named_when_refused() {
    // Items are posted only to a draft feed; a cancellation needs it open. The rows are reachable
    // (post, then activate), but the arrangement holds the feed `Open` while posting. Where it is
    // refused, the cause must name the creator, not the arrival state.
    let model = activated("state != Draft");
    let synthesis = ess_conformance::synthesize::synthesize(&ir_of(&model));
    if find(&synthesis.suite, CANCELLED).is_some() {
        witnessed(&model);
        return;
    }
    let cause = synthesis
        .refusals
        .iter()
        .find(|refusal| refusal.to_string().contains("Cancel/outcome/cancelled"))
        .map_or_else(
            || panic!("{:#?}", causes(&synthesis)),
            |refusal| refusal.cause.to_string(),
        );
    assert!(
        cause.contains("cascade.feed.Post/posted") && !cause.contains("arrives in"),
        "{cause}"
    );
}

// ---- determinism --------------------------------------------------------------------------------

#[test]
fn adv_the_cascade_scenario_bytes_are_the_same_on_every_run() {
    let model = post_with(
        FEED,
        "      - {name: hidden, type: Boolean}\n",
        &[
            UNKNOWN_FEED,
            BLANK,
            "      - {name: concealed, when: hidden == true, error: cascade.feed.Blank}\n",
        ],
    );
    // Separate threads: the witness memo is per thread, so each run searches afresh.
    let bytes: Vec<String> = (0..4)
        .map(|_| {
            let model = model.clone();
            std::thread::spawn(move || {
                let synthesis = ess_conformance::synthesize::synthesize(&ir_of(&model));
                let refusals = causes(&synthesis).join("\n");
                let suite = synthesis
                    .suite
                    .to_canonical_json()
                    .unwrap_or_else(|error| panic!("{error}"));
                format!("{suite}\n{refusals}")
            })
            .join()
            .expect("synthesis runs")
        })
        .collect();
    assert!(bytes[0].contains(CANCELLED), "{}", bytes[0]);
    for other in &bytes[1..] {
        assert!(other == &bytes[0], "two runs differ");
    }
}

// ---- refusal text -------------------------------------------------------------------------------

#[test]
fn adv_an_undriven_refusal_names_every_creator_and_its_own_help() {
    // Two commands create items, both refusing an open feed: no item of the feed a cancellation
    // names can be created.
    let open_feed = "      - name: open-feed
        when_related: {via: input.feed_id, predicate: state == Open}
        error: cascade.feed.WrongState
";
    let model = post_with(FEED, "", &[UNKNOWN_FEED, open_feed]);
    let post = "  - name: cascade.feed.Post\n";
    let cancel = "  - name: cascade.feed.Cancel\n";
    let start = model.find(post).expect("Post");
    let end = model.find(cancel).expect("Cancel");
    let repost = model[start..end].replacen(post, "  - name: cascade.feed.Repost\n", 1);
    let model = model
        .replacen(cancel, &format!("{repost}{cancel}"), 1)
        .replacen(
            "cascade.feed.Cancel]\n",
            "cascade.feed.Cancel, cascade.feed.Repost]\n",
            1,
        );
    let synthesis = ess_conformance::synthesize::synthesize(&ir_of(&model));
    assert!(find(&synthesis.suite, CANCELLED).is_none());
    let refusal = synthesis
        .refusals
        .iter()
        .find(|refusal| refusal.to_string().contains("Cancel/outcome/cancelled"))
        .unwrap_or_else(|| panic!("{:#?}", causes(&synthesis)));
    let cause = refusal.cause.to_string();
    assert!(
        cause.contains("cascade.feed.Post/posted")
            && cause.contains("cascade.feed.Repost/posted")
            && !cause.contains("arrives in"),
        "{cause}"
    );
    let help = refusal.hint();
    assert!(
        !help.contains("give the field a type that has a finite value") && help.contains("creat"),
        "{help}"
    );
}
