//! Row sets read from the store as it was before the branch is selected
//! (`docs/design/filtered-related-reads.md`; beyond10x/ess#228, #299).
//!
//! Every row of the selector's entity is a candidate, in no order that decides anything. A row is
//! selected where the selector holds of it, read over the row's fields, identity and held state,
//! the input under `input.`, the addressed subject under `subject.` and the decision's one instant
//! as `now`. A row the selector leaves `Unknown` — a value the store holds unobserved — stays a
//! possible member: it is never dropped, and a test or a read it leaves undecided decides nothing.
use std::collections::BTreeMap;

use ess_compiler::ir::{
    ResolvedCommand, ResolvedCondition, ResolvedEffect, ResolvedEntity, ResolvedField,
    ResolvedInstance, ResolvedRowSelection, ResolvedRowSetTest, ResolvedTypeRef, RowMember,
    Selected,
};
use ess_domain::command::ordered_head::Ties;
use ess_domain::command::OutcomeName;
use ess_domain::view::{Direction, Ranking};
use ess_primitives::facts::FactSource;
use ess_primitives::node::Node;
use ess_primitives::predicate::{Predicate, Truth};

use super::{caller, input, set_effects, Context, EssIr, Row, State, Undetermined};

/// Whether any branch of the command is guarded by a row set.
pub(super) fn uses(spec: &ResolvedCommand) -> bool {
    spec.outcomes
        .iter()
        .any(|outcome| matches!(outcome.condition, ResolvedCondition::RelatedSet { .. }))
}

/// What one decision reads its row sets against: the pre-outcome store, the input, the addressed
/// subject where the command names one, and the decision's instant.
pub(super) struct Reader<'a> {
    ir: &'a EssIr,
    store: &'a State,
    facts: input::InputFacts<'a>,
    now: Option<crate::occurrence_clock::DecisionInstant>,
    subject: Subject<'a>,
    /// The values the outcome publishes that its selectors read (ess/27, beyond10x/ess#549),
    /// resolved before any of them is read; `None` for a guard, which reads none.
    published: Option<&'a super::Published>,
}

/// The values one selector reads from its outcome's payload, each under its own root, and the
/// fields typing them.
type PublishedHeld = (Vec<ResolvedField>, BTreeMap<String, super::Value>);

/// The subject a selector's `subject.` reads.
enum Subject<'a> {
    /// The command addresses none.
    None,
    /// It addresses one the store does not hold: a read of it decides nothing.
    Missing,
    /// The row, as it was before the outcome.
    Held(&'a ResolvedEntity, &'a Node, &'a Row),
}

impl<'a> Reader<'a> {
    pub(super) fn new(
        ir: &'a EssIr,
        spec: &'a ResolvedCommand,
        store: &'a State,
        input: &Context<'_>,
    ) -> Result<Self, Undetermined> {
        let facts = input::flatten(ir, spec, input)
            .map_err(|errors| Undetermined::Request(errors.to_string()))?;
        Ok(Self {
            ir,
            store,
            facts,
            now: input.now,
            subject: subject(ir, spec, store, input),
            published: None,
        })
    }

    /// The same reader, its selectors reading the values `published` holds for their listed
    /// `payload.<Event>.<field>` operands (ess/27, beyond10x/ess#549).
    pub(super) fn with_published(mut self, published: &'a super::Published) -> Self {
        self.published = Some(published);
        self
    }

