//! The shapes adversary pass 1 found on `when_related:` (beyond10x/ess#211), each synthesized and
//! then run: against a hand-written target answering the model, which must pass every scenario of
//! the guarded command, and against mutants of it — one ignoring the related row, one inverting
//! `exists`, one reading the wrong row — each of which must fail at least one.
//!
//! * a predicate over a related enum field alone, witnessed on both sides (F2);
//! * a folder created inside an existing folder, the parent arranged through the root creator (F1);
//! * an accepting `when:` branch overlapping `exists: false`: on a missing row `exists: false`
//!   answers, whatever the input (F3, F4);
//! * `existing_instance:` beside `when_related:`: a taken identity is refused as taken even where
//!   the related row is missing, because the identity is checked first (F5).
use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::report::Status;
use ess_conformance::scenario::{CommandRef, ErrorRef, EventRef, OutcomeRef};
use ess_conformance::target::*;
use ess_conformance::{
    synthesize::Synthesis, AdmittedSuite, ConformanceScenario, Runner, ScenarioStep, ScenarioValue,
};
use ess_domain::{
    command::OutcomeName,
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::node::Node;

const SIGN_IN: &str = include_str!("fixtures/related-guard-sign-in.yaml");

fn replaced(text: &str, from: &str, to: &str) -> String {
    let out = text.replace(from, to);
    assert_ne!(out, text, "`{from}` is in the text");
    out
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn synthesis(text: &str) -> Synthesis {
    ess_conformance::synthesize::synthesize(&ir(text))
}

fn refusals_about(result: &Synthesis, command: &str) -> Vec<String> {
    result
        .refusals
        .iter()
        .filter(|refusal| format!("{refusal:?}").contains(command))
        .map(|refusal| format!("{}: {}", refusal.cause.code(), refusal.cause))
        .collect()
}

fn scenario<'a>(result: &'a Synthesis, id: &str) -> &'a ConformanceScenario {
    result
        .suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map_or_else(
            || {
                panic!(
                    "no scenario {id}: {:#?}\nrefusals: {:#?}",
                    result
                        .suite
                        .scenarios
                        .keys()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>(),
                    result
                        .refusals
                        .iter()
                        .map(|refusal| format!("{refusal:?}"))
                        .collect::<Vec<_>>()
                )
            },
            |(_, scenario)| scenario,
        )
}

/// Every `command` the scenario sends, with its input, in step order.
fn sent<'a>(
    scenario: &'a ConformanceScenario,
    command: &str,
) -> Vec<&'a BTreeMap<String, ScenarioValue>> {
    scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand {
                command: named,
                input,
                ..
            } if named.to_string() == command => Some(input),
            _ => None,
        })
        .collect()
}

fn text(node: Option<&Node>) -> String {
    match node {
        Some(Node::Text(value)) => value.clone(),
        other => panic!("expected text, got {other:?}"),
    }
}

// ---- a generic in-memory target ------------------------------------------------------------

/// Rows by entity, in creation order, and what this scenario published.
#[derive(Default)]
struct Store {
    minted: Cell<u64>,
    rows: RefCell<BTreeMap<String, Vec<BTreeMap<String, Node>>>>,
    published: RefCell<Vec<ObservedEvent>>,
}

impl Store {
    fn mint(&self) -> String {
        self.minted.set(self.minted.get() + 1);
        format!("00000000-0000-4000-8000-{:012}", self.minted.get())
    }

    fn insert(&self, entity: &str, row: BTreeMap<String, Node>) {
        self.rows
            .borrow_mut()
            .entry(entity.to_owned())
            .or_default()
            .push(row);
    }

    fn find(&self, entity: &str, key: &str, id: &str) -> Option<BTreeMap<String, Node>> {
        self.rows.borrow().get(entity).and_then(|rows| {
            rows.iter()
                .find(|row| row.get(key) == Some(&Node::Text(id.to_owned())))
                .cloned()
        })
    }

    fn first(&self, entity: &str) -> Option<BTreeMap<String, Node>> {
        self.rows
            .borrow()
            .get(entity)
            .and_then(|rows| rows.first().cloned())
    }
}

