//! The work the row-set search does for a command reading many selectors
//! (<https://github.com/beyond10x/ess/issues/624>).
//!
//! Since 0.63.0 a command reading several selectors is arranged over every combination of their
//! row counts, four per selector (beyond10x/ess#543). `Promote` below reads twelve: two over the
//! releases its own commands create, and ten over a checkout that is only observed, which no
//! command creates. Tried one by one, each of its row-set branches is 4^12 = 16,777,216
//! combinations, and a downstream specification of this shape took hours to synthesize. Only
//! attempts that cannot stand may be passed over: rows a selector cannot arrange fail every
//! attempt agreeing with it up to that selector, and a branch reading or needing a row no command
//! creates fails every attempt. So the work is bounded by the count of arrangements tried, and the
//! branches are what the full search would make of them.

use std::fmt::Write as _;

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::synthesize::{row_set_arrangements_tried, synthesize, Synthesis};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

/// The flags of the observed checkout `Promote` refuses on, one selector each, beside the
/// selector its label is read through.
const FLAGS: [&str; 9] = [
    "dirty", "detached", "unsigned", "shallow", "locked", "stale", "orphaned", "frozen", "patched",
];

/// What 0.63.0 to 0.66.0 tried for one branch of `Promote`: every combination of four counts for
/// each of its twelve selectors.
const UNPRUNED_PER_BRANCH: u64 = 16_777_216;

/// The most arrangements the whole synthesis of the model may try; 562 when the search was
/// bounded.
const BOUND: u64 = 1_000;

/// The model, with `FLAGS` written in at `FIELDS` (a checkout field each) and `OUTCOMES` (a
/// refusal of `Promote` each, guarded by a selector of its own).
const MODEL: &str = "format: ess/22
system: demo
version: v1
domain: demo.site
summary: Promotes a site's release, reading an observed checkout of the site through many selectors.
types:
  - {name: demo.site.CheckoutId, kind: newtype, of: Uuid}
  - {name: demo.site.ReleaseId, kind: newtype, of: Uuid}
  - {name: demo.site.PromotionId, kind: newtype, of: Uuid}
entities:
  - name: demo.site.Checkout
    identity: {name: checkout_id, type: demo.site.CheckoutId}
    fields:
      - {name: site, type: String}
      - {name: label, type: String}
FIELDS    lifecycle: {initial: Present, states: [Present], terminal: [Present], transitions: []}
  - name: demo.site.Release
    identity: {name: release_id, type: demo.site.ReleaseId}
    fields:
      - {name: site, type: String}
      - {name: live, type: Boolean}
    lifecycle: {initial: Present, states: [Present], terminal: [Present], transitions: []}
  - name: demo.site.Promotion
    identity: {name: promotion_id, type: demo.site.PromotionId}
    fields:
      - {name: site, type: String}
      - {name: label, type: String}
    lifecycle: {initial: Present, states: [Present], terminal: [Present], transitions: []}
errors:
  - {name: demo.site.Refused, summary: The site cannot be promoted., fields: []}
events:
  - name: demo.site.Published
    fields:
      - {name: release_id, type: demo.site.ReleaseId}
  - name: demo.site.Promoted
    fields:
      - {name: promotion_id, type: demo.site.PromotionId}
      - {name: label, type: String}
commands:
  - name: demo.site.Publish
    input:
      - {name: release_id, type: demo.site.ReleaseId}
      - {name: site, type: String}
      - {name: live, type: Boolean}
    outcomes:
      - name: published
        creates: demo.site.Release
        instance: release_id
        sets:
          site: input.site
          live: input.live
        emits: [demo.site.Published]
        payload:
          demo.site.Published: {release_id: input.release_id}
  - name: demo.site.Promote
    input:
      - {name: promotion_id, type: demo.site.PromotionId}
      - {name: site, type: String}
    outcomes:
      - name: unreleased
        when_related:
          entity: demo.site.Release
          where: site == input.site
          exists: false
        error: demo.site.Refused
      - name: already-live
        when_related:
          entity: demo.site.Release
          where: {all: [site == input.site, live == true]}
          exists: true
        error: demo.site.Refused
