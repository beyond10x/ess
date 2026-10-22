//! Private actual response authority. Candidate issuance is committed only with its completed step.
use super::{
    execute::{Step, Undetermined},
    protected::{self, Issued},
};
use crate::scenario::{CommandRef, OutcomeRef};
use ess_compiler::ir::{
    EssIr, ResolvedBody, ResolvedCommand, ResolvedField, ResolvedOutcome, ResolvedPayloadField,
    ResolvedPayloadValue, ResolvedTypeRef,
};
use ess_primitives::node::Node;
use std::collections::BTreeMap;

pub(super) type Value = BTreeMap<String, Node>;

// Deliberately no Debug or serialization: these maps may contain one-time plaintext.
pub(super) struct Prepared {
    pub(super) value: Option<Value>,
    pub(super) issued: Issued,
}

#[derive(Default)]
pub(super) struct Authority {
    original: Option<Issued>,
    completed: BTreeMap<OutcomeRef, Prepared>,
}

impl Authority {
    pub(super) fn native(issued: &Issued) -> Self {
        Self {
            original: Some(issued.clone()),
            completed: BTreeMap::new(),
        }
    }

    pub(super) fn prepare(
        &self,
        ir: &EssIr,
        command: &ResolvedCommand,
        outcome: &ResolvedOutcome,
    ) -> Result<Option<Prepared>, Undetermined> {
        let Some(mut issued) = self.original.clone() else {
            return Ok(None);
        };
        let command = CommandRef::new(command.name.clone());
        let selected = OutcomeRef::new(command.clone(), outcome.name.clone());
        let value = protected::response(ir, &command, Some(&selected), &mut issued)
            .map_err(|_| unavailable())?;
        Ok(Some(Prepared { value, issued }))
    }

    pub(super) fn completed(&mut self, selected: OutcomeRef, prepared: Option<Prepared>) {
        if let Some(prepared) = prepared {
            self.completed.insert(selected, prepared);
        }
    }

    /// Called only after the target has established there is one complete candidate.
    pub(super) fn finish(
        mut self,
        ir: &EssIr,
        step: &Step,
    ) -> Result<Option<Prepared>, Undetermined> {
        let Some(selected) = &step.outcome else {
            return Ok(None);
        };
        if let Some(prepared) = self.completed.remove(selected) {
            return Ok(Some(prepared));
        }
        // Early missing/wrong-state selection can produce a successful return of its own.
        let command = ir
            .commands()
            .get(selected.command.name())
            .ok_or_else(unavailable)?;
        let outcome = command
            .outcomes
            .iter()
            .find(|outcome| outcome.name == selected.outcome)
            .ok_or_else(unavailable)?;
        self.prepare(ir, command, outcome)
    }
}

pub(super) fn required(outcome: &ResolvedOutcome) -> bool {
    fn reads(field: &ResolvedPayloadField) -> bool {
        match &field.value {
            ResolvedPayloadValue::ResponseField { .. } => true,
            ResolvedPayloadValue::Struct { fields } => fields.iter().any(reads),
            _ => false,
        }
    }
    outcome
        .payload
        .iter()
        .any(|payload| payload.fields.iter().any(reads))
}

fn unavailable() -> Undetermined {
    Undetermined::NotInterpreted {
        construct: "a bounded actual command response".into(),
    }
}

