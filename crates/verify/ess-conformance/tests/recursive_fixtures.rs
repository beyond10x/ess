//! Finite recursive typed input fixtures (beyond10x/ess#416).
//!
//! A struct that reaches itself only through `List<T>` or `Optional<T>` has finite values, and a
//! fixture supplying one must be resolvable: synthesized with its typed binding retained, checked
//! structurally against the recursive contract, and executed. A recursion with no such boundary has
//! no finite value and stays refused, naming the input it was refused for; a recursive response
//! stays refused naming the response.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::{
    fixtures::Contract, interpret::Interpreted, report::Status, target::*, AdmittedSuite,
    ConformanceSuite, Runner, ScenarioStep,
};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;
use std::{cell::RefCell, collections::BTreeMap};

const SYSTEM: &str = include_str!("fixtures/recursive-fixtures/system.yaml");
const LIST: &str = include_str!("fixtures/recursive-fixtures/recursive.yaml");
const SCENARIO: &str = "fixtureprobe.recursive.Submit/outcome/accepted";
const VALUE: &str = "fixtureprobe.recursive.Value";

/// The issue's twin: the recursion terminated by `Optional` instead of `List`.
fn optional_twin() -> String {
    let twin = LIST.replace(
        "      - name: children\n        type: List<fixtureprobe.recursive.Value>\n",
        "      - name: next\n        type: Optional<fixtureprobe.recursive.Value>\n",
    );
    assert_ne!(twin, LIST, "the twin replaces the list boundary");
    twin
}

fn spec(domain: &str) -> Result<Specification, String> {
    Specification::assemble([
        (
            Source::new("system.yaml"),
            RawSpecFile::parse(SYSTEM).unwrap(),
        ),
        (
            Source::new("domains/recursive.yaml"),
            RawSpecFile::parse(domain).unwrap(),
        ),
    ])
    .map_err(|errors| errors.to_string())
}

fn ir(domain: &str) -> EssIr {
    let spec = spec(domain).unwrap_or_else(|errors| panic!("the model validates:\n{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn suite(domain: &str) -> ConformanceSuite {
    let synthesis = ess_conformance::synthesize::synthesize(&ir(domain));
    let refusals: Vec<String> = synthesis.refusals.iter().map(ToString::to_string).collect();
    assert_eq!(refusals.len(), 0, "{refusals:#?}");
    synthesis.suite
}

fn contract(suite: &ConformanceSuite) -> Contract {
    let ScenarioStep::ResolveFixtures { fixtures } =
        &suite.scenarios[&SCENARIO.parse().unwrap()].steps[0]
    else {
        panic!("fixture prelude");
    };
    fixtures.clone()
}

fn node(json: &str) -> Node {
    serde_json::from_str(json).unwrap_or_else(|error| panic!("{error}: {json}"))
}

/// A finite tree, three levels deep, through the list boundary.
const TREE: &str = r#"{"text":"root","children":[{"text":"a","children":[]},{"text":"b","children":[{"text":"c","children":[]}]}]}"#;
/// The same through the optional boundary: absent, null and present successors.
const CHAIN: &str = r#"{"text":"root","next":{"text":"a","next":{"text":"b","next":null}}}"#;

/// `levels` nested values, each the only child of the one above it: built rather than parsed,
/// because a JSON reader stops at its own nesting limit before the fixture guard is reached.
fn deep(levels: usize) -> Node {
    let mut value = Node::Seq(Vec::new());
    for _ in 0..levels {
        value = Node::Map(BTreeMap::from([
            ("text".to_owned(), Node::Text("deep".to_owned())),
            ("children".to_owned(), value),
        ]));
        value = Node::Seq(vec![value]);
    }
    let Node::Seq(mut root) = value else {
        unreachable!()
    };
    root.pop().expect("at least one level")
}

#[test]
fn the_issue_reproduction_synthesizes_with_its_typed_fixture_binding() {
    for (label, domain) in [("list", LIST.to_owned()), ("optional", optional_twin())] {
        let suite = suite(&domain);
        let json = serde_json::to_value(&suite).unwrap();
        let steps = json["scenarios"][SCENARIO]["steps"]
            .as_array()
            .unwrap_or_else(|| panic!("{label}: `{SCENARIO}` is synthesized: {json:#}"));
        assert_eq!(steps[0]["step"], "resolve_fixtures", "{label}");
        assert_eq!(
            steps[0]["fixtures"]["fields"],
            serde_json::json!([{"name": "finite-value", "type": VALUE}]),
            "{label}"
        );
        let declarations = steps[0]["fixtures"]["declarations"].as_object().unwrap();
        assert_eq!(declarations.keys().collect::<Vec<_>>(), [VALUE], "{label}");
        let execute = steps
            .iter()
            .find(|step| step["step"] == "execute_command")
            .unwrap_or_else(|| panic!("{label}: a command step"));
        assert_eq!(
            execute["input"]["value"],
            serde_json::json!({"kind": "fixture", "fixture": "finite-value"}),
            "{label}: the typed fixture binding is retained, not replaced by a literal"
        );
        contract(&suite).validate().unwrap();
        let reparsed: Contract =
            serde_json::from_value(steps[0]["fixtures"].clone()).expect("the contract re-admits");
        assert_eq!(reparsed, contract(&suite));
        AdmittedSuite::from_json(&suite.to_canonical_json().unwrap())
            .unwrap_or_else(|error| panic!("{label}: the suite is admitted: {error}"));
    }
}

/// The interpreter, given independently supplied fixture values.
struct Supplied {
    interpreted: Interpreted,
    value: Node,
    received: RefCell<Vec<BTreeMap<String, Node>>>,
}

impl Supplied {
    fn new(domain: &str, value: Node) -> Self {
        Self {
            interpreted: Interpreted::for_model(ir(domain)),
            value,
            received: RefCell::default(),
        }
    }
}

impl ConformanceTarget for Supplied {
    fn fixture_values(
        &self,
        _: &ScenarioContext,
        _: &Contract,
    ) -> Result<BTreeMap<String, Node>, TargetError> {
        Ok(BTreeMap::from([(
            "finite-value".into(),
            self.value.clone(),
        )]))
    }
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.interpreted.identity()
    }
    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.interpreted.begin_scenario(scenario)
    }
    fn end_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.interpreted.end_scenario(scenario)
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        self.received.borrow_mut().push(request.input.clone());
        self.interpreted.execute_command(request)
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        self.interpreted.query_view(request)
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        self.interpreted.observe_events(request)
    }
    fn configure_external_outcome(
        &self,
        request: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        self.interpreted.configure_external_outcome(request)
    }
    fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError> {
        self.interpreted.redeliver_event(request)
    }
}