/// How a mutant reads the related row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    Correct,
    /// Never reads it: takes the command's other branches as if the row existed and held nothing.
    IgnoresRelated,
    /// Reads existence the wrong way round.
    InvertsExists,
    /// Reads the first row of the entity rather than the one the input names.
    ReadsFirstRow,
}

/// What a handler sees of the related row: `None` where it reads none.
fn related(
    mode: Mode,
    store: &Store,
    entity: &str,
    key: &str,
    id: &str,
) -> Option<BTreeMap<String, Node>> {
    let named = store.find(entity, key, id);
    match mode {
        Mode::Correct => named,
        Mode::IgnoresRelated => Some(BTreeMap::new()),
        Mode::InvertsExists => match named {
            Some(_) => None,
            None => Some(store.first(entity).unwrap_or_default()),
        },
        Mode::ReadsFirstRow => store.first(entity),
    }
}

type Handler = fn(Mode, &Store, &CommandRef, &BTreeMap<String, Node>) -> SemanticCommandResult;

struct Service {
    mode: Mode,
    handler: Handler,
    /// The entity each view projects, and the state its rows are in.
    views: &'static [(&'static str, &'static str, &'static str)],
    store: Store,
}

impl Service {
    fn new(
        mode: Mode,
        handler: Handler,
        views: &'static [(&'static str, &'static str, &'static str)],
    ) -> Self {
        Self {
            mode,
            handler,
            views,
            store: Store::default(),
        }
    }
}

fn branch(command: &CommandRef, name: &str) -> OutcomeRef {
    OutcomeRef::new(command.clone(), OutcomeName::new(name).unwrap())
}

fn refusal(command: &CommandRef, name: &str, error: &str) -> SemanticCommandResult {
    SemanticCommandResult::took(branch(command, name))
        .with_error(DeclaredErrorValue::new(error.parse::<ErrorRef>().unwrap()))
}

fn took(
    command: &CommandRef,
    name: &str,
    event: &str,
    field: &str,
    id: &str,
) -> SemanticCommandResult {
    SemanticCommandResult::took(branch(command, name)).emitting(
        ObservedEvent::new(event.parse::<EventRef>().unwrap()).with(field, Node::Text(id.into())),
    )
}

impl ConformanceTarget for Service {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("related-guard-fixture", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.store.rows.replace(BTreeMap::new());
        self.store.published.replace(Vec::new());
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let result = (self.handler)(self.mode, &self.store, &request.command, &request.input);
        for event in &result.direct_events {
            self.store.published.borrow_mut().push(event.clone());
        }
        let token = self.store.mint();
        Ok(result.with_consistency(
            ess_primitives::consistency::ConsistencyToken::new(format!("seq:{token}")).unwrap(),
        ))
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let name = request.view.to_string();
        let (_, entity, state) = self
            .views
            .iter()
            .find(|(view, ..)| *view == name)
            .unwrap_or_else(|| panic!("no view {name}"));
        let rows = self
            .store
            .rows
            .borrow()
            .get(*entity)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .map(|mut row| {
                row.insert("state".to_owned(), Node::Text((*state).to_owned()));
                row
            })
            .collect::<Vec<_>>();
        Ok(SemanticViewResult::of(rows))
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Ok(self
            .store
            .published
            .borrow()
            .iter()
            .filter(|event| event.event == request.event)
            .cloned()
            .collect())
    }
    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        panic!("nothing here is externally decided")
    }
    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
        panic!("no bindings")
    }
}

/// Every scenario about `command` and its status against `target`.
fn statuses(result: &Synthesis, target: &Service, command: &str) -> BTreeMap<String, String> {
    let admitted =
        AdmittedSuite::from_suite(&result.suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, target)
        .into_report()
        .scenarios
        .into_iter()
        .filter(|run| run.scenario.to_string().contains(command))
        .map(|run| {
            (
                run.scenario.to_string(),
                if run.status == Status::Passed {
                    "passed".to_owned()
                } else {
                    format!("{:?}: {:?}", run.status, run.checks)
                },
            )
        })
        .collect()
}

fn failed(statuses: &BTreeMap<String, String>) -> Vec<&String> {
    statuses
        .iter()
        .filter(|(_, status)| *status != "passed")
        .map(|(id, _)| id)
        .collect()
}

