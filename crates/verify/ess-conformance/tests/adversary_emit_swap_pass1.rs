//! Adversary, pass 1, on `emit-swap` (beyond10x/ess#295).
//!
//! Drives the candidate rules of `docs/design/mutation-scope-and-known-failures.md`, "Single-event
//! mutation: emit-swap", and its "Implementation decisions for #295", at the cases the unit's own
//! fixtures leave out: more than one admissible candidate (so byte order decides), a field of a
//! different named type of the same shape, `List` against `Optional`, a different declaration
//! order, an extra field, an outcome without a `payload:` map, publication by domain ownership
//! against explicit listing, a command no component accepts, the exit-relevant count of a
//! component-scoped audit, and the manifest refusals the unit's tests do not reach.

use std::collections::BTreeMap;

use ess_compiler::source::SourceMap;
use ess_conformance::interpret::Interpreted;
use ess_conformance::mutate::{
    self, Document, MutantClass, Mutation, MutationReport, UnavailableReason, BASELINE_DIR,
    MANIFEST_FILE, REPORT_FILE, SUITE_FILE,
};
use ess_conformance::runner::Runner;
use ess_conformance::target::ConformanceTarget;
use ess_conformance::{AdmittedSuite, CountReport};
use ess_domain::spec::RawSpecFile;
use ess_domain::system::Source;
use serde_json::{json, Value};

const SWAP: &[MutantClass] = &[MutantClass::EmitSwap];

/// One command per candidate rule. `Place` emits `Placed { order_id: OrderId, note: String }`:
/// `AByRef` has `order_id: OrderRef` (a newtype of the same shape), `CExtra` one more field,
/// `DReordered` the same fields declared the other way round, `EAlso` the same fields in the same
/// order. `Tag` emits `Tagged { order_id, labels: List<String> }`: `ATaggedOptional` has
/// `Optional<String>`, `ATaggedOptionalList` `Optional<List<String>>`, `BTaggedExact` the same.
/// `Ping`'s `Ponged` differs only in order. `Cancel` and `Drop` have no alternative. One component
/// owns the domain and lists nothing.
const COMPAT: &str = r"format: ess/16
system: cmp
version: v1
domain: cmp.order
types:
  - {name: cmp.order.OrderId, kind: newtype, of: String}
  - {name: cmp.order.OrderRef, kind: newtype, of: String}
events:
  - name: cmp.order.AByRef
    fields:
      - {name: order_id, type: cmp.order.OrderRef}
      - {name: note, type: String}
  - name: cmp.order.ATaggedOptional
    fields:
      - {name: order_id, type: cmp.order.OrderId}
      - {name: labels, type: Optional<String>}
  - name: cmp.order.ATaggedOptionalList
    fields:
      - {name: order_id, type: cmp.order.OrderId}
      - {name: labels, type: Optional<List<String>>}
  - name: cmp.order.BTaggedExact
    fields:
      - {name: order_id, type: cmp.order.OrderId}
      - {name: labels, type: List<String>}
  - name: cmp.order.CExtra
    fields:
      - {name: order_id, type: cmp.order.OrderId}
      - {name: note, type: String}
      - {name: extra, type: String}
  - name: cmp.order.Cancelled
    fields:
      - {name: order_id, type: cmp.order.OrderId}
      - {name: reason, type: Integer}
  - name: cmp.order.DReordered
    fields:
      - {name: note, type: String}
      - {name: order_id, type: cmp.order.OrderId}
  - name: cmp.order.Dropped
    fields:
      - {name: order_id, type: cmp.order.OrderId}
      - {name: by, type: Boolean}
  - name: cmp.order.EAlso
    fields:
      - {name: order_id, type: cmp.order.OrderId}
      - {name: note, type: String}
  - name: cmp.order.Pinged
    fields:
      - {name: order_id, type: cmp.order.OrderId}
      - {name: at, type: Integer}
  - name: cmp.order.Placed
    fields:
      - {name: order_id, type: cmp.order.OrderId}
      - {name: note, type: String}
  - name: cmp.order.Ponged
    fields:
      - {name: at, type: Integer}
      - {name: order_id, type: cmp.order.OrderId}
  - name: cmp.order.Tagged
    fields:
      - {name: order_id, type: cmp.order.OrderId}
      - {name: labels, type: List<String>}
