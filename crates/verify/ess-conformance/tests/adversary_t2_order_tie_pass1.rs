//! Adversary, pass 1, on the synthesized tie of a ranked view's later keys (beyond10x/ess#708).
//!
//! The contract the story states: for a view ranked by keys k1..kn, the suite pins every key after
//! the first with two rows equal on the keys before it and different on it, or synthesis names a
//! standing gap for that view and key beside the scenario. So an `order-flip` of a later key is
//! killed, or a standing refusal names the view and the key. A survivor with no such refusal is a
//! key the suite silently does not pin.

use std::fmt::Write as _;

use ess_compiler::source::SourceMap;
use ess_conformance::interpret::Interpreted;
use ess_conformance::mutate::{self, MutantClass, MutationReport, Verdict};
use ess_conformance::synthesize::Synthesis;
use ess_domain::spec::RawSpecFile;
use ess_domain::system::Source;

/// A one-domain model: `Order` with `fields`, created by `PlaceOrder` writing `sets`, an optional
/// `ShipOrder` moving it to `Shipped`, and the view `OrderQueue` declared by `view`.
fn spec(fields: &[(&str, &str)], sets: &str, view: &str, ship: bool) -> String {
    let declared = fields.iter().fold(String::new(), |mut out, (name, kind)| {
        let _ = writeln!(out, "      - {{name: {name}, type: {kind}}}");
        out
    });
    let lifecycle = if ship {
        "{initial: Placed, states: [Placed, Shipped], terminal: [Shipped], transitions: [{name: ship, from: [Placed], to: Shipped}]}"
    } else {
        "{initial: Placed, states: [Placed], terminal: [Placed], transitions: []}"
    };
    let ship_command = if ship {
        "  - name: example.order.ShipOrder
    input:
      - {name: order_id, type: example.order.OrderId}
    outcomes:
      - name: shipped
        moves: example.order.Order.ship
        instance: order_id
        emits: [example.order.OrderShipped]
        payload:
          example.order.OrderShipped: {order_id: input.order_id}
"
    } else {
        ""
    };
    format!(
        "format: ess/32
system: example
version: v1
domain: example.order
summary: An adversary fixture for the tie of a later order key.
types:
  - {{name: example.order.OrderId, kind: newtype, of: Uuid}}
  - {{name: example.order.Kind, kind: enum, variants: [Rush, Normal]}}
entities:
  - name: example.order.Order
    identity: {{name: order_id, type: example.order.OrderId}}
    fields:
{declared}    lifecycle: {lifecycle}
events:
  - name: example.order.OrderPlaced
    fields: [{{name: order_id, type: example.order.OrderId}}]
  - name: example.order.OrderShipped
    fields: [{{name: order_id, type: example.order.OrderId}}]
commands:
  - name: example.order.PlaceOrder
    input:
{declared}    outcomes:
      - name: placed
        creates: example.order.Order
        instance: order_id
        emits: [example.order.OrderPlaced]
        payload:
          example.order.OrderPlaced: {{order_id: {{generated: true}}}}
        sets: {sets}
{ship_command}views:
  - name: example.order.OrderQueue
    source: example.order.Order
    consistency: read_your_writes
{view}"
    )
}

/// Every field written from the input of the same name.
fn from_input(fields: &[(&str, &str)]) -> String {
    let entries: Vec<String> = fields
        .iter()
        .map(|(name, _)| format!("{name}: input.{name}"))
        .collect();
    format!("{{{}}}", entries.join(", "))
}

fn view_fields(fields: &[(&str, &str)]) -> String {
    let mut text =
        String::from("    fields:\n      - {name: order_id, type: example.order.OrderId}\n");
    for (name, kind) in fields {
        let _ = writeln!(text, "      - {{name: {name}, type: {kind}}}");
    }
    text
}

fn compiled(yaml: &str) -> (Vec<mutate::Document>, SourceMap) {
    let mut texts = SourceMap::new();
    texts.insert("system.yaml".to_owned(), yaml.to_owned());
    let raw = RawSpecFile::parse(yaml).unwrap_or_else(|error| panic!("{error}\n{yaml}"));
    (vec![(Source::new("system.yaml"), raw)], texts)
}

