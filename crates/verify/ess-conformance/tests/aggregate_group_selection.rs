//! Aggregate group selection and isolated observations (beyond10x/ess#361, beyond10x/ess#362).
//!
//! `docs/design/aggregate-group-selection.md`. A parameter compared with a group key selects that
//! group's row, a key copied from a related row is selected the same way, and a view grouped by the
//! lifecycle state alone is observed exactly — under the suite's `scenario_initial_state: empty`.
//!
//! Everything above the first test is the target: a store of its own that computes every view of
//! the fixtures itself and never reads an expected value from the suite. It is run natively, and
//! the same source is compiled into the WASM runner (`wasm_verdicts`); the generated Go and
//! TypeScript runtimes run ports of it (`fixtures/aggregate-group-selection-runtime.go`,
//! `.mjs`). A `Fault` switches in one defect at a time.
use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;

use ess_compiler::refs::{CommandRef, OutcomeRef};
use ess_conformance::target::*;
use ess_domain::command::OutcomeName;
use ess_primitives::{
    facts::{FactValue, Number},
    node::Node,
    time::Rfc3339Instant,
};

/// Which fixture a store answers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Model {
    /// `fixtures/aggregate-group-selection.yaml`.
    Work,
    /// `fixtures/aggregate-group-selection-copied.yaml`.
    Depots,
    /// `fixtures/aggregate-group-parameter.yaml`, the issue's direct source.
    Direct,
    /// `fixtures/aggregate-copied-group-parameter.yaml`, the issue's copied-key source.
    Copied,
    /// `fixtures/aggregate-state-groups.yaml`, the issue's state-only source.
    States,
    /// `fixtures/aggregate-group-selection-owner.yaml`: a selector over an owner's identity.
    Ledger,
    /// `fixtures/aggregate-group-selection-guarded.yaml`: the #361 comment's shape, a selector
    /// and a copied key on a creating command a related guard refuses for a missing row.
    Goals,
    /// `fixtures/aggregate-list-parameter.yaml`: a list parameter over a group key
    /// (beyond10x/ess#438).
    Lists,
    /// `fixtures/aggregate-timestamp-range.yaml`: a window the caller resolved into two
    /// `Timestamp` parameters (beyond10x/ess#439).
    Window,
}

impl Model {
    const ALL: [Self; 9] = [
        Self::Work,
        Self::Depots,
        Self::Direct,
        Self::Copied,
        Self::States,
        Self::Ledger,
        Self::Goals,
        Self::Lists,
        Self::Window,
    ];

    fn name(self) -> &'static str {
        match self {
            Self::Work => "work",
            Self::Depots => "depots",
            Self::Direct => "direct",
            Self::Copied => "copied",
            Self::States => "states",
            Self::Ledger => "ledger",
            Self::Goals => "goals",
            Self::Lists => "lists",
            Self::Window => "window",
        }
    }

    fn of(name: &str) -> Self {
        Self::ALL
            .into_iter()
            .find(|model| model.name() == name)
            .unwrap_or_else(|| panic!("no model {name}"))
    }

    fn copies(self) -> bool {
        matches!(self, Self::Depots | Self::Copied)
    }
}

/// One defect an implementation could have.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Fault {
    None,
    /// A group parameter is not applied: every group comes back.
    IgnoresParam,
    /// The selection is replaced by the key of the first row ever created.
    SelectsFirstGroup,
    /// A text parameter is compared as a prefix of the key (a changed parameter).
    PrefixParam,
    /// Rows are grouped by the first key only: a one-key view's groups merge into one.
    MergesGroups,
    /// Rows whose group key is absent are dropped.
    DropsAbsentGroup,
    /// `count` counts every row of the group, the filter's refuted ones included.
    CountsAllRows,
    /// A transition answers but leaves the row in its state.
    StaleState,
    /// `begin_scenario` keeps the rows earlier scenarios made.
    RetainsRows,
    /// A grouped answer carries one extra group.
    SpuriousGroup,
    /// `sum` is one too many.
    WrongSum,
    /// `count` is one too many.
    WrongCount,
    /// `max` and `min` are swapped.
    WrongExtreme,
    /// A copied key is read from the related row created first, not the one named.
    CopiesFirstRelated,
    /// A copied key is read from the related row created last, not the one named.
    CopiesLastRelated,
    /// Numbers pass through binary64.
    LossyNumbers,
    /// `avg` truncates at the sixth place instead of rounding.
    TruncatesAvg,
    /// `count_distinct` counts rows.
    DistinctCountsRows,
    /// A list parameter is not applied: every group comes back.
    IgnoresList,
    /// An empty list selects nothing, where the view reads it as every group.
    EmptyMatchesNothing,
    /// Only the first element of a list parameter selects.
    FirstElementOnly,
    /// A window's `from` is not applied.
    IgnoresBound,
    /// A window's `to` is inclusive.
    InclusiveTo,
    /// Instants are compared by their spellings.
    ComparesText,
}

impl Fault {
    const ALL: [Self; 18] = [
        Self::None,
        Self::IgnoresParam,
        Self::SelectsFirstGroup,
        Self::PrefixParam,
        Self::MergesGroups,
        Self::DropsAbsentGroup,
        Self::CountsAllRows,
        Self::StaleState,
        Self::RetainsRows,
        Self::SpuriousGroup,
        Self::WrongSum,
        Self::WrongCount,
        Self::WrongExtreme,
        Self::CopiesFirstRelated,
        Self::CopiesLastRelated,
        Self::LossyNumbers,
        Self::TruncatesAvg,
        Self::DistinctCountsRows,
    ];

    /// The defects of a list parameter, run over [`Model::Lists`] only.
    const LISTS: [Self; 3] = [
        Self::IgnoresList,
        Self::EmptyMatchesNothing,
        Self::FirstElementOnly,
    ];

    /// The defects of a window, run over [`Model::Window`] only.
    const WINDOWS: [Self; 3] = [Self::IgnoresBound, Self::InclusiveTo, Self::ComparesText];

    fn name(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::IgnoresParam => "ignores-param",
            Self::SelectsFirstGroup => "selects-first-group",
            Self::PrefixParam => "prefix-param",
            Self::MergesGroups => "merges-groups",
            Self::DropsAbsentGroup => "drops-absent-group",
            Self::CountsAllRows => "counts-all-rows",
            Self::StaleState => "stale-state",
            Self::RetainsRows => "retains-rows",
            Self::SpuriousGroup => "spurious-group",
            Self::WrongSum => "wrong-sum",
            Self::WrongCount => "wrong-count",
            Self::WrongExtreme => "wrong-extreme",
            Self::CopiesFirstRelated => "copies-first-related",
            Self::CopiesLastRelated => "copies-last-related",
            Self::LossyNumbers => "lossy-numbers",
            Self::TruncatesAvg => "truncates-avg",
            Self::DistinctCountsRows => "distinct-counts-rows",
            Self::IgnoresList => "ignores-list",
            Self::EmptyMatchesNothing => "empty-matches-nothing",
            Self::FirstElementOnly => "first-element-only",
            Self::IgnoresBound => "ignores-bound",
            Self::InclusiveTo => "inclusive-to",
            Self::ComparesText => "compares-text",
        }
    }

    fn of(name: &str) -> Self {
        Self::ALL
            .into_iter()
            .chain(Self::LISTS)
            .chain(Self::WINDOWS)
            .find(|fault| fault.name() == name)
            .unwrap_or_else(|| panic!("no fault {name}"))
    }
}

#[derive(Debug, Clone, Copy)]
enum Agg {
    Count,
    Sum(&'static str),
    Max(&'static str),
    Min(&'static str),
    Avg(&'static str),
    Distinct(&'static str),
}

/// One view as the fixture declares it: a conjunction of `field == param.field`, where
/// `open_only` `state == Open`, and where `list` names one the membership of a field in a list
/// parameter.
struct ViewDef {
    name: &'static str,
    group_by: &'static [&'static str],
    params: &'static [&'static str],
    open_only: bool,
    fields: &'static [(&'static str, Agg)],
    list: Option<ListDef>,
    /// `<field> >= param.from` and `<field> < param.to`, compared as instants.
    range: Option<&'static str>,
}

/// `exists: {in: param.<param>, as: q, that: <field> == q}`; with `empty_all`, beside
/// `param.<param>.count == 0`.
#[derive(Clone, Copy)]
struct ListDef {
    param: &'static str,
    field: &'static str,
    empty_all: bool,
}

const fn view(
    name: &'static str,
    group_by: &'static [&'static str],
    params: &'static [&'static str],
    fields: &'static [(&'static str, Agg)],
) -> ViewDef {
    ViewDef {
        name,
        group_by,
        params,
        open_only: false,
        fields,
        list: None,
        range: None,
    }
}

const WORK: &[ViewDef] = &[
    view(
        "demo.work.ByTeam",
        &["team"],
        &["team"],
        &[
            ("count", Agg::Count),
            ("total", Agg::Sum("cents")),
            ("top", Agg::Max("cents")),
        ],
    ),
    ViewDef {
        open_only: true,
        ..view(
            "demo.work.OpenByTeam",
            &["team"],
            &["team"],
            &[("count", Agg::Count), ("total", Agg::Sum("cents"))],
        )
    },
    view(
        "demo.work.ByTeamState",
        &["team", "state"],
        &["team"],
        &[("count", Agg::Count)],
    ),
    view(
        "demo.work.ByTeamChannel",
        &["team", "channel"],
        &["team"],
        &[("count", Agg::Count), ("total", Agg::Sum("cents"))],
    ),
    view(
        "demo.work.ByUrgency",
        &["urgent"],
        &["urgent"],
        &[("count", Agg::Count), ("total", Agg::Sum("cents"))],
    ),
    view(
        "demo.work.ByBucket",
        &["bucket"],
        &["bucket"],
        &[("count", Agg::Count)],
    ),
    view(
        "demo.work.ByState",
        &["state"],
        &[],
        &[
            ("count", Agg::Count),
            ("total", Agg::Sum("cents")),
            ("low", Agg::Min("cents")),
        ],
    ),
    view(
        "demo.work.TeamsInLane",
        &["team"],
        &["lane", "team"],
        &[("count", Agg::Count)],
    ),
    view(
        "demo.work.ByTeamUrgency",
        &["team", "urgent"],
        &["team", "urgent"],
        &[("count", Agg::Count)],
    ),
    view(
        "demo.work.ByKind",
        &["kind"],
        &[],
        &[
            ("count", Agg::Count),
            ("mean", Agg::Avg("cents")),
            ("lanes", Agg::Distinct("lane")),
        ],
    ),
    view(
        "demo.work.ByUrgent",
        &["urgent"],
        &[],
        &[("count", Agg::Count)],
    ),
    ViewDef {
        open_only: true,
        ..view(
            "demo.work.TopOpen",
            &[],
            &[],
            &[("top", Agg::Max("cents")), ("low", Agg::Min("cents"))],
        )
    },
];

const LEDGER: &[ViewDef] = &[view(
    "demo.ledger.ByAccount",
    &["account_id"],
    &["account_id"],
    &[("count", Agg::Count), ("total", Agg::Sum("cents"))],
)];

const GOALS: &[ViewDef] = &[
    view(
        "demo.goals.EvidenceEvaluations",
        &["objective_id", "goal"],
        &["objective_id"],
        &[("count", Agg::Count)],
    ),
    view(
        "demo.goals.EvaluationsByGoal",
        &["goal"],
        &[],
        &[("count", Agg::Count), ("total", Agg::Sum("cents"))],
    ),
];

const DEPOTS: &[ViewDef] = &[
    view(
        "demo.work.ByTeam",
        &["team"],
        &["team"],
        &[("count", Agg::Count), ("total", Agg::Sum("cents"))],
    ),
    view(
        "demo.work.ItemsByState",
        &["state"],
        &[],
        &[("count", Agg::Count)],
    ),
];

const DIRECT: &[ViewDef] = &[view(
    "demo.work.ByTeam",
    &["team"],
    &["team"],
    &[("count", Agg::Count)],
)];