commands:
  - name: cmp.order.Place
    input:
      - {name: order_id, type: cmp.order.OrderId}
      - {name: note, type: String}
    outcomes:
      - name: placed
        emits: [cmp.order.Placed]
        payload:
          cmp.order.Placed: {order_id: input.order_id, note: input.note}
  - name: cmp.order.Tag
    input:
      - {name: order_id, type: cmp.order.OrderId}
      - {name: labels, type: List<String>}
    outcomes:
      - name: tagged
        emits: [cmp.order.Tagged]
        payload:
          cmp.order.Tagged: {order_id: input.order_id, labels: input.labels}
  - name: cmp.order.Ping
    input:
      - {name: order_id, type: cmp.order.OrderId}
      - {name: at, type: Integer}
    outcomes:
      - name: pinged
        emits: [cmp.order.Pinged]
        payload:
          cmp.order.Pinged: {order_id: input.order_id, at: input.at}
  - name: cmp.order.Cancel
    input:
      - {name: order_id, type: cmp.order.OrderId}
      - {name: reason, type: Integer}
    outcomes:
      - name: cancelled
        emits: [cmp.order.Cancelled]
        payload:
          cmp.order.Cancelled: {order_id: input.order_id, reason: input.reason}
  - name: cmp.order.Drop
    input:
      - {name: order_id, type: cmp.order.OrderId}
      - {name: by, type: Boolean}
    outcomes:
      - name: dropped
        emits: [cmp.order.Dropped]
        payload:
          cmp.order.Dropped: {order_id: input.order_id, by: input.by}
components:
  - component: svc
    owns: {domains: [cmp.order]}
";

/// `mem.order` is owned by nobody; `owner` owns `mem.events` and lists `Place`; `lister` lists
/// `Place` and publishes `Beta` and `Placed`. `Alpha` is byte-first, published by `owner` through
/// ownership and not by `lister`; `Beta` by `owner` through ownership and by `lister` by listing.
/// `mem.other` is owned by nobody and its `Orphan` is accepted by nobody.
const MEMBERSHIP: &str = r"format: ess/16
system: mem
version: v1
domain: mem.order
commands:
  - name: mem.order.Place
    input:
      - {name: order_id, type: String}
    outcomes:
      - name: placed
        emits: [mem.events.Placed]
        payload:
          mem.events.Placed: {order_id: input.order_id}
components:
  - component: owner
    owns: {domains: [mem.events]}
    accepts: {commands: [mem.order.Place]}
  - component: lister
    accepts: {commands: [mem.order.Place]}
    publishes: {events: [mem.events.Beta, mem.events.Placed]}
";

const MEMBERSHIP_EVENTS: &str = r"domain: mem.events
events:
  - name: mem.events.Alpha
    fields:
      - {name: order_id, type: String}
  - name: mem.events.Beta
    fields:
      - {name: order_id, type: String}
  - name: mem.events.Placed
    fields:
      - {name: order_id, type: String}
";

const MEMBERSHIP_OTHER: &str = r"domain: mem.other
events:
  - name: mem.other.Also
    fields:
      - {name: ref, type: String}
  - name: mem.other.Done
    fields:
      - {name: ref, type: String}
commands:
  - name: mem.other.Orphan
    input:
      - {name: ref, type: String}
    outcomes:
      - name: done
        emits: [mem.other.Done]
        payload:
          mem.other.Done: {ref: input.ref}
";

const PLACE_MEM: &str = "emit-swap/mem.order.Place/placed/mem.events.Placed/mem.events.Beta";
const ORPHAN: &str = "emit-swap/mem.other.Orphan/done/mem.other.Done";

fn spec(texts_in: &[(&str, &str)]) -> (Vec<Document>, SourceMap) {
    let mut texts = SourceMap::new();
    let mut parsed = Vec::new();
    for (label, text) in texts_in {
        let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{label}: {error}"));
        texts.insert((*label).to_owned(), (*text).to_owned());
        parsed.push((Source::new(*label), raw));
    }
    (parsed, texts)
}

fn compat() -> (Vec<Document>, SourceMap) {
    spec(&[("compat.yaml", COMPAT)])
}

