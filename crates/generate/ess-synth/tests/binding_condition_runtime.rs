//! A binding's event-payload condition in the generated Rust and Go applications (ess/22,
//! beyond10x/ess#268 slice 2, beyond10x/ess#194).
//!
//! `ess-conformance/tests/fixtures/binding-condition.yaml` with one component added is synthesized
//! to Rust and to Go, linked with the context ports of `tests/fixtures/binding-condition-harness/`
//! and `…-go-harness/` — which hold no behaviour and no binding — and the suite slice 1 synthesizes
//! for the fixture runs against each through a native adapter of their line protocol. The honest
//! applications pass every conditioned-binding scenario, and each deliberate defect, injected into
//! the *generated* system source and selected at run time by `ESS_FAULT`, fails a synthesized
//! scenario:
//!
//! | fault | what the generated code does wrong | fails |
//! |---|---|---|
//! | `ignore` | the condition always holds | `condition-false` |
//! | `invert` | the condition is negated | `mapping`, `condition-false` |
//! | `absence` | `defined(event.order)` holds for an absent order, and the proved member is read as a default | `condition-absent` |
//! | `unwrap` | the input is built — the absent member unwrapped to a default — and recorded before the condition is tested | `condition-false`, `condition-absent` |
//! | `absence-checked` | `defined(event.order)` holds for an absent order, and the transformation is the generated one | nothing: the presence check answers absent, and the binding reports `binding input` (`unsupported`) rather than invoking |
//! | `sibling` | an Unknown condition stops every binding on the occurrence | `condition-absent` of a condition that is Unknown there |
//!
//! `ess-conformance/tests/fixtures/binding-condition-selected.yaml` is the same shape on a
//! selection binding (beyond10x/ess#194): `received` selects the first leg and copies the tag
//! `defined(event.tag)` proves present into a required input. Its generated applications give the
//! interpreter's verdicts, and:
//!
//! | fault | what the generated code does wrong | fails |
//! |---|---|---|
//! | `absence` | the condition holds for an absent tag, and the transformation unwraps it to a default | `condition-absent` |
//! | `absence-checked` | the condition holds for an absent tag, and the transformation is the generated one | nothing: the binding reports `binding input` (`unsupported`) |
//!
//! The Go application is skipped, and said out loud, where the machine has no `go`.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::io::{BufRead as _, BufReader, Write as _};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Output, Stdio};

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::report::Status;
use ess_conformance::scenario::{CommandRef, ErrorRef, EventRef, OutcomeRef};
use ess_conformance::{
    AdmittedSuite, ConformanceSuite, ConformanceTarget, DeclaredErrorValue,
    EventObservationRequest, ExternalOutcomeControl, ImplementationIdentity,
    InvocationObservationRequest, ObservedEvent, ObservedInvocation, RedeliveryRequest, Runner,
    ScenarioContext, SemanticCommandRequest, SemanticCommandResult, SemanticViewRequest,
    SemanticViewResult, TargetError,
};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::node::Node;
use ess_synth::{synthesize_for, Target};

const MODEL: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/binding-condition.yaml");
const WHERE: &str = "      where: [defined(event.order), event.kind == ship]\n";
/// A condition that is Unknown where the order is absent: the comparison reads it.
const UNKNOWN_WHERE: &str = "      where: [event.kind == ship, event.order.id == o-1]\n";
const COMPONENT: &str = "components:
  - component: messages-service
    summary: Receives messages, and records them against their order.
    owns:
      domains: [demo.messages]
    accepts:
      commands: [demo.messages.ReceiveMessage, demo.messages.MessageEvent, demo.messages.LogMessage]
    publishes:
      events: [demo.messages.MessageReceived, demo.messages.OrderMessaged, demo.messages.MessageLogged]
    reached_by: network
";

const FLOW: &str = "received/binding/flow";
const MAPPING: &str = "received/binding/mapping";
const DELIVERY: &str = "received/binding/delivery";
const ON_FAILURE: &str = "received/binding/on-failure";
const FALSE: &str = "received/binding/condition-false";
const ABSENT: &str = "received/binding/condition-absent";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("the model parses");
    let spec = Specification::assemble([(Source::new("binding-condition.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model is admitted: {errors}"));
    compile(&spec, &SourceMap::new()).expect("the model compiles")
}

fn unknown_model() -> String {
    assert_eq!(MODEL.matches(WHERE).count(), 1);
    MODEL.replace(WHERE, UNKNOWN_WHERE)
}

/// The selecting variant (beyond10x/ess#194 on a selection binding): the message carries an
/// Optional `tag` and a list of legs, and `received` selects the first leg into the Optional `leg`
/// and copies the tag its condition proves present into the required `order_id`.
fn selected_model() -> String {
    include_str!("../../../verify/ess-conformance/tests/fixtures/binding-condition-selected.yaml")
        .to_owned()
}

/// The suite slice 1 synthesizes for `text`: the model as written, with no component.
fn suite(text: &str) -> ConformanceSuite {
    ess_conformance::synthesize::synthesize(&ir(text)).suite
}

fn scratch(label: &str) -> PathBuf {
    Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("binding-condition-{label}-{}", std::process::id()))
}

