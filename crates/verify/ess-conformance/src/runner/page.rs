//! Requiring one page of a paged view (suite/26, [`crate::view_paging`],
//! `docs/design/view-paging.md`).
//!
//! Three claims, each one a shared target cannot break with rows of its own: the page's exact
//! length (the scenario's rows fill it), a floor on the total (another user's rows only add to it),
//! and — for the page after the first — that it continues the page before it, which the run
//! snapshotted: in the declared order, ranked no earlier than that page's last row, and holding
//! none of its rows again. Two pages of one order stay in that order whatever else it holds.

use std::fmt::Write as _;

use super::{quote_row, ranked, Run, Verdict};
use crate::scenario::{ViewExpectation, ViewRef};
use crate::target::{SemanticViewResult, ViewRow};
use crate::view_paging::Follows;

/// A [`ViewExpectation::Page`], with the snapshot a continuation is read against.
pub(super) struct PageRequired {
    page: u64,
    size: u64,
    rows: usize,
    /// `rows` is a floor, and `size` the ceiling.
    at_least: bool,
    total_at_least: Option<u64>,
    /// How the page continues the one before it, and every row the snapshot of that page held.
    follows: Option<(Follows, Vec<ViewRow>)>,
}

impl PageRequired {
    /// The page `expectation` requires of `view`, or why it is no claim this run can check: a page
    /// that is no claim at all, or a continuation with no snapshot of the view before it.
    pub(super) fn of(
        view: &ViewRef,
        expectation: &ViewExpectation,
        run: &Run,
    ) -> Result<Self, String> {
        let ViewExpectation::Page {
            page,
            size,
            rows,
            at_least,
            total_at_least,
            follows,
        } = expectation
        else {
            return Err("not a page".to_owned());
        };
        if let Some(reason) = crate::view_paging::defect(
            *size,
            *rows,
            *total_at_least,
            follows
                .as_ref()
                .is_some_and(|follows| follows.order_by.is_empty()),
        ) {
            return Err(reason);
        }
        let follows = match follows {
            None => None,
            Some(follows) => Some((
                follows.clone(),
                run.view_snapshots.get(view).cloned().ok_or_else(|| {
                    "no view snapshot of the page before preceded this continuation".to_owned()
                })?,
            )),
        };
        Ok(Self {
            page: *page,
            size: *size,
            rows: *rows,
            at_least: *at_least,
            total_at_least: *total_at_least,
            follows,
        })
    }

    /// What the page must hold, as a report says it.
    pub(super) fn wanted(&self, view: &ViewRef) -> String {
        let mut wanted = format!(
            "page {} of {view}, of size {}, holds {}{} row(s)",
            self.page,
            self.size,
            if self.at_least { "at least " } else { "" },
            self.rows
        );
        if let Some(total) = self.total_at_least {
            let _ = write!(wanted, " and a total of at least {total}");
        }
        if let Some((follows, _)) = &self.follows {
            let keys = follows
                .order_by
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", then ");
            let _ = write!(wanted, ", continuing the page before it ordered by {keys}");
        }
        wanted
    }
}

/// Whether `result` is the page `required` names.
pub(super) fn page_of(required: &PageRequired, result: &SemanticViewResult) -> Verdict {
    let mut wrong = Vec::new();
    let held = result.rows.len();
    let ceiling = usize::try_from(required.size).unwrap_or(usize::MAX);
    if required.at_least && (held < required.rows || held > ceiling) {
        wrong.push(format!(
            "the page holds {held} row(s), and a page of size {} holds at least {} here",
            required.size, required.rows
        ));
    } else if !required.at_least && held != required.rows {
        wrong.push(format!(
            "the page holds {} row(s), and a page of size {} holds {} here",
            result.rows.len(),
            required.size,
            required.rows
        ));
    }
    if let Some(floor) = required.total_at_least {
        let held = u64::try_from(result.rows.len()).unwrap_or(u64::MAX);
        match result.total {
            None => wrong.push(
                "the answer carries no total, and the view declares `total: true`".to_owned(),
            ),
            Some(total) if total < floor => wrong.push(format!(
                "the total is {total}, fewer than the {floor} rows this scenario put in the view"
            )),
            Some(total) if total < held => wrong.push(format!(
                "the total is {total}, fewer than the {held} rows the page itself holds"
            )),
            Some(_) => {}
        }
    }
    if let Some((follows, before)) = &required.follows {
        // The snapshot's last row, then this page's rows: one run of the declared order.
        let run = SemanticViewResult::of(
            before
                .last()
                .cloned()
                .into_iter()
                .chain(result.rows.clone()),
        );
        match ranked(&follows.order_by, &run) {
            Verdict::Satisfied => {}
            Verdict::Unsatisfied(reason) => wrong.push(format!(
                "the page does not continue the one before it in the declared order: {reason}"
            )),
            undecidable @ Verdict::Undecidable { .. } => return undecidable,
        }
        if !follows.distinct_by.is_empty() {
            for row in &result.rows {
                let again = before.iter().any(|earlier| {
                    follows.distinct_by.iter().all(|field| {
                        earlier.get(field).is_some() && earlier.get(field) == row.get(field)
                    })
                });
                if again {
                    wrong.push(format!(
                        "{} was on the page before it as well",
                        quote_row(row)
                    ));
                }
            }
        }
    }
    if wrong.is_empty() {
        Verdict::Satisfied
    } else {
        Verdict::Unsatisfied(wrong.join("; "))
    }
}
