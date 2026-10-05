//! Two pages of a paged view, read beside its declared order (`paging:`, ess/16,
//! beyond10x/ess#174, `docs/design/view-paging.md`).
//!
//! A paged view is a ranked view, and the scenario that asserts its order has already arranged
//! [`RANKING_ROWS`](super::RANKING_ROWS) rows the caller's parameters admit — more than a page of
//! one row holds. So it reads the view twice more, each time with a page of one row: the first page,
//! which it snapshots, and the page after it. Each read requires exactly one row and, where the view
//! declares `total: true`, a total of at least the rows the scenario put there; the second requires
//! that it continues the first — ranked no earlier than the first page's row, and not the same row
//! again where the view projects the entity's identity.
//!
//! Every one of those claims holds on a target §8 permits to be shared. Another user's rows only add
//! to the total and push rows further back; they never shorten a page the scenario's own rows fill,
//! and they never put two pages of one order out of it. What is not claimed — which rows the pages
//! hold, and the exact total — is a claim about those other users.
//!
//! A read-your-writes view is read with [`QueryView`](ScenarioStep::QueryView) steps, and the
//! view's unpaged read is issued once more after the pages, so an expectation of the same view
//! appended after these reads is about every row the view holds, as it was before them.

use ess_domain::view::AssertionStyle;
use ess_primitives::facts::Number;
use ess_primitives::node::Node;
use ess_primitives::predicate::{Operand, Predicate};

use super::{
    bound, Arrangement, BTreeMap, BTreeSet, Determined, EssIr, ResolvedView, ScenarioStep,
    ScenarioValue, ViewExpectation, ViewRef,
};
use crate::view_paging::Follows;

/// The size of each page read: one row, so the two rows every ranked view is arranged with fill
/// two pages.
const PAGE_SIZE: u64 = 1;

/// The page reads of a paged view, read with the parameters `settled` binds: the first page and the
/// one after it, a page of one row each, and one page larger than every row this scenario put
/// there. Nothing where the view is not paged, holds no more of this scenario's rows than a page,
/// or is among the `unwitnessed` views whose order is not asserted.
///
/// The rows this scenario puts there are the subject's, where the filter admits it
/// (`admits_subject`), and every companion's the view shows.
pub(super) fn page_reads(
    ir: &EssIr,
    view: &ResolvedView,
    settled: &BTreeMap<String, Determined>,
    identity: Option<&ScenarioValue>,
    admits_subject: bool,
    companions: &[Arrangement],
    unwitnessed: &BTreeSet<ViewRef>,
) -> Vec<ScenarioStep> {
    let Some(paging) = &view.paging else {
        return Vec::new();
    };
    let name = &ViewRef::new(view.name.clone());
    let params = &bound(ir, view, settled, identity);
    let shown: Vec<&BTreeMap<String, Determined>> = admits_subject
        .then_some(settled)
        .into_iter()
        .chain(
            companions
                .iter()
                .filter(|row| row.shows(ir, view, params) == Ok(true))
                .map(|row| &row.settled),
        )
        .collect();
    let held = u64::try_from(shown.len()).unwrap_or(u64::MAX);
    if unwitnessed.contains(name) || view.order_by.is_empty() || held <= PAGE_SIZE {
        return Vec::new();
    }
    let total_at_least = paging.total.then_some(held);
    let follows = Follows {
        order_by: view.order_by.clone(),
        distinct_by: distinct_by(ir, view, &shown),
    };
    let pair = u64::try_from(PAIR).unwrap_or(u64::MAX);
    let mut reads = vec![
        // The first page, snapshotted.
        (0, PAGE_SIZE, 1, false, None, true),
        // The page after it, which continues it.
        (1, PAGE_SIZE, 1, false, Some(follows.clone()), false),
    ];
    if held >= 2 * pair {
        // Two pages of two rows: the second starts at `2`, where an implementation that reads the
        // page number as a row offset starts at `1`, inside the first.
        reads.push((0, pair, PAIR, false, None, true));
        reads.push((1, pair, PAIR, false, Some(follows), false));
    }
    reads.push(
        // A page larger than every row this scenario made: at least those rows, and — on a target
        // nobody else writes to — a partial last page, which a target that answers no partial last
        // page answers empty.
        (0, held + 1, shown.len(), true, None, false),
    );
    let mut steps = Vec::new();
    for (offset, size, rows, at_least, follows, snapshot) in reads {
        let page = paging.first_page + offset;
        let mut sent = params.clone();
        sent.insert(paging.page.clone(), whole(page));
        sent.insert(paging.size.clone(), whole(size));
        let expectation = ViewExpectation::Page {
            page,
            size,
            rows,
            at_least,
            total_at_least,
            follows,
        };
        match view.assertion_style {
            AssertionStyle::Expect => {
                steps.push(ScenarioStep::QueryView {
                    view: name.clone(),
                    params: sent,
                });
                steps.push(ScenarioStep::ExpectView {
                    view: name.clone(),
                    expectation,
                });
            }
            AssertionStyle::Eventually => steps.push(ScenarioStep::EventuallyView {
                view: name.clone(),
                params: sent,
                expectation,
            }),
        }
        if snapshot {
            steps.push(ScenarioStep::SnapshotView { view: name.clone() });
        }
    }
    if view.assertion_style == AssertionStyle::Expect {
        steps.push(ScenarioStep::QueryView {
            view: name.clone(),
            params: params.clone(),
        });
    }
    steps
}