fn shown(output: &Output) -> String {
    format!(
        "exit {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn go_found() -> bool {
    let found = Command::new("go")
        .arg("version")
        .output()
        .is_ok_and(|output| output.status.success());
    if !found {
        println!("skipped: no `go` on PATH, so the Go application was not run");
    }
    found
}

/// `text` with `from` replaced by `to`, where `from` occurs exactly once: a fault whose anchor the
/// generator stopped writing must fail loudly, never become a no-op.
fn once(text: &str, from: &str, to: &str) -> String {
    assert_eq!(
        text.matches(from).count(),
        1,
        "the injection anchor occurs once in the generated source:\n{from}\n----\n{text}"
    );
    text.replacen(from, to, 1)
}

// ---- injecting the faults into the generated source -------------------------------------------

/// The generated Rust system crate with every fault behind `ESS_FAULT`.
fn faulty_rust(lib: &str) -> String {
    let event = "demo_types::messages::MessageReceived";
    let mut lib = once(
        lib,
        &format!(
            "    pub fn received(event: &{event}) -> Option<bool> {{\n        let _ = event;\n"
        ),
        &format!(
            "    pub fn received(event: &{event}) -> Option<bool> {{\n        let honest = \
             honest_received(event);\n        if crate::fault(\"ignore\") {{ return Some(true); \
             }}\n        if crate::fault(\"invert\") {{ return honest.map(|holds| !holds); }}\n        \
             if crate::fault(\"absence\") || crate::fault(\"absence-checked\") {{ return Some(matches!(event.kind, \
             demo_types::messages::Kind::Ship)); }}\n        honest\n    }}\n\n    /// The \
             condition as generated.\n    pub fn honest_received(event: &{event}) -> Option<bool> \
             {{\n        let _ = event;\n"
        ),
    );
    lib = once(
        &lib,
        "None => return None } }",
        "None => if crate::fault(\"absence\") || crate::fault(\"unwrap\") { String::new() } else \
         { return None } } }",
    );
    lib = once(
        &lib,
        "        match conditions::received(event) {\n",
        "        if crate::fault(\"unwrap\") {\n            if let Some(input) = received(event) \
         {\n                self.invocations.push(BindingInvocation::Received(input));\n            \
         }\n        }\n        match conditions::received(event) {\n",
    );
    lib = once(
        &lib,
        "    fn deliver(&mut self, event: &SystemEvent) -> Result<(), \
         demo_types::obligation::UnmetObligation> {\n",
        "    fn deliver(&mut self, event: &SystemEvent) -> Result<(), \
         demo_types::obligation::UnmetObligation> {\n        if crate::fault(\"sibling\") {\n            \
         if let SystemEvent::MessageReceived(occurrence) = event {\n                if \
         conditions::received(occurrence).is_none() {\n                    return \
         Err(demo_types::obligation::UnmetObligation { capability: \"binding condition\", source: \
         \"received\" });\n                }\n            }\n        }\n",
    );
    lib
}

/// The generated Go system package with every fault behind `ESS_FAULT`.
fn faulty_go(system: &str) -> String {
    let mut system = once(
        system,
        "func ReceivedCondition(event messages.MessageReceived) int8 {\n\t_ = event\n",
        "func ReceivedCondition(event messages.MessageReceived) int8 {\n\thonest := \
         honestReceivedCondition(event)\n\tif fault(\"ignore\") {\n\t\treturn 1\n\t}\n\tif \
         fault(\"invert\") {\n\t\tif honest < 0 {\n\t\t\treturn honest\n\t\t}\n\t\treturn 1 - \
         honest\n\t}\n\tif fault(\"absence\") || fault(\"absence-checked\") {\n\t\tif _, ship := \
         event.Kind.(messages.KindShip); ship {\n\t\t\treturn 1\n\t\t}\n\t\treturn \
         0\n\t}\n\treturn honest\n}\n\nfunc honestReceivedCondition(event messages.MessageReceived) \
         int8 {\n\t_ = event\n",
    );
    system = once(
        &system,
        "\tif !present1 {\n\t\treturn messages.MessageEvent{}, false\n\t}\n",
        "\tif !present1 {\n\t\tif fault(\"absence\") || fault(\"unwrap\") {\n\t\t\tproved1 = \
         \"\"\n\t\t} else {\n\t\t\treturn messages.MessageEvent{}, false\n\t\t}\n\t}\n",
    );
    system = once(
        &system,
        "\tswitch ReceivedCondition(event) {\n",
        "\tif fault(\"unwrap\") {\n\t\tif input, present := Received(event); present \
         {\n\t\t\ts.invocations = append(s.invocations, BindingInvocationReceived{Input: \
         input})\n\t\t}\n\t}\n\tswitch ReceivedCondition(event) {\n",
    );
    once(
        &system,
        "func (s *System) deliver(event SystemEvent) *obligation.UnmetObligation {\n",
        "func (s *System) deliver(event SystemEvent) *obligation.UnmetObligation {\n\tif \
         fault(\"sibling\") {\n\t\tif occurrence, is := event.(SystemEventMessageReceived); is && \
         ReceivedCondition(occurrence.Event) == -1 {\n\t\t\treturn \
         &obligation.UnmetObligation{Capability: \"binding condition\", Source: \
         \"received\"}\n\t\t}\n\t}\n",
    )
}

/// The condition of the selecting variant with its faults behind `ESS_FAULT`: `absence` and
/// `absence-checked` let an absent tag through.
fn absence_rust(lib: &str) -> String {
    let event = "demo_types::messages::MessageReceived";
    once(
        lib,
        &format!(
            "    pub fn received(event: &{event}) -> Option<bool> {{\n        let _ = event;\n"
        ),
        &format!(
            "    pub fn received(event: &{event}) -> Option<bool> {{\n        if \
             crate::fault(\"absence\") || crate::fault(\"absence-checked\") {{ return \
             Some(matches!(event.kind, demo_types::messages::Kind::Ship)); }}\n        \
             honest_received(event)\n    }}\n\n    /// The condition as generated.\n    pub fn \
             honest_received(event: &{event}) -> Option<bool> {{\n        let _ = event;\n"
        ),
    )
}

/// The generated Rust system crate of the selecting variant with its faults behind `ESS_FAULT`:
/// `absence` also unwraps the absent tag to a default where the generated code checks it.
fn faulty_rust_selected(lib: &str) -> String {
    once(
        &absence_rust(lib),
        "match event.tag.clone() { Some(value) => value, None => return Ok(None) }",
        "match event.tag.clone() { Some(value) => value, None => if crate::fault(\"absence\") { \
         String::new() } else { return Ok(None) } }",
    )
}

/// The generated Go system package of the selecting variant with its faults behind `ESS_FAULT`.
fn faulty_go_selected(system: &str) -> String {
    let system = once(
        system,
        "func ReceivedCondition(event messages.MessageReceived) int8 {\n\t_ = event\n",
        "func ReceivedCondition(event messages.MessageReceived) int8 {\n\tif fault(\"absence\") || \
         fault(\"absence-checked\") {\n\t\tif _, ship := event.Kind.(messages.KindShip); ship \
         {\n\t\t\treturn 1\n\t\t}\n\t\treturn 0\n\t}\n\treturn \
         honestReceivedCondition(event)\n}\n\nfunc honestReceivedCondition(event \
         messages.MessageReceived) int8 {\n\t_ = event\n",
    );
    once(
        &system,
        "\tif proved1_0 == nil {\n\t\treturn messages.MessageEvent{}, false, nil\n\t}\n",
        "\tif proved1_0 == nil {\n\t\tif fault(\"absence\") {\n\t\t\tempty := \"\"\n\t\t\tproved1_0 \
         = &empty\n\t\t} else {\n\t\t\treturn messages.MessageEvent{}, false, nil\n\t\t}\n\t}\n",
    )
}

/// What the test adds beside the generated Rust system crate: the fault switch, and the unmet
/// obligation a pump failure names — `TransportFailure` is the pump's failure where a binding
/// selects occurrences, and a failed selection is its policy's, not an unmet obligation.
fn rust_harness_support(selects: bool) -> String {
    let unmet = if selects {
        "pub type HarnessFailure = TransportFailure;\n/// The unmet obligation a pump failure \
         names (a test harness only).\npub fn harness_unmet(failure: &HarnessFailure) -> \
         Option<(&'static str, &'static str)> {\n    match failure {\n        \
         TransportFailure::Obligation(unmet) => Some((unmet.capability, unmet.source)),\n        \
         TransportFailure::Selection(_) => None,\n    }\n}\n"
    } else {
        "pub type HarnessFailure = demo_types::obligation::UnmetObligation;\n/// The unmet \
         obligation a pump failure names (a test harness only).\npub fn harness_unmet(failure: \
         &HarnessFailure) -> Option<(&'static str, &'static str)> {\n    Some((failure.capability, \
         failure.source))\n}\n"
    };
    format!(
        "\n/// The deliberate defect this build runs with, named by `ESS_FAULT` (a test harness \
         only).\npub fn fault(name: &str) -> bool {{\n    std::env::var(\"ESS_FAULT\").is_ok_and(|value| \
         value == name)\n}}\n\n/// The pump's failure (a test harness only).\n{unmet}"
    )
}

/// What the test adds beside the generated Go server package: the generated wire codecs, exported
/// for the harness, and the unmet obligation a pump failure names.
fn go_harness_support(selects: bool) -> String {
    let unmet = if selects {
        "// HarnessUnmet names the unmet obligation a pump failure carries; a failed selection is \
         its policy's.\nfunc HarnessUnmet(failure *system.TransportFailure) any {\n\tif failure == \
         nil || failure.Obligation == nil {\n\t\treturn nil\n\t}\n\treturn \
         map[string]any{\"capability\": failure.Obligation.Capability, \"source\": \
         failure.Obligation.Source}\n}\n"
    } else {
        "// HarnessUnmet names the unmet obligation a pump failure carries.\nfunc \
         HarnessUnmet(failure *obligation.UnmetObligation) any {\n\tif failure == nil {\n\t\treturn \
         nil\n\t}\n\treturn map[string]any{\"capability\": failure.Capability, \"source\": \
         failure.Source}\n}\n"
    };
    let imports = if selects {
        "\t\"example.invalid/demo/system\"\n\t\"example.invalid/demo/types/messages\"\n"
    } else {
        "\t\"example.invalid/demo/system\"\n\t\"example.invalid/demo/types/messages\"\n\t\"example.invalid/demo/types/obligation\"\n"
    };
    format!(
        "package server\n\n// A test harness only: the generated wire codecs, exported.\n\nimport \
         (\n{imports})\n\n// HarnessReceiveMessage reads the command's input as the route \
         does.\nfunc HarnessReceiveMessage(value any) (messages.ReceiveMessage, error) {{\n\treturn \
         decodeCommandDemoMessagesReceiveMessage(value, \"input\")\n}}\n\n// HarnessMessageEvent \
         reads the command's input as the route does.\nfunc HarnessMessageEvent(value any) \
         (messages.MessageEvent, error) {{\n\treturn decodeCommandDemoMessagesMessageEvent(value, \
         \"input\")\n}}\n\n// HarnessLogMessage reads the command's input as the route does.\nfunc \
         HarnessLogMessage(value any) (messages.LogMessage, error) {{\n\treturn \
         decodeCommandDemoMessagesLogMessage(value, \"input\")\n}}\n\n// HarnessEvent writes one \
         logged event as the routes do.\nfunc HarnessEvent(value system.SystemEvent) \
         map[string]any {{\n\tswitch held := value.(type) {{\n\tcase \
         system.SystemEventMessageReceived:\n\t\treturn map[string]any{{\"event\": \
         \"demo.messages.MessageReceived\", \"payload\": \
         encodeEventDemoMessagesMessageReceived(held.Event)}}\n\tcase \
         system.SystemEventOrderMessaged:\n\t\treturn map[string]any{{\"event\": \
         \"demo.messages.OrderMessaged\", \"payload\": \
         encodeEventDemoMessagesOrderMessaged(held.Event)}}\n\tcase \
         system.SystemEventMessageLogged:\n\t\treturn map[string]any{{\"event\": \
         \"demo.messages.MessageLogged\", \"payload\": \
         encodeEventDemoMessagesMessageLogged(held.Event)}}\n\t}}\n\tpanic(\"an event this harness \
         does not know\")\n}}\n\n{unmet}"
    )
}

// ---- building the applications -----------------------------------------------------------------

/// The harness binaries built against one model's generated applications.
struct Built {
    rust: PathBuf,
    go: Option<PathBuf>,
}

fn write_artifacts(synthesis: &ess_synth::Synthesis, root: &Path) {
    for (relative, artifact) in &synthesis.artifacts {
        let destination = root.join(relative);
        std::fs::create_dir_all(destination.parent().unwrap()).unwrap();
        std::fs::write(destination, &artifact.contents).unwrap();
    }
}

fn build(label: &str, text: &str, selects: bool) -> Built {
    let ir = ir(&format!("{text}{COMPONENT}"));
    let root = scratch(label);
    let _ = std::fs::remove_dir_all(&root);
    let kept = root.join("kept");
    std::fs::create_dir_all(&kept).unwrap();

    let synthesis = synthesize_for(&ir, Target::Rust).expect("the model synthesizes to Rust");
    let tree = root.join("demo");
    write_artifacts(&synthesis, &tree);
    let lib = tree.join("crates/demo-system/src/lib.rs");
    let generated = std::fs::read_to_string(&lib).unwrap();
    let faulty = if selects {
        faulty_rust_selected(&generated)
    } else {
        faulty_rust(&generated)
    };
    std::fs::write(&lib, faulty + &rust_harness_support(selects)).unwrap();
    let harness = root.join("harness");
    std::fs::create_dir_all(harness.join("src")).unwrap();
    let dependencies = [
        "demo-server",
        "demo-system",
        "messages-service",
        "demo-types",
    ]
    .iter()
    .fold(String::new(), |mut out, name| {
        let _ = writeln!(out, "{name} = {{ path = \"../demo/crates/{name}\" }}");
        out
    });
    std::fs::write(
        harness.join("Cargo.toml"),
        format!(
            "[package]\nname = \"binding-condition-harness\"\nversion = \"0.0.0\"\nedition = \
             \"2021\"\npublish = false\n\n[workspace]\n\n[[bin]]\nname = \"harness\"\npath = \
             \"src/main.rs\"\n\n[dependencies]\n{dependencies}"
        ),
    )
    .unwrap();
    std::fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/binding-condition-harness/main.rs"),
        harness.join("src/main.rs"),
    )
    .unwrap();
    let target = root.join("target");
    let output = Command::new(std::env::var_os("CARGO").expect("Cargo supplies its executable"))
        .args(["build", "--offline", "--quiet", "--target-dir"])
        .arg(&target)
        .current_dir(&harness)
        .env_remove("CARGO_TARGET_DIR")
        .env_remove("CARGO_ENCODED_RUSTFLAGS")
        // No compiler wrapper: a cached result from a shared server is not this build.
        .env("RUSTC_WRAPPER", "")
        .env("CARGO_BUILD_RUSTC_WRAPPER", "")
        .env("RUSTC_WORKSPACE_WRAPPER", "")
        .env("CARGO_INCREMENTAL", "0")
        // The selection support every selecting system carries declares `selection_all` and
        // `selection_any` whether or not its predicates use them (pre-existing); only that warning
        // is let through, and only where the binding selects.
        .env(
            "RUSTFLAGS",
            if selects {
                "-D warnings -A dead_code"
            } else {
                "-D warnings"
            },
        )
        .output()
        .expect("cargo runs");
    if output.status.success() {
        std::fs::copy(target.join("debug/harness"), kept.join("rust")).unwrap();
    }
    // The generated system is in the binary; its build tree is not needed again, built or not.
    let _ = std::fs::remove_dir_all(&target);
    assert!(
        output.status.success(),
        "the Rust harness builds against the generated system: {}",
        shown(&output)
    );

    let go = go_found().then(|| build_go(&ir, &root, &kept, selects));
    Built {
        rust: kept.join("rust"),
        go,
    }
}