/// The correct service passes every scenario about `command` (and runs every `ids`), and each
/// mutant fails the scenario named beside it.
fn holds(
    text: &str,
    command: &str,
    ids: &[&str],
    handler: Handler,
    views: &'static [(&'static str, &'static str, &'static str)],
    mutants: &[(Mode, &str)],
) {
    let result = synthesis(text);
    assert_eq!(refusals_about(&result, command), Vec::<String>::new());
    let correct = statuses(
        &result,
        &Service::new(Mode::Correct, handler, views),
        command,
    );
    for id in ids {
        assert!(correct.contains_key(*id), "{id} is run: {correct:#?}");
    }
    assert!(failed(&correct).is_empty(), "{correct:#?}");
    for (mode, fails) in mutants {
        let run = statuses(&result, &Service::new(*mode, handler, views), command);
        assert!(
            failed(&run).contains(&&(*fails).to_owned()),
            "{mode:?} passes {fails}: {run:#?}"
        );
    }
}

// ---- F2: a predicate over a related enum field alone ----------------------------------------

/// The fixture with the configuration on a plan, and the refusal reading that stored field alone.
fn planned() -> String {
    let text = replaced(
        SIGN_IN,
        "  - {name: demo.signin.SignInId, kind: newtype, of: Uuid}\n",
        "  - {name: demo.signin.SignInId, kind: newtype, of: Uuid}\n  - name: demo.signin.Plan\n    kind: enum\n    variants: [Basic, Premium]\n",
    );
    let text = replaced(
        &text,
        "      - {name: redirect_client, type: demo.signin.ClientId}\n    lifecycle",
        "      - {name: redirect_client, type: demo.signin.ClientId}\n      - {name: plan, type: demo.signin.Plan}\n    lifecycle",
    );
    let text = replaced(
        &text,
        "      - {name: redirect_client, type: demo.signin.ClientId}\n    outcomes:",
        "      - {name: redirect_client, type: demo.signin.ClientId}\n      - {name: plan, type: demo.signin.Plan}\n    outcomes:",
    );
    let text = replaced(
        &text,
        "          redirect_client: input.redirect_client\n",
        "          redirect_client: input.redirect_client\n          plan: input.plan\n",
    );
    let text = replaced(
        &text,
        "      - {name: redirect_client, type: demo.signin.ClientId}\n  - name: demo.signin.SignIns",
        "      - {name: redirect_client, type: demo.signin.ClientId}\n      - {name: plan, type: demo.signin.Plan}\n  - name: demo.signin.SignIns",
    );
    replaced(
        &text,
        "        when_related: {via: input.tenant, predicate: redirect_client != input.client}\n",
        "        when_related: {via: input.tenant, predicate: plan == Basic}\n",
    )
}

const SIGN_IN_VIEWS: &[(&str, &str, &str)] = &[
    ("demo.signin.Configurations", "Configuration", "Active"),
    ("demo.signin.SignIns", "SignIn", "Initiated"),
];

fn planned_service(
    mode: Mode,
    store: &Store,
    command: &CommandRef,
    input: &BTreeMap<String, Node>,
) -> SemanticCommandResult {
    match command.to_string().as_str() {
        "demo.signin.ConfigureTenant" => {
            let tenant = store.mint();
            store.insert(
                "Configuration",
                BTreeMap::from([
                    ("tenant".to_owned(), Node::Text(tenant.clone())),
                    (
                        "redirect_client".to_owned(),
                        input["redirect_client"].clone(),
                    ),
                    ("plan".to_owned(), input["plan"].clone()),
                ]),
            );
            took(
                command,
                "configured",
                "demo.signin.TenantConfigured",
                "tenant",
                &tenant,
            )
        }
        "demo.signin.InitiateSignIn" => {
            let tenant = text(input.get("tenant"));
            match related(mode, store, "Configuration", "tenant", &tenant) {
                None => refusal(command, "no-configuration", "demo.signin.NoConfiguration"),
                Some(row) if row.get("plan") == Some(&Node::Text("Basic".into())) => {
                    refusal(command, "no-redirect-entry", "demo.signin.NoRedirectEntry")
                }
                Some(_) => {
                    let id = store.mint();
                    store.insert(
                        "SignIn",
                        BTreeMap::from([
                            ("sign_in_id".to_owned(), Node::Text(id.clone())),
                            ("tenant".to_owned(), Node::Text(tenant)),
                            ("client".to_owned(), input["client"].clone()),
                        ]),
                    );
                    took(
                        command,
                        "initiated",
                        "demo.signin.SignInInitiated",
                        "sign_in_id",
                        &id,
                    )
                }
            }
        }
        _ => SemanticCommandResult::undeclared(),
    }
}

