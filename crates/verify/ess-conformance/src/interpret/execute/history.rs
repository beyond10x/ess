//! Private execution values shared by concrete execution and outcome-only history search.
mod facts;
mod values;
pub(super) use facts::Facts;
pub(super) use values::{finite, generated, validate, Origin, Value};
pub(super) type Domains = std::cell::RefCell<BTreeMap<String, values::Feasibility>>;

use super::{
    DeclaredErrorValue, ErrorRef, EventRef, Instance, Node, ObservedEvent, OutcomeRef,
    QualifiedName, StateName, Step, Store, Undetermined,
};
use std::collections::BTreeMap;

/// One shared executor invocation. Only the private history entrypoint supplies an operation id.
pub(super) struct Context<'a> {
    pub(super) input: &'a BTreeMap<String, Node>,
    pub(super) caller: Option<&'a super::caller::Caller<'a>>,
    pub(super) operation: Option<&'a str>,
    pub(super) unresolved: std::cell::RefCell<Option<Undetermined>>,
    pub(super) domains: Domains,
}

impl std::ops::Deref for Context<'_> {
    type Target = BTreeMap<String, Node>;
    fn deref(&self) -> &Self::Target {
        self.input
    }
}

impl Context<'_> {
    /// Retain an unresolved history alternative without losing other proven provider choices.
    pub(super) fn defer(&self, why: Undetermined) -> Result<(), Undetermined> {
        if self.operation.is_some() && matches!(why, Undetermined::Undecidable { .. }) {
            self.unresolved.borrow_mut().get_or_insert(why);
            Ok(())
        } else {
            Err(why)
        }
    }
    pub(super) fn caller_value(
        &self,
        ir: &super::EssIr,
        attribute: &str,
        source: &super::ResolvedTypeRef,
    ) -> Result<Option<Node>, Undetermined> {
        super::Invocation {
            input: self.input,
            caller: self.caller,
        }
        .caller_value(ir, attribute, source)
    }
}

/// Internal rows carry explicit values. No public Store contains unknown sentinels or a skeleton.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Row {
    pub(super) state: StateName,
    pub(super) fields: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct State {
    pub(super) instances: BTreeMap<QualifiedName, BTreeMap<Node, Row>>,
    minted: u64,
}

impl State {
    pub(super) fn import(store: &Store) -> Self {
        Self {
            instances: store
                .instances
                .iter()
                .map(|(entity, rows)| {
                    (
                        entity.clone(),
                        rows.iter()
                            .map(|(identity, row)| {
                                (
                                    identity.clone(),
                                    Row {
                                        state: row.state.clone(),
                                        fields: row
                                            .fields
                                            .iter()
                                            .map(|(name, value)| {
                                                (name.clone(), Value::Known(value.clone()))
                                            })
                                            .collect(),
                                    },
                                )
                            })
                            .collect(),
                    )
                })
                .collect(),
            minted: store.minted,
        }
    }

    fn publish(self) -> Result<Store, Undetermined> {
        let instances = self
            .instances
            .into_iter()
            .map(|(entity, rows)| {
                let rows = rows
                    .into_iter()
                    .map(|(identity, row)| {
                        Ok((
                            identity,
                            Instance {
                                state: row.state,
                                fields: concrete_fields(row.fields)?,
                            },
                        ))
                    })
                    .collect::<Result<_, Undetermined>>()?;
                Ok((entity, rows))
            })
            .collect::<Result<_, Undetermined>>()?;
        Ok(Store {
            instances,
            minted: self.minted,
        })
    }

    pub(super) fn instance_typed(&self, entity: &QualifiedName, identity: &Node) -> Option<&Row> {
        self.instances.get(entity)?.get(identity)
    }

    pub(super) fn instances(&self) -> impl Iterator<Item = (&QualifiedName, &Node, &Row)> {
        self.instances.iter().flat_map(|(entity, rows)| {
            rows.iter()
                .map(move |(identity, row)| (entity, identity, row))
        })
    }