/// The Go harness built against the generated Go system, faults injected, kept as `kept/go`.
fn build_go(ir: &EssIr, root: &Path, kept: &Path, selects: bool) -> PathBuf {
    let synthesis = synthesize_for(ir, Target::Go).expect("the model synthesizes to Go");
    let tree = root.join("go-demo");
    write_artifacts(&synthesis, &tree);
    let system = tree.join("system/system.go");
    let generated = std::fs::read_to_string(&system).unwrap();
    let faulty = if selects {
        faulty_go_selected(&generated)
    } else {
        faulty_go(&generated)
    };
    std::fs::write(&system, faulty).unwrap();
    std::fs::write(tree.join("server/harness.go"), go_harness_support(selects)).unwrap();
    std::fs::write(
        tree.join("system/fault.go"),
        "package system\n\nimport \"os\"\n\n// fault is the deliberate defect this build runs \
         with, named by ESS_FAULT (a test harness only).\nfunc fault(name string) bool {\n\t\
         return os.Getenv(\"ESS_FAULT\") == name\n}\n",
    )
    .unwrap();
    let harness = root.join("go-harness");
    std::fs::create_dir_all(&harness).unwrap();
    std::fs::write(
        harness.join("go.mod"),
        "module bindingconditionharness\n\ngo 1.21\n\nrequire example.invalid/demo v0.0.0\n\n\
         replace example.invalid/demo => ../go-demo\n",
    )
    .unwrap();
    std::fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/binding-condition-go-harness/main.go"),
        harness.join("main.go"),
    )
    .unwrap();
    let binary = kept.join("go");
    let output = Command::new("go")
        .args(["build", "-o", binary.to_str().unwrap(), "."])
        .current_dir(&harness)
        .env("GOFLAGS", "-mod=mod")
        .env("GOPROXY", "off")
        .env("GOWORK", "off")
        .output()
        .expect("go runs");
    assert!(
        output.status.success(),
        "the Go harness builds against the generated system: {}",
        shown(&output)
    );
    binary
}

