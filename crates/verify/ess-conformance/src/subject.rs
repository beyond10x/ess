//! Typed authority for complete observations of one actual subject row.
use crate::scenario::{ScenarioStep, ScenarioValue};
use crate::selection::Declaration;
use ess_compiler::ir::{EssIr, ResolvedView};
use ess_domain::{Field, QualifiedName, TypeRef};
use ess_primitives::node::Node;
use std::collections::{BTreeMap, BTreeSet};

/// A complete projected row's finite typed schema, never expected target data.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "RawSubjectShape")]
pub struct SubjectShape {
    /// Required, non-optional projected identity field selecting this subject.
    pub identity_field: String,
    /// All declared fields of this projection, including state where projected.
    pub fields: Vec<Field>,
    /// Exactly the reachable finite nominal declarations.
    pub declarations: BTreeMap<QualifiedName, Declaration>,
}
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct RawSubjectShape {
    identity_field: String,
    fields: Vec<Field>,
    declarations: BTreeMap<QualifiedName, Declaration>,
}
impl TryFrom<RawSubjectShape> for SubjectShape {
    type Error = String;
    fn try_from(raw: RawSubjectShape) -> Result<Self, String> {
        let shape = Self {
            identity_field: raw.identity_field,
            fields: raw.fields,
            declarations: raw.declarations,
        };
        shape.validate()?;
        Ok(shape)
    }
}
impl SubjectShape {
    pub(crate) fn of(ir: &EssIr, view: &ResolvedView, identity: &str) -> Result<Self, String> {
        let fields: Vec<_> = view
            .fields
            .iter()
            .map(|field| Field::new(&field.name, crate::accessor::unresolve(&field.type_ref)))
            .collect();
        let shape = Self {
            identity_field: identity.into(),
            declarations: crate::replay::declarations_for(ir, &fields)?,
            fields,
        };
        shape.validate()?;
        Ok(shape)
    }
    /// Refuse unsupported or substituted declaration authority before any callbacks.
    pub fn validate(&self) -> Result<(), String> {
        crate::replay::ExactShape {
            fields: &self.fields,
            declarations: &self.declarations,
        }
        .validate()?;
        let identity = self
            .fields
            .iter()
            .find(|field| field.name == self.identity_field)
            .ok_or("complete subject identity is not a declared field")?;
        if self.optional(&identity.type_ref) {
            return Err("complete subject identity cannot be optional".into());
        }
        if serde_json::to_vec(self)
            .map_err(|error| error.to_string())?
            .len()
            > 1_048_576
        {
            return Err("complete subject schema byte limit".into());
        }
        Ok(())
    }
    fn optional(&self, ty: &TypeRef) -> bool {
        match ty {
            TypeRef::Optional(_) => true,
            TypeRef::Named(name) => match self.declarations.get(name) {
                Some(Declaration::Newtype { of }) => self.optional(of),
                _ => false,
            },
            _ => false,
        }
    }
    /// Require every declared value; extra row keys remain part of exact comparison.
    pub fn admit_row(&self, row: &BTreeMap<String, Node>) -> Result<(), String> {
        self.validate()?;
        let mut bytes = 0;
        for field in &self.fields {
            crate::selection::validate_response_value(
                &field.type_ref,
                row.get(&field.name),
                &self.declarations,
                &mut bytes,
            )?;
        }
        if bytes > 1_048_576
            || serde_json::to_vec(row)
                .map_err(|error| error.to_string())?
                .len()
                > 1_048_576
        {
            return Err("complete subject row byte limit".into());
        }
        Ok(())
    }
}

pub(crate) fn validate_steps(steps: &[ScenarioStep]) -> Result<(), String> {
    let mut bound = BTreeSet::new();
    let mut snapshots = BTreeMap::new();
    let mut invocation = 0_usize;
    let mut queried = None;
    for step in steps {
        match step {
            ScenarioStep::ExecuteCommand { .. } => {
                if snapshots.values().any(|(at, _)| *at < invocation) {
                    return Err("command interrupted complete subject comparison".into());
                }
                invocation += 1;
                queried = None;
            }
            ScenarioStep::QueryView { view, .. } => queried = Some((view, invocation)),
            ScenarioStep::CaptureInstance { instance, .. }
            | ScenarioStep::EstablishEntity { instance, .. } => {
                if snapshots
                    .values()
                    .any(|(_, selected)| selected == &instance)
                {
                    return Err("complete subject identity was overwritten".into());
                }
                bound.insert(instance);
            }
            ScenarioStep::SnapshotCompleteSubject {
                view,
                subject,
                shape,
            } => {
                shape.validate()?;
                let Some(ScenarioValue::Instance { instance }) = subject.get(&shape.identity_field)
                else {
                    return Err("complete snapshot requires its declared bound identity".into());
                };
                if subject.len() != 1
                    || !bound.contains(instance)
                    || invocation == 0
                    || queried != Some((view, invocation))
                    || snapshots.insert(view, (invocation, instance)).is_some()
                {
                    return Err(
                        "complete snapshot is unbound, overwritten or lacks a fresh query".into(),
                    );
                }
            }
            ScenarioStep::ExpectCompleteSubjectUnchanged { view } => {
                let Some((at, _)) = snapshots.remove(view) else {
                    return Err("complete comparison has no matching complete snapshot".into());
                };
                if invocation != at + 1 || queried != Some((view, invocation)) {
                    return Err(
                        "complete comparison must follow one command and a fresh query".into(),
                    );
                }
            }
            ScenarioStep::SnapshotSubject { view, .. }
            | ScenarioStep::ExpectSubjectUnchanged { view }
                if snapshots.contains_key(view) =>
            {
                return Err("legacy snapshot cannot replace complete subject authority".into());
            }
            _ => {}
        }
    }
    if !snapshots.is_empty() {
        return Err("complete subject snapshot has no comparison".into());
    }
    Ok(())
}
