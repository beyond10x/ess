//! A selector testing each candidate's related rows, `exists: {in: {entity: E}, as: b, that: P}`
//! (`docs/design/related-rows-quantifier.md`, "Synthesis"; source format `ess/27`,
//! beyond10x/ess#551).
//!
//! Both halves of the row-set arrangement read the test:
//!
//! * [`decided`] decides each rows quantifier of a selector for one walked candidate over the rows
//!   of `E` the steps left, so a branch is decided again over what the steps actually leave, the
//!   test included, as the interpreter decides it;
//! * the arrangement selects candidates by the selector without its rows tests ([`stripped`]),
//!   then gives each selected candidate the related rows that make its tests hold
//!   ([`for_candidates`]) and, for each test, arranges one candidate meeting every other conjunct
//!   that fails it ([`failing`]): for an `exists`, one related row per conjunct of the body other
//!   than the tie, meeting every conjunct but that one, and none meeting them all; for a `not:
//!   {exists: …}`, one related row meeting the whole body. Where the branch reads the head of the
//!   selection, the failing candidate ranks above the head on the first key, so a target that
//!   ignores the test, tests one side, or drops a conjunct of the body or the tie answers it.
//!
//! What the arrangement cannot do is named: a tie no creating command of `E` writes from its input
//! refuses the branch ([`untied`]), and a failing candidate no creating input ranks above the head
//! is a gap the scenario stands with.

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use ess_compiler::ir::{EntityHandle, EssIr, ResolvedCommand, ResolvedOutcome};
use ess_domain::name::QualifiedName;
use ess_domain::view::{Direction, Ranking};
use ess_primitives::facts::FactPath;
use ess_primitives::node::Node;
use ess_primitives::predicate::{Collection, CompareOp, Operand, Predicate, Quantified, Truth};

use super::super::{fact_value, map_paths, ActorRef, Arrangement, InstanceName, RefusalCause};
use super::super::{ScenarioStep, ScenarioValue, WitnessGap};
use super::{
    arrange_row, arranged_key, compared, misses, next, ordered_head, subject_fact, walk,
    written_in, Reading, Row,
};

/// The roots a rows quantifier's body reads that are neither the candidate nor the related row.
const ROOTS: [&str; 4] = [
    ess_domain::command::subject_fact::INPUT_NAMESPACE,
    ess_domain::command::set_effects::SUBJECT_NAMESPACE,
    ess_domain::command::published_operand::ROOT,
    "now",
];

/// The reason a branch is refused whose rows tests the arrangement cannot tie to a candidate.
const UNTIED: &str = "the related rows of a selector's test are not arranged; only this branch is \
     refused";
/// The reason a test's failing candidate is left out of a scenario that stands without it.
const UNRANKED: &str = "a target ignoring the test is not caught";

/// One top-level rows test of a selector's `where:`: `exists: {in: {entity: E}, as, that}`, or its
/// negation.
#[derive(Debug, Clone)]
pub(super) struct Test {
    /// The conjunct as the selector writes it.
    conjunct: Predicate,
    /// Whether it is `not: {exists: …}`.
    negated: bool,
    entity: String,
    bind: String,
    body: Predicate,
}

impl Test {
    fn of(conjunct: &Predicate) -> Option<Self> {
        let (negated, quantified) = match conjunct {
            Predicate::Exists(quantified) => (false, quantified),
            Predicate::Not(inner) => match &**inner {
                Predicate::Exists(quantified) => (true, quantified),
                _ => return None,
            },
            _ => return None,
        };
        let Quantified { over, bind, body } = &**quantified;
        let Collection::Rows { entity } = over else {
            return None;
        };
        Some(Self {
            conjunct: conjunct.clone(),
            negated,
            entity: entity.clone(),
            bind: bind.clone(),
            body: body.clone(),
        })
    }

    /// The conjuncts of the body.
    fn conjuncts(&self) -> Vec<Predicate> {
        match &self.body {
            Predicate::All(children) => children.clone(),
            other => vec![other.clone()],
        }
    }