/// A bounded implementation-owned choice from response declarations alone. Input examples,
/// fixture authority, outcome guards and suite expectations never participate in this search.
pub(super) fn base(
    ir: &EssIr,
    field: &ResolvedField,
    distinction: crate::witness::Distinction,
    depth: usize,
) -> Option<Node> {
    if depth > ess_domain::types::MAX_TYPE_DEPTH {
        return None;
    }
    if let Ok(mut fields) = crate::witness::fields(ir, std::slice::from_ref(field), distinction) {
        return fields.remove(&field.name);
    }
    // The general input witness uses `value` as union content. Actual responses must also
    // support a tag named `value`, whose declared content key is `content`.
    let mut inner = field.clone();
    let value = match &field.type_ref {
        ResolvedTypeRef::Optional { of } => {
            inner.type_ref = *of.clone();
            base(ir, &inner, distinction, depth + 1)?
        }
        ResolvedTypeRef::Declared { name } => match &ir.named_type(name).body {
            ResolvedBody::Newtype { of, invariants, .. } => {
                let mut candidates = Vec::new();
                for invariant in invariants {
                    literals(&invariant.predicate, &mut candidates);
                }
                if let Some(value) = candidates.into_iter().find(|value| {
                    crate::input::validate_typed_value(ir, &field.type_ref, value).is_ok()
                }) {
                    return Some(value);
                }
                inner.type_ref = of.clone();
                let value = base(ir, &inner, distinction, depth + 1)?;
                // The base text of the wrapped type, laid out again under this chain's rules: an
                // upper count bound, a text match or a prefix the base breaks.
                if let Node::Text(seed) = &value {
                    if crate::input::validate_typed_value(ir, &field.type_ref, &value).is_err() {
                        return text_choice(ir, &field.type_ref, seed);
                    }
                }
                value
            }
            ResolvedBody::Struct { fields, .. } => {
                let mut members = BTreeMap::new();
                for member in fields {
                    if let Some(value) = base(ir, member, distinction, depth + 1) {
                        members.insert(member.name.clone(), value);
                    }
                }
                Node::Map(members)
            }
            ResolvedBody::Union { tag, variants } => {
                let (label, variant) = variants.iter().next()?;
                let tagged = (tag.clone(), Node::Text(label.clone()));
                match variant {
                    Some(variant) => {
                        inner.type_ref = variant.clone();
                        let value = base(ir, &inner, distinction, depth + 1)?;
                        let content = if tag == "value" { "content" } else { "value" };
                        Node::Map(BTreeMap::from([tagged, (content.into(), value)]))
                    }
                    // A unit variant (ess/22) is the tag alone.
                    None => Node::Map(BTreeMap::from([tagged])),
                }
            }
            ResolvedBody::Enum { .. } => return None,
        },
        // One element, built by these rules, where the input witness built none its type admits.
        ResolvedTypeRef::List { of } => {
            inner.type_ref = *of.clone();
            Node::Seq(vec![base(ir, &inner, distinction, depth + 1)?])
        }
        ResolvedTypeRef::Map { key, value } => {
            inner.type_ref = ResolvedTypeRef::Primitive { name: *key };
            let Node::Text(key) = base(ir, &inner, distinction, depth + 1)? else {
                return None;
            };
            inner.type_ref = *value.clone();
            let value = base(ir, &inner, distinction, depth + 1)?;
            Node::Map(BTreeMap::from([(key, value)]))
        }
        ResolvedTypeRef::Primitive { .. } => return None,
    };
    crate::input::validate_typed_value(ir, &field.type_ref, &value).ok()?;
    Some(value)
}

/// Exact literals and integral neighbours are candidate choices, not a claim that arbitrary
/// predicates are solvable. Every choice is checked against the complete declared type.
fn literals(predicate: &ess_primitives::predicate::Predicate, values: &mut Vec<Node>) {
    use ess_primitives::{
        facts::FactValue,
        predicate::{Operand, Predicate},
    };
    if values.len() >= 512 {
        return;
    }
    match predicate {
        Predicate::All(children) | Predicate::Any(children) => {
            for child in children {
                literals(child, values);
            }
        }
        Predicate::Not(child) => literals(child, values),
        Predicate::Compare { left, right, .. } => {
            for operand in [left, right] {
                let Operand::Literal(value) = operand else {
                    continue;
                };
                match value {
                    FactValue::Number(number) => {
                        for value in [
                            Some(*number),
                            number.checked_add(1i64.into()),
                            number.checked_add((-1i64).into()),
                        ]
                        .into_iter()
                        .flatten()
                        {
                            if values.len() < 512 {
                                values.push(Node::Number(value));
                            }
                        }
                    }
                    FactValue::Text(text) => {
                        if values.len() < 512 {
                            values.push(Node::Text(text.clone()));
                        }
                    }
                    FactValue::Bool(value) => {
                        if values.len() < 512 {
                            values.push(Node::Bool(*value));
                        }
                    }
                }
            }
        }
        _ => {}
    }
}

/// The most texts [`text_choice`] tries for one returned value.
const MAX_TEXT_CHOICES: usize = 512;

/// The rules of a String newtype chain a returned text is built under: the effective alphabet and
/// prefix of every layer, and the positive text-match literals and `value.count` bounds of every
/// layer's invariants.
#[derive(Default)]
struct TextRules {
    alphabets: Vec<String>,
    prefix: String,
    suffix: String,
    contained: Vec<String>,
    counts: Vec<f64>,
}

