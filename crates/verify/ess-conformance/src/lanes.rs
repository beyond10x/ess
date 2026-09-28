//! A checked `ess-history/1` document, drawn as one lane per client.
//!
//! Every client that made a call has a lane. So does every declared client that made none, up to
//! 16 declared clients; above that the idle ones are one row that counts them.
//!
//! `ess verify conform web --history` renders what [`crate::linearize`] concluded about a history as
//! one self-contained HTML page: every operation's invoke–return interval on its client's lane, the
//! linearization points the search found, and for a command violation the operation where the
//! search failed and the operation it conflicts with, each with the state of the subject its
//! recorded answer needed and the state the order supplied ([`linearize::conflict`]). A violation's
//! shrunk history is drawn below it, the same way, where shrinking removed anything.
//!
//! # What the page is made of
//!
//! One HTML document with an inline stylesheet and inline SVG. It carries no script and names no
//! other resource, so it opens from disk and fetches nothing.
//!
//! # Determinism
//!
//! The page is a function of the model, the history and the budget, byte for byte: the check, the
//! search and the shrink are ([`crate::linearize`]), and the page adds no clock, no random identity
//! and no unordered collection. Instants are drawn by rank, so only their order shows, as only
//! their order counts.
//!
//! # Where a linearization point is drawn
//!
//! A history records when each call was invoked and returned, not when it took effect. The point is
//! drawn inside the operation's interval, after the point of the operation ordered before it on the
//! same subject — the earliest place the found order allows, not an observed instant. Calls that
//! share an instant share its band, and the band widens by one slot per point placed in it, so one
//! subject's points are always at strictly increasing x.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use ess_compiler::ir::EssIr;

use crate::history::{History, Operation, ReturnBound, Verdict};
use crate::linearize::{self, CheckRefusal, Checked, Conflict, ConflictSide};

/// Horizontal space before the first instant, for the lane label.
const LEFT: u64 = 96;
/// Horizontal space between two instant ranks' bands.
const COLUMN: u64 = 56;
/// Vertical space per lane.
const LANE: u64 = 48;
/// Vertical space above the first lane, for the axis.
const TOP: u64 = 28;
/// Height of an operation's interval bar.
const BAR: u64 = 22;
/// Width of one linearization-point slot in an instant's band.
const SLOT: u64 = 14;
/// Above this many clients, the clients that made no call are summarised in one row rather than
/// drawn as a lane each: `clients` may be as large as `ess-history/1` admits (2^53 - 1), and a
/// lane per idle client would be a page nobody could open.
const IDLE_LANES: u64 = 16;