fn conditioned() -> &'static Built {
    static BUILT: std::sync::OnceLock<Built> = std::sync::OnceLock::new();
    BUILT.get_or_init(|| build("conditioned", MODEL, false))
}

fn unknown() -> &'static Built {
    static BUILT: std::sync::OnceLock<Built> = std::sync::OnceLock::new();
    BUILT.get_or_init(|| build("unknown", &unknown_model(), false))
}

fn selected() -> &'static Built {
    static BUILT: std::sync::OnceLock<Built> = std::sync::OnceLock::new();
    BUILT.get_or_init(|| build("selected", &selected_model(), true))
}

fn applications(built: &Built) -> Vec<(&'static str, PathBuf)> {
    let mut found = vec![("rust", built.rust.clone())];
    found.extend(built.go.clone().map(|binary| ("go", binary)));
    found
}

// ---- the native adapter of the line protocol ----------------------------------------------------

/// One running application, and the unmet obligation each binding reported this scenario.
struct Application {
    name: String,
    child: RefCell<Child>,
    stdin: RefCell<ChildStdin>,
    stdout: RefCell<BufReader<ChildStdout>>,
    unmet: RefCell<BTreeMap<String, String>>,
}

impl Application {
    fn start(language: &str, binary: &Path, fault: Option<&str>) -> Self {
        let mut command = Command::new(binary);
        command.stdin(Stdio::piped()).stdout(Stdio::piped());
        if let Some(fault) = fault {
            command.env("ESS_FAULT", fault);
        }
        let mut child = command.spawn().expect("the harness starts");
        let stdin = child.stdin.take().expect("piped");
        let stdout = BufReader::new(child.stdout.take().expect("piped"));
        Self {
            name: format!("generated-{language}-{}", fault.unwrap_or("honest")),
            child: RefCell::new(child),
            stdin: RefCell::new(stdin),
            stdout: RefCell::new(stdout),
            unmet: RefCell::default(),
        }
    }