impl TextRules {
    /// The rules of the newtype chain `type_ref` is declared through, outermost layer first, or
    /// `None` when the chain does not end in `String`.
    fn of(ir: &EssIr, type_ref: &ResolvedTypeRef) -> Option<Self> {
        let mut rules = Self::default();
        let mut current = type_ref;
        for _ in 0..=ess_domain::types::MAX_TYPE_DEPTH {
            let ResolvedTypeRef::Declared { name } = current else {
                break;
            };
            let ResolvedBody::Newtype {
                of,
                alphabet,
                prefix,
                invariants,
            } = &ir.named_type(name).body
            else {
                return None;
            };
            rules.alphabets.extend(alphabet.iter().cloned());
            if let Some(prefix) = prefix {
                rules.starts(prefix);
            }
            for invariant in invariants {
                rules.read(&invariant.predicate, true);
            }
            current = of;
        }
        matches!(
            current,
            ResolvedTypeRef::Primitive {
                name: ess_domain::types::Primitive::String
            }
        )
        .then_some(rules)
    }

    /// The longest prefix kept: a later one that extends an earlier one is met only by a text
    /// built on the longer.
    fn starts(&mut self, literal: &str) {
        if literal.chars().count() > self.prefix.chars().count() {
            literal.clone_into(&mut self.prefix);
        }
    }

    fn read(&mut self, predicate: &ess_primitives::predicate::Predicate, positive: bool) {
        use ess_primitives::{
            facts::{FactPath, FactValue},
            predicate::{Operand, Predicate, TextOp, TextOperand},
        };
        let value = FactPath::new("value").expect("the newtype pseudo-field is a fact path");
        let count = value.child("count");
        match predicate {
            Predicate::All(children) | Predicate::Any(children) => {
                for child in children {
                    self.read(child, positive);
                }
            }
            Predicate::Not(inner) => self.read(inner, !positive),
            Predicate::Trimmed(inner) => self.read(inner, positive),
            Predicate::TextMatch {
                path,
                op,
                value: TextOperand::Literal(FactValue::Text(literal)),
            } if positive && *path == value => match op {
                TextOp::StartsWith => self.starts(literal),
                TextOp::EndsWith => {
                    if literal.chars().count() > self.suffix.chars().count() {
                        literal.clone_into(&mut self.suffix);
                    }
                }
                TextOp::Contains => self.contained.push(literal.clone()),
            },
            Predicate::Compare { left, right, .. } => {
                for (fact, literal) in [(left, right), (right, left)] {
                    if let (Operand::Fact(path), Operand::Literal(FactValue::Number(number))) =
                        (fact, literal)
                    {
                        if *path == count {
                            self.counts.push(number.get());
                        }
                    }
                }
            }
            _ => {}
        }
    }

    /// The texts tried, in order: the prefix, the contained literals and the suffix laid around
    /// `core` as the input witness lays a guard's literals, then each layout with the core resized
    /// so the whole text is as long as each length either side of every `value.count` bound.
    fn candidates(&self, seed: &str) -> Vec<String> {
        let alphabet: Option<Vec<char>> =
            ess_domain::types::effective_alphabet(self.alphabets.iter().map(String::as_str))
                .map(|alphabet| alphabet.chars().collect());
        let core: String = seed
            .strip_prefix(self.prefix.as_str())
            .unwrap_or(seed)
            .chars()
            .map(|character| match &alphabet {
                Some(held) if !held.is_empty() && !held.contains(&character) => {
                    held[u32::from(character) as usize % held.len()]
                }
                _ => character,
            })
            .collect();
        let mut contained = String::new();
        for literal in &self.contained {
            if !self.prefix.contains(literal.as_str())
                && !self.suffix.contains(literal.as_str())
                && !contained.contains(literal.as_str())
            {
                contained.push_str(literal);
            }
        }
        let (prefix, suffix) = (self.prefix.as_str(), self.suffix.as_str());
        let layouts = |core: &str| {
            [
                format!("{prefix}{contained}{core}{suffix}"),
                format!("{prefix}{core}{contained}{suffix}"),
                format!("{prefix}{contained}{suffix}"),
            ]
        };
        let mut texts: Vec<String> = Vec::new();
        let push = |text: String, texts: &mut Vec<String>| {
            if texts.len() < MAX_TEXT_CHOICES && !texts.contains(&text) {
                texts.push(text);
            }
        };
        for text in layouts(&core) {
            push(text, &mut texts);
        }
        let fixed = prefix.chars().count() + contained.chars().count() + suffix.chars().count();
        let fill: Vec<char> = match core.chars().next() {
            Some(_) => core.chars().collect(),
            None => vec![alphabet
                .as_ref()
                .and_then(|held| held.first().copied())
                .unwrap_or('x')],
        };
        for length in self.lengths() {
            let Some(own) = length.checked_sub(fixed) else {
                continue;
            };
            let resized: String = fill.iter().copied().cycle().take(own).collect();
            for text in layouts(&resized) {
                push(text, &mut texts);
            }
        }
        texts
    }