#[test]
fn f2_a_predicate_over_a_related_enum_field_is_witnessed_on_rows_holding_each_variant() {
    let result = synthesis(&planned());
    // The rows each branch is sent for hold the variant that selects it.
    for (id, plan) in [
        (
            "demo.signin.InitiateSignIn/outcome/no-redirect-entry",
            "Basic",
        ),
        ("demo.signin.InitiateSignIn/outcome/initiated", "Premium"),
    ] {
        let plans: Vec<String> = sent(scenario(&result, id), "demo.signin.ConfigureTenant")
            .iter()
            .map(|input| format!("{:?}", input.get("plan")))
            .collect();
        assert!(
            plans.iter().any(|sent| sent.contains(plan)),
            "{id} arranges a {plan} configuration: {plans:?}"
        );
    }
    holds(
        &planned(),
        "InitiateSignIn",
        &[
            "demo.signin.InitiateSignIn/outcome/no-configuration",
            "demo.signin.InitiateSignIn/outcome/no-redirect-entry",
            "demo.signin.InitiateSignIn/outcome/initiated",
        ],
        planned_service,
        SIGN_IN_VIEWS,
        &[
            (
                Mode::IgnoresRelated,
                "demo.signin.InitiateSignIn/outcome/no-configuration",
            ),
            (
                Mode::InvertsExists,
                "demo.signin.InitiateSignIn/outcome/initiated",
            ),
            (
                Mode::ReadsFirstRow,
                "demo.signin.InitiateSignIn/outcome/initiated",
            ),
        ],
    );
}

// ---- F1: a folder inside an existing folder --------------------------------------------------

const FOLDERS: &str = "format: ess/18
system: demo
version: v1
domain: demo.folders
types:
  - {name: demo.folders.FolderId, kind: newtype, of: Uuid}
  - {name: demo.folders.Label, kind: newtype, of: String}
entities:
  - name: demo.folders.Folder
    identity: {name: folder_id, type: demo.folders.FolderId}
    fields:
      - {name: label, type: demo.folders.Label}
    lifecycle: {initial: Active, states: [Active], terminal: [Active]}
errors:
  - {name: demo.folders.NoParent, summary: The parent folder does not exist., fields: []}
events:
  - name: demo.folders.FolderCreated
    fields:
      - {name: folder_id, type: demo.folders.FolderId}
actors:
  - name: demo.folders.Owner
    may: [demo.folders.CreateFolder, demo.folders.CreateRoot]
commands:
  - name: demo.folders.CreateFolder
    input:
      - {name: parent, type: demo.folders.FolderId}
      - {name: label, type: demo.folders.Label}
    outcomes:
      - name: no-parent
        when_related: {via: input.parent, exists: false}
        error: demo.folders.NoParent
      - name: created
        creates: demo.folders.Folder
        instance: folder_id
        emits: [demo.folders.FolderCreated]
        payload:
          demo.folders.FolderCreated: {folder_id: {generated: true}}
        sets:
          label: input.label
  - name: demo.folders.CreateRoot
    input:
      - {name: label, type: demo.folders.Label}
    outcomes:
      - name: created
        creates: demo.folders.Folder
        instance: folder_id
        emits: [demo.folders.FolderCreated]
        payload:
          demo.folders.FolderCreated: {folder_id: {generated: true}}
        sets:
          label: input.label
views:
  - name: demo.folders.Folders
    source: demo.folders.Folder
    consistency: read_your_writes
    fields:
      - {name: folder_id, type: demo.folders.FolderId}
      - {name: label, type: demo.folders.Label}
";

const FOLDER_VIEWS: &[(&str, &str, &str)] = &[("demo.folders.Folders", "Folder", "Active")];