fn run(suite: &ConformanceSuite, target: &Supplied) -> ess_conformance::report::ScenarioResult {
    let admitted = AdmittedSuite::from_suite(suite).unwrap();
    let report = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, target)
        .into_report();
    report
        .scenarios
        .into_iter()
        .find(|result| result.scenario.to_string() == SCENARIO)
        .expect("the scenario ran")
}

#[test]
fn a_conforming_finite_fixture_executes_against_the_interpreter() {
    for (label, domain, value) in [
        (
            "list leaf",
            LIST.to_owned(),
            r#"{"text":"finite","children":[]}"#,
        ),
        ("list tree", LIST.to_owned(), TREE),
        ("optional absent", optional_twin(), r#"{"text":"finite"}"#),
        ("optional chain", optional_twin(), CHAIN),
    ] {
        let suite = suite(&domain);
        let target = Supplied::new(&domain, node(value));
        let result = run(&suite, &target);
        assert_eq!(result.status, Status::Passed, "{label}: {result:#?}");
        assert_eq!(
            target.received.borrow().as_slice(),
            [BTreeMap::from([("value".to_owned(), node(value))])],
            "{label}: the command received the supplied tree unchanged"
        );
    }
}

#[test]
fn malformed_recursive_fixtures_refuse_naming_their_exact_position() {
    let list = contract(&suite(LIST));
    let optional = contract(&suite(&optional_twin()));
    // Level k of the tree is validated at depth 2k, so the guard (depth 128) trips on the first
    // field of level 64.
    let too_deep = format!("{}.text", vec!["children[0]"; 64].join("."));
    let cases = [
        (
            "wrong leaf deep in the tree",
            &list,
            node(&TREE.replace(r#""text":"c""#, r#""text":5"#)),
            "fixture finite-value: invalid_input at `children[1].children[0].text`".to_owned(),
        ),
        (
            "missing required field in a nested child",
            &list,
            node(r#"{"text":"root","children":[{"text":"a","children":[]},{"children":[]}]}"#),
            "fixture finite-value: invalid_input at `children[1].text`".to_owned(),
        ),
        (
            "unknown field in a nested child",
            &list,
            node(r#"{"text":"root","children":[{"text":"a","children":[],"extra":true}]}"#),
            "fixture finite-value: invalid_input at `children[0].extra`".to_owned(),
        ),
        (
            "deeper than the guard",
            &list,
            deep(70),
            format!("fixture finite-value: resource at `{too_deep}`"),
        ),
        (
            "wrong optional successor",
            &optional,
            node(&CHAIN.replace(r#""next":null"#, r#""next":"b""#)),
            "fixture finite-value: invalid_input at `next.next.next`".to_owned(),
        ),
        (
            "a root that is not a struct",
            &list,
            node("true"),
            "fixture finite-value: invalid_input".to_owned(),
        ),
    ];
    for (label, contract, value, expected) in cases {
        let values = BTreeMap::from([("finite-value".to_owned(), value)]);
        assert_eq!(
            contract.validate_values(&values).unwrap_err(),
            expected,
            "{label}"
        );
    }
    // A conforming deep tree just inside the guard is still a finite value.
    let values = BTreeMap::from([("finite-value".to_owned(), deep(64))]);
    list.validate_values(&values).unwrap();

    // The runner refuses each before the session opens, and reports where.
    let suite = suite(LIST);
    for (value, at) in [
        (
            node(&TREE.replace(r#""text":"c""#, r#""text":5"#)),
            "invalid_input at `children[1].children[0].text`".to_owned(),
        ),
        (deep(70), format!("resource at `{too_deep}`")),
    ] {
        let target = Supplied::new(LIST, value);
        let result = run(&suite, &target);
        assert_eq!(result.status, Status::Error, "{result:#?}");
        assert_eq!(target.received.borrow().len(), 0, "no command was sent");
        let observed: Vec<_> = result
            .checks
            .iter()
            .filter_map(|check| check.diagnostic.as_ref())
            .flat_map(|diagnostic| diagnostic.observed.clone())
            .collect();
        assert!(
            observed.iter().any(|line| line.ends_with(&at)),
            "{at}: {observed:#?}"
        );
    }
}

/// The list contract's JSON with `Value`'s declarations replaced.
fn mutated(declarations: serde_json::Value) -> Result<Contract, String> {
    let mut raw = serde_json::to_value(contract(&suite(LIST))).unwrap();
    raw["declarations"] = declarations;
    serde_json::from_value::<Contract>(raw).map_err(|error| error.to_string())
}

#[test]
fn every_termination_boundary_the_model_admits_is_admitted_by_the_contract() {
    for (label, declarations) in [
        (
            "list",
            serde_json::json!({VALUE: {"kind": "struct", "fields": [
                {"name": "text", "type": "String"},
                {"name": "children", "type": "List<fixtureprobe.recursive.Value>"}]}}),
        ),
        (
            "optional",
            serde_json::json!({VALUE: {"kind": "struct", "fields": [
                {"name": "text", "type": "String"},
                {"name": "next", "type": "Optional<fixtureprobe.recursive.Value>"}]}}),
        ),
        (
            "map",
            serde_json::json!({VALUE: {"kind": "struct", "fields": [
                {"name": "named", "type": "Map<String, fixtureprobe.recursive.Value>"}]}}),
        ),
        (
            "a union with a leaf variant",
            serde_json::json!({
                VALUE: {"kind": "union", "tag": "kind", "variants": {
                    "leaf": "String", "pair": "fixtureprobe.recursive.Pair"}},
                "fixtureprobe.recursive.Pair": {"kind": "struct", "fields": [
                    {"name": "left", "type": "fixtureprobe.recursive.Value"},
                    {"name": "right", "type": "fixtureprobe.recursive.Value"}]}}),
        ),
    ] {
        mutated(declarations).unwrap_or_else(|error| panic!("{label}: {error}"));
    }
}

#[test]
fn a_recursion_without_a_boundary_stays_refused_naming_the_input() {
    let boundary = "recurs with no Optional, List or Map boundary";
    for (label, declarations, at) in [
        (
            "required direct recursion",
            serde_json::json!({VALUE: {"kind": "struct", "fields": [
                {"name": "text", "type": "String"},
                {"name": "self", "type": "fixtureprobe.recursive.Value"}]}}),
            "finite-value.self",
        ),
        (
            "mutual required recursion",
            serde_json::json!({
                VALUE: {"kind": "struct", "fields": [
                    {"name": "text", "type": "String"},
                    {"name": "other", "type": "fixtureprobe.recursive.Other"}]},
                "fixtureprobe.recursive.Other": {"kind": "struct", "fields": [
                    {"name": "back", "type": "fixtureprobe.recursive.Value"}]}}),
            "finite-value.other.back",
        ),
        (
            "a union whose every variant recurses",
            serde_json::json!({
                VALUE: {"kind": "union", "tag": "kind", "variants": {
                    "pair": "fixtureprobe.recursive.Pair"}},
                "fixtureprobe.recursive.Pair": {"kind": "struct", "fields": [
                    {"name": "left", "type": "fixtureprobe.recursive.Value"}]}}),
            "finite-value.pair.left",
        ),
    ] {
        let refused = mutated(declarations).expect_err(label);
        assert_eq!(
            refused,
            format!(
                "fixture input `finite-value` has no finite value at `{at}`: `{VALUE}` {boundary}"
            ),
            "{label}"
        );
        assert!(!refused.contains("response"), "{label}: {refused}");
    }
    // The model refuses the same shape before any suite exists.
    let direct = LIST.replace(
        "        type: List<fixtureprobe.recursive.Value>\n",
        "        type: fixtureprobe.recursive.Value\n",
    );
    let refused = spec(&direct).expect_err("required direct recursion has no finite value");
    assert!(refused.contains("self_reference"), "{refused}");
}

#[test]
fn a_recursive_response_stays_refused_naming_the_response() {
    let domain = LIST
        .replace(
            "  - name: fixtureprobe.recursive.Accepted\n    fields:\n      - name: accepted\n        type: Boolean\n",
            "  - name: fixtureprobe.recursive.Accepted\n    fields:\n      - name: tree\n        type: fixtureprobe.recursive.Value\n",
        )
        .replace(
            "    response:\n      - name: accepted\n        type: Boolean\n",
            "    response:\n      - name: tree\n        type: fixtureprobe.recursive.Value\n",
        )
        .replace("            accepted: true\n", "            tree: {response: tree}\n");
    let ir = ir(&domain);
    let command = &ir.commands()[&"fixtureprobe.recursive.Submit".parse().unwrap()];
    let refused = ess_conformance::response::Observation::of(&ir, command, &command.outcomes[0])
        .expect_err("a recursive response has no finite observation");
    assert_eq!(
        refused,
        format!("recursive response type `{VALUE}` cannot be finitely admitted at response field `tree.children`")
    );
    let synthesis = ess_conformance::synthesize::synthesize(&ir);
    let refusals: Vec<String> = synthesis.refusals.iter().map(ToString::to_string).collect();
    assert!(
        refusals.iter().any(|refusal| refusal.contains(&format!(
            "fixtureprobe.recursive.Submit.response: {refused}"
        ))),
        "{refusals:#?}"
    );
}

#[test]
fn emitted_go_and_typescript_runtimes_resolve_the_recursive_fixture() {
    let suite = suite(LIST);
    let root = std::env::temp_dir().join(format!("ess-recursive-fixtures-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    for artifact in ess_conformance::go::emit(&suite).unwrap() {
        let path = root.join(artifact.path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, artifact.contents).unwrap();
    }
    std::fs::write(
        root.join("go.mod"),
        "module example.invalid/recursive\n\ngo 1.24\n",
    )
    .unwrap();
    std::fs::write(
        root.join("essconform/recursive_test.go"),
        include_str!("fixtures/recursive-fixtures/runtime_test.go"),
    )
    .unwrap();
    for artifact in ess_conformance::ts::emit(&suite).unwrap() {
        let path = root.join("typescript").join(artifact.path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, artifact.contents).unwrap();
    }
    let ts = root.join("typescript/essconform");
    std::fs::write(
        ts.join("recursive.mjs"),
        include_str!("fixtures/recursive-fixtures/runtime.mjs"),
    )
    .unwrap();
    std::fs::write(
        ts.join("runtime-test.tsconfig.json"),
        r#"{"extends":"./tsconfig.json","compilerOptions":{"types":[],"noCheck":true}}"#,
    )
    .unwrap();
    let compiled = std::process::Command::new("tsc")
        .args(["--project", "runtime-test.tsconfig.json"])
        .current_dir(&ts)
        .output()
        .unwrap();
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stdout)
    );
    for mode in ["valid", "wrong-leaf", "too-deep"] {
        for (tool, args, directory) in [
            ("go", vec!["test", "./essconform", "-count=1", "-v"], &root),
            ("node", vec!["--test", "recursive.mjs"], &ts),
        ] {
            let output = std::process::Command::new(tool)
                .args(args)
                .env("ESS_FIXTURE_CASE", mode)
                .env("ESS_REPORT_FORMAT", "2")
                .env("GOWORK", "off")
                .env("GOPROXY", "off")
                .env_remove("ESS_REPORT_OUT")
                .current_dir(directory)
                .output()
                .unwrap();
            let log = format!(
                "{}\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            let valid = mode == "valid";
            assert_eq!(output.status.success(), valid, "{tool} {mode}: {log}");
            assert_eq!(
                log.contains("recursive fixture received"),
                valid,
                "{tool} {mode}: {log}"
            );
            assert_eq!(
                log.contains("recursive session begun"),
                valid,
                "{tool} {mode}: invalid fixture data must not open a session: {log}"
            );
            if !valid {
                assert!(log.contains("fixture values"), "{tool} {mode}: {log}");
            }
        }
    }
    std::fs::remove_dir_all(root).unwrap();
}