    /// `⌊v⌋`, `⌊v⌋ + 1` and `⌊v⌋ − 1` for each `value.count` bound `v`, negatives, repeats and
    /// lengths above the input witness's own count limit dropped.
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss,
        reason = "a length is cast only once it lies in 0..=MAX_COUNT_WITNESS"
    )]
    fn lengths(&self) -> Vec<usize> {
        let limit = crate::witness::MAX_COUNT_WITNESS as f64;
        let mut lengths = Vec::new();
        for value in &self.counts {
            let floor = value.floor();
            for candidate in [floor, floor + 1.0, floor - 1.0] {
                if (0.0..=limit).contains(&candidate) {
                    let length = candidate as usize;
                    if !lengths.contains(&length) {
                        lengths.push(length);
                    }
                }
            }
        }
        lengths
    }
}

/// A text for a String newtype chain whose base text its own rules refuse: the first of
/// [`TextRules::candidates`] the complete declared type admits. A set of rules no text meets
/// (`value.count >= 5` beside `value.count <= 2`) tries at most [`MAX_TEXT_CHOICES`] texts and
/// ends in `None`, which the caller reports as an undetermined response.
fn text_choice(ir: &EssIr, type_ref: &ResolvedTypeRef, seed: &str) -> Option<Node> {
    TextRules::of(ir, type_ref)?
        .candidates(seed)
        .into_iter()
        .map(Node::Text)
        .find(|value| crate::input::validate_typed_value(ir, type_ref, value).is_ok())
}

/// Type, presence and bounded value checks depend only on the model and the actual response.
pub(super) fn validate(
    ir: &EssIr,
    command: &ResolvedCommand,
    value: &Value,
) -> Result<(), Undetermined> {
    if value.len() > 256 || command.response.len() > 256 {
        return Err(unavailable());
    }
    crate::input::bind(
        ir,
        &command.response,
        value,
        crate::input::Completeness::Total,
    )
    .map_err(|_| unavailable())?;
    let mut bytes = 0;
    for field in &command.response {
        match (field.naming.presence, value.get(&field.name)) {
            (Some(ess_domain::types::Presence::NullWhenAbsent), None)
            | (Some(ess_domain::types::Presence::OmittedWhenAbsent), Some(Node::Null)) => {
                return Err(unavailable())
            }
            _ => {}
        }
        bytes += field.name.len();
        if let Some(value) = value.get(&field.name) {
            crate::input::validate_typed_value(ir, &field.type_ref, value)
                .map_err(|_| unavailable())?;
            bounded(value, 0, &mut bytes)?;
        }
    }
    if bytes > 1_048_576 {
        return Err(unavailable());
    }
    Ok(())
}

