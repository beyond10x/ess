//! A one-time response's read cell names the reader (beyond10x/ess#614, option 2 of
//! `docs/design/one-time-read-cells-for-other-actors.md`).
//!
//! From suite/56 (/57 with coverage) the cell `<command>/disclosure/<outcome>/<field>/read/<view>/
//! as/actor/<A>` issues the value as an actor granted the issuing command, `A` where `A` is, and
//! then reads the view as `A` through a `read_as` step. An actor who may read the view but may not
//! issue the value gets a cell rather than a refusal. Below suite/56 a read cell keeps the shape it
//! always had: issued as `A`, read as no actor.
use std::cell::RefCell;

use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_conformance::interpret::Interpreted;
use ess_conformance::one_time_response::{Aspect, Cell};
use ess_conformance::scenario::{ActorRef, SuiteFormat};
use ess_conformance::target::{
    ConformanceTarget, EntitySetupRequest, EventObservationRequest, ExternalOutcomeControl,
    ImplementationIdentity, ObservedEvent, RecordedCommandCompletion, RedeliveryRequest,
    ScenarioContext, SemanticCommandRequest, SemanticCommandResult, SemanticViewRequest,
    SemanticViewResult, TargetError,
};
use ess_conformance::{report::Status, AdmittedSuite, ConformanceSuite, ScenarioId, ScenarioStep};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

/// `Owner` may issue a token and tag it, `Clerk` may only tag it, `Guest` may do nothing; the view
/// `vault.api.Tokens` is open to every caller.
const MODEL: &str = "format: ess/22
system: vault
version: v1
domain: vault.api
entities:
  - name: vault.api.Token
    identity: {name: token_id, type: Uuid}
    fields:
      - {name: owner, type: String}
      - {name: tag, type: Optional<String>}
    lifecycle:
      initial: Live
      states: [Live]
      terminal: [Live]
actors:
  - name: vault.api.Clerk
    attributes:
      - {name: user_id, type: String}
    may: [vault.api.TagToken]
  - name: vault.api.Guest
    attributes:
      - {name: user_id, type: String}
    may: []
  - name: vault.api.Owner
    attributes:
      - {name: user_id, type: String}
    may: [vault.api.IssueToken, vault.api.TagToken]
commands:
  - name: vault.api.IssueToken
    input:
      - {name: token_id, type: Uuid}
    response:
      - {name: secret, type: String}
    outcomes:
      - name: issued
        creates: vault.api.Token
        instance: token_id
        sets: {owner: {caller: user_id}}
        returns: true
        one_time_response: [secret]
        emits: [vault.api.TokenIssued]
        payload:
          vault.api.TokenIssued: {token_id: input.token_id}
  - name: vault.api.TagToken
    input:
      - {name: token_id, type: Uuid}
      - {name: tag, type: String}
    outcomes:
      - name: tagged
        updates: vault.api.Token
        instance: token_id
        sets: {tag: input.tag}
        emits: [vault.api.TokenTagged]
        payload:
          vault.api.TokenTagged: {token_id: input.token_id}
views:
  - name: vault.api.Tokens
    source: vault.api.Token
    consistency: read_your_writes
    fields:
      - {name: token_id, type: Uuid}
      - {name: owner, type: String}
events:
  - name: vault.api.TokenIssued
    fields:
      - {name: token_id, type: Uuid}
  - name: vault.api.TokenTagged
    fields:
      - {name: token_id, type: Uuid}
";

const ISSUER: &str = "vault.api.Owner";
const READERS: [&str; 3] = ["vault.api.Clerk", "vault.api.Guest", "vault.api.Owner"];

fn model(text: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("system.yaml"),
        RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}")),
    )])
    .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

/// The model with the view read-granted to `Owner` alone.
fn read_granted() -> String {
    MODEL.replace(
        "may: [vault.api.IssueToken, vault.api.TagToken]",
        "may: [vault.api.IssueToken, vault.api.TagToken, vault.api.Tokens]",
    )
}

