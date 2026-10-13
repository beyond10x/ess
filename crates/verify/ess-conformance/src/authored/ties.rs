//! A head an authored scenario leaves tied under `ties: unordered` (ess/25, beyond10x/ess#543,
//! `docs/design/ordered-related-head.md`, "Declared-unordered ties").
//!
//! An order declaring `ties: unordered` leaves rows tying on every key in no order the model fixes,
//! so an act whose head read ranks two such rows first claims a head that is the implementation's
//! choice. The file's own literals say when that happens: the store its `setup:` rows and its acts
//! build is replayed through the model interpreter, act by act, and an act whose read the
//! interpreter answers [`Undetermined::UnorderedTie`] is refused.
//!
//! Sound and deliberately weak. The replay stops — and refuses nothing more — at the first act it
//! cannot follow exactly: an input that is not a literal, a branch the interpreter does not decide
//! or that mints a value, an answer other than the branch the act names, and from the start where a
//! binding or a periodic cause may change the store between acts.

use std::collections::BTreeMap;

use ess_compiler::ir::{EssIr, ResolvedCommand};
use ess_domain::entity::StateName;
use ess_domain::name::QualifiedName;
use ess_primitives::node::Node;

use crate::interpret::execute::{
    execute_generating, Externals, Generated, Instance, Store, Undetermined,
};
use crate::scenario::ScenarioValue;

/// Two rows tying on every key of `order` and ranking first, in a read of `entity`.
pub(super) struct Tie {
    /// The entity whose rows tie.
    pub(super) entity: String,
    /// The ranking keys, as written.
    pub(super) order: String,
}

/// The store the file's literals build so far; `None` once an act is one the replay cannot follow.
pub(super) struct Replay {
    store: Option<Store>,
}

impl Replay {
    /// An empty store, where nothing but the file's acts changes it.
    pub(super) fn new(ir: &EssIr) -> Self {
        Self {
            store: ir.bindings().is_empty().then(Store::default),
        }
    }

    /// A replay that follows nothing more.
    pub(super) fn stopped() -> Self {
        Self { store: None }
    }

    /// A row a `setup:` states.
    pub(super) fn establish(
        &mut self,
        entity: QualifiedName,
        identity: Node,
        fields: BTreeMap<String, Node>,
        state: StateName,
    ) {
        if let Some(store) = &mut self.store {
            if !store.establish(entity, identity, Instance { state, fields }) {
                self.store = None;
            }
        }
    }

    /// The tie the act sending `command` the `input` reads, where the store before it leaves its
    /// head tied; and the store after it, where the interpreter takes the branch `expected` names
    /// (any, where none is named) and mints nothing.
    pub(super) fn act(
        &mut self,
        ir: &EssIr,
        command: &ResolvedCommand,
        input: &BTreeMap<String, ScenarioValue>,
        expected: Option<&str>,
    ) -> Option<Tie> {
        let store = self.store.take()?;
        let input: BTreeMap<String, Node> = input
            .iter()
            .map(|(field, value)| Some((field.clone(), value.as_literal()?.clone())))
            .collect::<Option<_>>()?;
        match execute_generating(
            ir,
            &store,
            &command.name,
            &input,
            &Externals::Withheld,
            &Generated::Given(BTreeMap::new()),
        ) {
            Err(Undetermined::UnorderedTie { entity, order }) => Some(Tie { entity, order }),
            Err(_) => None,
            Ok(mut steps) => {
                let taken = match steps.as_slice() {
                    [step] => step.outcome.as_ref().is_some_and(|taken| {
                        expected.is_none_or(|name| taken.outcome.as_str() == name)
                    }),
                    _ => false,
                };
                if taken {
                    self.store = Some(steps.remove(0).next);
                }
                None
            }
        }
    }
}