    /// The related row's field `path` reads, where it reads one: `<bind>.<field>`.
    fn related_field<'p>(&self, path: &'p FactPath) -> Option<&'p str> {
        match path.segments() {
            [bind, field] if *bind == self.bind => Some(field),
            _ => None,
        }
    }

    /// The candidate's field `path` reads, where it reads one bare.
    fn candidate_field<'p>(&self, path: &'p FactPath) -> Option<&'p str> {
        match path.segments() {
            [name] if *name != self.bind && !ROOTS.contains(&name.as_str()) => Some(name),
            _ => None,
        }
    }

    /// The related field and the candidate field `conjunct` ties, where it is a tie.
    fn tie<'p>(&self, conjunct: &'p Predicate) -> Option<(&'p str, &'p str)> {
        let Predicate::Compare {
            left: Operand::Fact(left),
            op: CompareOp::Eq,
            right: Operand::Fact(right),
            ..
        } = conjunct
        else {
            return None;
        };
        self.related_field(left)
            .zip(self.candidate_field(right))
            .or_else(|| self.related_field(right).zip(self.candidate_field(left)))
    }

    /// The related field of the body's first tie.
    fn tie_field(&self) -> Option<String> {
        self.conjuncts()
            .iter()
            .find_map(|conjunct| self.tie(conjunct).map(|(field, _)| field.to_owned()))
    }

    /// One goal over the body per conjunct that is not a tie, each refuting that conjunct alone.
    fn misses(&self) -> Vec<Predicate> {
        let conjuncts = self.conjuncts();
        if conjuncts.len() < 2 {
            return Vec::new();
        }
        misses(&self.body)
            .into_iter()
            .zip(&conjuncts)
            .filter(|(_, conjunct)| self.tie(conjunct).is_none())
            .map(|(miss, _)| miss)
            .collect()
    }
}

/// Every top-level rows test of `filter`; `Err` where a rows quantifier stands anywhere else, which
/// the arrangement does not follow.
pub(super) fn tests(filter: &Predicate) -> Result<Vec<Test>, ()> {
    let conjuncts = match filter {
        Predicate::All(children) => children.as_slice(),
        other => std::slice::from_ref(other),
    };
    let mut out = Vec::new();
    for conjunct in conjuncts {
        match Test::of(conjunct) {
            Some(test) if !ess_domain::command::related_rows::walks_rows(&test.body) => {
                out.push(test);
            }
            None if !ess_domain::command::related_rows::walks_rows(conjunct) => {}
            _ => return Err(()),
        }
    }
    Ok(out)
}

/// `filter` without its top-level rows tests.
pub(super) fn stripped(filter: &Predicate) -> Predicate {
    let Predicate::All(children) = filter else {
        return if Test::of(filter).is_some() {
            Predicate::Always
        } else {
            filter.clone()
        };
    };
    let kept: Vec<Predicate> = children
        .iter()
        .filter(|child| Test::of(child).is_none())
        .cloned()
        .collect();
    match <[Predicate; 1]>::try_from(kept) {
        Ok([only]) => only,
        Err(kept) if kept.is_empty() => Predicate::Always,
        Err(kept) => Predicate::All(kept),
    }
}

/// Why the branch reading `filter` is refused, where its rows tests cannot be arranged: a rows
/// quantifier standing other than as a top-level conjunct, or a tie no creating command of the
/// related entity writes from its input.
pub(super) fn untied(ir: &EssIr, at: &str, filter: &Predicate) -> Result<(), RefusalCause> {
    let refused = |path: String| {
        RefusalCause::NoWitness(WitnessGap {
            path,
            type_ref: "row set".into(),
            reason: UNTIED,
        })
    };
    let Ok(tests) = tests(filter) else {
        return Err(refused(format!(
            "{at}: a test over the rows of an entity stands inside `any:`, `not:` or another \
             test, where the arrangement does not follow it"
        )));
    };
    for test in &tests {
        let field = test.tie_field().unwrap_or_default();
        let tied = handle(ir, &test.entity)
            .is_some_and(|entity| ordered_head::steerable(ir, &entity, &field));
        if !tied {
            return Err(refused(format!(
                "{at}: the rows of `{}` related to a candidate are not arranged: no creating \
                 command writes `{field}` from its input",
                test.entity
            )));
        }
    }
    Ok(())
}

/// The handle of the entity a rows quantifier names, where a declared command creates it.
fn handle(ir: &EssIr, name: &str) -> Option<EntityHandle> {
    let name: QualifiedName = name.parse().ok()?;
    ir.drivers()
        .into_keys()
        .find(|entity| *entity.name() == name)
        .cloned()
}

// ---- the reading -------------------------------------------------------------------------------