fn audited(yaml: &str) -> (MutationReport, Synthesis) {
    let (files, texts) = compiled(yaml);
    let ir = mutate::compile(files.clone(), &texts)
        .unwrap_or_else(|stillborn| panic!("{stillborn:?}\n{yaml}"));
    let report = mutate::audit(&files, &texts, &[MutantClass::OrderFlip], || {
        Interpreted::for_model(ir.clone())
    })
    .unwrap_or_else(|refusal| panic!("the baseline is refused: {refusal}\n{yaml}"));
    (report, ess_conformance::synthesize::synthesize(&ir))
}

/// Every `order-flip` not killed is answered by a standing refusal naming its view and its key.
fn killed_or_named(yaml: &str) -> MutationReport {
    let (report, synthesis) = audited(yaml);
    let named: Vec<String> = synthesis
        .refusals
        .iter()
        .filter(|refusal| refusal.stands)
        .map(ToString::to_string)
        .collect();
    let silent: Vec<&str> = report
        .mutants
        .iter()
        .filter(|entry| entry.verdict != Verdict::Killed)
        .map(|entry| entry.id.as_str())
        .filter(|id| {
            let mut parts = id.splitn(3, '/').skip(1);
            let (view, key) = (parts.next().unwrap(), parts.next().unwrap());
            !named
                .iter()
                .any(|text| text.contains(view) && text.contains(&format!("`{key}`")))
        })
        .collect();
    assert!(
        silent.is_empty(),
        "order-flip mutants neither killed nor named by a standing gap: {silent:?}\n{}\nstanding \
         refusals: {named:#?}\n{yaml}",
        report.render_text()
    );
    report
}

fn counts(report: &MutationReport) -> (usize, usize, usize) {
    (
        report.counts.mutants,
        report.counts.killed,
        report.counts.survived,
    )
}

const PLAIN: [(&str, &str); 2] = [("region", "String"), ("priority", "Integer")];

/// Three keys, mixed directions, an enum key first and a timestamp key in the middle.
#[test]
fn three_keys_with_mixed_directions_over_enum_and_timestamp_are_all_killed() {
    let fields = [
        ("kind", "example.order.Kind"),
        ("placed_at", "Timestamp"),
        ("priority", "Integer"),
    ];
    let view = format!(
        "    order_by: [kind desc, placed_at asc, priority desc]\n{}",
        view_fields(&fields)
    );
    let report = killed_or_named(&spec(&fields, &from_input(&fields), &view, false));
    assert_eq!(counts(&report), (3, 3, 0), "{}", report.render_text());
}

/// The view's filter hides the subject, so the tie's base row is the first companion; the identity
/// as the last key needs a second row equal to it on both earlier keys.
#[test]
fn an_identity_last_key_of_a_view_hiding_the_subject_by_a_bound_is_pinned_or_named() {
    let view = format!(
        "    filter: priority >= 100\n    order_by: [region asc, priority asc, order_id asc]\n{}",
        view_fields(&PLAIN)
    );
    killed_or_named(&spec(&PLAIN, &from_input(&PLAIN), &view, false));
}

#[test]
fn an_identity_last_key_of_a_view_hiding_the_subject_by_an_enum_is_pinned_or_named() {
    let fields = [
        ("region", "String"),
        ("priority", "Integer"),
        ("kind", "example.order.Kind"),
    ];
    let view = format!(
        "    filter: kind == Normal\n    order_by: [region asc, priority asc, order_id asc]\n{}",
        view_fields(&fields)
    );
    killed_or_named(&spec(&fields, &from_input(&fields), &view, false));
}

#[test]
fn an_identity_last_key_of_a_view_of_a_later_state_is_pinned_or_named() {
    let view = format!(
        "    filter: state == Shipped\n    order_by: [region asc, priority asc, order_id asc]\n{}",
        view_fields(&PLAIN)
    );
    killed_or_named(&spec(&PLAIN, &from_input(&PLAIN), &view, true));
}