fn read_id(reader: Option<&str>) -> String {
    match reader {
        Some(reader) => format!(
            "vault.api.IssueToken/disclosure/issued/secret/read/vault.api.Tokens/as/actor/{reader}"
        ),
        None => "vault.api.IssueToken/disclosure/issued/secret/read/vault.api.Tokens/as/anonymous"
            .to_owned(),
    }
}

fn is_read(id: &ScenarioId) -> bool {
    matches!(id, ScenarioId::Disclosure { cell } if matches!(cell.aspect, Aspect::Read(_)))
}

/// The actor each `execute_command` of `IssueToken` is sent as, and the reader in force at each
/// read of the view, in step order.
fn senders_and_readers(steps: &[ScenarioStep]) -> (Vec<Option<String>>, Vec<Option<String>>) {
    let mut reader = None;
    let (mut senders, mut readers) = (Vec::new(), Vec::new());
    for step in steps {
        match step {
            ScenarioStep::ReadAs { actor } => reader = actor.as_ref().map(ToString::to_string),
            ScenarioStep::ExecuteCommand { command, actor, .. }
                if command.to_string() == "vault.api.IssueToken" =>
            {
                senders.push(actor.as_ref().map(ToString::to_string));
            }
            ScenarioStep::QueryView { view, .. } if view.to_string() == "vault.api.Tokens" => {
                readers.push(reader.clone());
            }
            _ => {}
        }
    }
    (senders, readers)
}

#[test]
fn every_actor_reads_the_view_as_itself_after_a_granted_actor_issues_the_value() {
    let synthesis = ess_conformance::synthesize(&model(MODEL));
    assert_eq!(
        synthesis.suite.provenance.suite_version.to_string(),
        "ess-conformance/56"
    );
    let refused: Vec<String> = synthesis
        .refusals
        .iter()
        .filter_map(|refusal| refusal.scenario.as_ref())
        .filter(|id| is_read(id))
        .map(ToString::to_string)
        .collect();
    assert_eq!(refused, Vec::<String>::new(), "no read cell is refused");
    for reader in READERS {
        let id = read_id(Some(reader));
        let scenario = synthesis
            .suite
            .scenario(&ScenarioId::parse(&id).unwrap())
            .unwrap_or_else(|| panic!("`{id}` is built"));
        let (senders, readers) = senders_and_readers(&scenario.steps);
        // Issued twice (origin and rotation), each time as the granted issuer.
        assert_eq!(senders, vec![Some(ISSUER.to_owned()); 2], "`{id}`");
        // The origin's own check reads on the harness's authority; from the cell's first read on,
        // before, within and after the rotation, every read is sent as the cell's actor.
        assert_eq!(readers.first(), Some(&None), "`{id}`");
        assert_eq!(
            cell_reads(&readers),
            vec![Some(reader.to_owned()); 3],
            "`{id}`"
        );
    }
}

/// The reads from the first one sent as an actor on.
fn cell_reads(readers: &[Option<String>]) -> Vec<Option<String>> {
    readers
        .iter()
        .skip_while(|reader| reader.is_none())
        .cloned()
        .collect()
}

#[test]
fn the_reader_cells_pass_on_the_interpreted_target() {
    let ir = model(MODEL);
    let synthesis = ess_conformance::synthesize(&ir);
    let admitted = AdmittedSuite::from_suite(&synthesis.suite).unwrap_or_else(|e| panic!("{e}"));
    let report = ess_conformance::Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(ir))
        .into_report();
    let failing: Vec<(String, Status)> = report
        .scenarios
        .iter()
        .filter(|scenario| scenario.status != Status::Passed)
        .map(|scenario| (scenario.scenario.to_string(), scenario.status))
        .collect();
    assert_eq!(failing, Vec::new());
    for reader in READERS {
        assert!(
            report
                .scenarios
                .iter()
                .any(|scenario| scenario.scenario.to_string() == read_id(Some(reader))),
            "`{reader}`'s read cell runs"
        );
    }
}

/// The interpreted target, showing the latest issued secret in the `owner` column of every row it
/// answers to a read sent as `leaks_to`.
struct LeaksToReader {
    inner: Interpreted,
    leaks_to: &'static str,
    secret: RefCell<Option<Node>>,
}

