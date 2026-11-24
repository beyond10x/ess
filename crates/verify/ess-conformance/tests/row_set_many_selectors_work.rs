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

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::num::NonZeroUsize;

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::synthesize::{
    row_set_arrangements_tried, row_set_preparations, row_set_preparations_unkept,
    row_set_selectors_arranged, synthesize, with_jobs, Preparations, Synthesis,
};
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

/// `Approve`, refused by a missing order and by `k` row-set guards over the order's lines, one per
/// flag (`line-f1` .. `line-f<k>`), accepting with `approved`: `k` selectors of one entity, every
/// combination of whose counts the search may try.
fn approvals(k: usize) -> String {
    let flags = |line: &dyn Fn(usize) -> String| (1..=k).map(line).collect::<String>();
    format!(
        "format: ess/37
system: shop
version: v1
domain: shop.orders
summary: Orders, their lines, and the approval that reads every line.
types:
  - {{name: shop.orders.OrderId, kind: newtype, of: Uuid}}
  - {{name: shop.orders.LineId, kind: newtype, of: Uuid}}
  - {{name: shop.orders.ApprovalId, kind: newtype, of: Uuid}}
errors:
  - {{name: shop.orders.Refused, summary: Refused., fields: []}}
events:
  - name: shop.orders.Placed
    fields: [{{name: order_id, type: shop.orders.OrderId}}]
  - name: shop.orders.LineAdded
    fields: [{{name: line_id, type: shop.orders.LineId}}]
  - name: shop.orders.Approved
    fields: [{{name: approval_id, type: shop.orders.ApprovalId}}]
entities:
  - name: shop.orders.Order
    identity: {{name: order_id, type: shop.orders.OrderId}}
    fields:
      - {{name: customer, type: String}}
    lifecycle: {{initial: Open, states: [Open], terminal: [Open]}}
  - name: shop.orders.Approval
    identity: {{name: approval_id, type: shop.orders.ApprovalId}}
    fields:
      - {{name: order_id, type: shop.orders.OrderId}}
    lifecycle: {{initial: Done, states: [Done], terminal: [Done]}}
  - name: shop.orders.Line
    identity: {{name: line_id, type: shop.orders.LineId}}
    fields:
      - {{name: order_id, type: shop.orders.OrderId}}
{fields}    relations:
      - {{name: order, kind: references, target: shop.orders.Order, cardinality: one, via: order_id}}
    lifecycle: {{initial: Present, states: [Present], terminal: [Present]}}
commands:
  - name: shop.orders.Place
    input:
      - {{name: order_id, type: shop.orders.OrderId}}
      - {{name: customer, type: String}}
    outcomes:
      - name: placed
        creates: shop.orders.Order
        instance: order_id
        sets: {{customer: input.customer}}
        emits: [shop.orders.Placed]
        payload: {{shop.orders.Placed: {{order_id: input.order_id}}}}
  - name: shop.orders.AddLine
    input:
      - {{name: line_id, type: shop.orders.LineId}}
      - {{name: order_id, type: shop.orders.OrderId}}
{fields}    outcomes:
      - name: no-order
        when_related: {{via: input.order_id, exists: false}}
        error: shop.orders.Refused
      - name: added
        creates: shop.orders.Line
        instance: line_id
        sets:
          order_id: input.order_id
{sets}        emits: [shop.orders.LineAdded]
        payload: {{shop.orders.LineAdded: {{line_id: input.line_id}}}}
  - name: shop.orders.Approve
    input:
      - {{name: approval_id, type: shop.orders.ApprovalId}}
      - {{name: order_id, type: shop.orders.OrderId}}
    outcomes:
      - name: no-order
        when_related: {{via: input.order_id, exists: false}}
        error: shop.orders.Refused
{refusals}      - name: approved
        creates: shop.orders.Approval
        instance: approval_id
        sets: {{order_id: input.order_id}}
        emits: [shop.orders.Approved]
        payload: {{shop.orders.Approved: {{approval_id: input.approval_id}}}}
views:
  - name: shop.orders.Orders
    source: shop.orders.Order
    consistency: read_your_writes
    fields:
      - {{name: order_id, type: shop.orders.OrderId}}
      - {{name: state, type: shop.orders.Order.State}}
",
        fields = flags(&|i| format!("      - {{name: f{i}, type: Boolean}}\n")),
        sets = flags(&|i| format!("          f{i}: input.f{i}\n")),
        refusals = flags(&|i| format!(
            "      - name: line-f{i}\n        when_related:\n          entity: shop.orders.Line\n          \
             where: {{all: [order_id == input.order_id, f{i} == true]}}\n          exists: true\n        \
             error: shop.orders.Refused\n"
        )),
    )
}

/// The most selectors the synthesis of [`approvals`]`(3)` may arrange rather than take from the
/// attempt before; 873 when every attempt arranged every selector again, 404 when each prefix of
/// counts is arranged once.
const SELECTORS_BOUND: u64 = 600;

/// The arrangements the synthesis of [`approvals`]`(3)` tries: reusing what an attempt arranged
/// changes none of them.
const APPROVALS_TRIED: u64 = 308;