const COPIED: &[ViewDef] = &[view(
    "demo.work.ByTeam",
    &["team"],
    &["team"],
    &[("count", Agg::Count), ("total", Agg::Sum("cents"))],
)];

const STATES: &[ViewDef] = &[view(
    "demo.work.ByState",
    &["state"],
    &[],
    &[("count", Agg::Count)],
)];

const LISTS: &[ViewDef] = &[
    ViewDef {
        list: Some(ListDef {
            param: "queues",
            field: "queue_id",
            empty_all: false,
        }),
        ..view(
            "demo.calls.InQueues",
            &["queue_id"],
            &[],
            &[("calls", Agg::Count), ("talk_ms", Agg::Sum("duration_ms"))],
        )
    },
    ViewDef {
        list: Some(ListDef {
            param: "queues",
            field: "queue_id",
            empty_all: true,
        }),
        ..view(
            "demo.calls.InQueuesOrAll",
            &["queue_id"],
            &[],
            &[("calls", Agg::Count)],
        )
    },
];

const WINDOWS: &[ViewDef] = &[
    ViewDef {
        range: Some("started_at"),
        ..view(
            "demo.window.CallsInRange",
            &[],
            &[],
            &[("calls", Agg::Count), ("talk_ms", Agg::Sum("duration_ms"))],
        )
    },
    ViewDef {
        range: Some("started_at"),
        ..view(
            "demo.window.QueueCallsInRange",
            &["queue_id"],
            &[],
            &[("calls", Agg::Count)],
        )
    },
];

/// The literal `demo.work.Open` writes into `bucket`: 2^53 + 1, which binary64 cannot hold.
const BUCKET: &str = "9007199254740993";

type Row = BTreeMap<String, Node>;

fn number(text: &str) -> Node {
    match FactValue::parse_literal(text) {
        FactValue::Number(number) => Node::Number(number),
        other => panic!("{other:?}"),
    }
}

fn integer(node: &Node) -> i128 {
    match node {
        Node::Number(number) => i128::from(number.as_i64().expect("an integer")),
        other => panic!("not a number: {other:?}"),
    }
}

/// The mean of integers to six places, half-even, spelled without trailing zeroes — or truncated.
fn mean(values: &[i128], truncate: bool) -> Node {
    if values.is_empty() {
        return Node::Null;
    }
    let sum: i128 = values.iter().sum();
    let n = i128::try_from(values.len()).expect("a count");
    let (mut q, r) = ((sum * 1_000_000) / n, (sum * 1_000_000) % n);
    if !truncate && (2 * r > n || (2 * r == n && q % 2 == 1)) {
        q += 1;
    }
    let text = format!("{}.{:06}", q / 1_000_000, q % 1_000_000);
    number(text.trim_end_matches('0').trim_end_matches('.'))
}

/// A number as a binary64 carries it.
fn lossy(node: &Node) -> Node {
    match node {
        Node::Number(number) => Node::Number(Number::new(number.get()).expect("finite")),
        other => other.clone(),
    }
}

struct Store {
    model: Model,
    fault: Fault,
    rows: RefCell<Vec<Row>>,
    /// The rows others name — depots, accounts or objectives — and the value each holds.
    depots: RefCell<Vec<(Node, Node)>>,
    minted: Cell<u64>,
}

impl Store {
    fn new(model: Model, fault: Fault) -> Self {
        Self {
            model,
            fault,
            rows: RefCell::default(),
            depots: RefCell::default(),
            minted: Cell::new(0),
        }
    }

    fn views(&self) -> &'static [ViewDef] {
        match self.model {
            Model::Work => WORK,
            Model::Depots => DEPOTS,
            Model::Direct => DIRECT,
            Model::Copied => COPIED,
            Model::States => STATES,
            Model::Ledger => LEDGER,
            Model::Goals => GOALS,
            Model::Lists => LISTS,
            Model::Window => WINDOWS,
        }
    }

    fn kept(&self, node: &Node) -> Node {
        if self.fault == Fault::LossyNumbers {
            lossy(node)
        } else {
            node.clone()
        }
    }

    fn selected(&self, held: &Node, param: &Node, first: Option<&Node>) -> bool {
        match self.fault {
            Fault::IgnoresParam => true,
            Fault::SelectsFirstGroup => first == Some(held),
            Fault::PrefixParam => match (held, param) {
                (Node::Text(held), Node::Text(param)) => held.starts_with(param.as_str()),
                _ => held == param,
            },
            _ => *held != Node::Null && *held == self.kept(param),
        }
    }

    fn admits(&self, view: &ViewDef, row: &Row, params: &BTreeMap<String, Node>) -> bool {
        let rows = self.rows.borrow();
        let first = rows.first();
        let groups = |field: &str| view.group_by.contains(&field);
        view.params.iter().all(|param| {
            let held = row.get(*param).unwrap_or(&Node::Null);
            let wanted = params.get(*param).unwrap_or(&Node::Null);
            if groups(param) {
                self.selected(held, wanted, first.and_then(|row| row.get(*param)))
            } else {
                *held == self.kept(wanted)
            }
        }) && (!view.open_only || row["state"] == Node::Text("Open".into()))
            && view.list.is_none_or(|list| self.listed(list, row, params))
            && view
                .range
                .is_none_or(|field| self.within(field, row, params))
    }

    /// Whether `row`'s instant is in `[param.from, param.to)`.
    fn within(&self, field: &str, row: &Row, params: &BTreeMap<String, Node>) -> bool {
        let text = |node: Option<&Node>| match node {
            Some(Node::Text(text)) => text.clone(),
            other => panic!("not an instant: {other:?}"),
        };
        let (held, from, to) = (
            text(row.get(field)),
            text(params.get("from")),
            text(params.get("to")),
        );
        if self.fault == Fault::ComparesText {
            return from <= held && held < to;
        }
        let at = |text: &str| Rfc3339Instant::parse_rfc3339(text).expect("an instant");
        let (held, from, to) = (at(&held), at(&from), at(&to));
        (self.fault == Fault::IgnoresBound || from <= held)
            && (held < to || (self.fault == Fault::InclusiveTo && held == to))
    }

    /// Whether `row`'s field is one the list parameter names.
    fn listed(&self, list: ListDef, row: &Row, params: &BTreeMap<String, Node>) -> bool {
        let held = row.get(list.field).unwrap_or(&Node::Null);
        let Some(Node::Seq(sent)) = params.get(list.param) else {
            return false;
        };
        match self.fault {
            Fault::IgnoresList => true,
            _ if sent.is_empty() => list.empty_all && self.fault != Fault::EmptyMatchesNothing,
            Fault::FirstElementOnly => *held == sent[0],
            _ => sent.contains(held),
        }
    }

    fn query(&self, view: &ViewDef, params: &BTreeMap<String, Node>) -> Vec<Row> {
        let keys: &[&str] = match self.fault {
            Fault::MergesGroups if view.group_by.len() > 1 => &view.group_by[..1],
            Fault::MergesGroups => &[],
            _ => view.group_by,
        };
        let rows = self.rows.borrow().clone();
        let mut groups: Vec<(Vec<Node>, Vec<Row>, Vec<Row>)> = Vec::new();
        for row in rows {
            let key: Vec<Node> = keys
                .iter()
                .map(|key| row.get(*key).cloned().unwrap_or(Node::Null))
                .collect();
            if self.fault == Fault::DropsAbsentGroup && key.contains(&Node::Null) {
                continue;
            }
            let admitted = self.admits(view, &row, params);
            let index = if let Some(index) = groups.iter().position(|(held, ..)| *held == key) {
                index
            } else {
                groups.push((key, Vec::new(), Vec::new()));
                groups.len() - 1
            };
            groups[index].1.push(row.clone());
            if admitted {
                groups[index].2.push(row);
            }
        }
        let mut out = Vec::new();
        for (_, all, members) in groups {
            let Some(first) = members.first() else {
                continue;
            };
            let mut reported = Row::new();
            for key in view.group_by {
                reported.insert(
                    (*key).to_owned(),
                    first.get(*key).cloned().unwrap_or(Node::Null),
                );
            }
            for (field, aggregate) in view.fields {
                reported.insert(
                    (*field).to_owned(),
                    self.aggregate(*aggregate, &all, &members),
                );
            }
            out.push(reported);
        }
        // An ungrouped view is one row, over no admitted row too.
        if view.group_by.is_empty() && out.is_empty() {
            let mut reported = Row::new();
            for (field, aggregate) in view.fields {
                reported.insert((*field).to_owned(), self.aggregate(*aggregate, &[], &[]));
            }
            out.push(reported);
        }
        if self.fault == Fault::SpuriousGroup && !view.group_by.is_empty() {
            if let Some(first) = out.first().cloned() {
                let mut extra = first;
                for key in view.group_by {
                    extra.insert((*key).to_owned(), Node::Text("spurious".into()));
                }
                out.push(extra);
            }
        }
        out
    }

    fn aggregate(&self, aggregate: Agg, all: &[Row], members: &[Row]) -> Node {
        let column = |field: &str| {
            members
                .iter()
                .map(|row| integer(&row[field]))
                .collect::<Vec<_>>()
        };
        let spelled = |value: i128| number(&value.to_string());
        match aggregate {
            Agg::Count => {
                let counted = if self.fault == Fault::CountsAllRows {
                    all.len()
                } else {
                    members.len()
                };
                let extra = usize::from(self.fault == Fault::WrongCount);
                spelled(i128::try_from(counted + extra).expect("a count"))
            }
            Agg::Sum(field) => {
                let extra = i128::from(self.fault == Fault::WrongSum);
                spelled(column(field).iter().sum::<i128>() + extra)
            }
            Agg::Max(field) | Agg::Min(field) => {
                let values = column(field);
                let max = matches!(aggregate, Agg::Max(_)) != (self.fault == Fault::WrongExtreme);
                let found = if max {
                    values.iter().max()
                } else {
                    values.iter().min()
                };
                found.map_or(Node::Null, |value| spelled(*value))
            }
            Agg::Avg(field) => mean(&column(field), self.fault == Fault::TruncatesAvg),
            Agg::Distinct(field) => {
                let mut seen: Vec<&Node> = Vec::new();
                for row in members {
                    if !seen.contains(&&row[field]) {
                        seen.push(&row[field]);
                    }
                }
                let counted = if self.fault == Fault::DistinctCountsRows {
                    members.len()
                } else {
                    seen.len()
                };
                spelled(i128::try_from(counted).expect("a count"))
            }
        }
    }

    /// The value copied from the related row `named` names — or, under a copying fault, from the
    /// first or last one created.
    fn read_related(&self, named: &Node) -> Option<Node> {
        let related = self.depots.borrow();
        let read = match self.fault {
            Fault::CopiesFirstRelated => related.first(),
            Fault::CopiesLastRelated => related.last(),
            _ => related.iter().find(|(id, _)| id == named),
        };
        read.map(|(_, value)| value.clone())
    }

    fn mint(&self) -> (u64, Node) {
        self.minted.set(self.minted.get() + 1);
        let n = self.minted.get();
        (n, Node::Text(format!("00000000-0000-4000-8000-{n:012}")))
    }
}

fn outcome(command: &CommandRef, name: &str) -> OutcomeRef {
    OutcomeRef::new(command.clone(), OutcomeName::new(name).unwrap())
}