    pub(crate) fn lifecycle(&self, entity: &QualifiedName, identity: &str) -> Option<&StateName> {
        self.instance_typed(entity, &Node::Text(identity.into()))
            .map(|row| &row.state)
    }

    pub(crate) fn text_states(&self) -> impl Iterator<Item = (&str, &StateName)> {
        self.instances().filter_map(|(_, identity, row)| {
            identity.as_text().map(|identity| (identity, &row.state))
        })
    }

    pub(super) fn tick(&mut self) -> u64 {
        self.minted += 1;
        self.minted
    }
}

/// Same selector/effect implementation as native execution, with no authority for unrecorded data.
pub(crate) struct Alternatives {
    pub(crate) proven: Vec<Transition>,
    pub(crate) unresolved: Option<Undetermined>,
}

pub(crate) fn execute(
    ir: &super::EssIr,
    state: &State,
    command: &QualifiedName,
    input: &BTreeMap<String, Node>,
    generated: &super::Generated,
    operation: &str,
) -> Result<Alternatives, Undetermined> {
    let context = Context {
        input,
        caller: None,
        operation: Some(operation),
        unresolved: std::cell::RefCell::default(),
        domains: Domains::default(),
    };
    let result = super::responding_core(
        ir,
        state,
        command,
        &context,
        &super::Externals::Open,
        generated,
        &mut super::super::response::Authority::default(),
    );
    let proven = match result {
        Ok(proven) => proven,
        Err(Undetermined::Request(_)) => Vec::new(),
        Err(why) => {
            context.defer(why)?;
            Vec::new()
        }
    };
    Ok(Alternatives {
        proven,
        unresolved: context.unresolved.into_inner(),
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Event {
    pub(super) event: EventRef,
    pub(super) payload: BTreeMap<String, Value>,
}

impl Event {
    pub(super) fn new(event: EventRef) -> Self {
        Self {
            event,
            payload: BTreeMap::new(),
        }
    }

    pub(super) fn with(mut self, field: String, value: Value) -> Self {
        self.payload.insert(field, value);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Error {
    pub(super) error: ErrorRef,
    pub(super) fields: BTreeMap<String, Value>,
}

impl Error {
    pub(super) fn new(error: ErrorRef) -> Self {
        Self {
            error,
            fields: BTreeMap::new(),
        }
    }

    pub(super) fn with(mut self, field: String, value: Value) -> Self {
        self.fields.insert(field, value);
        self
    }
}

/// Abstract events and errors stay private; a history records only the selected outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Transition {
    pub(crate) outcome: Option<OutcomeRef>,
    pub(super) error: Option<Error>,
    pub(super) events: Vec<Event>,
    pub(crate) next: State,
}

impl Transition {
    pub(super) fn publish(self) -> Result<Step, Undetermined> {
        Ok(Step {
            outcome: self.outcome,
            error: self
                .error
                .map(|error| {
                    Ok(DeclaredErrorValue {
                        error: error.error,
                        fields: concrete_fields(error.fields)?,
                    })
                })
                .transpose()?,
            events: self
                .events
                .into_iter()
                .map(|event| {
                    let mut concrete = ObservedEvent::new(event.event);
                    concrete.payload = concrete_fields(event.payload)?;
                    Ok(concrete)
                })
                .collect::<Result<_, Undetermined>>()?,
            next: self.next.publish()?,
        })
    }
}

fn concrete_fields(
    fields: BTreeMap<String, Value>,
) -> Result<BTreeMap<String, Node>, Undetermined> {
    let mut concrete = BTreeMap::new();
    for (name, value) in fields {
        let value = value.concrete().map_err(|()| Undetermined::Undecidable {
            outcome: "concrete execution result".into(),
            guard: format!("unobserved value at `{name}` has no concrete publication"),
        })?;
        if let Some(value) = value {
            concrete.insert(name, value);
        }
    }
    Ok(concrete)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ess_compiler::{resolve::compile, source::SourceMap};
    use ess_domain::{
        spec::{RawSpecFile, Specification},
        system::Source,
    };

    #[test]
    fn generated_origins_distinguish_effect_occurrences_rows_and_operations_but_preserve_copies() {
        let source = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../specify/ess-compiler/tests/fixtures/set-effects.yaml"))
            .replace("format: ess/16", "format: ess/20")
            .replace("  - name: demo.desk.Invite\n    input:\n", "  - name: demo.desk.Invite\n    input:\n      - {name: second, type: Boolean}\n")
            .replace("          on_hold: false\n        affects:", "          on_hold: false\n          note: {subject: note}\n        affects:")
            .replace("              on_hold: true\nviews:", "              on_hold: true\n              note: {generated: true}\n          - entity: demo.desk.Session\n            where: {all: [team == subject.team, input.second == true]}\n            sets: {note: {generated: true}}\nviews:");
        let specification = Specification::assemble([(
            Source::new("origins.yaml"),
            RawSpecFile::parse(&source).unwrap(),
        )])
        .unwrap();
        let ir = compile(&specification, &SourceMap::new()).unwrap();
        let mut concrete = Store::default();
        for id in ["subject", "a", "b"] {
            let mut steps = super::super::execute_generating(
                &ir,
                &concrete,
                &"demo.desk.Open".parse().unwrap(),
                &BTreeMap::from([
                    ("team".into(), Node::Text("one".into())),
                    ("note".into(), Node::Text("before".into())),
                ]),
                &super::super::Externals::Withheld,
                &super::super::Generated::Given(BTreeMap::from([(
                    super::super::GeneratedSlot::new(
                        "demo.desk.SessionOpened".parse().unwrap(),
                        "session_id",
                    ),
                    Node::Text(id.into()),
                )])),
            )
            .unwrap();
            assert_eq!(steps.len(), 1);
            concrete = steps.remove(0).next;
        }
        let initial = State::import(&concrete);
        let invoke = |state: &State, id: &str, second: bool, operation: &str| {
            let mut answer = execute(
                &ir,
                state,
                &"demo.desk.Invite".parse().unwrap(),
                &BTreeMap::from([
                    ("session_id".into(), Node::Text(id.into())),
                    ("second".into(), Node::Bool(second)),
                ]),
                &super::super::Generated::Counter,
                operation,
            )
            .unwrap();
            assert!(answer.unresolved.is_none(), "{:?}", answer.unresolved);
            assert_eq!(answer.proven.len(), 1);
            answer.proven.remove(0).next
        };
        let note = |state: &State, id: &str| {
            state
                .instance_typed(
                    &"demo.desk.Session".parse().unwrap(),
                    &Node::Text(id.into()),
                )
                .unwrap()
                .fields["note"]
                .clone()
        };
        let first = invoke(&initial, "subject", false, "same-operation");
        let second = invoke(&initial, "subject", true, "same-operation");
        assert!(matches!(note(&first, "a"), Value::Unknown { .. }));
        assert_ne!(
            note(&first, "a"),
            note(&second, "a"),
            "the second generation in one row/field needs a different occurrence origin"
        );
        assert_ne!(
            note(&first, "a"),
            note(&first, "b"),
            "different affected rows are distinct origins"
        );
        let later = invoke(&initial, "subject", false, "later-operation");
        assert_ne!(note(&first, "a"), note(&later, "a"));
        let copied = invoke(&first, "a", false, "copy-operation");
        assert_eq!(
            note(&first, "a"),
            note(&copied, "a"),
            "subject copy keeps the original unknown origin"
        );
        assert_eq!(
            initial,
            State::import(&concrete),
            "execution never mutates the input state"
        );
    }
}

#[cfg(test)]
mod transfer_tests {
    use super::*;
    use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
    use ess_domain::{
        spec::{RawSpecFile, Specification},
        system::Source,
    };

    const SOURCE: &str = r"format: ess/20
system: demo
version: v1
domain: demo.snapshot
types:
  - {name: demo.snapshot.RootId, kind: newtype, of: String}
  - {name: demo.snapshot.LinkId, kind: newtype, of: String}
entities:
  - name: demo.snapshot.Root
    identity: {name: root_id, type: demo.snapshot.RootId}
    fields: [{name: stamp, type: Optional<Timestamp>}, {name: flag, type: Boolean}]
    lifecycle: {initial: Active, states: [Active], terminal: [Active]}
  - name: demo.snapshot.Link
    identity: {name: link_id, type: demo.snapshot.LinkId}
    fields: [{name: root_id, type: demo.snapshot.RootId}, {name: copied, type: Optional<Timestamp>}]
    relations: [{name: root, kind: references, target: demo.snapshot.Root, cardinality: one, via: root_id}]
    lifecycle: {initial: Active, states: [Active], terminal: [Active]}
events:
  - {name: demo.snapshot.RootCreated, fields: [{name: root_id, type: demo.snapshot.RootId}]}
  - {name: demo.snapshot.LinkCreated, fields: [{name: link_id, type: demo.snapshot.LinkId}]}
  - {name: demo.snapshot.Changed, fields: []}
commands:
  - name: demo.snapshot.CreateRoot
    input: [{name: root_id, type: demo.snapshot.RootId}]
    outcomes:
      - name: created
        creates: demo.snapshot.Root
        instance: root_id
        sets: {stamp: {generated: true}, flag: false}
        emits: [demo.snapshot.RootCreated]
        payload: {demo.snapshot.RootCreated: {root_id: input.root_id}}
  - name: demo.snapshot.ClearRoot
    input: [{name: root_id, type: demo.snapshot.RootId}]
    outcomes:
      - {name: cleared, updates: demo.snapshot.Root, instance: root_id, sets: {stamp: {cleared: true}}, emits: [demo.snapshot.Changed]}
  - name: demo.snapshot.CreateLink
    input: [{name: link_id, type: demo.snapshot.LinkId}, {name: root_id, type: demo.snapshot.RootId}]
    outcomes:
      - name: created
        creates: demo.snapshot.Link
        instance: link_id
        sets: {root_id: input.root_id, copied: {related: {via: input.root_id, field: stamp}}}
        emits: [demo.snapshot.LinkCreated]
        payload: {demo.snapshot.LinkCreated: {link_id: input.link_id}}
  - name: demo.snapshot.ChangeLink
    input: [{name: link_id, type: demo.snapshot.LinkId}, {name: next, type: demo.snapshot.RootId}]
    outcomes:
      - name: changed
        updates: demo.snapshot.Link
        emits: [demo.snapshot.Changed]
        instance: link_id
        sets: {root_id: input.next, copied: {related: {via: root_id, field: stamp}}}
  - name: demo.snapshot.AllRoots
    outcomes:
      - name: changed
        updates: demo.snapshot.Root
        emits: [demo.snapshot.Changed]
        instances: {where: flag == false}
        sets: {flag: true, stamp: {generated: true}}
";

    fn model(source: &str) -> EssIr {
        let specification = Specification::assemble([(
            Source::new("snapshot.yaml"),
            RawSpecFile::parse(source).unwrap(),
        )])
        .unwrap();
        compile(&specification, &SourceMap::new()).unwrap()
    }
    fn answer(
        ir: &EssIr,
        state: &State,
        command: &str,
        input: &[(&str, &str)],
        operation: &str,
    ) -> Result<Alternatives, Undetermined> {
        execute(
            ir,
            state,
            &format!("demo.snapshot.{command}").parse().unwrap(),
            &input
                .iter()
                .map(|(k, v)| ((*k).into(), Node::Text((*v).into())))
                .collect(),
            &super::super::Generated::Counter,
            operation,
        )
    }
    fn next(
        ir: &EssIr,
        state: &State,
        command: &str,
        input: &[(&str, &str)],
        operation: &str,
    ) -> State {
        let mut answer = answer(ir, state, command, input, operation).unwrap();
        assert!(answer.unresolved.is_none(), "{:?}", answer.unresolved);
        assert_eq!(answer.proven.len(), 1);
        answer.proven.remove(0).next
    }
    fn field(state: &State, entity: &str, key: &str, field: &str) -> Value {
        state
            .instance_typed(
                &format!("demo.snapshot.{entity}").parse().unwrap(),
                &Node::Text(key.into()),
            )
            .unwrap()
            .fields
            .get(field)
            .cloned()
            .unwrap_or(Value::Absent)
    }

    #[test]
    fn changed_reference_reads_old_unknown_row_and_missing_is_not_absent() {
        let ir = model(SOURCE);
        let first = next(
            &ir,
            &State::default(),
            "CreateRoot",
            &[("root_id", "old")],
            "create-old",
        );
        let both = next(
            &ir,
            &first,
            "CreateRoot",
            &[("root_id", "new")],
            "create-new",
        );
        assert_ne!(
            field(&both, "Root", "old", "stamp"),
            field(&both, "Root", "new", "stamp")
        );
        let cleared = next(&ir, &both, "ClearRoot", &[("root_id", "new")], "clear-new");
        let linked = next(
            &ir,
            &cleared,
            "CreateLink",
            &[("link_id", "link"), ("root_id", "old")],
            "create-link",
        );
        let changed = next(
            &ir,
            &linked,
            "ChangeLink",
            &[("link_id", "link"), ("next", "new")],
            "change-link",
        );
        assert_eq!(
            field(&changed, "Link", "link", "root_id"),
            Value::Known(Node::Text("new".into()))
        );
        assert_eq!(
            field(&changed, "Link", "link", "copied"),
            field(&both, "Root", "old", "stamp")
        );
        assert!(matches!(
            field(&changed, "Link", "link", "copied"),
            Value::Unknown { .. }
        ));
        let absent = next(
            &ir,
            &cleared,
            "CreateLink",
            &[("link_id", "empty"), ("root_id", "new")],
            "create-empty",
        );
        assert_eq!(field(&absent, "Link", "empty", "copied"), Value::Absent);
        let before = absent.clone();
        let missing = answer(
            &ir,
            &absent,
            "CreateLink",
            &[("link_id", "missing"), ("root_id", "not-held")],
            "create-missing",
        );
        assert!(
            matches!(missing, Err(Undetermined::NoValue { .. })),
            "missing row must not become optional absence"
        );
        assert_eq!(absent, before);
    }

    #[test]
    fn unresolved_touched_row_validity_discards_all_staged_rows() {
        let source = SOURCE.replace("    identity: {name: root_id, type: demo.snapshot.RootId}",
            "    identity: {name: root_id, type: demo.snapshot.RootId}\n    invariants: [{any: ['flag == false', 'not defined(stamp)']}]");
        let ir = model(&source);
        let first = next(
            &ir,
            &State::default(),
            "CreateRoot",
            &[("root_id", "a")],
            "create-a",
        );
        let initial = next(&ir, &first, "CreateRoot", &[("root_id", "b")], "create-b");
        let before = initial.clone();
        let result = answer(&ir, &initial, "AllRoots", &[], "all-roots").unwrap();
        assert_eq!(result.proven.len(), 0, "{:?}", result.proven);
        assert!(matches!(
            result.unresolved,
            Some(Undetermined::Undecidable { .. })
        ));
        assert_eq!(initial, before);
        for key in ["a", "b"] {
            assert_eq!(
                field(&initial, "Root", key, "flag"),
                Value::Known(Node::Bool(false))
            );
        }
        let control = model(&source.replace(
            "sets: {flag: true, stamp: {generated: true}}",
            "sets: {flag: true, stamp: {cleared: true}}",
        ));
        let changed = next(&control, &initial, "AllRoots", &[], "all-roots");
        for key in ["a", "b"] {
            assert_eq!(
                field(&changed, "Root", key, "flag"),
                Value::Known(Node::Bool(true))
            );
            assert_eq!(field(&changed, "Root", key, "stamp"), Value::Absent);
        }
    }
}