impl ConformanceTarget for LeaksToReader {
    fn establish_entity(&self, request: EntitySetupRequest) -> Result<(), TargetError> {
        self.inner.establish_entity(request)
    }
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.inner.identity()
    }
    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        *self.secret.borrow_mut() = None;
        self.inner.begin_scenario(scenario)
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        self.execute_command_recorded(request).answer
    }
    fn execute_command_recorded(
        &self,
        request: SemanticCommandRequest,
    ) -> RecordedCommandCompletion {
        let completion = self.inner.execute_command_recorded(request);
        if let Ok(result) = &completion.answer {
            if let Some(secret) = result
                .response
                .as_ref()
                .and_then(|response| response.get("secret"))
            {
                *self.secret.borrow_mut() = Some(secret.clone());
            }
        }
        completion
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        self.inner.query_view(request)
    }
    fn query_view_as(
        &self,
        request: SemanticViewRequest,
        reader: &ActorRef,
    ) -> Result<SemanticViewResult, TargetError> {
        let mut result = self.inner.query_view_as(request, reader)?;
        if reader.to_string() == self.leaks_to {
            if let Some(secret) = &*self.secret.borrow() {
                for row in &mut result.rows {
                    row.insert("owner".to_owned(), secret.clone());
                }
            }
        }
        Ok(result)
    }
    fn query_view_anonymous(
        &self,
        request: SemanticViewRequest,
    ) -> Result<SemanticViewResult, TargetError> {
        self.inner.query_view_anonymous(request)
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        self.inner.observe_events(request)
    }
    fn configure_external_outcome(
        &self,
        request: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        self.inner.configure_external_outcome(request)
    }
    fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError> {
        self.inner.redeliver_event(request)
    }
    fn end_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.end_scenario(scenario)
    }
}

/// A target that shows the value to one reader who may not issue it fails exactly that reader's
/// cell, which a read sent as no actor could never catch.
#[test]
fn a_target_showing_the_value_to_a_reader_who_cannot_issue_it_fails_that_readers_cell() {
    let ir = model(MODEL);
    let synthesis = ess_conformance::synthesize(&ir);
    let admitted = AdmittedSuite::from_suite(&synthesis.suite).unwrap_or_else(|e| panic!("{e}"));
    let target = LeaksToReader {
        inner: Interpreted::for_model(ir),
        leaks_to: "vault.api.Guest",
        secret: RefCell::new(None),
    };
    let report = ess_conformance::Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &target)
        .into_report();
    let failing: Vec<(String, Status)> = report
        .scenarios
        .iter()
        .filter(|scenario| scenario.status != Status::Passed)
        .map(|scenario| (scenario.scenario.to_string(), scenario.status))
        .collect();
    assert_eq!(
        failing,
        vec![(read_id(Some("vault.api.Guest")), Status::Failed)]
    );
}

/// Where the view's grant does not name the reader, the read is refused before anything is read;
/// that refusal is the view-grant family's to check, so the cell is refused naming it. The
/// granted reader's cell is built and reads as that reader.
#[test]
fn a_reader_the_view_grant_does_not_name_is_refused_by_name() {
    let ir = model(&read_granted());
    let synthesis = ess_conformance::synthesize(&ir);
    let owner = read_id(Some(ISSUER));
    let scenario = synthesis
        .suite
        .scenario(&ScenarioId::parse(&owner).unwrap())
        .unwrap_or_else(|| panic!("`{owner}` is built"));
    assert_eq!(
        cell_reads(&senders_and_readers(&scenario.steps).1),
        vec![Some(ISSUER.to_owned()); 3]
    );
    for reader in ["vault.api.Clerk", "vault.api.Guest"] {
        let id = read_id(Some(reader));
        assert!(
            synthesis
                .suite
                .scenario(&ScenarioId::parse(&id).unwrap())
                .is_none(),
            "`{id}` is built"
        );
        let refusal = synthesis
            .refusals
            .iter()
            .find(|refusal| refusal.scenario.as_ref().map(ToString::to_string) == Some(id.clone()))
            .unwrap_or_else(|| panic!("`{id}` is refused"));
        let text = format!("{:?}", refusal.cause);
        assert!(
            text.contains("grant does not name the reader"),
            "`{id}`: {text}"
        );
    }
    let admitted = AdmittedSuite::from_suite(&synthesis.suite).unwrap_or_else(|e| panic!("{e}"));
    let report = ess_conformance::Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(ir))
        .into_report();
    let status = report
        .scenarios
        .iter()
        .find(|scenario| scenario.scenario.to_string() == owner)
        .map(|scenario| scenario.status);
    assert_eq!(status, Some(Status::Passed));
}