impl ConformanceTarget for Store {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("group-selection-fixture", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        if self.fault != Fault::RetainsRows {
            self.rows.replace(Vec::new());
            self.depots.replace(Vec::new());
        }
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
    #[allow(clippy::too_many_lines)] // One arm per command of every fixture, kept together.
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let (n, id) = self.mint();
        let token = ess_primitives::consistency::ConsistencyToken::new(format!("seq:{n}")).unwrap();
        let command = request.command.clone();
        let input = |field: &str| request.input.get(field).map(|value| self.kept(value));
        let commands = match self.model {
            Model::Ledger => 2,
            Model::Goals => 3,
            Model::Lists => 4,
            Model::Window => 5,
            other => u8::from(other.copies()),
        };
        let result = match (command.to_string().as_str(), commands) {
            ("demo.work.Open", 0) => {
                let mut row = Row::new();
                for field in ["team", "lane", "cents", "urgent", "channel", "kind"] {
                    row.insert(field.to_owned(), input(field).unwrap_or(Node::Null));
                }
                row.insert("id".into(), id.clone());
                row.insert("bucket".into(), self.kept(&number(BUCKET)));
                row.insert("state".into(), Node::Text("Open".into()));
                self.rows.borrow_mut().push(row);
                SemanticCommandResult::took(outcome(&command, "opened")).emitting(
                    ObservedEvent::new("demo.work.Opened".parse().unwrap()).with("id", id),
                )
            }
            ("demo.work.Finish", 0) => {
                let named = request.input.get("id").cloned().unwrap_or(Node::Null);
                let mut rows = self.rows.borrow_mut();
                let Some(row) = rows.iter_mut().find(|row| row["id"] == named) else {
                    return Ok(SemanticCommandResult::undeclared().with_consistency(token));
                };
                if row["state"] == Node::Text("Open".into()) {
                    if self.fault != Fault::StaleState {
                        row.insert("state".into(), Node::Text("Done".into()));
                    }
                    SemanticCommandResult::took(outcome(&command, "finished")).emitting(
                        ObservedEvent::new("demo.work.Finished".parse().unwrap()).with("id", named),
                    )
                } else {
                    SemanticCommandResult::took(outcome(&command, "unavailable"))
                }
            }
            ("demo.work.OpenDepot", 1) => {
                self.depots
                    .borrow_mut()
                    .push((id.clone(), input("team").unwrap_or(Node::Null)));
                SemanticCommandResult::took(outcome(&command, "opened")).emitting(
                    ObservedEvent::new("demo.work.DepotOpened".parse().unwrap())
                        .with("depot_id", id),
                )
            }
            ("demo.work.OpenItem", 1) => {
                let named = request.input.get("depot_id").cloned().unwrap_or(Node::Null);
                let Some(team) = self.read_related(&named) else {
                    return Ok(SemanticCommandResult::undeclared().with_consistency(token));
                };
                let row = Row::from([
                    ("item_id".to_owned(), id.clone()),
                    ("depot_id".to_owned(), named),
                    ("team".to_owned(), team),
                    ("cents".to_owned(), input("cents").unwrap_or(Node::Null)),
                    ("state".to_owned(), Node::Text("Open".into())),
                ]);
                self.rows.borrow_mut().push(row);
                SemanticCommandResult::took(outcome(&command, "opened")).emitting(
                    ObservedEvent::new("demo.work.ItemOpened".parse().unwrap()).with("item_id", id),
                )
            }
            ("demo.ledger.OpenAccount", 2) => {
                self.depots
                    .borrow_mut()
                    .push((id.clone(), input("holder").unwrap_or(Node::Null)));
                SemanticCommandResult::took(outcome(&command, "opened")).emitting(
                    ObservedEvent::new("demo.ledger.AccountOpened".parse().unwrap())
                        .with("account_id", id),
                )
            }
            ("demo.ledger.PostEntry", 2) => {
                let named = request
                    .input
                    .get("account_id")
                    .cloned()
                    .unwrap_or(Node::Null);
                if !self
                    .depots
                    .borrow()
                    .iter()
                    .any(|(account, _)| *account == named)
                {
                    return Ok(SemanticCommandResult::undeclared().with_consistency(token));
                }
                self.rows.borrow_mut().push(Row::from([
                    ("entry_id".to_owned(), id.clone()),
                    ("account_id".to_owned(), named),
                    ("cents".to_owned(), input("cents").unwrap_or(Node::Null)),
                    ("state".to_owned(), Node::Text("Posted".into())),
                ]));
                SemanticCommandResult::took(outcome(&command, "posted")).emitting(
                    ObservedEvent::new("demo.ledger.EntryPosted".parse().unwrap())
                        .with("entry_id", id),
                )
            }
            ("demo.goals.OpenObjective", 3) => {
                self.depots
                    .borrow_mut()
                    .push((id.clone(), input("goal").unwrap_or(Node::Null)));
                SemanticCommandResult::took(outcome(&command, "opened")).emitting(
                    ObservedEvent::new("demo.goals.ObjectiveOpened".parse().unwrap())
                        .with("objective_id", id),
                )
            }
            ("demo.goals.IntegrateCandidate", 3) => {
                let named = request
                    .input
                    .get("objective_id")
                    .cloned()
                    .unwrap_or(Node::Null);
                // A missing objective is the `no-objective` refusal, which no aggregate scenario
                // sends; it is answered as nothing declared.
                let Some(goal) = self.read_related(&named) else {
                    return Ok(SemanticCommandResult::undeclared().with_consistency(token));
                };
                self.rows.borrow_mut().push(Row::from([
                    ("evaluation_id".to_owned(), id.clone()),
                    ("objective_id".to_owned(), named),
                    ("goal".to_owned(), goal),
                    ("cents".to_owned(), input("cents").unwrap_or(Node::Null)),
                    ("state".to_owned(), Node::Text("Recorded".into())),
                ]));
                SemanticCommandResult::took(outcome(&command, "integrated")).emitting(
                    ObservedEvent::new("demo.goals.Evaluated".parse().unwrap())
                        .with("evaluation_id", id),
                )
            }
            ("demo.window.RecordCall", 5) => {
                let mut row = Row::new();
                for field in ["queue_id", "started_at", "duration_ms"] {
                    row.insert(field.to_owned(), input(field).unwrap_or(Node::Null));
                }
                row.insert("call_id".into(), id.clone());
                row.insert("state".into(), Node::Text("Recorded".into()));
                self.rows.borrow_mut().push(row);
                SemanticCommandResult::took(outcome(&command, "recorded")).emitting(
                    ObservedEvent::new("demo.window.CallRecorded".parse().unwrap())
                        .with("call_id", id),
                )
            }
            ("demo.calls.RecordCall", 4) => {
                let mut row = Row::new();
                for field in ["queue_id", "abandoned", "duration_ms"] {
                    row.insert(field.to_owned(), input(field).unwrap_or(Node::Null));
                }
                row.insert("call_id".into(), id.clone());
                row.insert("state".into(), Node::Text("Recorded".into()));
                self.rows.borrow_mut().push(row);
                SemanticCommandResult::took(outcome(&command, "recorded")).emitting(
                    ObservedEvent::new("demo.calls.CallRecorded".parse().unwrap())
                        .with("call_id", id),
                )
            }
            (other, _) => return Err(TargetError::unsupported("command", other)),
        };
        Ok(result.with_consistency(token))
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let name = request.view.to_string();
        let Some(view) = self.views().iter().find(|view| view.name == name) else {
            return Err(TargetError::unsupported("view", &name));
        };
        Ok(SemanticViewResult::of(self.query(view, &request.params)))
    }
    fn observe_events(
        &self,
        _: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Ok(Vec::new())
    }
    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        Err(TargetError::unsupported("external outcome", "none"))
    }
    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
        Err(TargetError::unsupported("redelivery", "none"))
    }
}

/// Every scenario's verdict against a store: `passed`, `failed`, `error` or `unsupported`.
fn run_store(suite: &ess_conformance::AdmittedSuite, model: Model, fault: Fault) -> Verdicts {
    let target = Store::new(model, fault);
    ess_conformance::Runner::for_suite(suite.suite())
        .run_admitted(suite, &target)
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| {
            let status = match result.status {
                ess_conformance::report::Status::Passed => "passed",
                ess_conformance::report::Status::Failed => "failed",
                ess_conformance::report::Status::Error => "error",
                ess_conformance::report::Status::Unsupported => "unsupported",
            };
            (result.scenario.to_string(), status.to_owned())
        })
        .collect()
}

type Verdicts = BTreeMap<String, String>;

// ---- the tests -------------------------------------------------------------------------------
// Nothing below is compiled into the WASM host.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::{
    scenario::{ScenarioId, ScenarioInitialState, ViewExpectation},
    synthesize::{synthesize, Synthesis},
    AdmittedSuite, ConformanceScenario, ConformanceSuite, ScenarioStep, ScenarioValue,
};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};

const WORK_YAML: &str = include_str!("fixtures/aggregate-group-selection.yaml");
const DEPOTS_YAML: &str = include_str!("fixtures/aggregate-group-selection-copied.yaml");
const DIRECT_YAML: &str = include_str!("fixtures/aggregate-group-parameter.yaml");
const COPIED_YAML: &str = include_str!("fixtures/aggregate-copied-group-parameter.yaml");
const STATES_YAML: &str = include_str!("fixtures/aggregate-state-groups.yaml");
const LEDGER_YAML: &str = include_str!("fixtures/aggregate-group-selection-owner.yaml");
const GOALS_YAML: &str = include_str!("fixtures/aggregate-group-selection-guarded.yaml");
const LIST_YAML: &str = include_str!("fixtures/aggregate-list-parameter.yaml");
const WINDOW_YAML: &str = include_str!("fixtures/aggregate-timestamp-range.yaml");

fn yaml(model: Model) -> &'static str {
    match model {
        Model::Work => WORK_YAML,
        Model::Depots => DEPOTS_YAML,
        Model::Direct => DIRECT_YAML,
        Model::Copied => COPIED_YAML,
        Model::States => STATES_YAML,
        Model::Ledger => LEDGER_YAML,
        Model::Goals => GOALS_YAML,
        Model::Lists => LIST_YAML,
        Model::Window => WINDOW_YAML,
    }
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("work.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn synthesis(text: &str) -> Synthesis {
    synthesize(&ir(text))
}

/// The aggregate refusals (`ESS-SYNTH-016`, `ESS-SYNTH-017`) a synthesis gives, rendered.
fn aggregate_refusals(result: &Synthesis) -> Vec<String> {
    result
        .refusals
        .iter()
        .filter(|refusal| {
            matches!(
                refusal.code().to_string().as_str(),
                "ESS-SYNTH-016" | "ESS-SYNTH-017"
            )
        })
        .map(ToString::to_string)
        .collect()
}

fn aggregate_ids(suite: &ConformanceSuite) -> Vec<String> {
    suite
        .scenarios
        .keys()
        .filter(|id| matches!(id, ScenarioId::Aggregate { .. }))
        .map(ToString::to_string)
        .collect()
}

fn scenario<'a>(suite: &'a ConformanceSuite, id: &str) -> &'a ConformanceScenario {
    suite
        .scenarios
        .iter()
        .find(|(held, _)| held.to_string() == id)
        .map_or_else(
            || {
                panic!(
                    "no scenario {id}; the suite holds {:?}",
                    aggregate_ids(suite)
                )
            },
            |(_, scenario)| scenario,
        )
}

/// The suite of a fixture's aggregate scenarios only, admitted.
fn aggregate_suite(model: Model) -> AdmittedSuite {
    let mut suite = synthesis(yaml(model)).suite;
    assert_eq!(
        suite.provenance.scenario_initial_state,
        Some(ScenarioInitialState::Empty)
    );
    assert_eq!(
        suite.provenance.suite_version.to_string(),
        "ess-conformance/34"
    );
    suite
        .scenarios
        .retain(|id, _| matches!(id, ScenarioId::Aggregate { .. }));
    // A lane over no scenario agrees with every other lane and checks nothing.
    assert_ne!(
        suite.scenarios.len(),
        0,
        "{}: no aggregate scenario to run",
        model.name()
    );
    AdmittedSuite::from_suite(&suite).unwrap_or_else(|error| panic!("{error}"))
}

/// One read: the parameters it binds, and what it expects.
type Read = (BTreeMap<String, ScenarioValue>, Vec<ViewExpectation>);

/// The reads of a scenario, in order.
fn reads(scenario: &ConformanceScenario) -> Vec<Read> {
    let mut out: Vec<Read> = Vec::new();
    for step in &scenario.steps {
        match step {
            ScenarioStep::QueryView { params, .. } => out.push((params.clone(), Vec::new())),
            ScenarioStep::ExpectView { expectation, .. } => out
                .last_mut()
                .expect("an expectation follows a query")
                .1
                .push(expectation.clone()),
            _ => {}
        }
    }
    out
}