fn folder_service(
    mode: Mode,
    store: &Store,
    command: &CommandRef,
    input: &BTreeMap<String, Node>,
) -> SemanticCommandResult {
    let create = |store: &Store| {
        let id = store.mint();
        store.insert(
            "Folder",
            BTreeMap::from([
                ("folder_id".to_owned(), Node::Text(id.clone())),
                ("label".to_owned(), input["label"].clone()),
            ]),
        );
        took(
            command,
            "created",
            "demo.folders.FolderCreated",
            "folder_id",
            &id,
        )
    };
    match command.to_string().as_str() {
        "demo.folders.CreateRoot" => create(store),
        "demo.folders.CreateFolder" => {
            let parent = text(input.get("parent"));
            match related(mode, store, "Folder", "folder_id", &parent) {
                None => refusal(command, "no-parent", "demo.folders.NoParent"),
                Some(_) => create(store),
            }
        }
        _ => SemanticCommandResult::undeclared(),
    }
}

#[test]
fn f1_a_folder_is_created_inside_a_parent_arranged_through_the_root_creator() {
    let result = synthesis(FOLDERS);
    let created = scenario(&result, "demo.folders.CreateFolder/outcome/created");
    assert!(
        !sent(created, "demo.folders.CreateRoot").is_empty(),
        "the parent is arranged through `CreateRoot`: {created:#?}"
    );
    let folder = sent(created, "demo.folders.CreateFolder");
    assert!(
        matches!(
            folder.last().and_then(|input| input.get("parent")),
            Some(ScenarioValue::Instance { .. })
        ),
        "the folder is created in the arranged parent: {folder:?}"
    );
    holds(
        FOLDERS,
        "CreateFolder",
        &[
            "demo.folders.CreateFolder/outcome/no-parent",
            "demo.folders.CreateFolder/outcome/created",
        ],
        folder_service,
        FOLDER_VIEWS,
        &[
            (
                Mode::IgnoresRelated,
                "demo.folders.CreateFolder/outcome/no-parent",
            ),
            (
                Mode::InvertsExists,
                "demo.folders.CreateFolder/outcome/created",
            ),
        ],
    );
}

// ---- F3, F4: an accepting `when:` branch overlapping `exists: false` --------------------------

fn fast_path() -> String {
    replaced(
        &planned(),
        "      - name: initiated\n",
        "      - name: fast-path
        when: client == \"console\"
        creates: demo.signin.SignIn
        instance: sign_in_id
        emits: [demo.signin.SignInInitiated]
        payload:
          demo.signin.SignInInitiated: {sign_in_id: {generated: true}}
        sets:
          tenant: input.tenant
          client: input.client
      - name: initiated
",
    )
}

/// A service taking the fast path before it reads the configuration: the order the ruling refuses.
#[derive(Clone, Copy)]
enum Order {
    RelatedFirst,
    InputFirst,
}

fn fast_path_answer(
    order: Order,
    mode: Mode,
    store: &Store,
    command: &CommandRef,
    input: &BTreeMap<String, Node>,
) -> SemanticCommandResult {
    if command.to_string() != "demo.signin.InitiateSignIn" {
        return planned_service(mode, store, command, input);
    }
    let tenant = text(input.get("tenant"));
    let client = text(input.get("client"));
    let create = |name: &str| {
        let id = store.mint();
        store.insert(
            "SignIn",
            BTreeMap::from([
                ("sign_in_id".to_owned(), Node::Text(id.clone())),
                ("tenant".to_owned(), Node::Text(tenant.clone())),
                ("client".to_owned(), Node::Text(client.clone())),
            ]),
        );
        took(
            command,
            name,
            "demo.signin.SignInInitiated",
            "sign_in_id",
            &id,
        )
    };
    if matches!(order, Order::InputFirst) && client == "console" {
        return create("fast-path");
    }
    match related(mode, store, "Configuration", "tenant", &tenant) {
        None => refusal(command, "no-configuration", "demo.signin.NoConfiguration"),
        Some(_) if client == "console" => create("fast-path"),
        Some(row) if row.get("plan") == Some(&Node::Text("Basic".into())) => {
            refusal(command, "no-redirect-entry", "demo.signin.NoRedirectEntry")
        }
        Some(_) => create("initiated"),
    }
}

fn fast_path_service(
    mode: Mode,
    store: &Store,
    command: &CommandRef,
    input: &BTreeMap<String, Node>,
) -> SemanticCommandResult {
    fast_path_answer(Order::RelatedFirst, mode, store, command, input)
}