    fn ask(&self, request: &serde_json::Value) -> Result<serde_json::Value, TargetError> {
        let mut stdin = self.stdin.borrow_mut();
        writeln!(stdin, "{request}")
            .and_then(|()| stdin.flush())
            .map_err(|error| {
                TargetError::unavailable("writing to the harness", error.to_string())
            })?;
        let mut line = String::new();
        self.stdout
            .borrow_mut()
            .read_line(&mut line)
            .map_err(|error| TargetError::unavailable("reading the harness", error.to_string()))?;
        if line.is_empty() {
            return Err(TargetError::unavailable(
                "reading the harness",
                "it exited without answering",
            ));
        }
        let answer: serde_json::Value = serde_json::from_str(&line)
            .map_err(|error| TargetError::unavailable("reading the harness", error.to_string()))?;
        if let Some(failure) = answer.get("failure") {
            return Err(TargetError::unavailable("the harness", failure.to_string()));
        }
        // An unmet obligation is the binding's that reported it: an Unknown condition, or a
        // proved member absent anyway. The command that set it off still took its outcome.
        if let Some(unmet) = answer.get("unmet").filter(|unmet| !unmet.is_null()) {
            self.unmet.borrow_mut().insert(
                unmet["source"].as_str().unwrap_or_default().to_owned(),
                unmet["capability"].as_str().unwrap_or_default().to_owned(),
            );
        }
        Ok(answer)
    }