fn text(value: &str) -> ScenarioValue {
    ScenarioValue::literal(Node::Text(value.to_owned()))
}

fn num(value: &str) -> ScenarioValue {
    ScenarioValue::literal(number(value))
}

fn fields(pairs: &[(&str, ScenarioValue)]) -> BTreeMap<String, ScenarioValue> {
    pairs
        .iter()
        .map(|(key, value)| ((*key).to_owned(), value.clone()))
        .collect()
}

fn contains(pairs: &[(&str, ScenarioValue)]) -> ViewExpectation {
    ViewExpectation::Contains {
        fields: fields(pairs),
    }
}

fn excludes(pairs: &[(&str, ScenarioValue)]) -> ViewExpectation {
    ViewExpectation::Excludes {
        fields: fields(pairs),
    }
}

fn rows(n: usize) -> ViewExpectation {
    ViewExpectation::Counts {
        at_least: Some(n),
        at_most: Some(n),
    }
}

// ---- synthesis: the issues' own sources --------------------------------------------------------

/// The red: each of the three sources validates and, on the code before this change, synthesized
/// no aggregate scenario (`ESS-SYNTH-017` twice, `ESS-SYNTH-016` once).
#[test]
fn the_issues_sources_synthesize_their_aggregate_scenarios() {
    let mut missing = Vec::new();
    for (model, id) in [
        (Model::Direct, "demo.work.ByTeam/aggregate"),
        (Model::Copied, "demo.work.ByTeam/aggregate"),
        (Model::States, "demo.work.ByState/aggregate"),
    ] {
        let result = synthesis(yaml(model));
        assert_eq!(
            result.suite.provenance.scenario_initial_state,
            Some(ScenarioInitialState::Empty)
        );
        let refused = aggregate_refusals(&result);
        if !aggregate_ids(&result.suite).contains(&id.to_owned()) || !refused.is_empty() {
            missing.push(format!(
                "{}: {id} missing, refused {refused:#?}",
                model.name()
            ));
        }
    }
    assert!(
        missing.is_empty(),
        "AGGREGATE_GROUP_SELECTION_GAP:\n{}",
        missing.join("\n")
    );
}

/// Direct: `team == param.team` over `group_by: [team]`. A's three rows and B's one are each
/// selected exactly, the other group excluded and the row count asserted; a team no row holds
/// answers no row. The filter is the selector alone, so no refuted row is forced.
#[test]
fn a_direct_group_parameter_selects_each_group_exactly() {
    let suite = synthesis(DIRECT_YAML).suite;
    let found = reads(scenario(&suite, "demo.work.ByTeam/aggregate"));
    let a = text("demo.work.ByTeam/A");
    let b = text("demo.work.ByTeam/B");
    let none = text("demo.work.ByTeam/unselected");
    assert_eq!(
        found,
        vec![
            (
                fields(&[("team", a.clone())]),
                vec![
                    contains(&[("team", a.clone()), ("count", num("3"))]),
                    excludes(&[("team", b.clone())]),
                    rows(1),
                ]
            ),
            (
                fields(&[("team", b.clone())]),
                vec![
                    excludes(&[("team", a.clone())]),
                    contains(&[("team", b.clone()), ("count", num("1"))]),
                    rows(1),
                ]
            ),
            (
                fields(&[("team", none)]),
                vec![excludes(&[("team", a)]), excludes(&[("team", b)]), rows(0),]
            ),
        ]
    );
}

/// Copied: the key is copied from the depot `input.depot_id` names. Every depot is opened before
/// the first item, each item names its own, and each team is selected exactly.
#[test]
fn a_copied_group_parameter_selects_each_group_exactly() {
    let suite = synthesis(COPIED_YAML).suite;
    let scenario = scenario(&suite, "demo.work.ByTeam/aggregate");
    let mut depots = Vec::new();
    let mut items = 0;
    for step in &scenario.steps {
        if let ScenarioStep::ExecuteCommand { command, input, .. } = step {
            match command.to_string().as_str() {
                "demo.work.OpenDepot" => {
                    assert_eq!(items, 0, "a depot opened after an item");
                    depots.push(input["team"].clone());
                }
                "demo.work.OpenItem" => {
                    assert!(
                        matches!(input["depot_id"], ScenarioValue::Instance { .. }),
                        "{input:?}"
                    );
                    items += 1;
                }
                other => panic!("{other}"),
            }
        }
    }
    assert_eq!(depots.len(), items, "one depot per item");
    let a = text("demo.work.ByTeam/A");
    let b = text("demo.work.ByTeam/B");
    assert_eq!(
        reads(scenario),
        vec![
            (
                fields(&[("team", a.clone())]),
                vec![
                    contains(&[
                        ("team", a.clone()),
                        ("count", num("3")),
                        ("total", num("5"))
                    ]),
                    excludes(&[("team", b.clone())]),
                    rows(1),
                ]
            ),
            (
                fields(&[("team", b.clone())]),
                vec![
                    excludes(&[("team", a.clone())]),
                    contains(&[
                        ("team", b.clone()),
                        ("count", num("1")),
                        ("total", num("85"))
                    ]),
                    rows(1),
                ]
            ),
            (
                fields(&[("team", text("demo.work.ByTeam/unselected"))]),
                vec![excludes(&[("team", a)]), excludes(&[("team", b)]), rows(0)]
            ),
        ]
    );
}

/// State only: one read, every reached state's exact count, and the number of rows. The pattern's
/// A group takes the first state in name order, `Done`, reached by `demo.work.Finish` and never
/// written directly; B's one row stays `Open`.
#[test]
fn a_state_only_grouping_is_observed_exactly() {
    let suite = synthesis(STATES_YAML).suite;
    let scenario = scenario(&suite, "demo.work.ByState/aggregate");
    assert!(scenario.steps.iter().any(|step| matches!(
        step,
        ScenarioStep::ExecuteCommand { command, .. } if command.to_string() == "demo.work.Finish"
    )));
    assert_eq!(
        reads(scenario),
        vec![(
            BTreeMap::new(),
            vec![
                contains(&[("state", text("Done")), ("count", num("3"))]),
                contains(&[("state", text("Open")), ("count", num("1"))]),
                rows(2),
            ]
        )]
    );
}

// ---- synthesis: the wider fixture ---------------------------------------------------------------

#[test]
fn every_view_of_the_wider_fixtures_is_witnessed() {
    for (model, expected) in [
        (
            Model::Work,
            vec![
                "demo.work.ByBucket/aggregate",
                "demo.work.ByKind/aggregate",
                "demo.work.ByState/aggregate",
                "demo.work.ByTeam/aggregate",
                "demo.work.ByTeamChannel/aggregate",
                "demo.work.ByTeamState/aggregate",
                "demo.work.ByTeamUrgency/aggregate",
                "demo.work.ByUrgency/aggregate",
                "demo.work.ByUrgent/aggregate",
                "demo.work.OpenByTeam/aggregate",
                "demo.work.TeamsInLane/aggregate",
                "demo.work.TopOpen/aggregate",
            ],
        ),
        (Model::Ledger, vec!["demo.ledger.ByAccount/aggregate"]),
        (
            Model::Goals,
            vec![
                "demo.goals.EvaluationsByGoal/aggregate",
                "demo.goals.EvidenceEvaluations/aggregate",
            ],
        ),
        (
            Model::Depots,
            vec![
                "demo.work.ByTeam/aggregate",
                "demo.work.ItemsByState/aggregate",
            ],
        ),
    ] {
        let result = synthesis(yaml(model));
        assert_eq!(
            aggregate_refusals(&result),
            Vec::<String>::new(),
            "{}",
            model.name()
        );
        assert_eq!(aggregate_ids(&result.suite), expected, "{}", model.name());
    }
}

/// Joint keys, an `Optional` absent group, two selectors and a value above 2^53.
#[test]
fn joint_keys_absent_groups_two_selectors_and_large_values_are_exact() {
    let suite = synthesis(WORK_YAML).suite;
    // `team == param.team` over `[team, channel]`: B's team answers B, B₂ and the group whose
    // channel is absent, each exactly.
    let channel = reads(scenario(&suite, "demo.work.ByTeamChannel/aggregate"));
    let b = text("demo.work.ByTeamChannel/B");
    let read_b = channel
        .iter()
        .find(|(params, _)| params.get("team") == Some(&b))
        .expect("B's team is selected");
    assert!(read_b.1.contains(&rows(3)), "{read_b:?}");
    assert!(
        read_b.1.iter().any(|expectation| matches!(
            expectation,
            ViewExpectation::Contains { fields }
                if fields.get("channel") == Some(&ScenarioValue::literal(Node::Null))
                    && fields.get("count") == Some(&num("1"))
        )),
        "{read_b:?}"
    );
    // Two selectors: every arranged pair, and a valid pair no group holds.
    let pairs = reads(scenario(&suite, "demo.work.ByTeamUrgency/aggregate"));
    assert!(
        pairs.iter().any(|(params, expectations)| params.len() == 2
            && expectations.contains(&rows(0))
            && params.get("team") == Some(&text("demo.work.ByTeamUrgency/A"))),
        "{pairs:#?}"
    );
    assert!(pairs.iter().filter(|(_, e)| !e.contains(&rows(0))).count() >= 2);
    // A `Boolean` key whose both values are arranged has no valid value no group holds: two reads,
    // and no nonmatching one.
    let urgency = reads(scenario(&suite, "demo.work.ByUrgency/aggregate"));
    assert_eq!(urgency.len(), 2, "{urgency:#?}");
    assert!(urgency.iter().all(|(_, e)| e.contains(&rows(1))));
    // 2^53 + 1 is sent and expected digit for digit, and a value no row holds answers nothing.
    let bucket = scenario(&suite, "demo.work.ByBucket/aggregate");
    let json = serde_json::to_string(&bucket.steps).unwrap();
    assert!(json.contains(BUCKET), "{json}");
    let bucket_reads = reads(bucket);
    assert_eq!(bucket_reads.len(), 2, "{bucket_reads:#?}");
    assert_eq!(bucket_reads[0].0, fields(&[("bucket", num(BUCKET))]));
    assert!(bucket_reads[1].1.contains(&rows(0)));
    // A non-group scope keeps its `in`/`out` values; the refuted row moved out of the lane.
    let lane = scenario(&suite, "demo.work.TeamsInLane/aggregate");
    let mut lanes: Vec<ScenarioValue> = Vec::new();
    for step in &lane.steps {
        if let ScenarioStep::ExecuteCommand { input, .. } = step {
            if let Some(value) = input.get("lane") {
                if !lanes.contains(value) {
                    lanes.push(value.clone());
                }
            }
        }
    }
    assert_eq!(
        lanes,
        vec![
            text("demo.work.TeamsInLane/in"),
            text("demo.work.TeamsInLane/out")
        ]
    );
    assert!(reads(lane)
        .iter()
        .all(|(params, _)| params.get("lane") == Some(&text("demo.work.TeamsInLane/in"))));
}

/// Views nothing scopes — grouped by an enum or a `Boolean` alone, or ungrouped with no `count`
/// or `sum` — are exact under `Empty` authority: every group, `avg` rounded at the sixth place
/// over a mean that truncation reports differently, and `count_distinct` over a column with
/// repeats (A's lanes 101, 101, 101, 103, 107, 113).
#[test]
fn enum_boolean_and_ungrouped_views_are_observed_exactly() {
    let suite = synthesis(WORK_YAML).suite;
    let bool_value = |value: bool| ScenarioValue::literal(Node::Bool(value));
    assert_eq!(
        reads(scenario(&suite, "demo.work.ByKind/aggregate")),
        vec![(
            BTreeMap::new(),
            vec![
                contains(&[
                    ("kind", text("Rush")),
                    ("count", num("6")),
                    ("mean", num("7.666667")),
                    ("lanes", num("4")),
                ]),
                contains(&[
                    ("kind", text("Plain")),
                    ("count", num("1")),
                    ("mean", num("85")),
                    ("lanes", num("1")),
                ]),
                contains(&[
                    ("kind", text("Hold")),
                    ("count", num("1")),
                    ("mean", num("87")),
                    ("lanes", num("1")),
                ]),
                rows(3),
            ]
        )]
    );
    assert_eq!(
        reads(scenario(&suite, "demo.work.ByUrgent/aggregate")),
        vec![(
            BTreeMap::new(),
            vec![
                contains(&[("urgent", bool_value(false)), ("count", num("3"))]),
                contains(&[("urgent", bool_value(true)), ("count", num("1"))]),
                rows(2),
            ]
        )]
    );
    // The refuted row (`Done`, 97) is in no read.
    assert_eq!(
        reads(scenario(&suite, "demo.work.TopOpen/aggregate")),
        vec![(
            BTreeMap::new(),
            vec![contains(&[("top", num("3")), ("low", num("1"))]), rows(1)]
        )]
    );
}