/// `predicate` with each rows quantifier decided for the walked `candidate` over the rows of its
/// entity the steps left: `Always` or `Never` where the rows decide it, and kept as written —
/// `Unknown` — where they do not, or where a step changed those rows in a way not followed.
pub(super) fn decided(reading: &Reading<'_>, candidate: &Row, predicate: &Predicate) -> Predicate {
    if !ess_domain::command::related_rows::walks_rows(predicate) {
        return predicate.clone();
    }
    let decide = |child: &Predicate| decided(reading, candidate, child);
    match predicate {
        Predicate::All(children) => Predicate::All(children.iter().map(decide).collect()),
        Predicate::Any(children) => Predicate::Any(children.iter().map(decide).collect()),
        Predicate::Not(inner) => Predicate::Not(Box::new(decide(inner))),
        Predicate::Forall(quantified) | Predicate::Exists(quantified) => {
            let Collection::Rows { entity } = &quantified.over else {
                return predicate.clone();
            };
            let universal = matches!(predicate, Predicate::Forall(_));
            match over_rows(reading, candidate, quantified, entity, universal) {
                Truth::True => Predicate::Always,
                Truth::False => Predicate::Never,
                Truth::Unknown => predicate.clone(),
            }
        }
        other => other.clone(),
    }
}

/// The Kleene fold of `quantified`'s body over every walked row of `entity`, for `candidate`.
fn over_rows(
    reading: &Reading<'_>,
    candidate: &Row,
    quantified: &Quantified,
    entity: &str,
    universal: bool,
) -> Truth {
    if reading.rows.all_opaque
        || reading
            .rows
            .opaque
            .iter()
            .any(|opaque| opaque.name().to_string() == entity)
    {
        return Truth::Unknown;
    }
    let test = Test {
        conjunct: Predicate::Always,
        negated: false,
        entity: entity.to_owned(),
        bind: quantified.bind.clone(),
        body: quantified.body.clone(),
    };
    let mut result = if universal { Truth::True } else { Truth::False };
    for row in reading
        .rows
        .held
        .iter()
        .filter(|row| row.entity.name().to_string() == entity)
    {
        let truth = bound(reading, &test, candidate, row)
            .map_or(Truth::Unknown, |body| reading.truth(row, &body));
        result = if universal {
            result.and(truth)
        } else {
            result.or(truth)
        };
    }
    result
}

/// The value the walked `row` holds for its field `name`: its identity, its held state, or a value
/// its steps settled.
fn walked_value(reading: &Reading<'_>, row: &Row, name: &str) -> Option<ScenarioValue> {
    let entity = reading.ir.entity(&row.entity);
    if entity.identity.name == name {
        return row.identity.clone();
    }
    if name == ess_domain::entity::EntitySpec::STATE
        && !entity.fields.iter().any(|field| field.name == name)
    {
        return Some(ScenarioValue::Literal {
            value: Node::Text(row.state.to_string()),
        });
    }
    row.settled.get(name).map(|held| held.value.clone())
}

/// Whether two walked values name the same thing, where the steps decide it: two literals, or two
/// arranged instances.
fn same(left: &ScenarioValue, right: &ScenarioValue) -> Option<bool> {
    match (left, right) {
        (
            ScenarioValue::Instance { instance: left },
            ScenarioValue::Instance { instance: right },
        ) => Some(left == right),
        _ => Some(left.as_literal()? == right.as_literal()?),
    }
}

/// The body of `test` read over the related walked `row` for `candidate`: each tie decided where
/// both sides are known, every other read of the candidate written in, and the binder's reads made
/// the row's own; `None` where a read of the candidate stays unknown.
fn bound(reading: &Reading<'_>, test: &Test, candidate: &Row, row: &Row) -> Option<Predicate> {
    fn ties(
        predicate: &Predicate,
        decide: &dyn Fn(&FactPath, &FactPath) -> Option<bool>,
    ) -> Predicate {
        let recurse = |child: &Predicate| ties(child, decide);
        match predicate {
            Predicate::All(children) => Predicate::All(children.iter().map(recurse).collect()),
            Predicate::Any(children) => Predicate::Any(children.iter().map(recurse).collect()),
            Predicate::Not(inner) => Predicate::Not(Box::new(recurse(inner))),
            Predicate::Compare {
                left: Operand::Fact(left),
                op: op @ (CompareOp::Eq | CompareOp::Ne),
                right: Operand::Fact(right),
                ..
            } => match decide(left, right) {
                Some(equal) if equal == (*op == CompareOp::Eq) => Predicate::Always,
                Some(_) => Predicate::Never,
                None => predicate.clone(),
            },
            other => other.clone(),
        }
    }
    let decide = |left: &FactPath, right: &FactPath| {
        let pair = |related: &FactPath, own: &FactPath| {
            let held = walked_value(reading, row, test.related_field(related)?)?;
            let read = walked_value(reading, candidate, test.candidate_field(own)?)?;
            same(&held, &read)
        };
        pair(left, right).or_else(|| pair(right, left))
    };
    let decided = ties(&test.body, &decide);
    let written = written_in(&decided, &|path| {
        let name = test.candidate_field(path)?;
        fact_value(walked_value(reading, candidate, name)?.as_literal()?)
    });
    if written
        .fact_paths()
        .iter()
        .any(|path| test.candidate_field(path).is_some())
    {
        return None;
    }
    Some(own_reads(&written, &test.bind))
}