    /// Lets the held-back attempts run, as the caller's schedule would.
    fn observed(&self) -> Result<serde_json::Value, TargetError> {
        self.ask(&serde_json::json!({"op": "pump"}))?;
        self.ask(&serde_json::json!({"op": "observe"}))
    }
}

impl Drop for Application {
    fn drop(&mut self) {
        let _ = self.child.borrow_mut().kill();
        let _ = self.child.borrow_mut().wait();
    }
}

fn node(value: &serde_json::Value) -> Node {
    serde_json::from_value(value.clone()).expect("the wire writes JSON a node reads")
}

fn members(value: &serde_json::Value) -> BTreeMap<String, Node> {
    value
        .as_object()
        .map(|members| {
            members
                .iter()
                .map(|(name, value)| (name.clone(), node(value)))
                .collect()
        })
        .unwrap_or_default()
}

fn json_of(input: &BTreeMap<String, Node>) -> serde_json::Value {
    serde_json::to_value(input).expect("a node writes JSON")
}

impl ConformanceTarget for Application {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new(self.name.clone(), "1"))
    }

    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.unmet.borrow_mut().clear();
        self.ask(&serde_json::json!({"op": "reset"})).map(|_| ())
    }

    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }

    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let answer = self.ask(&serde_json::json!({
            "op": "command",
            "command": request.command.to_string(),
            "input": json_of(&request.input),
        }))?;
        let outcome = &answer["outcome"];
        let name = outcome["outcome"].as_str().unwrap_or_default();
        let mut result = SemanticCommandResult::took(OutcomeRef::new(
            CommandRef::new(request.command.name().clone()),
            name.parse().expect("a declared outcome name"),
        ));
        for published in outcome["published"].as_array().into_iter().flatten() {
            let mut event = ObservedEvent::new(EventRef::new(
                published["event"].as_str().unwrap().parse().unwrap(),
            ));
            event.payload = members(&published["payload"]);
            result.direct_events.push(event);
        }
        if let Some(refusal) = outcome.get("refusal") {
            let mut error = DeclaredErrorValue::new(ErrorRef::new(
                refusal["error"].as_str().unwrap().parse().unwrap(),
            ));
            error.fields = members(&refusal["payload"]);
            result.error = Some(error);
        }
        Ok(result)
    }

    fn query_view(&self, _: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        Err(TargetError::unsupported("views", "the model declares none"))
    }

    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        let observed = self.observed()?;
        Ok(observed["published"]
            .as_array()
            .into_iter()
            .flatten()
            .filter(|published| published["event"] == request.event.to_string())
            .map(|published| {
                let mut event = ObservedEvent::new(request.event.clone());
                event.payload = members(&published["payload"]);
                event
            })
            .collect())
    }

    fn configure_external_outcome(
        &self,
        request: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        self.ask(&serde_json::json!({
            "op": "force",
            "command": request.force.command.to_string(),
            "outcome": request.force.outcome.to_string(),
        }))
        .map(|_| ())
    }

    fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError> {
        self.ask(&serde_json::json!({"op": "redeliver", "event": request.event.to_string()}))
            .map(|_| ())
    }

    fn observe_invocations(
        &self,
        request: InvocationObservationRequest,
    ) -> Result<Vec<ObservedInvocation>, TargetError> {
        let observed = self.observed()?;
        let binding = request.binding.to_string();
        if let Some(capability) = self.unmet.borrow().get(&binding) {
            return Err(TargetError::unsupported(
                format!("the invocations of `{binding}`"),
                format!("it reported its unmet obligation: {capability}"),
            ));
        }
        Ok(observed["invocations"]
            .as_array()
            .into_iter()
            .flatten()
            .filter(|invocation| {
                invocation["binding"] == binding
                    && invocation["command"] == request.command.to_string()
            })
            .map(|invocation| {
                let mut observed =
                    ObservedInvocation::new(request.binding.clone(), request.command.clone());
                observed.input = members(&invocation["input"]);
                observed
            })
            .collect())
    }
}