fn executed(scenario: &ConformanceScenario, command: &str) -> Vec<BTreeMap<String, ScenarioValue>> {
    scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand {
                command: sent,
                input,
                ..
            } if sent.to_string() == command => Some(input.clone()),
            _ => None,
        })
        .collect()
}

/// A selector over an owner's identity: each read binds the account the arrangement created, as
/// the instance it is, and no nonmatching account is invented — the target generates identities.
#[test]
fn a_selector_over_an_owner_identity_selects_each_owner_exactly() {
    let suite = synthesis(LEDGER_YAML).suite;
    let scenario = scenario(&suite, "demo.ledger.ByAccount/aggregate");
    assert_eq!(executed(scenario, "demo.ledger.OpenAccount").len(), 2);
    let found = reads(scenario);
    assert_eq!(found.len(), 2, "{found:#?}");
    let a = found[0].0["account_id"].clone();
    let b = found[1].0["account_id"].clone();
    assert!(matches!(a, ScenarioValue::Instance { .. }), "{a:?}");
    assert!(matches!(b, ScenarioValue::Instance { .. }), "{b:?}");
    assert_ne!(a, b);
    assert_eq!(
        found,
        vec![
            (
                fields(&[("account_id", a.clone())]),
                vec![
                    contains(&[
                        ("account_id", a.clone()),
                        ("count", num("3")),
                        ("total", num("5"))
                    ]),
                    excludes(&[("account_id", b.clone())]),
                    rows(1),
                ]
            ),
            (
                fields(&[("account_id", b.clone())]),
                vec![
                    excludes(&[("account_id", a)]),
                    contains(&[("account_id", b), ("count", num("1")), ("total", num("85"))]),
                    rows(1),
                ]
            ),
        ]
    );
}

/// The #361 comment's shape: a creating command a related guard refuses for a missing objective
/// (`no-objective`), a group key it copies from that objective, and a parameter over the
/// objective. On the code before this change `EvidenceEvaluations` was the group-parameter
/// refusal (`ESS-SYNTH-017`, "the parameter `objective_id` is read other than by one top-level
/// … conjunct over a field that is not a group key") and `EvaluationsByGoal`, the copied key
/// without the parameter, already synthesized: the guard was not the cause. Now every objective is
/// opened before the first candidate, every candidate names one and takes `integrated`, and each
/// read selects one objective exactly.
#[test]
fn the_comments_related_guard_case_is_a_group_selection() {
    let result = synthesis(GOALS_YAML);
    assert_eq!(aggregate_refusals(&result), Vec::<String>::new());
    let scenario = scenario(&result.suite, "demo.goals.EvidenceEvaluations/aggregate");
    let mut integrated = 0;
    let mut opened = 0;
    for step in &scenario.steps {
        match step {
            ScenarioStep::ExecuteCommand { command, input, .. } => {
                match command.to_string().as_str() {
                    "demo.goals.OpenObjective" => {
                        assert_eq!(integrated, 0, "an objective opened after a candidate");
                        opened += 1;
                    }
                    "demo.goals.IntegrateCandidate" => {
                        assert!(
                            matches!(input["objective_id"], ScenarioValue::Instance { .. }),
                            "{input:?}"
                        );
                        integrated += 1;
                    }
                    other => panic!("{other}"),
                }
            }
            ScenarioStep::ExpectOutcome { outcome }
                if outcome.to_string().contains("Integrate") =>
            {
                assert!(outcome.to_string().ends_with("integrated"), "{outcome}");
            }
            _ => {}
        }
    }
    // Each candidate is arranged on an objective of its own holding the goal its group copies
    // (`arrange_related`), so each objective is one group and one selection: no objective is
    // shared, and none invented for a nonmatching read.
    assert_eq!(
        opened, integrated,
        "{opened} objectives, {integrated} candidates"
    );
    let found = reads(scenario);
    assert_eq!(found.len(), opened, "{found:#?}");
    for (params, expectations) in &found {
        assert!(
            matches!(params["objective_id"], ScenarioValue::Instance { .. }),
            "{params:?}"
        );
        assert!(expectations.contains(&rows(1)), "{expectations:?}");
    }
}

/// A binding that moves the view's rows after the command that triggers it returns: no exact
/// number without a causal cut after its effects, so the view keeps a refusal that names it.
#[test]
fn a_view_a_binding_changes_keeps_a_named_refusal() {
    let bound = format!(
        "{STATES_YAML}bindings:\n  - id: finish-on-open\n    when: {{event: demo.work.Opened}}\n    invoke: {{command: demo.work.Finish}}\n    mapping: {{id: event.id}}\n    delivery: at_least_once\n    on_failure: drop\n"
    );
    let result = synthesis(&bound);
    let refused = aggregate_refusals(&result);
    assert_eq!(refused.len(), 1, "{refused:#?}");
    assert!(refused[0].contains("ESS-SYNTH-017"), "{refused:#?}");
    assert!(refused[0].contains("finish-on-open"), "{refused:#?}");
    assert!(refused[0].contains("binding cut"), "{refused:#?}");
}

/// A precondition that opens an item before every scenario: rows the arrangement does not
/// determine, refused by name rather than counted.
#[test]
fn a_view_a_precondition_feeds_keeps_a_named_refusal() {
    let seeded = format!(
        "{STATES_YAML}preconditions:\n  - command: demo.work.Open\n    input: {{team: seeded, cents: 17}}\n"
    );
    let result = synthesis(&seeded);
    let refused = aggregate_refusals(&result);
    assert_eq!(refused.len(), 1, "{refused:#?}");
    assert!(refused[0].contains("precondition"), "{refused:#?}");
    assert!(refused[0].contains("demo.work.Open"), "{refused:#?}");
}

// ---- beyond10x/ess#438: list-typed view parameters ---------------------------------------------

/// The source, compiled; or every refusal, with the stable code each compiles to, as one text.
fn validated(text: &str) -> Result<EssIr, String> {
    let raw = RawSpecFile::parse(text).map_err(|error| error.to_string())?;
    let spec = Specification::assemble([(Source::new("work.yaml"), raw)]).map_err(|errors| {
        let codes: Vec<String> = ess_compiler::resolve::diagnose(&errors, &SourceMap::new())
            .as_slice()
            .iter()
            .map(|diagnostic| format!("{} {diagnostic}", diagnostic.code))
            .collect();
        format!("{errors}\n{}", codes.join("\n"))
    })?;
    compile(&spec, &SourceMap::new()).map_err(|diagnostics| diagnostics.to_string())
}

fn refused(text: &str) -> String {
    match validated(text) {
        Ok(_) => panic!("must be refused:\n{text}"),
        Err(error) => error,
    }
}

/// `text` with `from` replaced by `to` once; fails where `from` is not there.
fn replaced(text: &str, from: &str, to: &str) -> String {
    assert!(text.contains(from), "the source holds `{from}`");
    text.replacen(from, to, 1)
}

const LIST_FILTER: &str = "filter: {exists: {in: param.queues, as: q, that: queue_id == q}}";

/// The JSON of `view`'s compiled filter.
fn filter_json(ir: &EssIr, view: &str) -> String {
    let (_, found) = ir
        .views()
        .iter()
        .find(|(name, _)| name.to_string() == view)
        .unwrap_or_else(|| panic!("no view {view}"));
    serde_json::to_string(&found.filter).unwrap()
}

/// A membership operand that names a parameter or an input is the text it spells, so `{in:
/// param.queues}` never asks whether the field is in the list the caller sent. Refused in every
/// operator spelling, naming the quantifier that does ask it.
#[test]
fn membership_operand_naming_a_parameter_is_refused_with_quantifier_hint() {
    for operator in ["in", "any_of", "one_of", "not_in", "none_of"] {
        let error = refused(&replaced(
            LIST_YAML,
            LIST_FILTER,
            &format!("filter: {{queue_id: {{{operator}: param.queues}}}}"),
        ));
        assert!(error.contains("[type_mismatch]"), "{operator}: {error}");
        assert!(
            error.contains("exists: {in: param.queues, as: x, that: queue_id == x}"),
            "{operator}: {error}"
        );
    }
    // A string field admits the text, so the type alone never refused it.
    let text_field = replaced(
        &replaced(
            LIST_YAML,
            "      - {name: duration_ms, type: Integer}\n    lifecycle",
            "      - {name: duration_ms, type: Integer}\n      - {name: label, type: String}\n    lifecycle",
        ),
        LIST_FILTER,
        "filter: {label: [param.queues]}",
    );
    let error = refused(&text_field);
    assert!(
        error.contains("exists: {in: param.queues, as: x, that: label == x}"),
        "{error}"
    );
    // A command guard over a list input.
    let guarded = replaced(
        &replaced(
            &replaced(
                LIST_YAML,
                "commands:\n",
                "errors:\n  - {name: demo.calls.Listed, summary: The queue is listed., fields: []}\ncommands:\n",
            ),
            "      - {name: duration_ms, type: Integer}\n    outcomes:",
            "      - {name: duration_ms, type: Integer}\n      - {name: allowed, type: List<Integer>}\n    outcomes:",
        ),
        "      - name: recorded\n",
        "      - name: listed\n        when: {queue_id: {in: input.allowed}}\n        error: demo.calls.Listed\n      - name: recorded\n",
    );
    let error = refused(&guarded);
    assert!(error.contains("[type_mismatch]"), "{error}");
    assert!(
        error.contains("exists: {in: input.allowed, as: x, that: queue_id == x}"),
        "{error}"
    );
}