fn membership() -> (Vec<Document>, SourceMap) {
    spec(&[
        ("membership.yaml", MEMBERSHIP),
        ("membership-events.yaml", MEMBERSHIP_EVENTS),
        ("membership-other.yaml", MEMBERSHIP_OTHER),
    ])
}

fn interpreted((files, texts): &(Vec<Document>, SourceMap)) -> impl Fn() -> Interpreted {
    let ir = mutate::compile(files.clone(), texts).expect("the fixture compiles");
    move || Interpreted::for_model(ir.clone())
}

fn swapped_to(selection: &mutate::Selection) -> BTreeMap<String, String> {
    selection
        .mutants
        .iter()
        .filter_map(|mutant| match &mutant.mutation {
            Mutation::EmitSwap { command, to, .. } => Some((command.clone(), to.clone())),
            _ => None,
        })
        .collect()
}

fn report_of(suite: &str, target: &impl ConformanceTarget) -> String {
    let admitted = AdmittedSuite::from_json(suite).expect("an emitted suite is admitted");
    let executed = Runner::for_suite(admitted.suite()).run_admitted(&admitted, target);
    CountReport::from_run(&executed, &admitted)
        .expect("a complete run")
        .to_canonical_json()
        .expect("serializes")
}

/// Every emitted suite answered by the model interpreter of `spec`.
fn answered(
    emission: &mutate::Emission,
    spec: &(Vec<Document>, SourceMap),
) -> BTreeMap<String, String> {
    let make = interpreted(spec);
    let mut written = emission.files.clone();
    let mut dirs = vec![BASELINE_DIR.to_owned()];
    dirs.extend(
        emission
            .manifest
            .mutants
            .iter()
            .filter_map(|mutant| mutant.dir.clone()),
    );
    for dir in dirs {
        let suite = &emission.files[&format!("{dir}/{SUITE_FILE}")];
        written.insert(format!("{dir}/{REPORT_FILE}"), report_of(suite, &make()));
    }
    written
}

// ---- candidate choice ---------------------------------------------------------------------------

/// Two admissible candidates: the design picks the first in byte order of name. `AByRef` (another
/// named type of the same shape) and `CExtra` (an extra field) come first and are not
/// alternatives; `DReordered` differs from `Placed` only in declaration order, which is not part
/// of field identity, and precedes `EAlso`.
#[test]
fn of_two_admissible_candidates_the_first_in_byte_order_is_the_swap() {
    let (files, texts) = compat();
    mutate::compile(files.clone(), &texts).expect("the compat fixture compiles");
    let selection = mutate::selection(&files, SWAP);
    let chosen = swapped_to(&selection);
    assert_eq!(
        chosen.get("cmp.order.Place").map(String::as_str),
        Some("cmp.order.DReordered"),
        "{selection:#?}"
    );
}

/// `List<String>` against `Optional<String>` and `Optional<List<String>>`: neither is the same
/// container structure, so the exact `BTaggedExact` is the swap although it sorts after both.
#[test]
fn list_against_optional_is_not_the_same_field_type() {
    let (files, _) = compat();
    let chosen = swapped_to(&mutate::selection(&files, SWAP));
    assert_eq!(
        chosen.get("cmp.order.Tag").map(String::as_str),
        Some("cmp.order.BTaggedExact")
    );
}

/// Before `ess/4` a payload may be left to inference, so an outcome may name no `payload:` map.
const IMPLICIT: &str = r"format: ess/3
system: imp
version: v1
domain: imp.order
events:
  - name: imp.order.Pinged
    fields:
      - {name: order_id, type: String}
  - name: imp.order.Ponged
    fields:
      - {name: order_id, type: String}
commands:
  - name: imp.order.Ping
    input:
      - {name: order_id, type: String}
    outcomes:
      - name: pinged
        emits: [imp.order.Pinged]
components:
  - component: svc
    owns: {domains: [imp.order]}
";

