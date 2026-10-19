//! A selector tests each candidate's related rows: the quantifier `exists:` over the rows of an
//! entity, `exists: {in: {entity: <E>}, as: <b>, that: <P>}` (source format `ess/27`,
//! beyond10x/ess#551, `docs/design/related-rows-quantifier.md`).
//!
//! The selector of a related read (`{related: {entity, where, field | order_by + first}}`) and of a
//! row-set `when_related:` (its `where:` and `forall:`) admit it; every other predicate site refuses
//! it through [`crate::expression::TypeEnvironment::related_rows`]. At the sites that admit it, these
//! rules hold before the predicate is typed:
//!
//! * the format is `ess/27` or later, and `E` is a declared entity;
//! * `exists` only: `forall` over rows is refused with its `not: {exists: …}` rewrite;
//! * one level: no rows quantifier inside another's `that:`;
//! * the binder names the related row only — no field of the candidate, `state`, a root (`input`,
//!   `subject`, `payload`, `now`) or a binder in scope — so a bare name keeps reading the
//!   candidate;
//! * the rows are tied to the candidate: `that:` is one comparison or an `all:` with a top-level
//!   conjunct `<b>.<f> == <g>`, either way round, where `<g>` is the candidate's identity or one of
//!   its declared fields.

use ess_primitives::error::{ConstructRef, ValidationCode, ValidationError, ValidationErrors};
use ess_primitives::facts::FactPath;
use ess_primitives::predicate::{Collection, CompareOp, Operand, Predicate, Quantified};

use crate::entity::EntitySpec;
use crate::name::QualifiedName;
use crate::spec::Specification;
use crate::system::FormatVersion;

/// The roots a binder may not shadow, beside the candidate's fields.
const ROOTS: [&str; 4] = [
    super::subject_fact::INPUT_NAMESPACE,
    super::set_effects::SUBJECT_NAMESPACE,
    super::published_operand::ROOT,
    "now",
];

/// Whether `spec` admits a quantifier over the rows of `entity` at a site that admits rows: the
/// format is `ess/27` or later and the entity is declared. Its declaration where it is.
pub(super) fn admitted<'s>(spec: &'s Specification, entity: &str) -> Option<&'s EntitySpec> {
    if spec.system().format.major() < FormatVersion::V27.major() {
        return None;
    }
    let name: QualifiedName = entity.parse().ok()?;
    spec.entities().get(&name)
}

/// Whether `predicate` walks the rows of any entity, at any depth.
pub fn walks_rows(predicate: &Predicate) -> bool {
    match predicate {
        Predicate::All(children) | Predicate::Any(children) => children.iter().any(walks_rows),
        Predicate::Not(inner) => walks_rows(inner),
        Predicate::Forall(quantified) | Predicate::Exists(quantified) => {
            matches!(quantified.over, Collection::Rows { .. }) || walks_rows(&quantified.body)
        }
        _ => false,
    }
}

/// The entity of every rows quantifier in `predicate`, at any depth, in the order they appear,
/// each as the quantifier writes it.
pub fn walked(predicate: &Predicate) -> Vec<&str> {
    fn walk<'p>(predicate: &'p Predicate, out: &mut Vec<&'p str>) {
        match predicate {
            Predicate::All(children) | Predicate::Any(children) => {
                for child in children {
                    walk(child, out);
                }
            }
            Predicate::Not(inner) => walk(inner, out),
            Predicate::Forall(quantified) | Predicate::Exists(quantified) => {
                if let Collection::Rows { entity } = &quantified.over {
                    out.push(entity);
                }
                walk(&quantified.body, out);
            }
            _ => {}
        }
    }
    let mut out = Vec::new();
    walk(predicate, &mut out);
    out
}

/// The rules of every rows quantifier in `predicate`, a selector over rows of `candidate`, refused
/// at `at`. Below `ess/27` one refusal names the format, whatever else is wrong.
pub(super) fn check(
    spec: &Specification,
    candidate: &EntitySpec,
    predicate: &Predicate,
    at: &ConstructRef,
) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    if !walks_rows(predicate) {
        return errors;
    }
    if spec.system().format.major() < FormatVersion::V27.major() {
        errors.push(
            ValidationError::at(
                at.clone(),
                ValidationCode::UnsupportedFormatVersion,
                "a test over the rows of an entity — `exists: {in: {entity: <Entity>}, as, that}` \
                 in a selector — requires specification format ess/27",
            )
            .with_hint("declare `format: ess/27`"),
        );
        return errors;
    }
    let rules = Rules {
        spec,
        candidate,
        at,
    };
    rules.walk(predicate, &mut Vec::new(), false, &mut errors);
    errors
}

struct Rules<'a> {
    spec: &'a Specification,
    candidate: &'a EntitySpec,
    at: &'a ConstructRef,
}