    /// The values `selection` reads from its outcome's payload, each under its own root, and the
    /// fields typing them; `None` where it reads none. A listed read the outcome has not resolved
    /// is a gap, never an absent operand: validation and `publish_reads` resolve every one first.
    fn published_facts(
        &self,
        selection: &ResolvedRowSelection,
    ) -> Result<Option<PublishedHeld>, Undetermined> {
        if selection.reads.is_empty() {
            return Ok(None);
        }
        let mut fields = Vec::with_capacity(selection.reads.len());
        let mut values = BTreeMap::new();
        for (index, read) in selection.reads.iter().enumerate() {
            let slot = set_effects::Published::slot(index);
            let value = self
                .published
                .and_then(|published| {
                    published.get(&(read.event.name().clone(), read.field.clone()))
                })
                .ok_or_else(|| Undetermined::NotInterpreted {
                    construct: format!(
                        "the published `{}.{}` a selector reads, before it is resolved",
                        read.event.name(),
                        read.field
                    ),
                })?;
            fields.push(ResolvedField {
                name: slot.clone(),
                type_ref: read.type_ref.clone(),
                naming: ess_domain::Naming::default(),
            });
            values.insert(slot, value.clone().unwrap_or(super::Value::Absent));
        }
        Ok(Some((fields, values)))
    }

    /// Every candidate row of `selection`'s entity, with whether it is selected and — where
    /// `satisfies` is given — whether it satisfies that predicate.
    fn members(
        &self,
        selection: &ResolvedRowSelection,
        satisfies: Option<&Predicate>,
    ) -> Result<Vec<(&'a Node, &'a Row, RowMember)>, Undetermined> {
        let entity = self.ir.entity(&selection.entity);
        let published_held = self.published_facts(selection)?;
        let published_facts = published_held
            .as_ref()
            .map(|(fields, values)| super::history::Facts::new(self.ir, fields, values))
            .transpose()?;
        let published = published_facts
            .as_ref()
            .map(|facts| set_effects::Published {
                facts,
                reads: &selection.reads,
            });
        let fields = set_effects::fields(entity);
        let subject_fields;
        let held = match &self.subject {
            Subject::Held(owner, key, row) => {
                subject_fields = set_effects::fields(owner);
                Some(set_effects::row_facts(
                    self.ir,
                    &subject_fields,
                    owner,
                    key,
                    row,
                )?)
            }
            Subject::None | Subject::Missing => None,
        };
        let subject = held.as_ref().map(|facts| facts as &dyn FactSource);
        if subject.is_none()
            && (reads_subject(&selection.filter) || satisfies.is_some_and(reads_subject))
        {
            return Err(Undetermined::Undecidable {
                outcome: format!("row set over {}", entity.name),
                guard: "a selector reading `subject.` where no addressed subject is held".into(),
            });
        }
        let mut members = Vec::new();
        for (_, key, row) in self
            .store
            .instances()
            .filter(|(name, _, _)| **name == entity.name)
        {
            let candidate = set_effects::row_facts(self.ir, &fields, entity, key, row)?;
            let truth = |predicate: &Predicate| {
                candidate.evaluate_with(|candidate| {
                    let read = set_effects::Facts {
                        row: candidate,
                        input: &self.facts,
                        subject,
                        published,
                    };
                    predicate.evaluate(&caller::Facts::new(&read, None, &[], self.now))
                })
            };
            let selected = truth(&selection.filter);
            let satisfies = match (satisfies, selected) {
                (Some(_), Truth::False) | (None, _) => Truth::True,
                (Some(predicate), _) => truth(predicate),
            };
            members.push((
                key,
                row,
                RowMember {
                    selected,
                    satisfies,
                },
            ));
        }
        Ok(members)
    }

    /// Whether `test` holds of the rows `selection` selects.
    pub(super) fn decide(
        &self,
        selection: &ResolvedRowSelection,
        test: &ResolvedRowSetTest,
    ) -> Result<Truth, Undetermined> {
        let members = self.members(selection, test.predicate())?;
        let members: Vec<RowMember> = members.into_iter().map(|(.., member)| member).collect();
        Ok(test.decide(&members))
    }

