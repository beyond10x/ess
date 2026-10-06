//! The issue fixture of beyond10x/ess#413 and a counter target, shared by the seed tests.
#![allow(dead_code)]

use std::cell::RefCell;
use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, refs::OutcomeRef, resolve::compile, source::SourceMap};
use ess_conformance::{
    authored,
    scenario::{ScenarioId, ScenarioStep, ScenarioValue},
    synthesize::{synthesize_with_seeds, AdmittedSeeds, SeedSelection, Synthesis},
    target::*,
    ConformanceSuite, InstanceName,
};
use ess_domain::{
    command::OutcomeName,
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::{consistency::ConsistencyToken, node::Node};
use serde_json::Value;
use sha2::{Digest as _, Sha256};

/// The issue's `system.yaml`, with `ess/20`.
pub const SYSTEM: &str = "format: ess/20
system: counter
version: v1
domains: [counter.model]
";

/// The issue's `counter.yaml`, verbatim.
pub const COUNTER: &str = r"domain: counter.model
entities:
  - name: counter.model.Counter
    identity: {name: id, type: Uuid}
    fields:
      - {name: revision, type: Integer}
    invariants: [revision >= 0, revision <= 9223372036854775807]
    lifecycle:
      initial: Active
      states: [Active]
      terminal: [Active]
commands:
  - name: counter.model.Create
    input: []
    outcomes:
      - name: created
        creates: counter.model.Counter
        instance: id
        sets: {revision: 0}
        emits: [counter.model.Created]
        payload:
          counter.model.Created: {id: {generated: true}}
  - name: counter.model.Authorize
    input:
      - {name: id, type: Uuid}
      - {name: expected_revision, type: Integer}
    outcomes:
      - name: invalid
        when: expected_revision < 0
        error: counter.model.Invalid
      - name: stale
        when_subject: {predicate: revision != input.expected_revision}
        error: counter.model.Stale
      - name: revision-exhausted
        when_subject: {predicate: revision >= 9223372036854775807}
        error: counter.model.Exhausted
      - name: authorized
        updates: counter.model.Counter
        instance: id
        sets: {revision: {increment: 1}}
        emits: [counter.model.Authorized]
        payload:
          counter.model.Authorized: {id: input.id}
      - name: unknown
        unknown_instance: true
        error: counter.model.Unknown
errors:
  - {name: counter.model.Invalid, fields: []}
  - {name: counter.model.Stale, fields: []}
  - {name: counter.model.Exhausted, fields: []}
  - {name: counter.model.Unknown, fields: []}
events:
  - name: counter.model.Authorized
    fields: [{name: id, type: Uuid}]
  - name: counter.model.Created
    fields: [{name: id, type: Uuid}]
views:
  - name: counter.model.Counters
    source: counter.model.Counter
    consistency: read_your_writes
    fields:
      - {name: id, type: Uuid}
      - {name: revision, type: Integer}
      - {name: state, type: counter.model.Counter.State}
";

/// SHA-256 of the canonical suite the base build writes for the issue fixture with no seed:
/// `ess verify conform synthesize --path . --out suite.json`. Re-pinned for beyond10x/ess#454: the
/// plain input refusal `invalid` is now also sent for an identity no row carries (step 2 of the
/// precedence order), so `Authorize/outcome/invalid` gains that send; nothing else moved.
pub const BASE_SUITE_SHA256: &str =
    "a1c2842ce87a08841bbfea6ed29225246f9d3476948fd0d7a7f97c61e1ac444e";

pub const MAX: i64 = i64::MAX;

pub fn ir_of(counter: &str) -> EssIr {
    let parsed = [
        (
            Source::new("system.yaml"),
            RawSpecFile::parse(SYSTEM).unwrap(),
        ),
        (
            Source::new("counter.yaml"),
            RawSpecFile::parse(counter).unwrap(),
        ),
    ];
    let spec = Specification::assemble(parsed).unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

/// The model these named files assemble to.
pub fn ir_files(files: &[(&str, &str)]) -> EssIr {
    let parsed: Vec<_> = files
        .iter()
        .map(|(name, text)| {
            (
                Source::new(*name),
                RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{name}: {error}")),
            )
        })
        .collect();
    let spec = Specification::assemble(parsed).unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

pub fn ir() -> EssIr {
    ir_of(COUNTER)
}

/// One `ess-scenario/2` document arranging `instance` at `revision` with literal `identity`.
pub fn seed_document(scenario: &str, instance: &str, identity: &str, revision: &str) -> String {
    format!(
        "type: ess-scenario/2
domain: counter.model
scenario: {scenario}
summary: A counter established at revision {revision}.
arrange:
  - instance: {instance}
    entity: counter.model.Counter
    setup:
      identity: {identity}
      fields: {{revision: {revision}}}
      state: Active
assert:
  - view: counter.model.Counters
    contains: {{id: {{$instance: {instance}}}, revision: {revision}}}
"
    )
}

pub const AT_MAX_ID: &str = "00000000-0000-4000-8000-00000000a001";
pub const BELOW_MAX_ID: &str = "00000000-0000-4000-8000-00000000a002";

pub fn max_source() -> authored::Source {
    authored::Source::new(
        "max.yaml",
        seed_document("counter-at-max", "at-max", AT_MAX_ID, &MAX.to_string()),
    )
}

pub fn below_source() -> authored::Source {
    authored::Source::new(
        "below.yaml",
        seed_document(
            "counter-below-max",
            "below-max",
            BELOW_MAX_ID,
            &(MAX - 1).to_string(),
        ),
    )
}

pub fn select(source: authored::Source, instance: &str) -> SeedSelection {
    SeedSelection {
        source,
        instance: InstanceName::new(instance).unwrap(),
    }
}

pub fn issue_selections() -> Vec<SeedSelection> {
    vec![
        select(max_source(), "at-max"),
        select(below_source(), "below-max"),
    ]
}

pub fn admitted(ir: &EssIr, selections: &[SeedSelection]) -> AdmittedSeeds {
    AdmittedSeeds::compile(ir, selections).unwrap_or_else(|error| panic!("{error}"))
}

pub fn seeded(ir: &EssIr) -> Synthesis {
    synthesize_with_seeds(ir, &admitted(ir, &issue_selections())).unwrap()
}

pub fn rendered(synthesis: &Synthesis) -> Vec<String> {
    synthesis.refusals.iter().map(ToString::to_string).collect()
}

pub fn id(text: &str) -> ScenarioId {
    ScenarioId::parse(text).unwrap()
}

pub const EXHAUSTED: &str = "counter.model.Authorize/outcome/revision-exhausted";
pub const AUTHORIZED: &str = "counter.model.Authorize/outcome/authorized";
pub const STALE: &str = "counter.model.Authorize/outcome/stale";

pub fn sha256(text: &str) -> String {
    use std::fmt::Write as _;
    Sha256::digest(text.as_bytes())
        .iter()
        .fold(String::new(), |mut out, byte| {
            write!(out, "{byte:02x}").expect("writing to a String");
            out
        })
}

pub fn integer(value: i64) -> Node {
    serde_json::from_str::<Node>(&value.to_string()).unwrap()
}

/// The steps of one seeded segment, starting at its `establish_entity`.
pub fn segment(steps: &[ScenarioStep], identity: &str) -> (usize, InstanceName, usize) {
    let (at, instance) = steps
        .iter()
        .enumerate()
        .find_map(|(at, step)| match step {
            ScenarioStep::EstablishEntity {
                identity: Node::Text(held),
                instance,
                ..
            } if held == identity => Some((at, instance.clone())),
            _ => None,
        })
        .unwrap_or_else(|| panic!("no setup of {identity}: {steps:#?}"));
    let command = steps
        .iter()
        .enumerate()
        .skip(at + 1)
        .find_map(|(position, step)| match step {
            ScenarioStep::ExecuteCommand { input, .. }
                if input.get("id") == Some(&ScenarioValue::instance(instance.clone())) =>
            {
                Some(position)
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("no command addressing {instance}: {steps:#?}"));
    (at, instance, command)
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Fault {
    None,
    /// The compare-and-swap is not checked.
    CasIgnored,
    /// The exhaustion guard is omitted, so MAX advances.
    GuardOmitted,
    /// Exhaustion one revision early.
    ThresholdEarly,
    /// Setup is acknowledged and nothing is stored.
    AcknowledgedOnly,
    /// Setup is not supported at all.
    Unsupported,
}

pub struct Counters<'ir> {
    ir: &'ir EssIr,
    fault: Fault,
    limit: i128,
    step: i128,
    rows: RefCell<Vec<BTreeMap<String, Node>>>,
    minted: RefCell<u64>,
}

impl<'ir> Counters<'ir> {
    /// The issue's counter: exhausted at `i64::MAX`, moved by one.
    pub fn new(ir: &'ir EssIr, fault: Fault) -> Self {
        Self::with(ir, fault, i128::from(MAX), 1)
    }
    /// A counter exhausted at `limit` and moved by `step`.
    pub fn with(ir: &'ir EssIr, fault: Fault, limit: i128, step: i128) -> Self {
        Self {
            ir,
            fault,
            limit,
            step,
            rows: RefCell::default(),
            minted: RefCell::new(0),
        }
    }
    fn token(&self) -> ConsistencyToken {
        ConsistencyToken::new(format!("seq:{}", self.minted.borrow())).unwrap()
    }
}

pub fn exact(node: &Node) -> i128 {
    let Node::Number(number) = node else {
        panic!("an integer: {node:?}")
    };
    number
        .exact_text()
        .parse::<i128>()
        .unwrap_or_else(|_| panic!("an integer: {node:?}"))
}

pub fn wide(value: i128) -> Node {
    serde_json::from_str::<Node>(&value.to_string()).unwrap()
}

impl ConformanceTarget for Counters<'_> {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("counter-fixture", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.rows.replace(Vec::new());
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.rows.replace(Vec::new());
        Ok(())
    }
    fn establish_entity(&self, request: EntitySetupRequest) -> Result<(), TargetError> {
        if self.fault == Fault::Unsupported {
            return Err(TargetError::unsupported(
                "entity setup",
                "no fixture capability",
            ));
        }
        ess_conformance::input::validate_entity_setup(
            self.ir,
            &request.entity,
            &request.identity,
            &request.fields,
            &request.state,
        )
        .map_err(|detail| TargetError::unavailable("invalid entity setup", detail))?;
        if self
            .rows
            .borrow()
            .iter()
            .any(|row| row["id"] == request.identity)
        {
            return Err(TargetError::unavailable("entity setup", "identity exists"));
        }
        if self.fault == Fault::AcknowledgedOnly {
            return Ok(());
        }
        let mut row = request.fields;
        row.insert("id".into(), request.identity);
        row.insert("state".into(), Node::Text(request.state.to_string()));
        self.rows.borrow_mut().push(row);
        Ok(())
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let command = request.command.clone();
        let took = |name: &str| {
            SemanticCommandResult::took(OutcomeRef::new(
                command.clone(),
                OutcomeName::new(name).unwrap(),
            ))
        };
        let error = |name: &str| DeclaredErrorValue::new(name.parse().unwrap());
        let event = |name: &str, id: Node| {
            ObservedEvent::new(format!("counter.model.{name}").parse().unwrap()).with("id", id)
        };
        let result = match command.to_string().as_str() {
            "counter.model.Create" => {
                *self.minted.borrow_mut() += 1;
                let id = Node::Text(format!(
                    "00000000-0000-4000-8000-{:012}",
                    *self.minted.borrow()
                ));
                let mut row = BTreeMap::new();
                row.insert("id".to_owned(), id.clone());
                row.insert("state".to_owned(), Node::Text("Active".into()));
                row.insert("revision".to_owned(), wide(0));
                self.rows.borrow_mut().push(row);
                took("created").emitting(event("Created", id))
            }
            "counter.model.Authorize" => {
                let id = request.input["id"].clone();
                let expected = exact(&request.input["expected_revision"]);
                let mut rows = self.rows.borrow_mut();
                if expected < 0 {
                    took("invalid").with_error(error("counter.model.Invalid"))
                } else if let Some(row) = rows.iter_mut().find(|row| row["id"] == id) {
                    let revision = exact(&row["revision"]);
                    let limit = self.limit - i128::from(self.fault == Fault::ThresholdEarly);
                    if self.fault != Fault::CasIgnored && revision != expected {
                        took("stale").with_error(error("counter.model.Stale"))
                    } else if self.fault != Fault::GuardOmitted && revision >= limit {
                        took("revision-exhausted").with_error(error("counter.model.Exhausted"))
                    } else {
                        row.insert("revision".to_owned(), wide(revision + self.step));
                        took("authorized").emitting(event("Authorized", id))
                    }
                } else {
                    took("unknown").with_error(error("counter.model.Unknown"))
                }
            }
            other => return Err(TargetError::unsupported("command", other)),
        };
        *self.minted.borrow_mut() += 1;
        Ok(result.with_consistency(self.token()))
    }
    fn query_view(&self, _: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        Ok(SemanticViewResult::of(self.rows.borrow().clone()))
    }
    fn observe_events(
        &self,
        _: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Ok(Vec::new())
    }
    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        panic!("nothing here is externally decided")
    }
    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
        panic!("no bindings")
    }
}

/// One forged seeded document and the refusal each reader gives it.
pub struct Forgery {
    /// What was forged.
    pub label: String,
    /// A substring of the Rust admission refusal.
    pub rust: &'static str,
    /// A substring of the generated Go and TypeScript admission refusal.
    pub runtimes: &'static str,
    /// The document.
    pub document: Value,
}

/// The seeded suite of the issue fixture as JSON, admitted as it is.
pub fn wire(suite: &ConformanceSuite) -> Value {
    serde_json::from_str(&suite.to_canonical_json().unwrap()).unwrap()
}

/// The seed record of a suite document.
fn seeds(document: &mut Value) -> &mut Value {
    &mut document["provenance"]["synthesis_seeds"]
}

/// Every forged or dangling variant of a valid seeded ordinary suite `good` (beyond10x/ess#413),
/// with what each reader must say. `authored` is `good` with one authored setup scenario added.
#[allow(clippy::too_many_lines)]
pub fn forgeries(good: &Value, authored: &Value, authored_id: &str) -> Vec<Forgery> {
    let mut out = Vec::new();
    let mut add =
        |label: &str, rust: &'static str, runtimes: &'static str, edit: &dyn Fn(&mut Value)| {
            let mut document = good.clone();
            edit(&mut document);
            out.push(Forgery {
                label: label.to_owned(),
                rust,
                runtimes,
                document,
            });
        };
    let held = "an application names a scenario the suite does not hold";
    add("dangling-scenario", held, held, &|d| {
        seeds(d)["applications"][0]["scenario"] = "counter.model.Authorize/outcome/nowhere".into();
    });
    let unknown = "an application names an unknown selection";
    add("unknown-selection", unknown, unknown, &|d| {
        seeds(d)["applications"][0]["instance"] = "other".into();
    });
    let not_setup = "establish_step is not an establish_entity step";
    add("establish-off-by-one", not_setup, not_setup, &|d| {
        let at = seeds(d)["applications"][0]["establish_step"]
            .as_u64()
            .unwrap();
        seeds(d)["applications"][0]["establish_step"] = (at + 1).into();
    });
    add("establish-out-of-range", not_setup, not_setup, &|d| {
        seeds(d)["applications"][0]["establish_step"] = 100_000.into();
    });
    let command = "command_step is not the asserted command sent to the established row";
    add("command-before-setup", command, command, &|d| {
        seeds(d)["applications"][0]["command_step"] = 0.into();
    });
    add("command-not-the-sent-one", command, command, &|d| {
        let at = seeds(d)["applications"][0]["command_step"]
            .as_u64()
            .unwrap();
        seeds(d)["applications"][0]["command_step"] = (at + 1).into();
    });
    let differs = "the established row differs from the selection";
    add("selection-row-changed", differs, differs, &|d| {
        seeds(d)["selections"][0]["fields"]["revision"] = serde_json::json!(5);
    });
    add("selection-max-rounded", differs, differs, &|d| {
        // 2^63 is the binary64 both MAX and MAX−1 round to; an exact reader tells them apart.
        let raw = serde_json::to_string(&*seeds(d))
            .unwrap()
            .replace("9223372036854775806", "9223372036854775807");
        *seeds(d) = serde_json::from_str(&raw).unwrap();
    });
    let unbound = "establishes a row no application binds";
    add("applications-dropped", unbound, unbound, &|d| {
        seeds(d)["applications"] = serde_json::json!([]);
    });
    let sorted = "applications must be sorted and distinct";
    add("applications-unsorted", sorted, sorted, &|d| {
        let mut applications = seeds(d)["applications"].as_array().unwrap().clone();
        applications.reverse();
        seeds(d)["applications"] = applications.into();
    });
    add("applications-duplicated", sorted, sorted, &|d| {
        let mut applications = seeds(d)["applications"].as_array().unwrap().clone();
        applications.push(applications[1].clone());
        seeds(d)["applications"] = applications.into();
    });
    let selections_sorted = "selections must be sorted and distinct";
    add(
        "selections-unsorted",
        selections_sorted,
        selections_sorted,
        &|d| {
            let mut selections = seeds(d)["selections"].as_array().unwrap().clone();
            selections.reverse();
            seeds(d)["selections"] = selections.into();
        },
    );
    let orphan = "a source is selected by no selection";
    add("orphan-source", orphan, orphan, &|d| {
        seeds(d)["sources"]["orphan.yaml"] = format!("sha256:{}", "0".repeat(64)).into();
    });
    let digest = "invalid source digest";
    add("short-digest", digest, digest, &|d| {
        seeds(d)["sources"]["max.yaml"] = "sha256:short".into();
    });
    add(
        "unknown-member",
        "UnknownField",
        "unknown field extra",
        &|d| {
            seeds(d)["extra"] = serde_json::json!(1);
        },
    );
    add(
        "empty-selections",
        "sources and selections must be nonempty",
        "selections must hold 1 to 64 records",
        &|d| {
            seeds(d)["selections"] = serde_json::json!([]);
        },
    );
    add(
        "seeds-under-34",
        "synthesis seeds require suite/42 or /43",
        "synthesis_seeds is required exactly in suite/42 and /43",
        &|d| {
            d["provenance"]["suite_version"] = "ess-conformance/34".into();
        },
    );
    add(
        "42-without-seeds",
        "suite/42 and /43 require synthesis_seeds",
        "synthesis_seeds is required exactly in suite/42 and /43",
        &|d| {
            d["provenance"]
                .as_object_mut()
                .unwrap()
                .remove("synthesis_seeds");
        },
    );
    add(
        "43-without-coverage",
        "coverage is required",
        "coverage is required",
        &|d| {
            d["provenance"]["suite_version"] = "ess-conformance/43".into();
        },
    );
    // Admitted majors that are not the seed pair: the record itself is what is refused.
    for major in [36, 40] {
        add(
            &format!("seeds-in-major-{major}"),
            "synthesis seeds require suite/42 or /43",
            "synthesis_seeds is required exactly in suite/42 and /43",
            &move |d| {
                d["provenance"]["suite_version"] = format!("ess-conformance/{major}").into();
            },
        );
    }
    for major in [46, 47, 99] {
        add(
            &format!("unimplemented-major-{major}"),
            "execution readers admit suite majors",
            "unsupported suite version",
            &move |d| {
                d["provenance"]["suite_version"] = format!("ess-conformance/{major}").into();
            },
        );
    }
    // A use bound to an authored scenario, added beside the two generated uses.
    let mut document = authored.clone();
    let mut applications = document["provenance"]["synthesis_seeds"]["applications"]
        .as_array()
        .unwrap()
        .clone();
    let mut bound = applications[1].clone();
    bound["scenario"] = authored_id.into();
    bound["establish_step"] = 0.into();
    applications.push(bound);
    document["provenance"]["synthesis_seeds"]["applications"] = applications.into();
    let authored_use = "an application names an authored scenario";
    out.push(Forgery {
        label: "authored-use".to_owned(),
        rust: authored_use,
        runtimes: authored_use,
        document,
    });
    out
}

/// A document as pretty JSON with its final newline.
pub fn text(value: &Value) -> String {
    serde_json::to_string_pretty(value).unwrap() + "\n"
}

/// The seeded suite, the same suite with the seed source's own authored scenario appended, and that
/// scenario's id.
pub fn documents(ir: &EssIr) -> (Value, Value, String) {
    let synthesis = seeded(ir);
    let good = wire(&synthesis.suite);
    let mut suite = synthesis.suite.clone();
    let authoring = authored::compile(ir, &[max_source()]);
    assert!(authoring.is_complete(), "{:?}", authoring.refusals);
    let (authored_id, authored_scenario) = authoring.scenarios.into_iter().next().unwrap();
    suite
        .scenarios
        .insert(authored_id.clone(), authored_scenario);
    (good, wire(&suite), authored_id.to_string())
}