fn input_first_service(
    mode: Mode,
    store: &Store,
    command: &CommandRef,
    input: &BTreeMap<String, Node>,
) -> SemanticCommandResult {
    fast_path_answer(Order::InputFirst, mode, store, command, input)
}

#[test]
fn f3_f4_on_a_missing_row_exists_false_answers_before_an_accepting_input_branch() {
    let result = synthesis(&fast_path());
    let refused = scenario(
        &result,
        "demo.signin.InitiateSignIn/outcome/no-configuration",
    );
    let sent = sent(refused, "demo.signin.InitiateSignIn");
    let last = sent.last().expect("the refusal is sent");
    assert_eq!(
        last.get("client"),
        Some(&ScenarioValue::literal(Node::Text("console".into()))),
        "the witness sends the input the accepting branch takes: {last:?}"
    );
    holds(
        &fast_path(),
        "InitiateSignIn",
        &[
            "demo.signin.InitiateSignIn/outcome/no-configuration",
            "demo.signin.InitiateSignIn/outcome/fast-path",
            "demo.signin.InitiateSignIn/outcome/initiated",
        ],
        fast_path_service,
        SIGN_IN_VIEWS,
        &[(
            Mode::IgnoresRelated,
            "demo.signin.InitiateSignIn/outcome/no-configuration",
        )],
    );
    let input_first = statuses(
        &result,
        &Service::new(Mode::Correct, input_first_service, SIGN_IN_VIEWS),
        "InitiateSignIn",
    );
    assert!(
        failed(&input_first)
            .contains(&&"demo.signin.InitiateSignIn/outcome/no-configuration".to_owned()),
        "a service taking the input branch before the missing row passes: {input_first:#?}"
    );
}

// ---- F5: `existing_instance:` beside `when_related:` -----------------------------------------

const ORDERS: &str = "format: ess/18
system: demo
version: v1
domain: demo.orders
types:
  - {name: demo.orders.OrderId, kind: newtype, of: String}
  - {name: demo.orders.CustomerId, kind: newtype, of: Uuid}
  - {name: demo.orders.Label, kind: newtype, of: String}
entities:
  - name: demo.orders.Customer
    identity: {name: customer_id, type: demo.orders.CustomerId}
    fields:
      - {name: label, type: demo.orders.Label}
    lifecycle: {initial: Active, states: [Active], terminal: [Active]}
  - name: demo.orders.Order
    identity: {name: order_id, type: demo.orders.OrderId}
    fields:
      - {name: customer, type: demo.orders.CustomerId}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
events:
  - name: demo.orders.CustomerAdded
    fields:
      - {name: customer_id, type: demo.orders.CustomerId}
  - name: demo.orders.OrderPlaced
    fields:
      - {name: order_id, type: demo.orders.OrderId}
errors:
  - {name: demo.orders.OrderExists, summary: The order id is taken., fields: []}
  - {name: demo.orders.NoCustomer, summary: The customer does not exist., fields: []}
actors:
  - {name: demo.orders.Clerk, may: [demo.orders.AddCustomer, demo.orders.PlaceOrder]}
commands:
  - name: demo.orders.AddCustomer
    input:
      - {name: label, type: demo.orders.Label}
    outcomes:
      - name: added
        creates: demo.orders.Customer
        instance: customer_id
        emits: [demo.orders.CustomerAdded]
        payload:
          demo.orders.CustomerAdded: {customer_id: {generated: true}}
        sets:
          label: input.label
  - name: demo.orders.PlaceOrder
    input:
      - {name: order_id, type: demo.orders.OrderId}
      - {name: customer, type: demo.orders.CustomerId}
    outcomes:
      - {name: duplicate, existing_instance: true, error: demo.orders.OrderExists}
      - name: no-customer
        when_related: {via: input.customer, exists: false}
        error: demo.orders.NoCustomer
      - name: placed
        creates: demo.orders.Order
        instance: order_id
        emits: [demo.orders.OrderPlaced]
        payload:
          demo.orders.OrderPlaced: {order_id: input.order_id}
        sets:
          customer: input.customer
views:
  - name: demo.orders.Customers
    source: demo.orders.Customer
    consistency: read_your_writes
    fields:
      - {name: customer_id, type: demo.orders.CustomerId}
      - {name: label, type: demo.orders.Label}
  - name: demo.orders.Orders
    source: demo.orders.Order
    consistency: read_your_writes
    fields:
      - {name: order_id, type: demo.orders.OrderId}
      - {name: customer, type: demo.orders.CustomerId}