    /// The one row `selection` selects, or why there is none to read.
    pub(super) fn one(
        &self,
        selection: &ResolvedRowSelection,
    ) -> Result<(&'a Node, &'a Row), Undetermined> {
        let members = self.members(selection, None)?;
        let read: Vec<RowMember> = members.iter().map(|(.., member)| *member).collect();
        let entity = &self.ir.entity(&selection.entity).name;
        match Selected::of(&read) {
            Selected::One(index) => {
                let (key, row, _) = members[index];
                Ok((key, row))
            }
            Selected::None => Err(Undetermined::NoValue {
                what: format!("a field of the one `{entity}` the selector selects: none is"),
            }),
            Selected::Several => Err(Undetermined::NoValue {
                what: format!(
                    "a field of the one `{entity}` the selector selects: several are, and none is \
                     chosen"
                ),
            }),
            Selected::Unknown => Err(Undetermined::Undecidable {
                outcome: format!("filtered read of {entity}"),
                guard: format!("{:?}", selection.filter),
            }),
        }
    }

    /// The first row `selection` selects, ranked by `order_by` (ess/27, beyond10x/ess#543,
    /// `docs/design/ordered-related-head.md`), or why there is none to read.
    ///
    /// The rows are ranked with the comparator a view's order uses, so the two cannot disagree.
    /// A total order's last key is the identity, so exactly one row is first. Under `ties:
    /// unordered` two rows may tie on every key and rank first, and then the head is the
    /// implementation's choice: [`Undetermined::UnorderedTie`], and no row is picked. Sound and
    /// deliberately weak over a history: a row whose membership is unknown, or a selected row
    /// whose key the store does not hold, leaves the head undecidable, unless one row is selected
    /// and no other may be.
    pub(super) fn head(
        &self,
        selection: &ResolvedRowSelection,
        order_by: &[Ranking],
        ties: Ties,
    ) -> Result<(&'a Node, &'a Row), Undetermined> {
        let members = self.members(selection, None)?;
        let entity = self.ir.entity(&selection.entity);
        let undecidable = |why: &str| Undetermined::Undecidable {
            outcome: format!("ordered read of {}", entity.name),
            guard: format!(
                "the first row by `{}` of those `{}` selects: {why}",
                order_by
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", "),
                selection.filter
            ),
        };
        if members
            .iter()
            .any(|(.., member)| member.selected == Truth::Unknown)
        {
            return Err(undecidable("a row's membership is unknown"));
        }
        let selected: Vec<(&'a Node, &'a Row)> = members
            .iter()
            .filter(|(.., member)| member.selected == Truth::True)
            .map(|(key, row, _)| (*key, *row))
            .collect();
        match selected.as_slice() {
            [] => {
                return Err(Undetermined::NoValue {
                    what: format!(
                        "a field of the first `{}` the selector selects: none is",
                        entity.name
                    ),
                })
            }
            [one] => return Ok(*one),
            _ => {}
        }
        let types: Vec<ResolvedTypeRef> = order_by
            .iter()
            .map(|ranking| key_type(entity, &ranking.field))
            .collect();
        let mut ranked = Vec::with_capacity(selected.len());
        for (key, row) in selected {
            let mut values = Vec::with_capacity(order_by.len());
            for ranking in order_by {
                let Some(value) = key_value(entity, key, row, &ranking.field) else {
                    return Err(undecidable(&format!(
                        "the store does not hold `{}` of a selected row",
                        ranking.field
                    )));
                };
                values.push(value);
            }
            ranked.push((key, row, values));
        }
        let order = |left: &[Node], right: &[Node]| {
            for ((ranking, ty), (left, right)) in order_by
                .iter()
                .zip(&types)
                .zip(left.iter().zip(right.iter()))
            {
                let compare = super::super::views::compare(self.ir, ty, Some(left), Some(right));
                if !compare.is_eq() {
                    return if ranking.direction == Direction::Descending {
                        compare.reverse()
                    } else {
                        compare
                    };
                }
            }
            std::cmp::Ordering::Equal
        };
        ranked.sort_by(|(.., left), (.., right)| order(left, right));
        if let [(.., first), (.., second), ..] = ranked.as_slice() {
            if order(first, second).is_eq() {
                // Two rows rank first. A total order ends in the identity, which no two rows share,
                // so a tie under one is no answer the model gives either.
                if ties.is_total() {
                    return Err(undecidable("two rows tie on every key"));
                }
                return Err(Undetermined::UnorderedTie {
                    entity: entity.name.to_string(),
                    order: order_by
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(", "),
                });
            }
        }
        ranked
            .into_iter()
            .next()
            .map(|(key, row, _)| (key, row))
            .ok_or_else(|| undecidable("no row is first"))
    }
}