const STYLE: &str = "\
:root{--bg:#f7f8fa;--panel:#fff;--edge:#dfe3ea;--ink:#1b1f27;--dim:#6b7484;--op:#dbe6f7;\
--opline:#4a6fa5;--fail:#c8372d;--conflict:#d28a12;--point:#1c8a5f}
*{box-sizing:border-box}
body{margin:0;background:var(--bg);color:var(--ink);font:14px/1.5 system-ui,sans-serif}
header,section{padding:14px 20px}
header{border-bottom:1px solid var(--edge)}
h1{font-size:16px;margin:0}
h2{font-size:14px;margin:0 0 8px}
code,.mono{font-family:ui-monospace,Menlo,monospace;font-size:12px}
.meta{color:var(--dim);font-size:12px}
section{background:var(--panel);border-bottom:1px solid var(--edge)}
.verdict-Linearizable{color:var(--point)}
.verdict-Violation{color:var(--fail)}
.verdict-Unknown{color:var(--conflict)}
ul.conflicts{padding-left:18px}
li.conflict{margin-bottom:6px}
svg{display:block;margin:10px 0;font-family:ui-monospace,Menlo,monospace;font-size:11px}
svg .axis{stroke:var(--edge)}
svg .tick{fill:var(--dim)}
svg .lane-label{fill:var(--dim)}
svg rect.op{fill:var(--op);stroke:var(--opline)}
svg rect.op.open{stroke-dasharray:4 3}
svg rect.op.failing{fill:#f6d5d2;stroke:var(--fail);stroke-width:2}
svg rect.op.conflict{fill:#f8e6c4;stroke:var(--conflict);stroke-width:2}
svg circle.point{fill:var(--point)}
svg .order{fill:var(--point);font-size:10px}
table{border-collapse:collapse;font-size:12px}
th,td{border:1px solid var(--edge);padding:3px 7px;text-align:left}
th{color:var(--dim);font-weight:600}
tr.failing td{background:#f6d5d2}
tr.conflict td{background:#f8e6c4}
";

/// Renders `history`, checked against `ir` with at most `budget` executions of the model per
/// search, as one self-contained HTML page.
///
/// # Errors
///
/// [`CheckRefusal`] where the history cannot be checked against this model at all.
pub fn render(ir: &EssIr, history: &History, budget: u64) -> Result<String, CheckRefusal> {
    let checked = linearize::check(ir, history, budget)?;
    let mut page = String::new();
    let _ = write!(
        page,
        "<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n\
         <title>{system} {version} — history {id}</title>\n<style>\n{STYLE}</style>\n</head>\n\
         <body>\n<header>\n<h1>{system} {version} — history <code>{id}</code></h1>\n\
         <div class=\"meta\">seed {seed} · {clients} client(s) · {operations} operation(s) · \
         budget {budget} · verdict <b class=\"verdict-{verdict}\">{verdict}</b></div>\n</header>\n",
        system = escape(&ir.system().to_string()),
        version = escape(&ir.version().to_string()),
        id = escape(history.history_id.as_str()),
        seed = history.seed,
        clients = history.clients,
        operations = history.operations.len(),
        verdict = verdict(checked.verdict),
    );
    section(&mut page, ir, "The history", history, &checked, budget)?;
    if checked.verdict == Verdict::Violation {
        let shrunk = linearize::shrink(ir, history, budget)?;
        if shrunk != *history {
            let checked = linearize::check(ir, &shrunk, budget)?;
            section(
                &mut page,
                ir,
                "The shrunk history",
                &shrunk,
                &checked,
                budget,
            )?;
        }
    }
    page.push_str("</body>\n</html>\n");
    Ok(page)
}

fn verdict(verdict: Verdict) -> &'static str {
    match verdict {
        Verdict::Linearizable => "Linearizable",
        Verdict::Violation => "Violation",
        Verdict::Unknown => "Unknown",
    }
}

/// `text` safe inside an element or a double-quoted attribute.
fn escape(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for character in text.chars() {
        match character {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            other => escaped.push(other),
        }
    }
    escaped
}

/// The last dotted segment of a qualified name: `PayInvoice` for `billing.invoice.PayInvoice`.
fn short(name: &str) -> &str {
    name.rsplit('.').next().unwrap_or(name)
}

/// What an operation answered, as drawn.
fn answered(operation: &Operation) -> String {
    operation.outcome.as_ref().map_or_else(
        || "no answer".to_owned(),
        |outcome| short(outcome.as_str()).to_owned(),
    )
}

/// How an operation is marked on the page.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Mark {
    Plain,
    Failing,
    Conflict,
}

impl Mark {
    fn class(self) -> &'static str {
        match self {
            Self::Plain => "",
            Self::Failing => " failing",
            Self::Conflict => " conflict",
        }
    }
}

/// One drawn history: its summary, its conflict, its lanes and its table.
fn section(
    page: &mut String,
    ir: &EssIr,
    title: &str,
    history: &History,
    checked: &Checked,
    budget: u64,
) -> Result<(), CheckRefusal> {
    let orders = linearize::orders(ir, history, budget)?;
    let conflict = linearize::conflict(ir, history, checked)?;
    let failing: Option<String> = match (&conflict, &checked.read, checked.verdict) {
        (Some(conflict), _, _) => Some(conflict.failing.operation_id.clone()),
        // A read violation fails at the read the check judged.
        (None, Some(read), Verdict::Violation) => Some(read.operation_id.clone()),
        (None, None, Verdict::Violation) => first_unplaced(ir, history, checked),
        _ => None,
    };
    let mark = |operation: &Operation| {
        let id = operation.operation_id.as_str();
        if failing.as_deref() == Some(id) {
            Mark::Failing
        } else if conflict
            .as_ref()
            .is_some_and(|conflict| conflict.against.operation_id == id)
        {
            Mark::Conflict
        } else {
            Mark::Plain
        }
    };

    let _ = write!(
        page,
        "<section class=\"history\">\n<h2>{title}</h2>\n<div class=\"meta\">verdict \
         <b class=\"verdict-{verdict}\">{verdict}</b> · {operations} operation(s) · {partitions} \
         subject partition(s) · {steps} of {budget} step(s)</div>\n",
        title = escape(title),
        verdict = verdict(checked.verdict),
        operations = history.operations.len(),
        partitions = checked.partitions,
        steps = checked.steps,
    );
    if checked.verdict == Verdict::Unknown {
        page.push_str(
            "<p>The budget ran out before the search finished; Unknown is not a pass.</p>\n",
        );
    }
    order_line(page, checked);
    if let Some(conflict) = &conflict {
        conflict_list(page, history, conflict);
    } else if let (Some(id), None) = (&failing, &checked.read) {
        let _ = writeln!(
            page,
            "<p>The search failed at operation <code>{}</code>: no order the model accepts \
             places it.</p>",
            escape(id)
        );
    }
    if let Some(read) = &checked.read {
        let _ = writeln!(
            page,
            "<p>Read violation: client {} read <code>{}</code> of <code>{}</code>, declared {}: \
             {} — it {} <code>{}</code>.</p>",
            read.client,
            escape(&read.operation_id),
            escape(&read.view),
            escape(&read.consistency),
            read.anomaly.as_str(),
            if read.shown { "shows" } else { "does not show" },
            escape(&read.subject_key)
        );
    }
    lanes(page, history, &orders, &mark);
    table(page, history, &orders, &mark);
    if !checked.not_judged.is_empty() {
        page.push_str("<ul class=\"not-judged\">\n");
        for read in &checked.not_judged {
            let _ = writeln!(
                page,
                "<li>not judged: <code>{}</code> reads <code>{}</code>, declared {}: {}</li>",
                escape(&read.operation_id),
                escape(&read.view),
                escape(&read.consistency),
                escape(&read.reason)
            );
        }
        page.push_str("</ul>\n");
    }
    page.push_str("</section>\n");
    Ok(())
}

/// The order found for the subject that decided a violation or an `Unknown`.
///
/// For a command violation it is the longest partial linearization. For a read violation every
/// command linearized, and it is one order found for the subject the read shows wrongly; the read
/// was judged against every such order.
fn order_line(page: &mut String, checked: &Checked) {
    let Some(subject) = &checked.subject_key else {
        return;
    };
    let (label, suffix) = if checked.read.is_some() {
        (
            "An order of the commands on subject",
            " — one of the orders the read was judged against",
        )
    } else {
        ("Longest partial linearization of subject", "")
    };
    let order = if checked.linearization.is_empty() {
        "none".to_owned()
    } else {
        checked
            .linearization
            .iter()
            .map(|id| format!("<code>{}</code>", escape(id)))
            .collect::<Vec<_>>()
            .join(" → ")
    };
    let _ = writeln!(
        page,
        "<p>{label} <code>{}</code>{suffix}: {order}</p>",
        escape(subject)
    );
}

/// The first operation of the deciding partition, in invoke order, the longest order left out.
fn first_unplaced(ir: &EssIr, history: &History, checked: &Checked) -> Option<String> {
    let subject = checked.subject_key.as_deref()?;
    let mut candidates: Vec<(usize, &Operation)> = history
        .operations
        .iter()
        .enumerate()
        .filter(|(_, operation)| {
            operation.subject_key == subject
                && !ess_domain::name::QualifiedName::new(operation.command.as_str())
                    .is_ok_and(|name| ir.views().contains_key(&name))
                && !checked
                    .linearization
                    .iter()
                    .any(|id| id == operation.operation_id.as_str())
        })
        .collect();
    candidates.sort_by_key(|(index, operation)| (operation.invoked_at, *index));
    candidates
        .first()
        .map(|(_, operation)| operation.operation_id.as_str().to_owned())
}

/// The conflict, as the two orders it names.
fn conflict_list(page: &mut String, history: &History, conflict: &Conflict) {
    let describe = |id: &str| -> String {
        history
            .operations
            .iter()
            .find(|operation| operation.operation_id.as_str() == id)
            .map_or_else(
                || format!("<code>{}</code>", escape(id)),
                |operation| {
                    format!(
                        "<code>{}</code> (client {}, {} → {})",
                        escape(id),
                        operation.client,
                        escape(short(operation.command.as_str())),
                        escape(&answered(operation))
                    )
                },
            )
    };
    let states = |states: &[String]| {
        states
            .iter()
            .map(|state| format!("<code>{}</code>", escape(state)))
            .collect::<Vec<_>>()
            .join(" or ")
    };
    let item = |side: &ConflictSide, first: &ConflictSide| {
        format!(
            "<li class=\"conflict\" data-operation=\"{id}\" data-required=\"{required}\" \
             data-supplied=\"{supplied}\">Ordered after {first}, {this} needed subject \
             <code>{subject}</code> in {required_states}; that order supplied it in \
             {supplied_states}.</li>\n",
            id = escape(&side.operation_id),
            required = escape(&side.required.join(",")),
            supplied = escape(&side.supplied.join(",")),
            first = describe(&first.operation_id),
            this = describe(&side.operation_id),
            subject = escape(&conflict.subject_key),
            required_states = states(&side.required),
            supplied_states = states(&side.supplied),
        )
    };
    let _ = write!(
        page,
        "<p>The search failed at operation <code>{}</code>. It conflicts with <code>{}</code>: \
         neither order of the two explains what both answered.</p>\n<ul class=\"conflicts\">\n{}{}\
         </ul>\n",
        escape(&conflict.failing.operation_id),
        escape(&conflict.against.operation_id),
        item(&conflict.failing, &conflict.against),
        item(&conflict.against, &conflict.failing),
    );
}

/// Each instant's rank among every instant the history records.
fn ranks(history: &History) -> BTreeMap<u64, u64> {
    let instants: BTreeSet<u64> = history
        .operations
        .iter()
        .flat_map(|operation| [Some(operation.invoked_at), operation.returned_at])
        .flatten()
        .collect();
    instants
        .into_iter()
        .enumerate()
        .map(|(rank, instant)| (instant, rank as u64))
        .collect()
}

/// Where everything on the time axis is drawn.
///
/// Each instant rank is a band: every call invoked at it starts at the band's left edge, every
/// call that returned at it ends at the band's right edge, and the linearization points placed at
/// it sit in slots across the band, one slot per point, so the band widens with the number of
/// points it holds. Bands are [`COLUMN`] apart. A call's point is placed at the latest of its own
/// invoke rank and the rank of the point ordered before it on the same subject — in the next slot
/// when that is the same rank — so each subject's points are at strictly increasing x in the order
/// found, and every point lies inside its call's interval: an order the search found never puts a
/// call after one that was invoked after it returned.
struct Layout {
    /// Each instant's rank.
    ranks: BTreeMap<u64, u64>,
    /// The left edge of each rank's band, and one more for a call that never answered.
    left: Vec<u64>,
    /// The width of each rank's band, and one more for a call that never answered.
    band: Vec<u64>,
    /// Each placed operation's point, and its place in its subject's order.
    points: BTreeMap<String, (u64, usize)>,
}

impl Layout {
    fn new(history: &History, orders: &[(String, Vec<String>)]) -> Self {
        let ranks = ranks(history);
        let open = ranks.len();
        let rank = |instant: u64| ranks.get(&instant).copied().unwrap_or(0);
        let by_id: BTreeMap<&str, &Operation> = history
            .operations
            .iter()
            .map(|operation| (operation.operation_id.as_str(), operation))
            .collect();
        // (rank, slot from 1, place in the order), per placed operation.
        let mut slotted: BTreeMap<String, (u64, u64, usize)> = BTreeMap::new();
        let mut slots = vec![0_u64; open + 1];
        for (_, order) in orders {
            let mut previous: Option<(u64, u64)> = None;
            for (position, id) in order.iter().enumerate() {
                let Some(operation) = by_id.get(id.as_str()) else {
                    continue;
                };
                let invoked = rank(operation.invoked_at);
                let (at, slot) = match previous {
                    Some((before, slot)) if before >= invoked => (before, slot + 1),
                    _ => (invoked, 1),
                };
                previous = Some((at, slot));
                let index = usize::try_from(at).unwrap_or(open);
                slots[index] = slots[index].max(slot);
                slotted.insert(id.clone(), (at, slot, position + 1));
            }
        }
        let band: Vec<u64> = slots.iter().map(|count| (count + 1) * SLOT).collect();
        let mut left = Vec::with_capacity(open + 1);
        let mut x = LEFT;
        for width in &band {
            left.push(x);
            x += width + COLUMN;
        }
        let points = slotted
            .into_iter()
            .map(|(id, (at, slot, position))| {
                let index = usize::try_from(at).unwrap_or(open);
                (id, (left[index] + slot * SLOT, position))
            })
            .collect();
        Self {
            ranks,
            left,
            band,
            points,
        }
    }

    /// The index of `instant`'s band; the last band for a call that never answered.
    fn index(&self, instant: Option<u64>) -> usize {
        instant
            .and_then(|instant| self.ranks.get(&instant))
            .and_then(|rank| usize::try_from(*rank).ok())
            .unwrap_or(self.ranks.len())
    }

    /// An operation's interval: from its invoke band's left edge to its return band's right edge.
    fn span(&self, operation: &Operation) -> (u64, u64) {
        let start = self.left[self.index(Some(operation.invoked_at))];
        let returned = match operation.return_bound() {
            ReturnBound::At(instant) => Some(instant),
            ReturnBound::AfterEveryOther => None,
        };
        let index = self.index(returned);
        (
            start,
            (self.left[index] + self.band[index]).max(start + SLOT),
        )
    }

    /// The width of the drawing.
    fn width(&self) -> u64 {
        self.left.last().copied().unwrap_or(LEFT) + self.band.last().copied().unwrap_or(0) + 24
    }
}

/// The lanes: one row per client, one bar per operation, one point per placed operation.
fn lanes(
    page: &mut String,
    history: &History,
    orders: &[(String, Vec<String>)],
    mark: &dyn Fn(&Operation) -> Mark,
) {
    let layout = Layout::new(history, orders);
    // Every client that made a call has a lane. Up to `IDLE_LANES` declared clients, so does every
    // one that made none; above it, those are counted in one row, never enumerated.
    let calling: BTreeSet<u64> = history.operations.iter().map(|it| it.client).collect();
    let clients: BTreeSet<u64> = if history.clients <= IDLE_LANES {
        (0..history.clients)
            .chain(calling.iter().copied())
            .collect()
    } else {
        calling.clone()
    };
    let idle = if history.clients <= IDLE_LANES {
        0
    } else {
        history.clients.saturating_sub(calling.len() as u64)
    };
    let rows = clients.len() as u64 + u64::from(idle > 0);
    let width = layout.width();
    let height = TOP + rows * LANE + 8;
    let _ = writeln!(
        page,
        "<svg class=\"lanes\" width=\"{width}\" height=\"{height}\" viewBox=\"0 0 {width} \
         {height}\" role=\"img\">"
    );
    for (instant, rank) in &layout.ranks {
        let x = usize::try_from(*rank).map_or(LEFT, |index| layout.left[index]);
        let _ = writeln!(
            page,
            "<line class=\"axis\" x1=\"{x}\" y1=\"{top}\" x2=\"{x}\" y2=\"{bottom}\"/>\
             <text class=\"tick\" x=\"{x}\" y=\"{label}\" text-anchor=\"middle\">{instant}</text>",
            top = TOP - 6,
            bottom = height - 4,
            label = TOP - 10,
        );
    }
    for (lane, client) in clients.iter().enumerate() {
        let y = TOP + lane as u64 * LANE;
        let _ = writeln!(
            page,
            "<g class=\"lane\" data-client=\"{client}\">\n<text class=\"lane-label\" x=\"8\" \
             y=\"{label}\">client {client}</text>",
            label = y + BAR / 2 + 8,
        );
        for operation in history
            .operations
            .iter()
            .filter(|operation| operation.client == *client)
        {
            let (start, end) = layout.span(operation);
            let open = if operation.returned_at.is_none() {
                " open"
            } else {
                ""
            };
            let id = escape(operation.operation_id.as_str());
            let _ = writeln!(
                page,
                "<rect class=\"op{mark}{open}\" data-operation=\"{id}\" x=\"{start}\" \
                 y=\"{top}\" width=\"{width}\" height=\"{BAR}\" rx=\"4\"><title>{id} · {command} \
                 `{subject}` → {answer}</title></rect>\n<text x=\"{label}\" y=\"{baseline}\">\
                 {short} → {answer}</text>",
                mark = mark(operation).class(),
                top = y + 6,
                width = end - start,
                command = escape(operation.command.as_str()),
                subject = escape(&operation.subject_key),
                answer = escape(&answered(operation)),
                label = start + 4,
                baseline = y + 6 + BAR / 2 + 4,
                short = escape(short(operation.command.as_str())),
            );
            if let Some((x, position)) = layout.points.get(operation.operation_id.as_str()) {
                let _ = writeln!(
                    page,
                    "<circle class=\"point\" cx=\"{x}\" cy=\"{cy}\" r=\"4\"><title>linearization \
                     point {position} of subject `{subject}`</title></circle><text class=\"order\" \
                     x=\"{x}\" y=\"{above}\" text-anchor=\"middle\">{position}</text>",
                    cy = y + 6 + BAR,
                    subject = escape(&operation.subject_key),
                    above = y + 6 + BAR + 14,
                );
            }
        }
        page.push_str("</g>\n");
    }
    if idle > 0 {
        let _ = writeln!(
            page,
            "<g class=\"idle\" data-clients=\"{idle}\"><text class=\"lane-label\" x=\"8\" \
             y=\"{label}\">{idle} clients made no calls</text></g>",
            label = TOP + clients.len() as u64 * LANE + BAR / 2 + 8,
        );
    }
    page.push_str("</svg>\n");
}

/// Every operation, in document order, with its interval and its place in the order found.
fn table(
    page: &mut String,
    history: &History,
    orders: &[(String, Vec<String>)],
    mark: &dyn Fn(&Operation) -> Mark,
) {
    let position: BTreeMap<&str, usize> = orders
        .iter()
        .flat_map(|(_, order)| {
            order
                .iter()
                .enumerate()
                .map(|(index, id)| (id.as_str(), index + 1))
        })
        .collect();
    page.push_str(
        "<table>\n<tr><th>operation</th><th>client</th><th>command</th><th>subject</th>\
         <th>invoked</th><th>returned</th><th>answer</th><th>order</th></tr>\n",
    );
    for operation in &history.operations {
        let id = operation.operation_id.as_str();
        let class = match mark(operation) {
            Mark::Plain => "",
            Mark::Failing => " class=\"failing\"",
            Mark::Conflict => " class=\"conflict\"",
        };
        let _ = writeln!(
            page,
            "<tr{class}><td class=\"mono\">{id}</td><td>{client}</td><td class=\"mono\">{command}\
             </td><td class=\"mono\">{subject}</td><td>{invoked}</td><td>{returned}</td>\
             <td class=\"mono\">{answer}</td><td>{order}</td></tr>",
            id = escape(id),
            client = operation.client,
            command = escape(operation.command.as_str()),
            subject = escape(&operation.subject_key),
            invoked = operation.invoked_at,
            returned = operation
                .returned_at
                .map_or_else(|| "never".to_owned(), |instant| instant.to_string()),
            answer = escape(&answered(operation)),
            order = position
                .get(id)
                .map_or_else(|| "—".to_owned(), ToString::to_string),
        );
    }
    page.push_str("</table>\n");
}