/// The class behind the membership refusal's hint (correction round 1): whatever the operand names
/// — a list or a single value, a view parameter or a command input, under `in` or `in_ignore_case`
/// — the form the hint names is one the author can apply, and applied it validates.
#[test]
fn membership_hint_names_a_form_that_validates() {
    let labelled = replaced(
        LIST_YAML,
        "      - {name: duration_ms, type: Integer}\n    lifecycle",
        "      - {name: duration_ms, type: Integer}\n      - {name: label, type: String}\n    lifecycle",
    );
    let listed_params = "params: [{name: queues, type: List<Integer>}]";
    let view = |params: &str, filter: &str| {
        replaced(
            &labelled,
            &format!("{listed_params}\n    {LIST_FILTER}"),
            &format!("{params}\n    filter: {filter}"),
        )
    };
    let guarded = |when: &str| {
        replaced(
            &replaced(
                &replaced(
                    LIST_YAML,
                    "commands:\n",
                    "errors:\n  - {name: demo.calls.Listed, summary: The queue is listed., fields: []}\ncommands:\n",
                ),
                "      - {name: duration_ms, type: Integer}\n    outcomes:",
                "      - {name: duration_ms, type: Integer}\n      - {name: allowed, type: List<Integer>}\n      - {name: limit, type: Integer}\n    outcomes:",
            ),
            "      - name: recorded\n",
            &format!(
                "      - name: listed\n        when: {when}\n        error: demo.calls.Listed\n      - name: recorded\n"
            ),
        )
    };
    let cases = [
        (
            view(listed_params, "{queue_id: {in: param.queues}}"),
            "{queue_id: {in: param.queues}}",
            "exists: {in: param.queues, as: x, that: queue_id == x}",
        ),
        (
            view(
                "params: [{name: queue, type: Integer}]",
                "{queue_id: {in: param.queue}}",
            ),
            "{queue_id: {in: param.queue}}",
            "queue_id == param.queue",
        ),
        (
            view(
                "params: [{name: labels, type: List<String>}]",
                "{label: {in_ignore_case: [param.labels]}}",
            ),
            "{label: {in_ignore_case: [param.labels]}}",
            "exists: {in: param.labels, as: x, that: label == x}",
        ),
        (
            view(
                "params: [{name: wanted, type: String}]",
                "{label: {in_ignore_case: [param.wanted]}}",
            ),
            "{label: {in_ignore_case: [param.wanted]}}",
            "label == param.wanted",
        ),
        (
            guarded("{queue_id: {in: input.allowed}}"),
            "{queue_id: {in: input.allowed}}",
            "exists: {in: input.allowed, as: x, that: queue_id == x}",
        ),
        (
            guarded("{queue_id: {not_in: input.limit}}"),
            "{queue_id: {not_in: input.limit}}",
            "queue_id != input.limit",
        ),
        (
            view(listed_params, "{queue_id: {none_of: param.queues}}"),
            "{queue_id: {none_of: param.queues}}",
            "not: {exists: {in: param.queues, as: x, that: queue_id == x}}",
        ),
    ];
    for (text, written, hint) in cases {
        let error = refused(&text);
        assert!(error.contains("[type_mismatch]"), "{written}: {error}");
        assert!(error.contains(hint), "{written} hints `{hint}`: {error}");
        // A mapping form is written as a flow mapping, a comparison as it stands.
        let form = if hint.contains(": ") {
            format!("{{{hint}}}")
        } else {
            hint.to_owned()
        };
        let applied = replaced(&text, written, &form);
        if let Err(error) = validated(&applied) {
            panic!("{written}: the hinted `{hint}` is refused:\n{error}");
        }
    }
}

const LONG_VIEW: &str = "  - name: demo.calls.Long
    source: demo.calls.Call
    consistency: read_your_writes
    params: [{name: limit_ms, type: Integer}]
    filter: duration_ms > param.limit_ms + 1000
    group_by: [queue_id]
    fields:
      - {name: queue_id, type: Integer}
      - {name: calls, type: Integer, aggregate: {count: {}}}
";

/// A parameter scaled by a constant is not an expression ESS has: the operand is the text it
/// spells, refused as `type_mismatch`, and the refusal says what to write instead.
#[test]
fn comparison_operand_with_trailing_text_after_a_parameter_is_refused() {
    let scaled = format!(
        "{LIST_YAML}{}",
        LONG_VIEW
            .replace("limit_ms, type", "limit_s, type")
            .replace("param.limit_ms + 1000", "param.limit_s * 1000")
    );
    let error = refused(&scaled);
    assert!(error.contains("[type_mismatch]"), "{error}");
    assert!(
        error.contains("Text literal `param.limit_s * 1000`"),
        "{error}"
    );
    assert!(
        error.contains("one constant offset") && error.contains("`param.limit_s + 1000`"),
        "the hint names the offset form: {error}"
    );
    assert!(
        error.contains("declare the parameter in the unit the field is stored in"),
        "the hint names the stored unit: {error}"
    );
}

/// One constant offset of a parameter keeps validating, and compiles to the bytes it did.
#[test]
fn parameter_offset_stays_admitted() {
    let ir =
        validated(&format!("{LIST_YAML}{LONG_VIEW}")).unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(
        filter_json(&ir, "demo.calls.Long"),
        r#"{"duration_ms":{"gt":{"offset":{"add":1000.0,"fact":"param.limit_ms"}}}}"#
    );
}

/// Literal membership lists, in the operator form and the list shorthand, keep their meaning and
/// their bytes.
#[test]
fn membership_literal_lists_keep_meaning_and_bytes() {
    let ir = validated(&replaced(
        LIST_YAML,
        &format!("params: [{{name: queues, type: List<Integer>}}]\n    {LIST_FILTER}"),
        "filter: {queue_id: {in: [1, 2]}}",
    ))
    .unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(
        filter_json(&ir, "demo.calls.InQueues"),
        r#"{"queue_id":{"any_of":[1.0,2.0]}}"#
    );
    let ir = validated(&replaced(
        WORK_YAML,
        "filter: team == param.team\n",
        "filter: {all: [team == param.team, {channel: [Web, Store]}]}\n",
    ))
    .unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(
        filter_json(&ir, "demo.work.ByTeam"),
        r#"{"all":["team == param.team",{"channel":{"any_of":["Web","Store"]}}]}"#
    );
}

/// The fixture states the three shapes the request needed with the constructs ESS has: membership
/// through `exists`, an empty list read as unfiltered, and a switch that is `false` when left out.
#[test]
fn list_parameter_quantifier_validates() {
    let ir = validated(LIST_YAML).unwrap_or_else(|error| panic!("{error}"));
    for (view, filter) in [
        ("demo.calls.InQueues", "exists"),
        ("demo.calls.InQueuesOrAll", "param.queues.count == 0"),
        ("demo.calls.ByAbandonment", "defined(param.abandoned)"),
    ] {
        let (_, found) = ir
            .views()
            .iter()
            .find(|(name, _)| name.to_string() == view)
            .unwrap_or_else(|| panic!("no view {view}"));
        let written = found.filter.as_ref().expect("a filter").to_string();
        assert!(written.contains(filter), "{view}: {written}");
    }
}

const GUIDE: &str = include_str!("../../../../website/docs/guides/specify/aggregate-views.md");
const GUIDE_HEADING: &str = "## A list parameter, an empty list and a default";

/// The guide section states the list, empty-list and default idioms, says what is not admitted,
/// and every model it fences validates.
#[test]
fn list_parameter_guide_section_states_the_idioms() {
    let marker = format!("\n{GUIDE_HEADING}\n");
    let body = GUIDE
        .split(marker.as_str())
        .nth(1)
        .unwrap_or_else(|| panic!("the guide has no heading `{GUIDE_HEADING}`"))
        .split("\n## ")
        .next()
        .unwrap();
    let normalized = body.split_whitespace().collect::<Vec<_>>().join(" ");
    let missing: Vec<&str> = [
        "exists: {in: param.queues, as: q, that: queue_id == q}",
        "param.queues.count == 0",
        "not defined(param.abandoned)",
        "`default:`",
        "`{in: param.queues}`",
        "not admitted",
    ]
    .into_iter()
    .filter(|phrase| !normalized.contains(phrase))
    .collect();
    assert_eq!(
        missing,
        Vec::<&str>::new(),
        "the section does not state these"
    );
    let fences: Vec<&str> = body
        .split("```yaml\n")
        .skip(1)
        .map(|fence| fence.split("```").next().unwrap())
        .collect();
    assert_ne!(fences.len(), 0, "the section fences a model");
    for fence in fences {
        validated(fence).unwrap_or_else(|error| panic!("{error}\n{fence}"));
    }
}

/// The reads of `view`'s aggregate scenario in the list fixture.
fn list_reads(view: &str) -> Vec<Read> {
    let result = synthesis(LIST_YAML);
    let refused = aggregate_refusals(&result);
    // The default switch reads its parameter twice, which no selection binds: it validates and
    // keeps its named refusal.
    assert_eq!(refused.len(), 1, "{refused:#?}");
    assert!(
        refused[0].contains("ESS-SYNTH-017") && refused[0].contains("demo.calls.ByAbandonment"),
        "{refused:#?}"
    );
    reads(scenario(&result.suite, &format!("{view}/aggregate")))
}

fn list_of(values: &[ScenarioValue]) -> ScenarioValue {
    ScenarioValue::literal(Node::Seq(
        values
            .iter()
            .map(|value| match value {
                ScenarioValue::Literal { value } => value.clone(),
                other => panic!("not a literal: {other:?}"),
            })
            .collect(),
    ))
}

/// The key a `Contains` asserts, where it asserts one.
fn contained_key(expectation: &ViewExpectation) -> Option<ScenarioValue> {
    match expectation {
        ViewExpectation::Contains { fields } => fields.get("queue_id").cloned(),
        _ => None,
    }
}

/// `[key_A]` selects group A exactly and leaves the decoy B out; `[]` is no such read without the
/// empty-list disjunct.
#[test]
fn aggregate_list_parameter_selects_listed_groups() {
    for view in ["demo.calls.InQueues", "demo.calls.InQueuesOrAll"] {
        let found = list_reads(view);
        let (params, expectations) = found.first().expect("a read");
        let contained: Vec<ScenarioValue> = expectations.iter().filter_map(contained_key).collect();
        assert_eq!(contained.len(), 1, "{view}: {found:#?}");
        let a = contained[0].clone();
        assert_eq!(
            params,
            &fields(&[("queues", list_of(std::slice::from_ref(&a)))]),
            "{view}"
        );
        assert!(expectations.contains(&rows(1)), "{view}: {expectations:#?}");
        let decoys: Vec<&ViewExpectation> = expectations
            .iter()
            .filter(|expectation| {
                matches!(expectation, ViewExpectation::Excludes { fields }
                    if fields.get("queue_id").is_some_and(|key| *key != a))
            })
            .collect();
        assert_ne!(decoys.len(), 0, "{view}: a decoy group is excluded");
        // Two listed keys, the first of them not A's: a target matching the first element only
        // answers another number of groups.
        assert!(
            found.iter().any(|(params, expectations)| matches!(
                &params["queues"],
                ScenarioValue::Literal { value: Node::Seq(listed) } if listed.len() > 1
            ) && expectations
                .iter()
                .filter_map(contained_key)
                .count()
                > 1),
            "{view}: a read lists several keys: {found:#?}"
        );
    }
}

/// With `param.queues.count == 0` beside the membership, `[]` reads every group; without it, no
/// `[]` read is synthesized.
#[test]
fn aggregate_list_parameter_empty_reads_unfiltered() {
    let empty = fields(&[("queues", ScenarioValue::literal(Node::Seq(Vec::new())))]);
    let all = list_reads("demo.calls.InQueuesOrAll");
    let groups: BTreeSet<String> = all
        .iter()
        .flat_map(|(_, expectations)| expectations.iter().filter_map(contained_key))
        .map(|key| format!("{key:?}"))
        .collect();
    assert!(groups.len() >= 2, "{all:#?}");
    let (_, expectations) = all
        .iter()
        .find(|(params, _)| *params == empty)
        .unwrap_or_else(|| panic!("no `[]` read: {all:#?}"));
    let answered: BTreeSet<String> = expectations
        .iter()
        .filter_map(contained_key)
        .map(|key| format!("{key:?}"))
        .collect();
    assert_eq!(answered, groups, "`[]` asserts every group");
    assert!(
        expectations.contains(&rows(groups.len())),
        "{expectations:#?}"
    );
    let listed = list_reads("demo.calls.InQueues");
    assert!(
        !listed.iter().any(|(params, _)| *params == empty),
        "no `[]` read without the disjunct: {listed:#?}"
    );
}

// ---- beyond10x/ess#439: a window the caller resolved into two instants -------------------------

/// The `started_at` of every row a window scenario creates, as written, and its reads.
fn window_scenario(view: &str) -> (Vec<String>, Vec<Read>) {
    let result = synthesis(WINDOW_YAML);
    assert_eq!(aggregate_refusals(&result), Vec::<String>::new());
    let scenario = scenario(&result.suite, &format!("{view}/aggregate"));
    let spelled = executed(scenario, "demo.window.RecordCall")
        .into_iter()
        .map(|input| match &input["started_at"] {
            ScenarioValue::Literal {
                value: Node::Text(text),
            } => text.clone(),
            other => panic!("{other:?}"),
        })
        .collect();
    (spelled, reads(scenario))
}

fn instant(text: &str) -> Rfc3339Instant {
    Rfc3339Instant::parse_rfc3339(text).unwrap_or_else(|| panic!("an instant: {text}"))
}