/// A later key that may be absent.
#[test]
fn an_optional_later_key_is_pinned_or_named() {
    let fields = [("region", "String"), ("due", "Optional<Timestamp>")];
    let view = format!(
        "    order_by: [region asc, due desc]\n{}",
        view_fields(&fields)
    );
    killed_or_named(&spec(&fields, &from_input(&fields), &view, false));
}

/// An earlier key that may be absent.
#[test]
fn an_optional_earlier_key_is_pinned_or_named() {
    let fields = [("team", "Optional<String>"), ("priority", "Integer")];
    let view = format!(
        "    order_by: [team asc, priority desc]\n{}",
        view_fields(&fields)
    );
    killed_or_named(&spec(&fields, &from_input(&fields), &view, false));
}

/// A paged view: the tie rows join the four rows the pages want, and the baseline still passes.
#[test]
fn a_paged_view_keeps_its_baseline_and_kills_both_flips() {
    let view = format!(
        "    params:\n      - {{name: page, type: Integer}}\n      - {{name: size, type: Integer}}\n    \
         order_by: [region asc, priority desc]\n    paging: {{page: page, size: size}}\n{}",
        view_fields(&PLAIN)
    );
    let report = killed_or_named(&spec(&PLAIN, &from_input(&PLAIN), &view, false));
    assert_eq!(counts(&report), (2, 2, 0), "{}", report.render_text());
}

#[test]
fn a_paged_view_with_the_identity_last_kills_all_three_flips() {
    let view = format!(
        "    params:\n      - {{name: page, type: Integer}}\n      - {{name: size, type: Integer}}\n    \
         order_by: [region asc, priority asc, order_id asc]\n    paging: {{page: page, size: size}}\n{}",
        view_fields(&PLAIN)
    );
    let report = killed_or_named(&spec(&PLAIN, &from_input(&PLAIN), &view, false));
    assert_eq!(counts(&report), (3, 3, 0), "{}", report.render_text());
}

/// A later key the creating command writes from the same input as the first: no row agrees on the
/// first and differs on it, so the gap must be named.
#[test]
fn a_key_the_creator_cannot_set_apart_from_the_first_is_named() {
    let fields = [("region", "String"), ("zone", "String")];
    let view = format!(
        "    order_by: [region asc, zone desc]\n{}",
        view_fields(&fields)
    );
    let yaml = spec(
        &fields,
        "{region: input.region, zone: input.region}",
        &view,
        false,
    )
    .replacen(
        "    input:\n      - {name: region, type: String}\n      - {name: zone, type: String}\n",
        "    input:\n      - {name: region, type: String}\n",
        1,
    );
    killed_or_named(&yaml);
}

/// The identity first, two keys after it: each is a named gap beside the one scenario.
#[test]
fn the_identity_first_names_a_gap_for_each_later_key() {
    let view = format!(
        "    order_by: [order_id asc, region asc, priority desc]\n{}",
        view_fields(&PLAIN)
    );
    let yaml = spec(&PLAIN, &from_input(&PLAIN), &view, false);
    let report = killed_or_named(&yaml);
    assert_eq!(counts(&report).1, 1, "{}", report.render_text());
    let (_, synthesis) = audited(&yaml);
    let standing: Vec<String> = synthesis
        .refusals
        .iter()
        .filter(|refusal| refusal.stands)
        .map(|refusal| {
            format!(
                "{} {}",
                refusal
                    .scenario
                    .as_ref()
                    .map(ToString::to_string)
                    .unwrap_or_default(),
                refusal
            )
        })
        .collect();
    assert_eq!(standing.len(), 2, "{standing:#?}");
    for key in ["`region`", "`priority`"] {
        assert!(
            standing.iter().any(|text| text
                .starts_with("example.order.PlaceOrder/outcome/placed ")
                && text.contains("example.order.OrderQueue")
                && text.contains(key)),
            "{key}: {standing:#?}"
        );
    }
}