/// The rotation a cell appends checks its own outcome by reading every view of the token. A second
/// view read-granted to `Owner` alone is read there as `Owner`, and the cell's reader is put back
/// for its closing read, so every cell passes on a target enforcing the grant.
#[test]
fn the_rotation_reads_a_view_the_reader_may_not_read_as_a_reader_its_grant_names() {
    let text = MODEL
        .replace(
            "may: [vault.api.IssueToken, vault.api.TagToken]",
            "may: [vault.api.IssueToken, vault.api.TagToken, vault.api.Owners]",
        )
        .replace(
            "events:\n",
            "  - name: vault.api.Owners\n    source: vault.api.Token\n    consistency: \
             read_your_writes\n    fields:\n      - {name: token_id, type: Uuid}\n      - {name: \
             owner, type: String}\nevents:\n",
        );
    let ir = model(&text);
    let synthesis = ess_conformance::synthesize(&ir);
    let id = read_id(Some("vault.api.Clerk"));
    let scenario = synthesis
        .suite
        .scenario(&ScenarioId::parse(&id).unwrap())
        .unwrap_or_else(|| panic!("`{id}` is built"));
    let mut reader = None;
    let mut owners_read_as = Vec::new();
    for step in &scenario.steps {
        match step {
            ScenarioStep::ReadAs { actor } => reader = actor.as_ref().map(ToString::to_string),
            ScenarioStep::QueryView { view, .. } if view.to_string() == "vault.api.Owners" => {
                owners_read_as.push(reader.clone());
            }
            _ => {}
        }
    }
    assert!(
        owners_read_as.contains(&Some(ISSUER.to_owned())),
        "`{id}` reads `vault.api.Owners` as its granted reader: {owners_read_as:?}"
    );
    assert!(
        !owners_read_as.contains(&Some("vault.api.Clerk".to_owned())),
        "{owners_read_as:?}"
    );
    assert_eq!(
        senders_and_readers(&scenario.steps).1.last(),
        Some(&Some("vault.api.Clerk".to_owned()))
    );
    let admitted = AdmittedSuite::from_suite(&synthesis.suite).unwrap_or_else(|e| panic!("{e}"));
    let report = ess_conformance::Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(ir))
        .into_report();
    let failing: Vec<(String, Status)> = report
        .scenarios
        .iter()
        .filter(|scenario| scenario.status != Status::Passed)
        .map(|scenario| (scenario.scenario.to_string(), scenario.status))
        .collect();
    assert_eq!(failing, Vec::new());
}

/// A model that declares no actor reads as no actor, as before: its read cell gains no step and
/// its suite keeps its format.
#[test]
fn a_model_without_actors_keeps_its_read_cell_and_its_format() {
    let text = MODEL
        .split("actors:\n")
        .next()
        .map(|head| {
            let tail = MODEL.split("commands:\n").nth(1).unwrap();
            format!("{head}commands:\n{tail}")
        })
        .unwrap()
        .replace(
            "sets: {owner: {caller: user_id}}",
            "sets: {owner: \"vault\"}",
        );
    let synthesis = ess_conformance::synthesize(&model(&text));
    assert_eq!(
        synthesis.suite.provenance.suite_version.to_string(),
        "ess-conformance/52"
    );
    let id = read_id(None);
    let scenario = synthesis
        .suite
        .scenario(&ScenarioId::parse(&id).unwrap())
        .unwrap_or_else(|| panic!("`{id}` is built"));
    assert!(
        !scenario
            .steps
            .iter()
            .any(|step| matches!(step, ScenarioStep::ReadAs { .. })),
        "`{id}` reads as no actor without a step saying so"
    );
}