fn bounded(value: &Node, depth: usize, bytes: &mut usize) -> Result<(), Undetermined> {
    if depth > 128 {
        return Err(unavailable());
    }
    match value {
        Node::Text(text) => {
            if text.len() > 1_048_576 {
                return Err(unavailable());
            }
            *bytes += text.len();
        }
        Node::Seq(values) => {
            if values.len() > 65_536 {
                return Err(unavailable());
            }
            for value in values {
                bounded(value, depth + 1, bytes)?;
            }
        }
        Node::Map(values) => {
            if values.len() > 65_536 {
                return Err(unavailable());
            }
            for (key, value) in values {
                if key.len() > 1_048_576 {
                    return Err(unavailable());
                }
                *bytes += key.len();
                bounded(value, depth + 1, bytes)?;
            }
        }
        Node::Null | Node::Bool(_) | Node::Number(_) => *bytes += 32,
    }
    if *bytes > 1_048_576 {
        return Err(unavailable());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn model() -> EssIr {
        let source = r"format: ess/21
system: demo
version: v1
domain: demo.response
types:
  - {name: demo.response.Large, kind: newtype, of: Integer, invariants: ['value == 9007199254740993']}
commands:
  - name: demo.response.Read
    response: [{name: value, type: demo.response.Large}]
    outcomes: [{name: read, returns: true}]
";
        model_text(source)
    }

    fn model_text(source: &str) -> EssIr {
        let specification = ess_domain::Specification::assemble([(
            ess_domain::system::Source::new("response.yaml"),
            ess_domain::spec::RawSpecFile::parse(source).unwrap(),
        )])
        .unwrap();
        ess_compiler::resolve::compile(&specification, &ess_compiler::source::SourceMap::new())
            .unwrap()
    }

    #[test]
    fn exact_constrained_response_value_is_generated_without_numeric_rounding() {
        let ir = model();
        let command = ir.commands().values().next().unwrap();
        let field = &command.response[0];
        let exact = Node::Number(9_007_199_254_740_993i64.into());
        crate::input::validate_typed_value(&ir, &field.type_ref, &exact).unwrap();
        assert_eq!(
            base(
                &ir,
                field,
                crate::witness::Distinction::further(1_000_001),
                0
            ),
            Some(exact)
        );
    }

    #[test]
    fn presence_policies_and_response_resource_limits_are_checked_without_disclosure() {
        use ess_domain::types::{Presence, Primitive};
        let ir = model();
        let mut command = ir.commands().values().next().unwrap().clone();
        command.response[0].type_ref = ResolvedTypeRef::Optional {
            of: Box::new(ResolvedTypeRef::Primitive {
                name: Primitive::String,
            }),
        };
        for presence in [
            None,
            Some(Presence::NullWhenAbsent),
            Some(Presence::OmittedWhenAbsent),
        ] {
            command.response[0].naming.presence = presence;
            for value in [None, Some(Node::Null), Some(Node::Text("actual".into()))] {
                let actual = value.clone().map_or_else(BTreeMap::new, |value| {
                    BTreeMap::from([("value".into(), value)])
                });
                let valid = !matches!(
                    (presence, value),
                    (Some(Presence::NullWhenAbsent), None)
                        | (Some(Presence::OmittedWhenAbsent), Some(Node::Null))
                );
                assert_eq!(validate(&ir, &command, &actual).is_ok(), valid);
            }
        }
        let private = "private-payload".repeat(80_000);
        let error = validate(
            &ir,
            &command,
            &BTreeMap::from([("value".into(), Node::Text(private.clone()))]),
        )
        .unwrap_err();
        assert!(!error.to_string().contains(&private));
        assert!(bounded(&Node::Seq(vec![Node::Null; 65_537]), 0, &mut 0).is_err());
        assert!(bounded(&Node::Null, 129, &mut 0).is_err());
    }

    #[test]
    fn nonunique_candidates_discard_their_private_issuance() {
        let ir = model_text(
            r"format: ess/21
system: demo
version: v1
domain: demo.response
commands:
  - name: demo.response.Read
    response: [{name: secret, type: String}]
    outcomes:
      - {name: external, external: provider succeeds, returns: true, one_time_response: [secret]}
      - {name: returned, returns: true, one_time_response: [secret]}
",
        );
        let command = ir.commands().values().next().unwrap();
        let original = Issued::default();
        let mut authority = Authority::native(&original);
        let steps = super::super::execute::responding(
            &ir,
            &super::super::execute::Store::default(),
            &command.name,
            &super::super::execute::caller::Invocation {
                input: &BTreeMap::new(),
                caller: None,
            },
            &super::super::execute::Externals::Open,
            &super::super::execute::Generated::Counter,
            &mut authority,
            None,
        )
        .unwrap();
        assert_eq!(steps.len(), 2);
        assert_eq!(authority.completed.len(), 2);
        drop(authority);
        let next = Authority::native(&original)
            .prepare(&ir, command, &command.outcomes[1])
            .unwrap()
            .unwrap();
        let control = Authority::native(&Issued::default())
            .prepare(&ir, command, &command.outcomes[1])
            .unwrap()
            .unwrap();
        assert_eq!(next.value, control.value);
        for step in steps {
            assert!(!format!("{step:?}")
                .contains(next.value.as_ref().unwrap()["secret"].as_text().unwrap()));
        }
    }
}