impl Rules<'_> {
    fn walk<'p>(
        &self,
        predicate: &'p Predicate,
        binders: &mut Vec<&'p str>,
        inside_rows: bool,
        errors: &mut ValidationErrors,
    ) {
        match predicate {
            Predicate::All(children) | Predicate::Any(children) => {
                for child in children {
                    self.walk(child, binders, inside_rows, errors);
                }
            }
            Predicate::Not(inner) => self.walk(inner, binders, inside_rows, errors),
            Predicate::Forall(quantified) | Predicate::Exists(quantified) => {
                let universal = matches!(predicate, Predicate::Forall(_));
                let rows = match &quantified.over {
                    Collection::Fact(_) => false,
                    Collection::Rows { entity } => {
                        if let Some(refused) =
                            self.refusal(quantified, entity, universal, binders, inside_rows)
                        {
                            errors.push(refused);
                            return;
                        }
                        true
                    }
                };
                binders.push(&quantified.bind);
                self.walk(&quantified.body, binders, inside_rows || rows, errors);
                binders.pop();
            }
            _ => {}
        }
    }

    /// The first rule one rows quantifier breaks, in the order a reader repairs them: the entity,
    /// `forall`, nesting, the binder, the tie.
    fn refusal(
        &self,
        quantified: &Quantified,
        entity: &str,
        universal: bool,
        binders: &[&str],
        inside_rows: bool,
    ) -> Option<ValidationError> {
        let bind = quantified.bind.as_str();
        if admitted(self.spec, entity).is_none() {
            return Some(
                ValidationError::at(
                    self.at.clone(),
                    ValidationCode::UndeclaredReference,
                    format!("`{entity}` is not a declared entity"),
                )
                .with_hint(format!(
                    "declared entities: {}",
                    super::join(self.spec.entities().keys())
                )),
            );
        }
        if universal {
            return Some(
                ValidationError::at(
                    self.at.clone(),
                    ValidationCode::UnsupportedConstruct,
                    format!(
                        "`forall:` over the rows of `{entity}` is not admitted: a claim about every \
                         row of an entity needs the tie to the candidate as its antecedent, and a \
                         predicate has no implication"
                    ),
                )
                .with_hint(format!(
                    "write `not: {{exists: {{in: {{entity: {entity}}}, as: {bind}, that: {{all: \
                     [<tie>, {{not: <condition>}}]}}}}}}`"
                )),
            );
        }
        if inside_rows {
            return Some(
                ValidationError::at(
                    self.at.clone(),
                    ValidationCode::UnsupportedConstruct,
                    format!(
                        "the test over the rows of `{entity}` stands inside another test over \
                         rows; one level is admitted in this cut"
                    ),
                )
                .with_hint(
                    "test the inner rows in a selector of their own, or over a list field of the \
                     related row",
                ),
            );
        }
        if let Some(shadowed) = self.shadows(bind, binders) {
            return Some(
                ValidationError::at(
                    self.at.clone(),
                    ValidationCode::ConflictingDeclaration,
                    format!(
                        "the binder `{bind}` of the test over the rows of `{entity}` shadows \
                         {shadowed}, so a bare name in its body would read the related row"
                    ),
                )
                .with_hint("name the binder after the related row, as `as: member`"),
            );
        }
        if !self.tied(quantified, binders) {
            return Some(
                ValidationError::at(
                    self.at.clone(),
                    ValidationCode::MissingDeclaration,
                    format!(
                        "the rows of `{entity}` this test walks are not tied to the candidate: \
                         `that:` compares no `{bind}.<field>` with a field of `{}` in a top-level \
                         equality",
                        self.candidate.name
                    ),
                )
                .with_hint(format!(
                    "write `{bind}.<field> == <candidate field>` as a conjunct of `that:`, or \
                     guard with `when_related: {{entity: {entity}, where: …, exists: true}}` for a \
                     test that does not depend on the candidate"
                )),
            );
        }
        None
    }

    /// What `bind` shadows, rendered, where it shadows anything.
    fn shadows(&self, bind: &str, binders: &[&str]) -> Option<String> {
        if binders.contains(&bind) {
            return Some(format!("the binder `{bind}` of an enclosing quantifier"));
        }
        if super::row_set::candidate_fields(self.candidate)
            .iter()
            .any(|field| field.name == bind)
        {
            return Some(format!(
                "`{bind}`, a field of the candidate `{}`",
                self.candidate.name
            ));
        }
        ROOTS
            .contains(&bind)
            .then(|| format!("`{bind}`, which a selector reads as a root"))
    }

    /// Whether `that:` holds a top-level `<bind>.<f> == <g>`, either way round, with `<g>` the
    /// candidate's identity or a declared field, not `state` and not a binder.
    fn tied(&self, quantified: &Quantified, binders: &[&str]) -> bool {
        let bind = quantified.bind.as_str();
        let candidate = |path: &FactPath| {
            let [name] = path.segments() else {
                return false;
            };
            !binders.contains(&name.as_str())
                && name != bind
                && (name == &self.candidate.identity.name
                    || self
                        .candidate
                        .fields
                        .iter()
                        .any(|field| &field.name == name))
        };
        let related = |path: &FactPath| path.namespace() == bind && path.segments().len() > 1;
        let tie = |conjunct: &Predicate| {
            let Predicate::Compare {
                left: Operand::Fact(left),
                op: CompareOp::Eq,
                right: Operand::Fact(right),
                ..
            } = conjunct
            else {
                return false;
            };
            (related(left) && candidate(right)) || (related(right) && candidate(left))
        };
        match &quantified.body {
            Predicate::All(children) => children.iter().any(tie),
            other => tie(other),
        }
    }
}