/// An outcome with no `payload:` map is still a site, its swap adds no payload entry, and the
/// healthy interpreter kills it.
#[test]
fn an_outcome_without_a_payload_map_swaps_and_gains_no_payload_entry() {
    let spec = spec(&[("implicit.yaml", IMPLICIT)]);
    let (files, texts) = &spec;
    mutate::compile(files.clone(), texts).expect("the implicit fixture compiles");
    let selection = mutate::selection(files, SWAP);
    let mutant = selection
        .mutants
        .iter()
        .find(|mutant| {
            mutant.id == "emit-swap/imp.order.Ping/pinged/imp.order.Pinged/imp.order.Ponged"
        })
        .unwrap_or_else(|| panic!("Ping swaps to Ponged: {selection:#?}"));
    let mutated = mutate::apply(files, &mutant.mutation).unwrap();
    let outcome = mutated
        .iter()
        .flat_map(|(_, file)| &file.commands)
        .find(|command| command.name.to_string() == "imp.order.Ping")
        .and_then(|command| command.outcomes.first())
        .unwrap();
    assert_eq!(outcome.payload.0.len(), 0, "no payload key was invented");
    let emitted: Vec<String> = outcome.emits.iter().map(ToString::to_string).collect();
    assert_eq!(emitted, ["imp.order.Ponged"]);
    let report = mutate::audit(files, texts, SWAP, interpreted(&spec)).expect("an audit");
    assert_eq!(report.counts.killed, 1, "{}", report.render_text());
}

#[test]
fn the_compat_sites_are_exactly_three_swaps_and_two_unavailable_sites() {
    let (files, _) = compat();
    let selection = mutate::selection(&files, SWAP);
    let ids: Vec<&str> = selection.mutants.iter().map(|it| it.id.as_str()).collect();
    assert_eq!(
        ids,
        [
            "emit-swap/cmp.order.Ping/pinged/cmp.order.Pinged/cmp.order.Ponged",
            "emit-swap/cmp.order.Place/placed/cmp.order.Placed/cmp.order.DReordered",
            "emit-swap/cmp.order.Tag/tagged/cmp.order.Tagged/cmp.order.BTaggedExact",
        ]
    );
    let unavailable: Vec<&str> = selection
        .unavailable
        .iter()
        .map(|it| it.id.as_str())
        .collect();
    assert_eq!(
        unavailable,
        [
            "emit-swap/cmp.order.Cancel/cancelled/cmp.order.Cancelled",
            "emit-swap/cmp.order.Drop/dropped/cmp.order.Dropped",
        ]
    );
}

/// Every compat swap is killed by the healthy interpreter of the unmutated model.
#[test]
fn a_healthy_target_kills_every_compat_swap() {
    let spec = compat();
    let report = mutate::audit(&spec.0, &spec.1, SWAP, interpreted(&spec)).expect("an audit");
    assert_eq!(report.counts.mutants, 3, "{}", report.render_text());
    assert_eq!(report.counts.killed, 3, "{}", report.render_text());
    assert_eq!(report.unaudited_sites(), 2);
}

/// The choice is a function of names, not of which document declares what.
#[test]
fn the_choice_does_not_depend_on_document_order() {
    let (files, _) = membership();
    let mut reversed = files.clone();
    reversed.reverse();
    assert_eq!(
        mutate::selection(&reversed, SWAP),
        mutate::selection(&files, SWAP)
    );
}

// ---- publication and acceptance -----------------------------------------------------------------

/// `owner` accepts `Place` by owning its domain, `lister` by listing it. `Alpha` is published by
/// `owner` (ownership) and not `lister`; `Beta` by both. `Orphan` is accepted by nobody.
#[test]
fn ownership_and_listing_both_count_and_a_command_nobody_accepts_has_no_alternative() {
    let (files, texts) = membership();
    mutate::compile(files.clone(), &texts).expect("the membership fixture compiles");
    let selection = mutate::selection(&files, SWAP);
    let ids: Vec<&str> = selection.mutants.iter().map(|it| it.id.as_str()).collect();
    assert_eq!(ids, [PLACE_MEM]);
    let unavailable: Vec<(&str, UnavailableReason)> = selection
        .unavailable
        .iter()
        .map(|it| (it.id.as_str(), it.reason))
        .collect();
    assert_eq!(
        unavailable,
        [(ORPHAN, UnavailableReason::NoCompatibleEventAlternative)]
    );
}

