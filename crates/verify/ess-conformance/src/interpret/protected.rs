//! Actual interpreted issuance and reads, with private, non-serializable finite value state.
use crate::{
    scenario::{CommandRef, OutcomeRef},
    target::TargetError,
};
use ess_compiler::EssIr;
use ess_primitives::node::Node;
use std::collections::BTreeMap;

#[derive(Default)]
pub(super) struct Issued {
    next: usize,
    values: Vec<String>,
    bytes: usize,
}

impl std::fmt::Debug for Issued {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("private one-time values")
    }
}

fn unsupported() -> TargetError {
    TargetError::unsupported(
        "protected interpreted observation",
        "the declared observation has no bounded interpreted witness",
    )
}

pub(super) fn response(
    ir: &EssIr,
    command: &CommandRef,
    selected: Option<&OutcomeRef>,
    issued: &mut Issued,
) -> Result<Option<BTreeMap<String, Node>>, TargetError> {
    let spec = ir.commands().get(command.name()).ok_or_else(unsupported)?;
    let Some(outcome) = selected.and_then(|selected| {
        spec.outcomes
            .iter()
            .find(|outcome| outcome.name == selected.outcome)
    }) else {
        return Ok(None);
    };
    if !outcome.returns || outcome.error.is_some() {
        return Ok(None);
    }
    for _ in 0..512 {
        issued.next = issued.next.checked_add(1).ok_or_else(unsupported)?;
        let distinction = crate::witness::Distinction::further(
            1_000_000usize
                .checked_add(issued.next)
                .ok_or_else(unsupported)?,
        );
        let mut response = BTreeMap::new();
        let mut chosen = Vec::<String>::new();
        for field in &spec.response {
            let base = crate::witness::fields(ir, std::slice::from_ref(field), distinction)
                .ok()
                .and_then(|mut fields| fields.remove(&field.name));
            if !outcome.one_time_response.contains(&field.name) {
                response.insert(field.name.clone(), base.ok_or_else(unsupported)?);
                continue;
            }
            let candidates = base
                .into_iter()
                .chain(string_candidates(ir, &field.type_ref)?);
            let value = candidates
                .into_iter()
                .find(|node| {
                    let Node::Text(value) = node else {
                        return false;
                    };
                    !value.is_empty()
                        && crate::input::validate_typed_value(ir, &field.type_ref, node).is_ok()
                        && !issued.values.iter().any(|old| value.contains(old))
                        && !chosen
                            .iter()
                            .any(|other| value.contains(other) || other.contains(value))
                        && !spec.response.iter().any(|field| field.name.contains(value))
                })
                .ok_or_else(unsupported)?;
            if let Node::Text(value) = &value {
                chosen.push(value.clone());
            }
            response.insert(field.name.clone(), value);
        }
        let new: Option<Vec<_>> = outcome
            .one_time_response
            .iter()
            .map(|field| match response.get(field) {
                Some(Node::Text(text)) if !text.is_empty() => Some(text.clone()),
                _ => None,
            })
            .collect();
        let Some(new) = new else {
            continue;
        };
        if new.iter().enumerate().any(|(index, value)| {
            issued.values.iter().any(|old| value.contains(old))
                || new[..index]
                    .iter()
                    .any(|other| value.contains(other) || other.contains(value))
        }) {
            continue;
        }
        // The whole return must avoid old values, including unmarked fields and dynamic keys.
        if response.iter().any(|(key, value)| {
            issued
                .values
                .iter()
                .any(|old| key.contains(old) || contains(value, old))
        }) {
            continue;
        }
        if response.iter().any(|(key, value)| {
            new.iter().any(|new| {
                key.contains(new)
                    || (!outcome.one_time_response.contains(key) && contains(value, new))
            })
        }) {
            continue;
        }
        let bytes = new.iter().map(String::len).sum::<usize>();
        if issued.values.len().saturating_add(new.len()) > crate::one_time_response::MAX_CAPTURES
            || issued.bytes.saturating_add(bytes) > crate::one_time_response::MAX_CAPTURE_BYTES
        {
            return Err(unsupported());
        }
        issued.bytes += bytes;
        issued.values.extend(new);
        return Ok(Some(response));
    }
    Err(unsupported())
}

fn string_candidates(
    ir: &EssIr,
    ty: &ess_compiler::ir::ResolvedTypeRef,
) -> Result<Vec<Node>, TargetError> {
    let mut ty = ty;
    let mut alphabets = Vec::new();
    let mut prefix = String::new();
    while let ess_compiler::ir::ResolvedTypeRef::Declared { name: reference } = ty {
        let ess_compiler::ir::ResolvedBody::Newtype {
            of,
            alphabet,
            prefix: declared,
            ..
        } = &ir.named_type(reference).body
        else {
            return Err(unsupported());
        };
        if let Some(alphabet) = alphabet {
            alphabets.push(alphabet.as_str());
        }
        if let Some(declared) = declared {
            if declared.len() > prefix.len() {
                prefix.clone_from(declared);
            }
        }
        ty = of;
    }
    let alphabet = ess_domain::types::effective_alphabet(alphabets)
        .unwrap_or_else(|| "abcdefghijklmnopqrstuvwxyz0123456789".into());
    let alphabet: Vec<_> = alphabet.chars().collect();
    if alphabet.is_empty() {
        return Err(unsupported());
    }
    let mut candidates = Vec::new();
    if !prefix.is_empty() {
        candidates.push(Node::Text(prefix.clone()));
    }
    for index in 1..=512 {
        let mut ordinal = index;
        let mut suffix = Vec::new();
        while ordinal > 0 {
            ordinal -= 1;
            suffix.push(alphabet[ordinal % alphabet.len()]);
            ordinal /= alphabet.len();
        }
        let suffix: String = suffix.into_iter().rev().collect();
        candidates.push(Node::Text(format!("{prefix}{suffix}")));
    }
    Ok(candidates)
}

fn contains(node: &Node, value: &str) -> bool {
    match node {
        Node::Text(text) => text.contains(value),
        Node::Map(fields) => fields
            .iter()
            .any(|(key, node)| key.contains(value) || contains(node, value)),
        Node::Seq(values) => values.iter().any(|node| contains(node, value)),
        Node::Null | Node::Bool(_) | Node::Number(_) => false,
    }
}