/// The fields that tell this scenario's rows apart in the view, so the page after the first cannot
/// answer the first page's row again unnoticed.
///
/// The entity's identity, where the view projects it. Otherwise the projected fields of which the
/// scenario knows a literal value for every one of its rows — where no two of its rows agree on all
/// of them. Otherwise nothing: rows the view shows alike cannot be told
/// apart by any reader, and the continuation is then asserted by its order alone
/// (`docs/design/view-paging.md`, "Stated limits").
fn distinct_by(
    ir: &EssIr,
    view: &ResolvedView,
    shown: &[&BTreeMap<String, Determined>],
) -> Vec<String> {
    let identity = &ir.entity(&view.source).identity.name;
    if view.field(identity).is_some() {
        return vec![identity.clone()];
    }
    let literal = |row: &BTreeMap<String, Determined>, field: &str| {
        row.get(field)
            .and_then(|held| held.value.as_literal().cloned())
    };
    let fields: Vec<String> = view
        .fields
        .iter()
        .map(|field| field.name.clone())
        .filter(|field| shown.iter().all(|row| literal(row, field).is_some()))
        .collect();
    if fields.is_empty() {
        return Vec::new();
    }
    let tuples: Vec<Vec<Option<Node>>> = shown
        .iter()
        .map(|row| fields.iter().map(|field| literal(row, field)).collect())
        .collect();
    let apart = tuples
        .iter()
        .enumerate()
        .all(|(index, tuple)| tuples[index + 1..].iter().all(|other| other != tuple));
    if apart {
        fields
    } else {
        Vec::new()
    }
}

/// A page number or size as the literal a paging parameter is sent.
fn whole(value: u64) -> ScenarioValue {
    let value = usize::try_from(value).unwrap_or(usize::MAX);
    ScenarioValue::literal(Node::Number(Number::from(value)))
}

/// `view`, with every comparison against a parameter this scenario bound to a literal made against
/// that literal — or `None` where the filter compares against no such parameter.
///
/// The search for a further row of a ranked view ([`arrange_toward`](super::arrange_toward)) maps
/// the filter onto the creating command's input and asks for inputs that satisfy it. A filter
/// `type == param.type` then reads a path no input carries, and no row was ever found: a ranked view
/// filtered by its caller's parameter was refused `ESS-SYNTH-014`, and so was every paged view
/// written the way #174 writes one. Bound, it reads `type == "<the subject's type>"`, which the
/// input can satisfy; the row found is still checked against the view as the caller reads it.
pub(super) fn with_bound_params(
    view: &ResolvedView,
    params: &BTreeMap<String, ScenarioValue>,
) -> Option<ResolvedView> {
    let filter = view.filter.as_ref()?;
    let bound = bind(filter, params);
    (bound != *filter).then(|| ResolvedView {
        filter: Some(bound),
        ..view.clone()
    })
}

fn bind(predicate: &Predicate, params: &BTreeMap<String, ScenarioValue>) -> Predicate {
    match predicate {
        Predicate::All(children) => {
            Predicate::All(children.iter().map(|child| bind(child, params)).collect())
        }
        Predicate::Any(children) => {
            Predicate::Any(children.iter().map(|child| bind(child, params)).collect())
        }
        Predicate::Not(inner) => Predicate::Not(Box::new(bind(inner, params))),
        Predicate::Compare {
            left,
            op,
            right,
            kind,
        } => Predicate::Compare {
            kind: *kind,
            left: operand(left, params),
            op: *op,
            right: operand(right, params),
        },
        other => other.clone(),
    }
}

fn operand(it: &Operand, params: &BTreeMap<String, ScenarioValue>) -> Operand {
    if let Operand::Fact(path) = it {
        if let [namespace, name] = path.segments() {
            if namespace == ess_domain::view::ViewSpec::PARAM {
                if let Some(value) = params
                    .get(name)
                    .and_then(ScenarioValue::as_literal)
                    .and_then(super::fact_value)
                {
                    return Operand::Literal(value);
                }
            }
        }
    }
    it.clone()
}

/// Whether `param` is one of the view's paging parameters, which only a page read sends: an input
/// or a field of the same name never binds it on the view's other reads.
pub(super) fn reads(view: &ResolvedView, param: &str) -> bool {
    view.paging
        .as_ref()
        .is_some_and(|paging| paging.reads(param))
}

/// Whether a read that sends no parameter reads every row of `view`: it declares none, or only the
/// page and the size, which `paging:` declares a read may omit and then answers every row.
///
/// Every witness that reads a view whole — a deletion, a preserved row, a one-row snapshot —
/// selects its views by this rather than by `params` being empty, so declaring `paging:` on an
/// otherwise unparameterised view costs it none of them.
pub(crate) fn read_whole(view: &ResolvedView) -> bool {
    view.params.iter().all(|param| reads(view, &param.name))
}

/// How many of this scenario's rows a view is arranged with before it is read: two for an order,
/// and four for a paged view, which fill two pages of two rows — the one read past the first page
/// whose size is above one, and so the one that tells `page * size` from `page` as the offset.
pub(super) fn rows_wanted(view: &ResolvedView) -> usize {
    if view.paging.is_some() {
        2 * PAIR
    } else {
        super::RANKING_ROWS
    }
}

/// The size of the pages that tell `page * size` from `page` apart.
const PAIR: usize = 2;
