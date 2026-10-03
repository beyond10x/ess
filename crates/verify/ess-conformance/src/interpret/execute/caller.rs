//! Typed authentication facts live for exactly one invocation, separate from its input.
use super::{input, Completeness, EssIr, Node, TypedFacts, Undetermined};
use crate::scenario::ActorRef;
use ess_compiler::ir::{ResolvedField, ResolvedTypeRef};
use ess_primitives::facts::{FactPath, FactSource, FactValue, Scales};
use std::collections::BTreeMap;

pub(in crate::interpret) struct Caller<'ir> {
    values: BTreeMap<String, Node>,
    facts: TypedFacts<'ir>,
}

impl<'ir> Caller<'ir> {
    pub(in crate::interpret) fn bind(
        ir: &'ir EssIr,
        actor: Option<&ActorRef>,
        values: Option<&BTreeMap<String, Node>>,
    ) -> Result<Option<Self>, Undetermined> {
        let Some(values) = values else {
            return Ok(None);
        };
        let Some(actor) = actor.and_then(|actor| ir.actors().get(actor.name())) else {
            return if values.is_empty() {
                Ok(None)
            } else {
                Err(Undetermined::Request(
                    "caller attributes require a declared actor".into(),
                ))
            };
        };
        let bound = input::bind(ir, &actor.attributes, values, Completeness::Total)
            .map_err(|error| Undetermined::Request(error.to_string()))?;
        Ok(Some(Self {
            values: values.clone(),
            facts: TypedFacts::new(ir, &actor.attributes, bound),
        }))
    }

    pub(super) fn value(
        &self,
        ir: &EssIr,
        attribute: &str,
        source: &ResolvedTypeRef,
    ) -> Result<Option<Node>, Undetermined> {
        let value = self
            .values
            .get(attribute)
            .filter(|value| **value != Node::Null);
        match value {
            Some(value) => {
                input::validate_typed_value(ir, source, value).map_err(Undetermined::Request)?;
                Ok(Some(value.clone()))
            }
            None if source.is_optional() => Ok(None),
            None => Err(Undetermined::NoValue {
                what: format!("caller attribute `{attribute}`"),
            }),
        }
    }
}

pub(in crate::interpret) struct Invocation<'a> {
    pub(in crate::interpret) input: &'a BTreeMap<String, Node>,
    pub(in crate::interpret) caller: Option<&'a Caller<'a>>,
}

impl std::ops::Deref for Invocation<'_> {
    type Target = BTreeMap<String, Node>;
    fn deref(&self) -> &Self::Target {
        self.input
    }
}

impl Invocation<'_> {
    pub(super) fn caller_value(
        &self,
        ir: &EssIr,
        attribute: &str,
        source: &ResolvedTypeRef,
    ) -> Result<Option<Node>, Undetermined> {
        self.caller
            .ok_or_else(|| Undetermined::NoValue {
                what: format!("authenticated caller attribute `{attribute}`"),
            })?
            .value(ir, attribute, source)
    }
}

pub(super) struct Facts<'a> {
    base: &'a dyn FactSource,
    caller: Option<&'a dyn FactSource>,
    shadowed: bool,
}

impl<'a> Facts<'a> {
    pub(super) fn new(
        base: &'a dyn FactSource,
        caller: Option<&'a Caller<'_>>,
        roots: &[ResolvedField],
    ) -> Self {
        Self {
            base,
            caller: caller.map(|caller| &caller.facts as &dyn FactSource),
            shadowed: roots.iter().any(|field| field.name == "caller"),
        }
    }

    fn source(&self, path: &FactPath) -> Option<(&dyn FactSource, FactPath)> {
        let (root, tail) = path.segments().split_first()?;
        if root == "caller" && !tail.is_empty() && !self.shadowed {
            Some((self.caller?, FactPath::from_segments(tail)))
        } else {
            Some((self.base, path.clone()))
        }
    }
}

impl FactSource for Facts<'_> {
    fn fact(&self, path: &FactPath) -> Option<FactValue> {
        self.source(path)
            .and_then(|(source, path)| source.fact(&path))
    }
    fn present(&self, path: &FactPath) -> bool {
        self.source(path)
            .is_some_and(|(source, path)| source.present(&path))
    }
    fn scales(&self) -> &Scales {
        self.base.scales()
    }
    fn orders_as_instant(&self, path: &FactPath) -> bool {
        self.source(path)
            .is_some_and(|(source, path)| source.orders_as_instant(&path))
    }
    fn orders_text_by_bytes(&self, path: &FactPath) -> bool {
        self.source(path)
            .is_some_and(|(source, path)| source.orders_text_by_bytes(&path))
    }
}