/// `predicate` with each `<bind>.<rest>` read as `<rest>`.
fn own_reads(predicate: &Predicate, bind: &str) -> Predicate {
    map_paths(predicate, &|path| {
        if path.namespace() == bind && path.segments().len() > 1 {
            FactPath::from_segments(&path.segments()[1..])
        } else {
            path.clone()
        }
    })
}

// ---- the arrangement ---------------------------------------------------------------------------

/// What arranging related rows reads of its caller.
pub(super) struct Context<'c> {
    pub(super) ir: &'c EssIr,
    pub(super) command: &'c ResolvedCommand,
    pub(super) outcome: &'c ResolvedOutcome,
    pub(super) supplied: &'c BTreeMap<String, ScenarioValue>,
    pub(super) actors: &'c BTreeMap<QualifiedName, ActorRef>,
    pub(super) taken: &'c mut BTreeSet<InstanceName>,
    pub(super) steps: &'c mut Vec<ScenarioStep>,
}

impl Context<'_> {
    /// What `predicate` decides of the arranged `candidate`, a row of `owner`, over every row the
    /// steps so far leave, as the branch is decided again.
    fn decides(
        &self,
        owner: &EntityHandle,
        candidate: &Arrangement,
        predicate: &Predicate,
    ) -> Truth {
        let Ok(rows) = walk(self.ir, self.steps, false) else {
            return Truth::Unknown;
        };
        let Ok(reading) = Reading::new(self.ir, self.command, self.supplied, &rows) else {
            return Truth::Unknown;
        };
        let reading = reading.for_outcome(self.outcome);
        rows.held
            .iter()
            .find(|row| row.entity == *owner && row.instance.as_ref() == Some(&candidate.instance))
            .map_or(Truth::Unknown, |row| reading.truth(row, predicate))
    }

    /// One row of `test`'s entity related to `candidate`, a row of `owner`, meeting `goal` — a
    /// predicate over the body's reads — and, where `forbid` is given, not the whole of it; placed
    /// after the steps so far where the steps then still decide every row-set command they send.
    /// A row already arranged in `related` that serves is used again instead.
    fn related_row(
        &mut self,
        test: &Test,
        (owner, candidate): (&EntityHandle, &Arrangement),
        goal: &Predicate,
        forbid: Option<&Predicate>,
        related: &mut Vec<Arrangement>,
    ) -> Option<()> {
        let ir = self.ir;
        let entity = handle(ir, &test.entity)?;
        let over_row = |predicate: &Predicate| -> Option<Predicate> {
            let written = written_in(predicate, &|path| {
                let name = test.candidate_field(path)?;
                fact_value(&arranged_key(ir, owner, candidate, name)?)
            });
            if written
                .fact_paths()
                .iter()
                .any(|path| test.candidate_field(path).is_some())
            {
                return None;
            }
            Some(own_reads(&written, &test.bind))
        };
        let goal = over_row(goal)?;
        let forbid = match forbid {
            Some(forbid) => Some(over_row(forbid)?),
            None => None,
        };
        let truth = |row: &Arrangement, predicate: &Predicate| {
            subject_fact::row_truth_with(
                ir,
                &entity,
                &row.settled,
                &row.unwritten,
                Some(&row.state),
                predicate,
                None,
            )
        };
        let accept = |row: &Arrangement| {
            truth(row, &goal) == Truth::True
                && forbid
                    .as_ref()
                    .is_none_or(|forbid| truth(row, forbid) == Truth::False)
        };
        if related.iter().any(&accept) {
            return Some(());
        }
        let distinction = next(ir, &entity, self.taken);
        let row = arrange_row(
            ir,
            &entity,
            &goal,
            self.actors,
            (distinction, None),
            &accept,
        )?;
        let mut after = self.steps.clone();
        after.extend(row.steps.iter().cloned());
        if walk(ir, &after, true).is_err() {
            return None;
        }
        *self.steps = after;
        related.push(row);
        Some(())
    }

    /// The related rows that make every test of `tests` hold of `candidate`, a row of `owner`: for
    /// an `exists`, one row meeting the body; for a `not: {exists: …}`, one row per conjunct of the
    /// body other than the tie, meeting every conjunct but that one.
    fn holding(
        &mut self,
        tests: &[&Test],
        (owner, candidate): (&EntityHandle, &Arrangement),
        related: &mut Vec<Arrangement>,
    ) -> Option<()> {
        for test in tests {
            if test.negated {
                for miss in test.misses() {
                    self.related_row(test, (owner, candidate), &miss, Some(&test.body), related)?;
                }
            } else {
                self.related_row(test, (owner, candidate), &test.body, None, related)?;
            }
        }
        Some(())
    }
}