";

const ORDER_VIEWS: &[(&str, &str, &str)] = &[
    ("demo.orders.Customers", "Customer", "Active"),
    ("demo.orders.Orders", "Order", "Open"),
];

/// `true` in the handler's `mode` slot for the mutant that reads the customer before the identity.
fn orders(
    related_first: bool,
    mode: Mode,
    store: &Store,
    command: &CommandRef,
    input: &BTreeMap<String, Node>,
) -> SemanticCommandResult {
    match command.to_string().as_str() {
        "demo.orders.AddCustomer" => {
            let id = store.mint();
            store.insert(
                "Customer",
                BTreeMap::from([
                    ("customer_id".to_owned(), Node::Text(id.clone())),
                    ("label".to_owned(), input["label"].clone()),
                ]),
            );
            took(
                command,
                "added",
                "demo.orders.CustomerAdded",
                "customer_id",
                &id,
            )
        }
        "demo.orders.PlaceOrder" => {
            let order = text(input.get("order_id"));
            let customer = text(input.get("customer"));
            let taken = store.find("Order", "order_id", &order).is_some();
            let missing = related(mode, store, "Customer", "customer_id", &customer).is_none();
            if related_first && missing {
                return refusal(command, "no-customer", "demo.orders.NoCustomer");
            }
            if taken {
                return refusal(command, "duplicate", "demo.orders.OrderExists");
            }
            if missing {
                return refusal(command, "no-customer", "demo.orders.NoCustomer");
            }
            store.insert(
                "Order",
                BTreeMap::from([
                    ("order_id".to_owned(), Node::Text(order.clone())),
                    ("customer".to_owned(), Node::Text(customer)),
                ]),
            );
            took(
                command,
                "placed",
                "demo.orders.OrderPlaced",
                "order_id",
                &order,
            )
        }
        _ => SemanticCommandResult::undeclared(),
    }
}

fn orders_service(
    mode: Mode,
    store: &Store,
    command: &CommandRef,
    input: &BTreeMap<String, Node>,
) -> SemanticCommandResult {
    orders(false, mode, store, command, input)
}

fn related_first_service(
    mode: Mode,
    store: &Store,
    command: &CommandRef,
    input: &BTreeMap<String, Node>,
) -> SemanticCommandResult {
    orders(true, mode, store, command, input)
}

#[test]
fn f5_a_taken_identity_is_refused_as_taken_before_the_missing_related_row() {
    let result = synthesis(ORDERS);
    let duplicate = scenario(&result, "demo.orders.PlaceOrder/outcome/duplicate");
    let placed = sent(duplicate, "demo.orders.PlaceOrder");
    assert!(placed.len() >= 2, "sent twice for one identity: {placed:?}");
    assert!(
        matches!(
            placed[0].get("customer"),
            Some(ScenarioValue::Instance { .. })
        ),
        "the first call names an arranged customer: {:?}",
        placed[0]
    );
    assert!(
        matches!(
            placed[placed.len() - 1].get("customer"),
            Some(ScenarioValue::Literal { .. })
        ),
        "the second call names a customer no row carries: {:?}",
        placed[placed.len() - 1]
    );
    holds(
        ORDERS,
        "PlaceOrder",
        &[
            "demo.orders.PlaceOrder/outcome/duplicate",
            "demo.orders.PlaceOrder/outcome/no-customer",
            "demo.orders.PlaceOrder/outcome/placed",
        ],
        orders_service,
        ORDER_VIEWS,
        &[
            (
                Mode::IgnoresRelated,
                "demo.orders.PlaceOrder/outcome/no-customer",
            ),
            (Mode::InvertsExists, "demo.orders.PlaceOrder/outcome/placed"),
        ],
    );
    let related_first = statuses(
        &result,
        &Service::new(Mode::Correct, related_first_service, ORDER_VIEWS),
        "PlaceOrder",
    );
    assert!(
        failed(&related_first).contains(&&"demo.orders.PlaceOrder/outcome/duplicate".to_owned()),
        "a service reading the customer before the identity passes: {related_first:#?}"
    );
}