// ---- running the suite ---------------------------------------------------------------------------

fn run(suite: &ConformanceSuite, target: &impl ConformanceTarget) -> BTreeMap<String, Status> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, target)
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| (result.scenario.to_string(), result.status))
        .collect()
}

fn verdicts(
    built: &Built,
    suite: &ConformanceSuite,
    fault: Option<&str>,
) -> Vec<(&'static str, BTreeMap<String, Status>)> {
    applications(built)
        .into_iter()
        .map(|(language, binary)| {
            let application = Application::start(language, &binary, fault);
            (language, run(suite, &application))
        })
        .collect()
}

fn failed(statuses: &BTreeMap<String, Status>) -> Vec<&str> {
    statuses
        .iter()
        .filter(|(_, status)| **status == Status::Failed)
        .map(|(id, _)| id.as_str())
        .collect()
}

#[test]
fn the_generated_applications_pass_every_conditioned_binding_scenario() {
    let suite = suite(MODEL);
    let reference = run(
        &suite,
        &ess_conformance::interpret::Interpreted::for_model(ir(MODEL)),
    );
    for (language, statuses) in verdicts(conditioned(), &suite, None) {
        for id in [FLOW, MAPPING, DELIVERY, ON_FAILURE, FALSE, ABSENT] {
            assert_eq!(
                statuses.get(id),
                Some(&Status::Passed),
                "{language}: {id}: {statuses:#?}"
            );
        }
        assert_eq!(failed(&statuses), Vec::<&str>::new(), "{language}");
        for (id, status) in &reference {
            if id.starts_with("received/") || id.starts_with("logged/") {
                assert_eq!(
                    statuses.get(id),
                    Some(status),
                    "{language} gives the interpreter's verdict on {id}"
                );
            }
        }
    }
}

#[test]
fn a_generated_application_that_ignores_the_condition_fails_condition_false() {
    for (language, statuses) in verdicts(conditioned(), &suite(MODEL), Some("ignore")) {
        assert_eq!(statuses[FALSE], Status::Failed, "{language}: {statuses:#?}");
    }
}

#[test]
fn a_generated_application_that_inverts_the_condition_fails_both_sides() {
    for (language, statuses) in verdicts(conditioned(), &suite(MODEL), Some("invert")) {
        assert_eq!(
            statuses[MAPPING],
            Status::Failed,
            "{language}: {statuses:#?}"
        );
        assert_eq!(statuses[FALSE], Status::Failed, "{language}: {statuses:#?}");
    }
}

#[test]
fn a_generated_application_that_fires_on_absence_fails_condition_absent() {
    for (language, statuses) in verdicts(conditioned(), &suite(MODEL), Some("absence")) {
        assert_eq!(
            statuses[ABSENT],
            Status::Failed,
            "{language}: {statuses:#?}"
        );
        assert_eq!(statuses[FALSE], Status::Passed, "{language}: {statuses:#?}");
    }
}

#[test]
fn a_generated_application_that_unwraps_before_testing_fails_both_negative_witnesses() {
    for (language, statuses) in verdicts(conditioned(), &suite(MODEL), Some("unwrap")) {
        assert_eq!(statuses[FALSE], Status::Failed, "{language}: {statuses:#?}");
        assert_eq!(
            statuses[ABSENT],
            Status::Failed,
            "{language}: {statuses:#?}"
        );
    }
}

/// Unknown is the conditioned binding's own unmet obligation: the honest application reports it
/// for `received` alone (`unsupported`, as the interpreter does), and the binding beside it still
/// invokes; stopping that sibling fails the scenario outright.
#[test]
fn an_unknown_condition_stops_no_sibling_and_a_generated_application_that_does_fails() {
    let text = unknown_model();
    let suite = suite(&text);
    for (language, statuses) in verdicts(unknown(), &suite, None) {
        assert_eq!(
            statuses[ABSENT],
            Status::Unsupported,
            "{language}: {statuses:#?}"
        );
        assert_eq!(failed(&statuses), Vec::<&str>::new(), "{language}");
    }
    for (language, statuses) in verdicts(unknown(), &suite, Some("sibling")) {
        assert_eq!(
            statuses[ABSENT],
            Status::Failed,
            "{language}: {statuses:#?}"
        );
    }
}

/// beyond10x/ess#194, at the generated seam: a condition that wrongly lets an absent order through
/// still invokes nothing, because the generated transformation checks the member the condition
/// was to prove present instead of unwrapping it, and the binding reports its unmet input.
#[test]
fn a_proved_member_absent_anyway_is_checked_never_unwrapped() {
    for (language, statuses) in verdicts(conditioned(), &suite(MODEL), Some("absence-checked")) {
        assert_eq!(
            statuses[ABSENT],
            Status::Unsupported,
            "{language}: {statuses:#?}"
        );
        assert_eq!(failed(&statuses), Vec::<&str>::new(), "{language}");
    }
}