/// The related rows of every candidate of `owner` among `rows` that `own`, the selector without
/// its rows tests, selects, each holding every test of `tests`; `None` where one cannot be
/// arranged.
pub(super) fn for_selected(
    context: &mut Context<'_>,
    tests: &[Test],
    owner: &EntityHandle,
    own: &Predicate,
    rows: &[Arrangement],
) -> Option<Vec<Arrangement>> {
    let ir = context.ir;
    let all: Vec<&Test> = tests.iter().collect();
    let mut out = Vec::new();
    for candidate in rows.iter().filter(|row| {
        subject_fact::row_truth_with(
            ir,
            owner,
            &row.settled,
            &row.unwritten,
            Some(&row.state),
            own,
            None,
        ) == Truth::True
    }) {
        let mut related = Vec::new();
        context.holding(&all, (owner, candidate), &mut related)?;
        out.extend(related);
    }
    Some(out)
}

/// The head a failing candidate ranks above: the first key, and the head's value of it.
pub(super) struct Above<'a> {
    pub(super) first: &'a Ranking,
    pub(super) value: Option<Node>,
}

/// How many fresh candidates are tried for one test before its failing candidate is named a gap:
/// a related row arranged for another selector may name a fresh candidate's identity by chance.
const ATTEMPTS: usize = 3;

/// One candidate of `owner` per test of `tests`, meeting `stripped` and every other test, and
/// failing that one, ranked above the head on the first key where `above` names one; with the
/// related rows that make it so, checked over the rows the steps leave. A candidate no creating
/// input ranks above the head is arranged unranked, and named in `gaps`; one no creating input
/// arranges at all is named and left out.
pub(super) fn failing(
    context: &mut Context<'_>,
    (at, owner): (&str, &EntityHandle),
    stripped: &Predicate,
    tests: &[Test],
    above: Option<&Above<'_>>,
    gaps: &mut Vec<RefusalCause>,
) -> Vec<Arrangement> {
    let gap = |path: String| {
        RefusalCause::NoWitness(WitnessGap {
            path,
            type_ref: "row set".into(),
            reason: UNRANKED,
        })
    };
    let mut out = Vec::new();
    for index in 0..tests.len() {
        let mut made = None;
        if above.is_some() {
            made = (0..ATTEMPTS)
                .find_map(|_| one_failing(context, owner, stripped, (tests, index), above));
            if made.is_none() {
                gaps.push(gap(format!(
                    "{at}: no candidate failing `{}` ranks above the head of `{}`",
                    tests[index].conjunct,
                    owner.name()
                )));
            }
        }
        if made.is_none() {
            made = (0..ATTEMPTS)
                .find_map(|_| one_failing(context, owner, stripped, (tests, index), None));
        }
        match made {
            Some(rows) => out.extend(rows),
            None => gaps.push(gap(format!(
                "{at}: no candidate of `{}` failing `{}` is arranged",
                owner.name(),
                tests[index].conjunct
            ))),
        }
    }
    out
}