/// Scoped to `owner`, the orphan's site is another component's (`outside_component`) and does not
/// keep the audit from succeeding: `unaudited_sites` is what the CLI's exit status reads.
#[test]
fn a_scoped_audit_does_not_count_an_outside_component_site() {
    let spec = membership();
    let (files, texts) = &spec;
    let unscoped = mutate::audit(files, texts, SWAP, interpreted(&spec)).expect("an audit");
    assert_eq!(unscoped.unaudited_sites(), 1);
    for component in ["owner", "lister"] {
        let emission = mutate::emit_for(files, texts, SWAP, Some(component)).unwrap();
        let sites = emission.manifest.unavailable_sites.as_deref().unwrap();
        assert_eq!(sites.len(), 1, "{component}");
        assert_eq!(sites[0].id, ORPHAN);
        assert_eq!(sites[0].reason, UnavailableReason::OutsideComponent);
        let written = answered(&emission, &spec);
        let report: MutationReport =
            mutate::collect_for(|path| written.get(path).cloned(), Some(component))
                .unwrap_or_else(|refusal| panic!("{component}: {refusal}"));
        assert_eq!(report.counts.killed, 1, "{}", report.render_text());
        assert_eq!(report.counts.survived, 0);
        assert_eq!(report.counts.inconclusive, 0);
        assert_eq!(report.counts.unwitnessed, 0);
        assert_eq!(
            report.unaudited_sites(),
            0,
            "{component}: an outside_component site counted"
        );
        assert!(report
            .render_text()
            .contains(&format!("unavailable {ORPHAN}: outside_component")));
    }
}

// ---- manifest admission -------------------------------------------------------------------------

fn manifest_value() -> Value {
    let (files, texts) = compat();
    let emission = mutate::emit(&files, &texts, SWAP).unwrap();
    serde_json::from_str(&emission.files[MANIFEST_FILE]).unwrap()
}

fn refused(value: &Value) -> String {
    mutate::Manifest::from_json(&value.to_string()).expect_err("an incoherent unavailable list")
}

#[test]
fn the_manifest_reads_back_with_two_unavailable_sites() {
    let value = manifest_value();
    assert_eq!(value["unavailable_sites"].as_array().map(Vec::len), Some(2));
    mutate::Manifest::from_json(&value.to_string()).expect("the emitted manifest reads back");
}

#[test]
fn an_empty_unavailable_list_is_refused() {
    let mut value = manifest_value();
    value["unavailable_sites"] = json!([]);
    let message = refused(&value);
    assert!(message.contains("empty"), "{message}");
}

#[test]
fn unavailable_sites_out_of_byte_order_are_refused() {
    let mut value = manifest_value();
    value["unavailable_sites"].as_array_mut().unwrap().reverse();
    let message = refused(&value);
    assert!(message.contains("byte order"), "{message}");
}

#[test]
fn a_duplicated_unavailable_site_is_refused() {
    let mut value = manifest_value();
    let first = value["unavailable_sites"][0].clone();
    value["unavailable_sites"]
        .as_array_mut()
        .unwrap()
        .insert(0, first);
    let message = refused(&value);
    assert!(message.contains("once each"), "{message}");
}

#[test]
fn an_unavailable_site_that_is_also_a_mutants_site_is_refused() {
    let mut value = manifest_value();
    let command = "cmp.order.Place";
    let event = "cmp.order.Placed";
    let site = json!({
        "class": "emit-swap",
        "command": command,
        "event": event,
        "id": format!("emit-swap/{command}/placed/{event}"),
        "outcome": "placed",
        "reason": "no_compatible_event_alternative",
        "unaudited": format!(
            "single-event substitution was not audited here: no declared event other than \
             `{event}` has exactly its fields, is published by every component that accepts \
             `{command}`, and compiles in its place"
        ),
    });
    let sites = value["unavailable_sites"].as_array_mut().unwrap();
    sites.push(site);
    sites.sort_by(|left, right| left["id"].as_str().cmp(&right["id"].as_str()));
    let message = refused(&value);
    assert!(
        message.contains("both an unavailable site and a mutant's site"),
        "{message}"
    );
}

#[test]
fn an_unavailable_site_with_another_sentence_is_refused() {
    let mut value = manifest_value();
    value["unavailable_sites"][0]["unaudited"] = json!("audited after all");
    let message = refused(&value);
    assert!(
        message.contains("does not say what was not audited"),
        "{message}"
    );
}