// ---- a selection binding reading a proved member (beyond10x/ess#194) ---------------------------

/// The selecting variant in every lane: the interpreter passes its conditioned-binding scenarios,
/// and the generated Rust and Go applications give the interpreter's verdict on every one.
#[test]
fn a_selection_binding_reading_a_proved_member_runs_in_every_lane() {
    let text = selected_model();
    let suite = suite(&text);
    let reference = run(
        &suite,
        &ess_conformance::interpret::Interpreted::for_model(ir(&text)),
    );
    for id in [FLOW, MAPPING, FALSE, ABSENT] {
        assert_eq!(
            reference.get(id),
            Some(&Status::Passed),
            "interpreter: {id}: {reference:#?}"
        );
    }
    assert_eq!(failed(&reference), Vec::<&str>::new(), "interpreter");
    for (language, statuses) in verdicts(selected(), &suite, None) {
        for (id, status) in &reference {
            if id.starts_with("received/") || id.starts_with("logged/") {
                assert_eq!(
                    statuses.get(id),
                    Some(status),
                    "{language} gives the interpreter's verdict on {id}: {statuses:#?}"
                );
            }
        }
        assert_eq!(failed(&statuses), Vec::<&str>::new(), "{language}");
    }
}

/// A selecting application whose condition lets an absent tag through and whose transformation
/// unwraps it to a default invokes for the absent occurrence, and fails `condition-absent`; with
/// the generated transformation in place, the same wrong condition still invokes nothing.
#[test]
fn a_selecting_application_that_unwraps_an_absent_member_fails_condition_absent() {
    let suite = suite(&selected_model());
    for (language, statuses) in verdicts(selected(), &suite, Some("absence")) {
        assert_eq!(
            statuses[ABSENT],
            Status::Failed,
            "{language}: {statuses:#?}"
        );
    }
    for (language, statuses) in verdicts(selected(), &suite, Some("absence-checked")) {
        assert_eq!(
            statuses[ABSENT],
            Status::Unsupported,
            "{language}: {statuses:#?}"
        );
        assert_eq!(failed(&statuses), Vec::<&str>::new(), "{language}");
    }
}

// ---- an external binding's workspace (beyond10x/ess#268 slice 2) -------------------------------

/// `delivery-context.yaml`, plain and with a condition on its external binding: both generate in
/// Rust and Go with the delivery a named obligation, and the generated workspaces compile.
#[test]
fn an_external_binding_workspace_builds_in_rust_and_go() {
    let inbox =
        include_str!("../../../verify/ess-conformance/tests/fixtures/delivery-context.yaml");
    let conditioned = once(
        &once(
            &once(inbox, "format: ess/18\n", "format: ess/22\n"),
            "      - {name: from, type: String}\n",
            "      - {name: from, type: String}\n      - {name: urgent, type: Optional<String>}\n",
        ),
        "      context_authority: account-messages\n",
        "      context_authority: account-messages\n      where: defined(event.urgent)\n",
    );
    for (label, text) in [
        ("external", inbox.to_owned()),
        ("external-where", conditioned),
    ] {
        let ir = ir(&text);
        let root = scratch(label);
        let _ = std::fs::remove_dir_all(&root);
        let rust = synthesize_for(&ir, Target::Rust).expect("the model synthesizes to Rust");
        let tree = root.join("rust");
        write_artifacts(&rust, &tree);
        let output =
            Command::new(std::env::var_os("CARGO").expect("Cargo supplies its executable"))
                .args([
                    "check",
                    "--offline",
                    "--quiet",
                    "--workspace",
                    "--target-dir",
                ])
                .arg(root.join("target"))
                .current_dir(&tree)
                .env_remove("CARGO_TARGET_DIR")
                .env_remove("CARGO_ENCODED_RUSTFLAGS")
                .env("RUSTC_WRAPPER", "")
                .env("CARGO_BUILD_RUSTC_WRAPPER", "")
                .env("RUSTC_WORKSPACE_WRAPPER", "")
                .env("CARGO_INCREMENTAL", "0")
                .env("RUSTFLAGS", "-D warnings")
                .output()
                .expect("cargo runs");
        let _ = std::fs::remove_dir_all(root.join("target"));
        assert!(
            output.status.success(),
            "{label}: the generated Rust workspace compiles: {}",
            shown(&output)
        );
        if go_found() {
            let go = synthesize_for(&ir, Target::Go).expect("the model synthesizes to Go");
            let tree = root.join("go");
            write_artifacts(&go, &tree);
            let output = Command::new("go")
                .args(["vet", "./..."])
                .current_dir(&tree)
                .env("GOFLAGS", "-mod=mod")
                .env("GOPROXY", "off")
                .env("GOWORK", "off")
                .output()
                .expect("go runs");
            assert!(
                output.status.success(),
                "{label}: the generated Go module compiles: {}",
                shown(&output)
            );
        }
        let _ = std::fs::remove_dir_all(&root);
    }
}