/// A read's `from` and `to`, as written.
fn bounds(read: &Read) -> (String, String) {
    let text = |name: &str| match &read.0[name] {
        ScenarioValue::Literal {
            value: Node::Text(text),
        } => text.clone(),
        other => panic!("{name}: {other:?}"),
    };
    (text("from"), text("to"))
}

/// The `calls` every `Contains` of a read asserts, summed.
fn counted(read: &Read) -> i128 {
    read.1
        .iter()
        .filter_map(|expectation| match expectation {
            ViewExpectation::Contains { fields } => match fields.get("calls") {
                Some(ScenarioValue::Literal { value }) => Some(integer(value)),
                _ => None,
            },
            _ => None,
        })
        .sum()
}

/// `started_at >= param.from` and `started_at < param.to` at the top of the filter select rows by
/// their instants: rows a second before `from`, at `from`, a second before `to` and at `to` are
/// created, and each read counts exactly the rows in `[from, to)`.
#[test]
fn aggregate_timestamp_range_parameters_select_rows() {
    for view in ["demo.window.CallsInRange", "demo.window.QueueCallsInRange"] {
        let (spelled, found) = window_scenario(view);
        assert_ne!(found.len(), 0, "{view}: a read");
        for read in &found {
            let (from, to) = bounds(read);
            let (from, to) = (instant(&from), instant(&to));
            let instants: Vec<Rfc3339Instant> = spelled.iter().map(|text| instant(text)).collect();
            for (edge, name) in [
                (from.plus_seconds(-1).unwrap(), "from - 1s"),
                (from, "from"),
                (to.plus_seconds(-1).unwrap(), "to - 1s"),
                (to, "to"),
            ] {
                assert!(
                    instants.contains(&edge),
                    "{view}: a row at {name}: {spelled:?}"
                );
            }
            let inside = instants
                .iter()
                .filter(|at| from <= **at && **at < to)
                .count();
            assert!(inside < instants.len(), "{view}: a row is outside");
            assert_eq!(
                counted(read),
                i128::try_from(inside).unwrap(),
                "{view}: {read:#?}"
            );
        }
    }
}

/// One row is spelled at an offset under which its text sorts on the other side of a bound than its
/// instant does, so a target comparing text counts another number.
#[test]
fn aggregate_timestamp_range_compares_instants_not_text() {
    for view in ["demo.window.CallsInRange", "demo.window.QueueCallsInRange"] {
        let (spelled, found) = window_scenario(view);
        let (from, to) = bounds(&found[0]);
        let by_text = |text: &String| from.as_str() <= text.as_str() && text.as_str() < to.as_str();
        let by_instant =
            |text: &String| instant(&from) <= instant(text) && instant(text) < instant(&to);
        assert!(
            spelled.iter().any(|text| by_text(text) != by_instant(text)),
            "{view}: no row tells text from instant: {spelled:?} in [{from}, {to})"
        );
    }
}