/// One candidate failing `tests[index]` and holding every other test, with its related rows, the
/// steps extended by them; `None`, the steps as they were, where the rows the steps then leave do
/// not decide it so.
fn one_failing(
    context: &mut Context<'_>,
    owner: &EntityHandle,
    stripped: &Predicate,
    (tests, index): (&[Test], usize),
    above: Option<&Above<'_>>,
) -> Option<Vec<Arrangement>> {
    let ir = context.ir;
    let test = &tests[index];
    let truth = |row: &Arrangement, predicate: &Predicate| {
        subject_fact::row_truth_with(
            ir,
            owner,
            &row.settled,
            &row.unwritten,
            Some(&row.state),
            predicate,
            None,
        )
    };
    let mut goal = stripped.clone();
    if let Some(above) = above {
        let better = if above.first.direction == Direction::Descending {
            CompareOp::Gt
        } else {
            CompareOp::Lt
        };
        let steer = compared(&above.first.field, better, above.value.as_ref()?)?;
        goal = Predicate::All(vec![goal, steer]);
    }
    let accept = |row: &Arrangement| {
        truth(row, stripped) == Truth::True
            && above.is_none_or(|above| {
                ordered_head::on(
                    ir,
                    owner,
                    above.first,
                    arranged_key(ir, owner, row, &above.first.field).as_ref(),
                    above.value.as_ref(),
                ) == Some(Ordering::Less)
            })
    };
    let distinction = next(ir, owner, context.taken);
    let candidate = arrange_row(
        ir,
        owner,
        &goal,
        context.actors,
        (distinction, None),
        &accept,
    )?;
    let before = context.steps.clone();
    context.steps.extend(candidate.steps.iter().cloned());
    let mut related = Vec::new();
    let others: Vec<&Test> = tests
        .iter()
        .enumerate()
        .filter(|(other, _)| *other != index)
        .map(|(_, other)| other)
        .collect();
    let arranged = failing_rows(context, (owner, &candidate), test, &others, &mut related);
    // Checked, never assumed: over every row the steps leave, the candidate meets the rest of the
    // selector and fails this test.
    let rest = Predicate::All(
        std::iter::once(stripped.clone())
            .chain(others.iter().map(|other| other.conjunct.clone()))
            .collect(),
    );
    let decided = arranged.is_some()
        && context.decides(owner, &candidate, &rest) == Truth::True
        && context.decides(owner, &candidate, &test.conjunct) == Truth::False;
    if !decided {
        *context.steps = before;
        return None;
    }
    let mut out = vec![candidate];
    out.extend(related);
    Some(out)
}

/// The related rows of `candidate`, a row of `owner`, that make it fail `test` and hold every test
/// of `others`: for a `not: {exists: …}`, one row meeting its body; for an `exists`, rows holding
/// the others none of which meets its body, and one row per conjunct of its body other than the
/// tie, meeting every conjunct but that one.
fn failing_rows(
    context: &mut Context<'_>,
    (owner, candidate): (&EntityHandle, &Arrangement),
    test: &Test,
    others: &[&Test],
    related: &mut Vec<Arrangement>,
) -> Option<()> {
    if test.negated {
        context.related_row(test, (owner, candidate), &test.body, None, related)?;
        return context.holding(others, (owner, candidate), related);
    }
    for other in others {
        if other.negated {
            for miss in other.misses() {
                context.related_row(
                    other,
                    (owner, candidate),
                    &miss,
                    Some(&other.body),
                    related,
                )?;
            }
            continue;
        }
        // Where both walk one entity, that row does not meet this test's body.
        let goal = if other.entity == test.entity {
            Predicate::All(vec![
                other.body.clone(),
                Predicate::Not(Box::new(rebound(&test.body, &test.bind, &other.bind))),
            ])
        } else {
            other.body.clone()
        };
        context.related_row(other, (owner, candidate), &goal, None, related)?;
    }
    for miss in test.misses() {
        context.related_row(test, (owner, candidate), &miss, Some(&test.body), related)?;
    }
    Some(())
}

/// `body`, written over the binder `from`, read over the binder `to`.
fn rebound(body: &Predicate, from: &str, to: &str) -> Predicate {
    if from == to {
        return body.clone();
    }
    map_paths(body, &|path| {
        if path.namespace() == from {
            FactPath::from_segments(
                std::iter::once(to.to_owned()).chain(path.segments()[1..].iter().cloned()),
            )
        } else {
            path.clone()
        }
    })
}
