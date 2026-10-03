//! Typed facts from the actual pre-command subject; missing values remain unknown.
use super::{input, EssIr, ResolvedCondition, Row, Undetermined};
use ess_compiler::ir::{ResolvedEntity, ResolvedRelatedTest};
use ess_domain::entity::StateName;
use ess_primitives::facts::{FactPath, FactSource, FactValue, Scales};
use ess_primitives::predicate::{CompareOp, Operand, Predicate, Truth};

pub(super) struct Held<'a> {
    state: &'a StateName,
    row: super::history::Facts<'a>,
    fields: &'a [ess_compiler::ir::ResolvedField],
}
impl<'a> Held<'a> {
    pub(super) fn new(
        ir: &'a EssIr,
        entity: &'a ResolvedEntity,
        instance: &'a Row,
    ) -> Result<Self, Undetermined> {
        let row = super::history::Facts::row(ir, &entity.fields, instance)?;
        Ok(Self {
            state: &instance.state,
            row,
            fields: &entity.fields,
        })
    }

    pub(super) fn selects(
        &self,
        condition: &ResolvedCondition,
        input: &input::InputFacts<'_>,
        caller: Option<&super::caller::Caller<'_>>,
        outcome: String,
    ) -> Result<Option<bool>, Undetermined> {
        let (stored, additional) = match condition {
            ResolvedCondition::SubjectState { state, predicate } => (
                if state.contains(self.state) {
                    Truth::True
                } else {
                    Truth::False
                },
                predicate.as_ref(),
            ),
            ResolvedCondition::StateChange {
                states, predicate, ..
            } => (
                if states.contains(self.state) {
                    Truth::True
                } else {
                    Truth::False
                },
                predicate.as_ref(),
            ),
            ResolvedCondition::SubjectPredicate {
                predicate,
                input: additional,
            }
            | ResolvedCondition::Related {
                test: ResolvedRelatedTest::Holds { predicate },
                input: additional,
                ..
            } => (
                self.row.evaluate_with(|row| {
                    predicate.evaluate(&super::caller::Facts::new(
                        &RowAndInput { row, input },
                        caller,
                        self.fields,
                    ))
                }),
                additional.as_ref(),
            ),
            ResolvedCondition::Related {
                test: ResolvedRelatedTest::Absent,
                ..
            } => (Truth::False, None),
            ResolvedCondition::SubjectField {
                field,
                equals,
                predicate,
            } => (
                self.row.evaluate_with(|row| {
                    Predicate::Compare {
                        left: Operand::Fact(FactPath::new(field).expect("resolved subject field")),
                        op: CompareOp::Eq,
                        right: Operand::Literal(FactValue::text(equals.clone())),
                    }
                    .evaluate(row)
                }),
                predicate.as_ref(),
            ),
            _ => return Ok(None),
        };
        let input = super::caller::Facts::new(input, caller, &input.command().input);
        match stored.and(additional.map_or(Truth::True, |guard| guard.evaluate(&input))) {
            Truth::True => Ok(Some(true)),
            Truth::False => Ok(Some(false)),
            Truth::Unknown => Err(Undetermined::Undecidable {
                outcome,
                guard: format!("{condition:?}"),
            }),
        }
    }
}

struct RowAndInput<'a, 'ir> {
    row: &'a super::history::Facts<'ir>,
    input: &'a input::InputFacts<'ir>,
}
fn input_path(path: &FactPath) -> Option<FactPath> {
    let (root, rest) = path.segments().split_first()?;
    (root == ess_domain::command::subject_fact::INPUT_NAMESPACE && !rest.is_empty())
        .then(|| FactPath::from_segments(rest))
}
impl FactSource for RowAndInput<'_, '_> {
    fn fact(&self, path: &FactPath) -> Option<FactValue> {
        input_path(path).map_or_else(|| self.row.fact(path), |path| self.input.fact(&path))
    }
    fn present(&self, path: &FactPath) -> bool {
        input_path(path).map_or_else(|| self.row.present(path), |path| self.input.present(&path))
    }
    fn observed_presence(&self, path: &FactPath) -> Option<bool> {
        input_path(path).map_or_else(
            || self.row.observed_presence(path),
            |path| self.input.observed_presence(&path),
        )
    }
    fn observe(&self, path: &FactPath) -> Option<FactValue> {
        input_path(path).map_or_else(|| self.row.observe(path), |path| self.input.observe(&path))
    }
    fn cardinality(&self, path: &FactPath) -> Option<usize> {
        input_path(path).map_or_else(
            || self.row.cardinality(path),
            |path| self.input.cardinality(&path),
        )
    }
    fn scales(&self) -> &Scales {
        self.row.scales()
    }
    fn orders_as_instant(&self, path: &FactPath) -> bool {
        input_path(path).map_or_else(
            || self.row.orders_as_instant(path),
            |path| self.input.orders_as_instant(&path),
        )
    }
    fn orders_text_by_bytes(&self, path: &FactPath) -> bool {
        input_path(path).map_or_else(
            || self.row.orders_text_by_bytes(path),
            |path| self.input.orders_text_by_bytes(&path),
        )
    }
}