#[test]
fn a_command_refusing_on_several_row_sets_arranges_each_prefix_once() {
    let ir = compiled(&approvals(3));
    let ((synthesis, tried), arranged) =
        row_set_selectors_arranged(|| row_set_arrangements_tried(|| synthesize(&ir)));
    let refusals: Vec<String> = synthesis.refusals.iter().map(ToString::to_string).collect();
    let scenarios: Vec<String> = synthesis
        .suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .filter(|id| id.starts_with("shop.orders.Approve/"))
        .collect();
    assert!(
        arranged <= SELECTORS_BOUND,
        "{arranged} selectors arranged, more than {SELECTORS_BOUND}; {tried} arrangements tried\n\
         scenarios: {scenarios:#?}\nrefusals: {refusals:#?}"
    );
    assert_eq!(tried, APPROVALS_TRIED, "the same arrangements are tried");
    assert_eq!(
        scenarios,
        [
            "shop.orders.Approve/outcome/approved",
            "shop.orders.Approve/outcome/line-f1",
            "shop.orders.Approve/outcome/line-f2",
            "shop.orders.Approve/outcome/no-order",
        ],
        "the scenarios of `Approve`\nrefusals: {refusals:#?}"
    );
}

/// The branches of each model below are prepared more than once from one context: before, every
/// request searched every arrangement afresh (8 made for 8 asked under 7 keys, and 3 for 3 under
/// 2). A later request reads the first's preparation. Where a branch is made under two keys, the
/// message names the parts of its keys that differ.
#[test]
fn a_row_set_branch_asked_again_in_the_same_context_is_prepared_once() {
    for name in [
        "filtered-related-reads.yaml",
        "row-set-refusal-beside-the-addressed-related-row.yaml",
    ] {
        let ir = compiled(&fixture(name));
        let (_, preparations) = with_jobs(NonZeroUsize::MIN, || {
            row_set_preparations(|| synthesize(&ir))
        });
        let Preparations { asked, made, keys } = &preparations;
        let mut branches: BTreeMap<String, Vec<&String>> = BTreeMap::new();
        for key in keys {
            let branch = key.splitn(3, '|').take(2).collect::<Vec<_>>().join("|");
            branches.entry(branch).or_default().push(key);
        }
        let differing: Vec<String> = branches
            .iter()
            .filter(|(_, keys)| keys.len() > 1)
            .map(|(branch, keys)| {
                let parts: Vec<Vec<&str>> =
                    keys.iter().map(|key| key.split('|').collect()).collect();
                let apart: Vec<String> = (0..parts[0].len())
                    .filter(|at| {
                        parts
                            .iter()
                            .any(|other| other.get(*at) != parts[0].get(*at))
                    })
                    .map(|at| {
                        parts
                            .iter()
                            .map(|key| key.get(at).copied().unwrap_or("-"))
                            .collect::<Vec<_>>()
                            .join(" / ")
                    })
                    .collect();
                format!("{branch}: {apart:?}")
            })
            .collect();
        assert!(
            made < asked,
            "{name}: {made} preparations made for {asked} asked, under {} keys: none answered \
             from memory\nbranches made under several keys: {differing:#?}",
            keys.len()
        );
        assert_eq!(
            *made,
            keys.len() as u64,
            "{name}: a preparation made twice under one key\nbranches made under several keys: \
             {differing:#?}"
        );
    }
}

/// The suite and the refusals of `text`, synthesized at one job with the preparations kept and
/// with each made afresh.
fn kept_and_fresh(text: &str) -> (String, String) {
    let ir = compiled(text);
    let shown = |synthesis: &Synthesis| {
        format!(
            "{}\n{:?}",
            serde_json::to_string(&synthesis.suite).expect("the suite serializes"),
            synthesis.refusals
        )
    };
    with_jobs(NonZeroUsize::MIN, || {
        let kept = synthesize(&ir);
        let fresh = row_set_preparations_unkept(|| synthesize(&ir));
        (shown(&kept), shown(&fresh))
    })
}

/// A model the row-set targets read, under `tests/fixtures/`.
fn fixture(name: &str) -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

/// One test per model: its suite is the same bytes with the preparations kept and made afresh.
macro_rules! kept_preparations_write_the_fresh_bytes {
    ($($test:ident: $text:expr;)*) => {
        $(
            #[test]
            fn $test() {
                let (kept, fresh) = kept_and_fresh(&$text);
                assert!(kept == fresh, "the suite differs with the preparations kept");
            }
        )*
    };
}

kept_preparations_write_the_fresh_bytes! {
    kept_preparations_approvals_2: approvals(2);
    kept_preparations_approvals_3: approvals(3);
    kept_preparations_approvals_4: approvals(4);
    kept_preparations_filtered_related_reads: fixture("filtered-related-reads.yaml");
    kept_preparations_identity_beside_row_set: fixture("../../../../specify/ess-domain/tests/fixtures/identity-beside-row-set.yaml");
    kept_preparations_nested_ordered_heads: fixture("nested-ordered-heads.yaml");
    kept_preparations_now_stored_rows: fixture("now-stored-rows.yaml");
    kept_preparations_row_set_creator_predicate_guard: fixture("row-set-creator-predicate-guard.yaml");
    kept_preparations_row_set_refusal_beside_a_named_related_row:
        fixture("row-set-refusal-beside-a-named-related-row.yaml");
    kept_preparations_row_set_refusal_beside_the_addressed_related_row:
        fixture("row-set-refusal-beside-the-addressed-related-row.yaml");
    kept_preparations_row_set_upsert: fixture("row-set-upsert.yaml");
    kept_preparations_two_related_selectors: fixture("two-related-selectors.yaml");
    kept_preparations_two_sided_pick: fixture("two-sided-pick.yaml");
    kept_preparations_unique_within_scope: fixture("unique-within-scope.yaml");
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