/// The type a ranking key of `entity` ranks at: a declared field's, the identity's, or the
/// entity's `State` for `state`.
fn key_type(entity: &ResolvedEntity, name: &str) -> ResolvedTypeRef {
    entity
        .fields
        .iter()
        .chain(std::iter::once(&entity.identity))
        .find(|field| field.name == name)
        .map_or_else(
            || entity.state_field().type_ref,
            |field| field.type_ref.clone(),
        )
}

/// The value of the ranking key `name` on the row `key` names, as the store holds it: a declared
/// field, the identity, or the held state's name; `None` where the store holds none.
fn key_value(entity: &ResolvedEntity, key: &Node, row: &Row, name: &str) -> Option<Node> {
    if entity.fields.iter().any(|field| field.name == name) {
        return match row.fields.get(name) {
            Some(super::Value::Known(value)) => Some(value.clone()),
            _ => None,
        };
    }
    if entity.identity.name == name {
        return Some(key.clone());
    }
    (name == ess_domain::entity::EntitySpec::STATE)
        .then(|| Node::Text(row.state.as_str().to_owned()))
}

/// Whether `predicate` reads the addressed subject.
fn reads_subject(predicate: &Predicate) -> bool {
    predicate.fact_paths().iter().any(|path| {
        path.segments().len() > 1
            && path.namespace() == ess_domain::command::set_effects::SUBJECT_NAMESPACE
    })
}

/// The subject the command's branches address through the input, as the store held it.
fn subject<'a>(
    ir: &'a EssIr,
    spec: &ResolvedCommand,
    store: &'a State,
    input: &Context<'_>,
) -> Subject<'a> {
    let Some((entity, field)) = spec.outcomes.iter().find_map(|outcome| {
        let subject = outcome.subject.as_ref()?;
        match (&subject.effect, &subject.instance) {
            (ResolvedEffect::Creates, _) | (_, ResolvedInstance::Observed { .. }) => None,
            (_, ResolvedInstance::Supplied { field }) => Some((&subject.entity, &field.name)),
        }
    }) else {
        return Subject::None;
    };
    let entity: &ResolvedEntity = ir.entity(entity);
    let Some(key) = input.get(field) else {
        return Subject::Missing;
    };
    let Some((key, row)) = store
        .instances()
        .find(|(name, held, _)| **name == entity.name && *held == key)
        .map(|(_, key, row)| (key, row))
    else {
        return Subject::Missing;
    };
    Subject::Held(entity, key, row)
}

/// What each row-set branch's test decides, by branch: computed once, read only where the
/// precedence order reaches the branch, so a test that cannot be decided decides nothing earlier.
pub(super) fn decisions(
    reader: Option<&Reader<'_>>,
    spec: &ResolvedCommand,
) -> BTreeMap<OutcomeName, Result<Truth, String>> {
    let mut out = BTreeMap::new();
    let Some(reader) = reader else {
        return out;
    };
    for outcome in &spec.outcomes {
        if let ResolvedCondition::RelatedSet {
            selection, test, ..
        } = &outcome.condition
        {
            out.insert(
                outcome.name.clone(),
                reader
                    .decide(selection, test)
                    .map_err(|why| why.to_string()),
            );
        }
    }
    out
}