/// A target that ignores `from`, makes `to` inclusive or compares the spellings fails both window
/// observations natively; the Go, TypeScript and WASM lanes below run the same cases.
#[test]
fn aggregate_timestamp_range_faults_fail() {
    let suite = aggregate_suite(Model::Window);
    assert_eq!(
        failed(&run_store(&suite, Model::Window, Fault::None)),
        BTreeSet::new(),
        "the healthy store passes"
    );
    let both: BTreeSet<String> = [
        "demo.window.CallsInRange/aggregate",
        "demo.window.QueueCallsInRange/aggregate",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect();
    for fault in Fault::WINDOWS {
        assert_eq!(
            failed(&run_store(&suite, Model::Window, fault)),
            both,
            "{fault:?}"
        );
    }
}

#[test]
fn synthesis_is_deterministic() {
    for model in Model::ALL {
        let once = synthesis(yaml(model)).suite.to_canonical_json().unwrap();
        let twice = synthesis(yaml(model)).suite.to_canonical_json().unwrap();
        assert_eq!(once, twice, "{}", model.name());
    }
}

// ---- execution -----------------------------------------------------------------------------------

/// What each fault must fail, per wider fixture, and why the rest are equivalent there.
#[allow(clippy::too_many_lines)] // The whole fault table, in one place.
fn expected_failures(model: Model, fault: Fault) -> BTreeSet<&'static str> {
    let work =
        |views: &[&'static str]| -> BTreeSet<&'static str> { views.iter().copied().collect() };
    let selectors = [
        "demo.work.ByBucket/aggregate",
        "demo.work.ByTeam/aggregate",
        "demo.work.ByTeamChannel/aggregate",
        "demo.work.ByTeamState/aggregate",
        "demo.work.ByTeamUrgency/aggregate",
        "demo.work.ByUrgency/aggregate",
        "demo.work.OpenByTeam/aggregate",
        "demo.work.TeamsInLane/aggregate",
    ];
    let every: Vec<&'static str> = selectors
        .iter()
        .copied()
        .chain([
            "demo.work.ByState/aggregate",
            "demo.work.ByKind/aggregate",
            "demo.work.ByUrgent/aggregate",
        ])
        .collect();
    match (model, fault) {
        (Model::Work, Fault::IgnoresParam | Fault::SelectsFirstGroup) => work(&selectors),
        // Only `ByTeamChannel` arranges a team (`…/B1`) another team's spelling is a prefix of,
        // and only it groups by a key a row may leave absent.
        (Model::Work, Fault::PrefixParam | Fault::DropsAbsentGroup) => {
            work(&["demo.work.ByTeamChannel/aggregate"])
        }
        // Merging is observable only where one read admits two groups.
        (Model::Work, Fault::MergesGroups) => work(&[
            "demo.work.ByKind/aggregate",
            "demo.work.ByState/aggregate",
            "demo.work.ByTeamChannel/aggregate",
            "demo.work.ByTeamState/aggregate",
            "demo.work.ByUrgent/aggregate",
        ]),
        // Only a residual filter refutes a row of a selected group.
        (Model::Work, Fault::CountsAllRows) => work(&[
            "demo.work.OpenByTeam/aggregate",
            "demo.work.TeamsInLane/aggregate",
        ]),
        (Model::Work, Fault::StaleState) => work(&[
            "demo.work.ByState/aggregate",
            "demo.work.ByTeamState/aggregate",
            "demo.work.OpenByTeam/aggregate",
            "demo.work.TopOpen/aggregate",
        ]),
        (Model::Work, Fault::SpuriousGroup | Fault::WrongCount) => work(&every),
        (Model::Work, Fault::WrongSum) => work(&[
            "demo.work.ByState/aggregate",
            "demo.work.ByTeam/aggregate",
            "demo.work.ByTeamChannel/aggregate",
            "demo.work.ByUrgency/aggregate",
            "demo.work.OpenByTeam/aggregate",
        ]),
        (Model::Work, Fault::WrongExtreme) => work(&[
            "demo.work.ByState/aggregate",
            "demo.work.ByTeam/aggregate",
            "demo.work.TopOpen/aggregate",
        ]),
        (Model::Work, Fault::TruncatesAvg | Fault::DistinctCountsRows) => {
            work(&["demo.work.ByKind/aggregate"])
        }
        (
            Model::Ledger,
            Fault::IgnoresParam
            | Fault::SelectsFirstGroup
            | Fault::SpuriousGroup
            | Fault::WrongSum
            | Fault::WrongCount,
        ) => work(&["demo.ledger.ByAccount/aggregate"]),
        (Model::Work, Fault::LossyNumbers) => work(&["demo.work.ByBucket/aggregate"]),
        // `EvidenceEvaluations` is exact; `EvaluationsByGoal`, scoped by its copied key, keeps its
        // old observation, which asserts no row count.
        (Model::Goals, Fault::IgnoresParam | Fault::SelectsFirstGroup | Fault::SpuriousGroup) => {
            work(&["demo.goals.EvidenceEvaluations/aggregate"])
        }
        (
            Model::Goals,
            Fault::WrongCount | Fault::CopiesFirstRelated | Fault::CopiesLastRelated,
        ) => work(&[
            "demo.goals.EvaluationsByGoal/aggregate",
            "demo.goals.EvidenceEvaluations/aggregate",
        ]),
        (Model::Goals, Fault::MergesGroups | Fault::WrongSum) => {
            work(&["demo.goals.EvaluationsByGoal/aggregate"])
        }
        (
            Model::Depots,
            Fault::IgnoresParam
            | Fault::SelectsFirstGroup
            | Fault::WrongSum
            | Fault::CopiesFirstRelated
            | Fault::CopiesLastRelated,
        ) => work(&["demo.work.ByTeam/aggregate"]),
        (Model::Depots, Fault::SpuriousGroup | Fault::WrongCount) => work(&[
            "demo.work.ByTeam/aggregate",
            "demo.work.ItemsByState/aggregate",
        ]),
        // A later scenario holds the earlier ones' rows. A scoped team is this view's alone, so
        // only the views whose selected or grouped values other scenarios also write see them;
        // `ByBucket` runs first.
        (Model::Work, Fault::RetainsRows) => work(&[
            "demo.work.ByKind/aggregate",
            "demo.work.ByState/aggregate",
            "demo.work.ByUrgency/aggregate",
            "demo.work.ByUrgent/aggregate",
            "demo.work.TopOpen/aggregate",
        ]),
        (Model::Depots, Fault::RetainsRows) => work(&["demo.work.ItemsByState/aggregate"]),
        // Every list read selects through the list; only the view reading `[]` as every group
        // tells an empty list that selects nothing from one read correctly.
        (Model::Lists, Fault::IgnoresList | Fault::FirstElementOnly) => work(&[
            "demo.calls.InQueues/aggregate",
            "demo.calls.InQueuesOrAll/aggregate",
        ]),
        // Each window read holds a row a second before `from`, one at `to` and one whose spelling
        // sorts on the other side of a bound than its instant.
        (Model::Window, Fault::IgnoresBound | Fault::InclusiveTo | Fault::ComparesText) => work(&[
            "demo.window.CallsInRange/aggregate",
            "demo.window.QueueCallsInRange/aggregate",
        ]),
        (Model::Lists, Fault::EmptyMatchesNothing) => work(&["demo.calls.InQueuesOrAll/aggregate"]),
        // Equivalent by construction: no related row in `Work`; in `Depots` one-key selections
        // admit one group, the single-state lifecycle has one honest group and no transition,
        // no team spelling is another's prefix, no key is absent, no filter refutes a row, no
        // extreme is declared and no number reaches 2^53; in `Ledger` an owner's identity is
        // this scenario's alone and fixed-width, so retained rows, prefixes and merges are unseen.
        _ => BTreeSet::new(),
    }
}

fn failed(verdicts: &Verdicts) -> BTreeSet<String> {
    verdicts
        .iter()
        .filter(|(_, status)| *status != "passed")
        .map(|(id, _)| id.clone())
        .collect()
}

/// Every (model, fault) a lane runs: each model healthy, every fault over the wider fixtures.
fn matrix() -> Vec<(Model, Fault)> {
    let mut out: Vec<(Model, Fault)> = Model::ALL
        .iter()
        .map(|model| (*model, Fault::None))
        .collect();
    for model in [Model::Work, Model::Depots, Model::Ledger, Model::Goals] {
        for fault in &Fault::ALL[1..] {
            out.push((model, *fault));
        }
    }
    for fault in Fault::LISTS {
        out.push((Model::Lists, fault));
    }
    for fault in Fault::WINDOWS {
        out.push((Model::Window, fault));
    }
    out
}

/// The native verdicts of every case of the matrix.
fn native_verdicts() -> BTreeMap<(Model, Fault), Verdicts> {
    let suites: BTreeMap<Model, AdmittedSuite> = Model::ALL
        .into_iter()
        .map(|model| (model, aggregate_suite(model)))
        .collect();
    matrix()
        .into_iter()
        .map(|(model, fault)| {
            // The names the other lanes are driven by are this lane's own.
            assert_eq!(
                (Model::of(model.name()), Fault::of(fault.name())),
                (model, fault)
            );
            ((model, fault), run_store(&suites[&model], model, fault))
        })
        .collect()
}

#[test]
fn healthy_stores_pass_and_every_fault_fails_what_it_must() {
    let mut wrong = Vec::new();
    for ((model, fault), verdicts) in native_verdicts() {
        let expected = aggregate_ids(aggregate_suite(model).suite()).len();
        assert_eq!(
            verdicts.len(),
            expected,
            "{model:?} {fault:?}: whole report"
        );
        let failed: BTreeSet<String> = failed(&verdicts);
        let wanted: BTreeSet<String> = expected_failures(model, fault)
            .into_iter()
            .map(str::to_owned)
            .collect();
        if failed != wanted {
            wrong.push(format!(
                "{model:?} {fault:?}: failed {failed:?}, expected {wanted:?}"
            ));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// A target that ignores the list, reads `[]` as matching nothing or matches only the first element
/// fails the named list observation natively; the Go, TypeScript and WASM lanes below run the same
/// cases and must agree verdict for verdict.
#[test]
fn aggregate_list_parameter_faults_fail() {
    let suite = aggregate_suite(Model::Lists);
    assert_eq!(
        failed(&run_store(&suite, Model::Lists, Fault::None)),
        BTreeSet::new(),
        "the healthy store passes"
    );
    for (fault, named) in [
        (
            Fault::IgnoresList,
            &[
                "demo.calls.InQueues/aggregate",
                "demo.calls.InQueuesOrAll/aggregate",
            ][..],
        ),
        (
            Fault::EmptyMatchesNothing,
            &["demo.calls.InQueuesOrAll/aggregate"][..],
        ),
        (
            Fault::FirstElementOnly,
            &[
                "demo.calls.InQueues/aggregate",
                "demo.calls.InQueuesOrAll/aggregate",
            ][..],
        ),
    ] {
        let wanted: BTreeSet<String> = named.iter().map(|id| (*id).to_owned()).collect();
        assert_eq!(
            failed(&run_store(&suite, Model::Lists, fault)),
            wanted,
            "{fault:?}"
        );
    }
}

// ---- the generated Go and TypeScript runtimes, and the WASM runner -------------------------------

fn scratch(name: &str) -> PathBuf {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("aggregate-group-selection")
        .join(format!("{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    root
}

fn run(command: &mut Command) -> std::process::Output {
    let output = command
        .output()
        .unwrap_or_else(|error| panic!("{command:?}: {error}"));
    assert!(
        output.status.success(),
        "{command:?}\n{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

/// The verdicts a report/2 document records.
fn report_verdicts(path: &Path, log: &str) -> Verdicts {
    let text = std::fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("no report at {}: {error}\n{log}", path.display()));
    let document: serde_json::Value = serde_json::from_str(&text).unwrap();
    let mut out = Verdicts::new();
    for (status, ids) in document["outcomes"].as_object().expect("outcomes") {
        for id in ids.as_array().expect("a list") {
            out.insert(id.as_str().unwrap().to_owned(), status.clone());
        }
    }
    out
}

fn assert_agrees(lane: &str, observed: &BTreeMap<(Model, Fault), Verdicts>) {
    let native = native_verdicts();
    let mut wrong = Vec::new();
    for (case, verdicts) in &native {
        let found = observed.get(case);
        if found != Some(verdicts) {
            wrong.push(format!(
                "{case:?}: {lane} {:?}, native {:?}",
                found.map(failed),
                failed(verdicts)
            ));
        }
    }
    assert!(wrong.is_empty(), "{lane} disagrees:\n{}", wrong.join("\n"));
    let failing = native.values().filter(|v| !failed(v).is_empty()).count();
    println!(
        "{lane}: {} cases, {failing} with a failing scenario, every verdict the native runner's",
        native.len()
    );
}

#[test]
fn the_generated_go_runtime_agrees_with_the_native_runner() {
    let mut observed = BTreeMap::new();
    for model in Model::ALL {
        let suite = aggregate_suite(model);
        let root = scratch(&format!("go-{}", model.name()));
        for artifact in ess_conformance::go::emit(suite.suite()).unwrap() {
            let path = root.join(artifact.path);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, artifact.contents).unwrap();
        }
        std::fs::write(
            root.join("go.mod"),
            "module example.invalid/groups\n\ngo 1.24\n",
        )
        .unwrap();
        std::fs::write(
            root.join("essconform/groups_test.go"),
            include_str!("fixtures/aggregate-group-selection-runtime.go"),
        )
        .unwrap();
        for (case_model, fault) in matrix().into_iter().filter(|(m, _)| *m == model) {
            let report = root.join(format!("report-{}.json", fault.name()));
            let output = Command::new("go")
                .args(["test", "./essconform", "-count=1", "-run", "TestGroups"])
                .env("ESS_GROUPS_MODEL", case_model.name())
                .env("ESS_GROUPS_FAULT", fault.name())
                .env("ESS_REPORT_FORMAT", "2")
                .env("ESS_REPORT_OUT", &report)
                .env("GOWORK", "off")
                .env("GOFLAGS", "-mod=mod")
                .current_dir(&root)
                .output()
                .unwrap();
            let log = format!(
                "{}{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            observed.insert((case_model, fault), report_verdicts(&report, &log));
        }
        std::fs::remove_dir_all(&root).ok();
    }
    assert_agrees("go", &observed);
}

#[test]
fn the_generated_typescript_runtime_agrees_with_the_native_runner() {
    let mut observed = BTreeMap::new();
    for model in Model::ALL {
        let suite = aggregate_suite(model);
        let root = scratch(&format!("ts-{}", model.name()));
        for artifact in ess_conformance::ts::emit(suite.suite()).unwrap() {
            let path = root.join(artifact.path);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, artifact.contents).unwrap();
        }
        let dir = root.join(ess_conformance::ts::PACKAGE);
        for (name, contents) in [
            (
                "target.mjs",
                include_str!("fixtures/aggregate-group-selection-runtime.mjs"),
            ),
            (
                "driver.mjs",
                include_str!("fixtures/typescript-parity-driver.mjs"),
            ),
            (
                "runtime-test.tsconfig.json",
                r#"{"extends":"./tsconfig.json","compilerOptions":{"types":[],"noCheck":true}}"#,
            ),
        ] {
            std::fs::write(dir.join(name), contents).unwrap();
        }
        run(Command::new("tsc")
            .args(["--project", "runtime-test.tsconfig.json"])
            .current_dir(&dir));
        for (case_model, fault) in matrix().into_iter().filter(|(m, _)| *m == model) {
            let report = dir.join(format!("report-{}.json", fault.name()));
            let output = Command::new("node")
                .args(["--test", "driver.mjs"])
                .env("ESS_GROUPS_MODEL", case_model.name())
                .env("ESS_GROUPS_FAULT", fault.name())
                .env("ESS_REPORT_FORMAT", "2")
                .env("ESS_REPORT_OUT", &report)
                .current_dir(&dir)
                .output()
                .unwrap();
            let log = format!(
                "{}{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            observed.insert((case_model, fault), report_verdicts(&report, &log));
        }
        std::fs::remove_dir_all(&root).ok();
    }
    assert_agrees("typescript", &observed);
}

/// The native runner and this file's store, compiled into WebAssembly and driven through the
/// browser glue `ess-synth` ships, as `tests/support_initial_state` drives its fixtures.
#[test]
fn the_wasm_runner_agrees_with_the_native_runner() {
    use std::fmt::Write as _;
    let root = scratch("wasm");
    std::fs::create_dir_all(root.join("src")).unwrap();
    let conformance = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut manifest = "[package]\nname = \"group-selection-host\"\nversion = \"0.0.0\"\nedition = \"2021\"\n[workspace]\n[lib]\ncrate-type = [\"cdylib\"]\n[dependencies]\nserde_json = {version = \"1\", features = [\"arbitrary_precision\"]}\n".to_owned();
    for (name, path) in [
        ("ess-conformance", conformance.to_owned()),
        (
            "ess-primitives",
            conformance.join("../../specify/ess-primitives"),
        ),
        (
            "ess-compiler",
            conformance.join("../../specify/ess-compiler"),
        ),
        ("ess-domain", conformance.join("../../specify/ess-domain")),
    ] {
        writeln!(
            manifest,
            "{name} = {{path = {}}}",
            serde_json::to_string(&path.canonicalize().unwrap()).unwrap()
        )
        .unwrap();
    }
    std::fs::write(root.join("Cargo.toml"), manifest).unwrap();
    let source = include_str!("aggregate_group_selection.rs")
        .split("// ---- the tests ----")
        .next()
        .unwrap();
    std::fs::write(
        root.join("src/lib.rs"),
        format!("#![allow(dead_code, unused_imports)]\n{source}\n{WASM_HOST}"),
    )
    .unwrap();
    std::fs::copy(
        conformance.join("../../../Cargo.lock"),
        root.join("Cargo.lock"),
    )
    .unwrap();
    let cache =
        std::env::var_os("CARGO_TARGET_DIR").map_or_else(|| root.join("target"), PathBuf::from);
    run(Command::new("cargo")
        .args(["build", "--offline", "--target", "wasm32-unknown-unknown"])
        .arg("--target-dir")
        .arg(&cache)
        .current_dir(&root));
    let page = include_str!("../../../generate/ess-synth/src/web/page.rs");
    let glue = page
        .split("const GLUE_BODY: &str = r#\"")
        .nth(1)
        .unwrap()
        .split("\"#;")
        .next()
        .unwrap();
    std::fs::write(
        root.join("bridge.mjs"),
        format!("{glue}\nexport const EXPORTS=['ess_input_reserve','ess_dispatch','ess_output_len']; export const REALIZE='ess_realize';\n"),
    )
    .unwrap();
    let mut requests = Vec::new();
    for (model, fault) in matrix() {
        requests.push(serde_json::json!({
            "suite": aggregate_suite(model).original_json(),
            "model": model.name(),
            "fault": fault.name(),
        }));
    }
    std::fs::write(
        root.join("requests.json"),
        serde_json::to_vec(&requests).unwrap(),
    )
    .unwrap();
    std::fs::write(
        root.join("driver.mjs"),
        "import {readFileSync} from 'node:fs';import {open} from './bridge.mjs';\nconst module=await open(readFileSync(process.argv[2]));\nconst requests=JSON.parse(readFileSync('requests.json','utf8'));\nconsole.log(JSON.stringify(requests.map(request=>module.request(request))));\n",
    )
    .unwrap();
    let output = run(Command::new("node")
        .arg("driver.mjs")
        .arg(cache.join("wasm32-unknown-unknown/debug/group_selection_host.wasm"))
        .current_dir(&root));
    let answers: Vec<serde_json::Value> = serde_json::from_slice(&output.stdout).unwrap();
    let mut observed = BTreeMap::new();
    for (case, answer) in matrix().into_iter().zip(answers) {
        let verdicts: Verdicts = serde_json::from_value(answer).unwrap();
        observed.insert(case, verdicts);
    }
    std::fs::remove_dir_all(&root).ok();
    assert_agrees("wasm", &observed);
}

const WASM_HOST: &str = r#"
thread_local! {
 static INPUT: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
 static OUTPUT: RefCell<String> = const { RefCell::new(String::new()) };
}
#[no_mangle] pub extern "C" fn ess_input_reserve(length:u32)->u32 {
 INPUT.with(|held| { let mut held=held.borrow_mut(); *held=vec![0;length as usize];held.as_mut_ptr() as u32 })
}
#[no_mangle] pub extern "C" fn ess_dispatch()->u32 {
 let answer=INPUT.with(|held| {
  let request:serde_json::Value=serde_json::from_slice(&held.borrow()).unwrap();
  let suite=ess_conformance::AdmittedSuite::from_json(request["suite"].as_str().unwrap()).unwrap();
  let model=Model::of(request["model"].as_str().unwrap());
  let fault=Fault::of(request["fault"].as_str().unwrap());
  serde_json::to_string(&run_store(&suite,model,fault)).unwrap()
 });
 OUTPUT.with(|out| { *out.borrow_mut()=answer; out.borrow().as_ptr() as u32 })
}
#[no_mangle] pub extern "C" fn ess_output_len()->u32 { OUTPUT.with(|out| out.borrow().len() as u32) }
"#;