/// The suite with every read cell's `read_as` removed and labelled `version`.
fn without_read_as(suite: &ConformanceSuite, version: &str) -> String {
    let mut document: serde_json::Value =
        serde_json::from_str(&suite.to_canonical_json().unwrap()).unwrap();
    document["provenance"]["suite_version"] = version.into();
    for (id, scenario) in document["scenarios"].as_object_mut().unwrap() {
        if is_read(&ScenarioId::parse(id).unwrap()) {
            scenario["steps"]
                .as_array_mut()
                .unwrap()
                .retain(|step| step["step"] != "read_as");
        }
    }
    document.to_string()
}

/// At suite/56 a read cell's actor is the reader: a cell read as anyone else is refused. Below
/// it the actor is the issuer, as it always was: a suite of such cells is admitted, and one whose
/// cell names a reader who did not issue the value is refused.
#[test]
fn admission_reads_the_cells_actor_by_the_suites_major() {
    let suite = ess_conformance::synthesize(&model(MODEL)).suite;
    AdmittedSuite::from_json(&suite.to_canonical_json().unwrap())
        .unwrap_or_else(|error| panic!("the synthesized suite is admitted: {error}"));

    let refused = AdmittedSuite::from_json(&without_read_as(&suite, "ess-conformance/56"))
        .expect_err("a suite/56 read cell read as no actor is refused");
    assert!(
        refused
            .to_string()
            .contains("disclosure identity has no matching follow-up obligation"),
        "{refused}"
    );

    // The issuer's own cell, read as no actor, is what every major below suite/56 holds.
    let mut older = suite.clone();
    older
        .scenarios
        .retain(|id, _| !is_read(id) || id.to_string() == read_id(Some(ISSUER)));
    AdmittedSuite::from_json(&without_read_as(&older, "ess-conformance/54"))
        .unwrap_or_else(|error| panic!("an issuer's read cell is admitted at /54: {error}"));
    AdmittedSuite::from_json(&without_read_as(&suite, "ess-conformance/54"))
        .expect_err("at /54 a read cell's actor issues the value");
}

#[test]
fn coverage_suites_holding_a_reader_cell_are_57() {
    let input = ess_conformance::coverage_build::build(
        &model(MODEL),
        &[],
        ess_conformance::coverage::Scope::System,
        ess_conformance::coverage::Origins::Generated,
    )
    .unwrap_or_else(|error| panic!("{error}"));
    let envelope: serde_json::Value =
        serde_json::from_str(&input.document().to_canonical_json().unwrap()).unwrap();
    let document = envelope["suite_json"].as_str().expect("the coverage suite");
    let suite = ConformanceSuite::from_json(document).unwrap();
    assert_eq!(
        suite.provenance.suite_version.to_string(),
        "ess-conformance/57"
    );
    AdmittedSuite::from_json(document).unwrap_or_else(|error| panic!("{error}"));
}

#[test]
fn the_first_unread_majors_are_62_and_63() {
    for major in [56, 57, 58, 59, 60, 61] {
        assert!(SuiteFormat::parse(&format!("ess-conformance/{major}"))
            .unwrap()
            .is_supported());
    }
    for major in [62, 63] {
        assert!(!SuiteFormat::parse(&format!("ess-conformance/{major}"))
            .unwrap()
            .is_supported());
    }
}

/// The cell's id still parses to the reader: the identity spelling is unchanged.
#[test]
fn the_cell_identity_is_unchanged() {
    let id = read_id(Some("vault.api.Guest"));
    let cell = Cell::parse(&id).unwrap();
    assert_eq!(
        cell.actor.as_ref().map(ToString::to_string).as_deref(),
        Some("vault.api.Guest")
    );
    assert_eq!(cell.to_string(), id);
}