OUTCOMES      - name: promoted
        creates: demo.site.Promotion
        instance: promotion_id
        sets:
          site: input.site
          label:
            related:
              entity: demo.site.Checkout
              where: site == input.site
              field: label
        emits: [demo.site.Promoted]
        payload:
          demo.site.Promoted:
            promotion_id: input.promotion_id
            label:
              related:
                entity: demo.site.Checkout
                where: site == input.site
                field: label
";

fn model() -> String {
    let mut fields = String::new();
    let mut outcomes = String::new();
    for flag in FLAGS {
        writeln!(fields, "      - {{name: {flag}, type: Boolean}}").expect("written");
        write!(
            outcomes,
            "      - name: {flag}\n        when_related:\n          entity: demo.site.Checkout\n          \
             where: {{all: [site == input.site, {flag} == true]}}\n          exists: true\n        \
             error: demo.site.Refused\n"
        )
        .expect("written");
    }
    MODEL
        .replacen("FIELDS", &fields, 1)
        .replacen("OUTCOMES", &outcomes, 1)
}

fn compiled(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let specification = Specification::assemble([(Source::new("site.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model validates:\n{errors}\n{text}"));
    compile(&specification, &SourceMap::new())
        .unwrap_or_else(|diagnostics| panic!("the model resolves:\n{diagnostics}"))
}

/// The refusal of `Promote`'s branch `branch`, as it prints.
fn refusal_of(synthesis: &Synthesis, branch: &str) -> Option<String> {
    let named = format!("outcome demo.site.Promote/{branch} has no scenario");
    synthesis
        .refusals
        .iter()
        .map(ToString::to_string)
        .find(|refusal| refusal.contains(&named))
}

#[test]
fn a_command_reading_twelve_selectors_is_synthesized_within_a_bound_on_the_arrangements_tried() {
    let ir = compiled(&model());
    let (synthesis, tried) = row_set_arrangements_tried(|| synthesize(&ir));
    let refusals: Vec<String> = synthesis.refusals.iter().map(ToString::to_string).collect();
    let scenarios: Vec<String> = synthesis
        .suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .collect();
    assert!(
        tried <= BOUND,
        "{tried} arrangements tried, more than {BOUND}; one branch unpruned tries \
         {UNPRUNED_PER_BRANCH}\nscenarios: {scenarios:#?}\nrefusals: {refusals:#?}"
    );
    // The branches the releases decide stand on rows the search arranges.
    for branch in ["unreleased", "already-live"] {
        let id = format!("demo.site.Promote/outcome/{branch}");
        assert!(
            scenarios.contains(&id),
            "{id} is synthesized: {scenarios:#?}\nrefusals: {refusals:#?}"
        );
    }
    // Every branch reading or needing a checkout, which no command creates, ends where the full
    // search ends: refused by the checkout field no command writes, not cut short.
    for branch in FLAGS.iter().chain(&["promoted"]) {
        let refusal = refusal_of(&synthesis, branch)
            .unwrap_or_else(|| panic!("`{branch}` is refused: {refusals:#?}"));
        assert!(
            refusal.contains(
                "`demo.site.Checkout.site` is `String`, which a filter reads and no \
                 declared command writes"
            ),
            "`{branch}` is refused by the unwritten field: {refusal}"
        );
    }
    assert!(
        refusals
            .iter()
            .all(|refusal| !refusal.contains("than synthesis tries for one branch")),
        "no search is cut: {refusals:#?}"
    );
}

#[test]
fn the_bounded_search_is_deterministic() {
    let ir = compiled(&model());
    let (first, tried) = row_set_arrangements_tried(|| synthesize(&ir));
    let (second, again) = row_set_arrangements_tried(|| synthesize(&ir));
    assert_eq!(tried, again, "the same arrangements are tried each time");
    assert_eq!(
        serde_json::to_string(&first.suite).expect("the suite serializes"),
        serde_json::to_string(&second.suite).expect("the suite serializes"),
    );
    assert_eq!(
        format!("{:?}", first.refusals),
        format!("{:?}", second.refusals)
    );
}
