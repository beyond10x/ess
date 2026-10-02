//! Adversary pass 1 on beyond10x/ess#310: the generated Rust server selecting a branch by whether
//! the addressed record exists, driven by variants of `upsert-by-existence.yaml` and by mutants of
//! the generated behaviour. Each case builds the generated server beside
//! `tests/fixtures/upsert-by-existence-adversary-harness/` (storage and context ports only, no
//! behaviour) and runs the suite the same specification synthesizes against it.

use std::cell::RefCell;
use std::fmt::Write as _;
use std::io::{BufRead as _, BufReader, Write as _};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Output, Stdio};

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::report::Status;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::node::Node;
use ess_synth::{synthesize_for, CapabilityKind, SynthesisDisposition, Target};

const MODEL: &str =
    include_str!("../../../specify/ess-compiler/tests/fixtures/upsert-by-existence.yaml");

/// The fixture's `BookSlot`, verbatim, which every variant below replaces.
const BOOK_SLOT: &str = "  - name: demo.items.BookSlot
    input:
      - {name: slot_id, type: demo.items.ItemId}
      - {name: label, type: demo.items.Label}
    outcomes:
      - name: booked
        creates: demo.items.Slot
        instance: slot_id
        emits: [demo.items.SlotBooked]
        payload:
          demo.items.SlotBooked: {slot_id: input.slot_id, label: input.label}
        sets:
          label: input.label
      - {name: already-booked, existing_instance: true, error: demo.items.SlotTaken}
";

fn component(commands: &[&str], events: &[&str]) -> String {
    format!(
        "components:
  - component: items-service
    owns: {{domains: [demo.items]}}
    accepts: {{commands: [{}]}}
    publishes: {{events: [{}]}}
    reached_by: network
",
        commands.join(", "),
        events.join(", ")
    )
}

fn served_model(book_slot: &str, extra: &str) -> String {
    assert!(MODEL.contains(BOOK_SLOT), "the fixture's BookSlot moved");
    let mut model = MODEL.replace(BOOK_SLOT, &format!("{book_slot}{extra}"));
    let mut commands = vec!["demo.items.PutItem", "demo.items.BookSlot"];
    let mut events = vec!["demo.items.ItemStored", "demo.items.SlotBooked"];
    if !extra.is_empty() {
        commands.push("demo.items.FreeSlot");
        events.push("demo.items.SlotFreed");
        model = model
            .replace(
                "events:\n",
                "events:\n  - name: demo.items.SlotFreed\n    fields:\n      - {name: slot_id, \
                 type: demo.items.ItemId}\n",
            )
            .replace(
                "errors:\n",
                "errors:\n  - name: demo.items.NoSuchSlot\n    summary: No slot with this id.\n    \
                 fields: []\n",
            )
            .replace(
                "may: [demo.items.PutItem, demo.items.BookSlot]",
                "may: [demo.items.PutItem, demo.items.BookSlot, demo.items.FreeSlot]",
            );
    }
    format!("{model}{}", component(&commands, &events))
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap();
    let spec = Specification::assemble([(Source::new("upsert-by-existence.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("{errors:?}"))
}

// ---- building the generated server ---------------------------------------------------------------

fn cargo(directory: &Path, target: &Path, features: &[&str]) -> Output {
    let mut command =
        Command::new(std::env::var_os("CARGO").expect("Cargo supplies its executable"));
    command
        .args(["build", "--offline", "--quiet", "--target-dir"])
        .arg(target)
        .current_dir(directory)
        .env_remove("CARGO_TARGET_DIR")
        .env_remove("CARGO_ENCODED_RUSTFLAGS")
        .env_remove("RUSTC_WRAPPER")
        .env("CARGO_INCREMENTAL", "0")
        .env("RUSTFLAGS", "-D warnings");
    if !features.is_empty() {
        command.args(["--features", &features.join(",")]);
    }
    let output = command.output().expect("cargo runs");
    eprintln!(
        "cargo build in {}\n{}{}",
        directory.display(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

/// One synthesized model, written beside the adversary harness, with `build` producing a harness
/// binary from the generated behaviour as `mutate` leaves it.
struct Workspace {
    root: PathBuf,
    harness: PathBuf,
    behaviour: PathBuf,
    original: String,
    features: Vec<&'static str>,
}

impl Workspace {
    fn new(label: &str, model: &str, features: &[&'static str]) -> Self {
        let synthesis = synthesize_for(&ir(model), Target::Rust).expect("the model synthesizes");
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("adversary-310-{label}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let tree = root.join("demo");
        for (relative, artifact) in &synthesis.artifacts {
            let destination = tree.join(relative);
            std::fs::create_dir_all(destination.parent().unwrap()).unwrap();
            std::fs::write(&destination, &artifact.contents).unwrap();
        }
        let harness = root.join("harness");
        std::fs::create_dir_all(harness.join("src")).unwrap();
        let dependencies = ["demo-server", "demo-system", "items-service", "demo-types"]
            .iter()
            .fold(String::new(), |mut out, name| {
                let _ = writeln!(out, "{name} = {{ path = \"../demo/crates/{name}\" }}");
                out
            });
        std::fs::write(
            harness.join("Cargo.toml"),
            format!(
                "[package]\nname = \"existence-harness\"\nversion = \"0.0.0\"\nedition = \
                 \"2021\"\npublish = false\n\n[workspace]\n\n[features]\ncontext = []\nfreed = \
                 []\n\n[[bin]]\nname = \"harness\"\npath = \"src/main.rs\"\n\n[dependencies]\n\
                 {dependencies}"
            ),
        )
        .unwrap();
        std::fs::copy(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/upsert-by-existence-adversary-harness/main.rs"),
            harness.join("src/main.rs"),
        )
        .unwrap();
        let behaviour = tree.join("crates/demo-types/src/behaviour.rs");
        let original = std::fs::read_to_string(&behaviour).unwrap();
        Self {
            root,
            harness,
            behaviour,
            original,
            features: features.to_vec(),
        }
    }

    /// The harness built over the generated behaviour with `mutate` applied, kept as `name`.
    fn build(&self, name: &str, mutate: impl FnOnce(&str) -> String) -> PathBuf {
        let source = mutate(&self.original);
        std::fs::write(&self.behaviour, &source).unwrap();
        let target = self.root.join("target");
        assert!(
            cargo(&self.harness, &target, &self.features)
                .status
                .success(),
            "the harness builds over `{name}`"
        );
        let kept = self.root.join("kept");
        std::fs::create_dir_all(&kept).unwrap();
        std::fs::copy(target.join("debug/harness"), kept.join(name)).unwrap();
        kept.join(name)
    }
}

/// Every scenario of the suite `model` synthesizes, run through one harness binary.
fn run(binary: &Path, model: &str, env: &[(&str, &str)]) -> Vec<(String, Status)> {
    let ir = ir(model);
    let suite = ess_conformance::synthesize(&ir).suite;
    let admitted = ess_conformance::AdmittedSuite::from_suite(&suite).expect("admits");
    let target = Served::start(binary, &ir, env);
    let report = ess_conformance::Runner::for_suite(&suite)
        .run_admitted(&admitted, &target)
        .into_report();
    for scenario in &report.scenarios {
        if scenario.status != Status::Passed {
            eprintln!(
                "{} via {}: {scenario:#?}",
                scenario.scenario,
                binary.display()
            );
        }
    }
    report
        .scenarios
        .into_iter()
        .map(|scenario| (scenario.scenario.to_string(), scenario.status))
        .collect()
}

fn failed(ran: &[(String, Status)]) -> Vec<&str> {
    ran.iter()
        .filter(|(_, status)| *status != Status::Passed)
        .map(|(id, _)| id.as_str())
        .collect()
}

const ALREADY_BOOKED: &str = "demo.items.BookSlot/outcome/already-booked";

// ---- 1. the lookup reads the storage of the first creation only ----------------------------------

/// Two creations beside one `existing_instance:`, of different entities, both taking their identity
/// from `input.slot_id`. The plan generates the command (`existence_identity` checks the field and
/// not the entity) and the generated lookup reads `ItemStorage` only, so the `Slot` path creates
/// over an existing slot.
const TWO_ENTITIES: &str = "  - name: demo.items.BookSlot
    input:
      - {name: slot_id, type: demo.items.ItemId}
      - {name: label, type: demo.items.Label}
      - {name: as_item, type: Boolean}
    outcomes:
      - name: booked-item
        when: as_item == true
        creates: demo.items.Item
        instance: item_id
        emits: [demo.items.ItemStored]
        payload:
          demo.items.ItemStored: {item_id: input.slot_id, label: input.label}
        sets:
          label: input.label
      - name: booked
        creates: demo.items.Slot
        instance: slot_id
        emits: [demo.items.SlotBooked]
        payload:
          demo.items.SlotBooked: {slot_id: input.slot_id, label: input.label}
        sets:
          label: input.label
      - {name: already-booked, existing_instance: true, error: demo.items.SlotTaken}
";

#[test]
fn existing_instance_beside_creations_of_two_entities_refuses_on_every_creating_path() {
    let model = served_model(TWO_ENTITIES, "");
    let synthesis = synthesize_for(&ir(&model), Target::Rust).expect("Rust synthesizes");
    let generated = synthesis
        .plan
        .disposition_of(CapabilityKind::CommandBehavior, "demo.items.BookSlot")
        == Some(&SynthesisDisposition::Generated);
    if !generated {
        // An obligation is a correct answer too: the plan did not claim what it cannot generate.
        return;
    }
    let workspace = Workspace::new("two-entities", &model, &[]);
    let served = workspace.build("served", str::to_owned);
    let ran = run(&served, &model, &[]);
    assert!(
        ran.iter().any(|(id, _)| id == ALREADY_BOOKED),
        "the suite witnesses the refusal: {ran:?}"
    );
    assert_eq!(
        failed(&ran),
        Vec::<&str>::new(),
        "the generated server answers its own suite"
    );
}

// ---- 2. a plain creation ignores the identity its payload takes from the input -------------------

fn plain_creation() -> String {
    let book_slot = BOOK_SLOT.replace(
        "      - {name: already-booked, existing_instance: true, error: demo.items.SlotTaken}\n",
        "",
    );
    served_model(&book_slot, "")
}

/// The existing Go storage/HTTP harness, with only an optional identity-generating context added.
/// All command behavior still comes from the projection under test.
fn plain_go(label: &str, model: &str, context: bool) -> PathBuf {
    let synthesis = synthesize_for(&ir(model), Target::Go).expect("Go synthesizes");
    let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("plain-creation-go-{label}-{}", std::process::id()));
    for (relative, artifact) in &synthesis.artifacts {
        let destination = root.join("demo").join(relative);
        std::fs::create_dir_all(destination.parent().unwrap()).unwrap();
        std::fs::write(destination, &artifact.contents).unwrap();
    }
    let harness = root.join("harness");
    std::fs::create_dir_all(&harness).unwrap();
    std::fs::write(
        harness.join("go.mod"),
        "module existenceharness\n\ngo 1.21\n\nrequire example.invalid/demo v0.0.0\n\n\
         replace example.invalid/demo => ../demo\n",
    )
    .unwrap();
    let mut source = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/upsert-by-existence-go-harness/main.go"),
    )
    .unwrap();
    if context {
        source = source.replace(
            "behaviour.Ports{",
            "behaviour.Ports{Context: &identityContext{},",
        );
        source.push_str(
            "\ntype identityContext struct { minted int }\n\
             func (c *identityContext) GenerateDemoItemsItemId() items.ItemId {\n\
             c.minted++\nreturn items.NewItemId(fmt.Sprintf(\"minted-%d\", c.minted))\n}\n",
        );
    }
    std::fs::write(harness.join("main.go"), source).unwrap();
    let binary = root.join("served");
    let output = Command::new("go")
        .args(["build", "-o"])
        .arg(&binary)
        .arg(".")
        .current_dir(&harness)
        .env("GOPROXY", "off")
        .env("GOWORK", "off")
        .output()
        .expect("the Go toolchain runs");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    binary
}

fn assert_created_identity(target: &Served, input: &serde_json::Value, expected: &str) {
    let answer = target.ask(&serde_json::json!({
        "op": "command", "command": "demo.items.BookSlot", "actor": "demo.items.Admin",
        "body": input.to_string(),
    }));
    assert_eq!(answer["answer"]["outcome"], "booked", "{answer}");
    assert_eq!(
        answer["answer"]["published"][0]["payload"]["slot_id"], expected,
        "{answer}"
    );
    let rows = target.ask(&serde_json::json!({"op": "view", "view": "demo.items.SlotDetails"}));
    assert!(
        rows["answer"]["rows"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["slot_id"] == expected),
        "{rows}"
    );
}

/// Required caller-selected identities need no context at all; both emitted servers execute it.
#[test]
fn a_plain_creation_stores_and_publishes_the_identity_its_payload_takes_from_the_input() {
    let model = plain_creation();
    let synthesis = synthesize_for(&ir(&model), Target::Rust).expect("Rust synthesizes");
    assert_eq!(
        synthesis
            .plan
            .disposition_of(CapabilityKind::CommandBehavior, "demo.items.BookSlot"),
        Some(&SynthesisDisposition::Generated),
        "the plan claims the creation is fully determined"
    );
    let workspace = Workspace::new("plain-creation", &model, &[]);
    let served = workspace.build("served", str::to_owned);
    for binary in [served, plain_go("required", &model, false)] {
        let target = Served::start(&binary, &ir(&model), &[]);
        assert_created_identity(
            &target,
            &serde_json::json!({"slot_id": "slot-chosen", "label": "first"}),
            "slot-chosen",
        );
    }
}

#[test]
fn a_plain_creation_optional_identity_uses_input_then_generated_fallback() {
    let model = plain_creation()
        .replace("  - name: demo.items.BookSlot\n    input:\n      - {name: slot_id, type: demo.items.ItemId}", "  - name: demo.items.BookSlot\n    input:\n      - {name: slot_id, type: Optional<demo.items.ItemId>}")
        .replace("{slot_id: input.slot_id, label: input.label}", "{slot_id: {input: slot_id, else: {generated: true}}, label: input.label}");
    let workspace = Workspace::new("plain-optional", &model, &["context"]);
    for binary in [
        plain_go("optional", &model, true),
        workspace.build("served", str::to_owned),
    ] {
        let target = Served::start(&binary, &ir(&model), &[]);
        assert_created_identity(
            &target,
            &serde_json::json!({"slot_id": "slot-chosen", "label": "first"}),
            "slot-chosen",
        );
        // minted-1 also proves the supplied case did not consume a generated identity.
        assert_created_identity(
            &target,
            &serde_json::json!({"label": "fallback"}),
            "minted-1",
        );
    }
}

// ---- 3. mutants the suite must kill --------------------------------------------------------------

/// The fixture's `BookSlot` with a second input of the identity's type, so a lookup can read the
/// wrong one and still compile.
fn with_ref_id() -> String {
    BOOK_SLOT.replace(
        "      - {name: label, type: demo.items.Label}\n    outcomes:",
        "      - {name: label, type: demo.items.Label}\n      - {name: ref_id, type: \
         demo.items.ItemId}\n    outcomes:",
    )
}

/// A rewrite of the generated behaviour.
type Mutant = Box<dyn Fn(&str) -> String>;

const LOOKUP: &str = "if SlotStorage::get(&self.ports, &input.slot_id).is_some() {";
const REFUSAL: &str =
    "return Ok(crate::items::BookSlotOutcome::AlreadyBooked { error: crate::items::SlotTaken });";

fn once(source: &str, needle: &str) {
    assert_eq!(
        source.matches(needle).count(),
        1,
        "one `{needle}`:\n{source}"
    );
}

/// The line that stores the new slot, and what follows it.
fn after_put(source: &str, inserted: &str) -> String {
    let at = source
        .find("SlotStorage::put(&mut self.ports,")
        .expect("BookSlot stores the slot");
    let end = at + source[at..].find('\n').expect("a line") + 1;
    format!("{}{inserted}{}", &source[..end], &source[end..])
}

#[test]
fn every_lookup_mutant_fails_the_existing_instance_scenario() {
    let model = served_model(&with_ref_id(), "");
    let workspace = Workspace::new("mutants", &model, &[]);
    once(&workspace.original, LOOKUP);
    once(&workspace.original, REFUSAL);
    once(&workspace.original, "SlotStorage::put(&mut self.ports,");
    let served = workspace.build("served", str::to_owned);
    assert_eq!(
        failed(&run(&served, &model, &[])),
        Vec::<&str>::new(),
        "the unmutated server passes"
    );
    let mutants: [(&str, Mutant); 3] = [
        (
            "wrong-key",
            Box::new(|source: &str| {
                source.replace(
                    LOOKUP,
                    "if SlotStorage::get(&self.ports, &input.ref_id).is_some() {",
                )
            }),
        ),
        (
            "checked-after-create",
            Box::new(|source: &str| {
                after_put(
                    &source.replace(LOOKUP, "if false {"),
                    &format!(
                        "        if SlotStorage::get(&self.ports, &input.slot_id).is_some() {{ \
                         {REFUSAL} }}\n"
                    ),
                )
            }),
        ),
        (
            "overwrite-then-refuse",
            Box::new(|source: &str| {
                after_put(
                    &source.replace(
                        LOOKUP,
                        "let existed = SlotStorage::get(&self.ports, &input.slot_id).is_some();\n        \
                         if false {",
                    ),
                    &format!("        if existed {{ {REFUSAL} }}\n"),
                )
            }),
        ),
    ];
    let mut survived = Vec::new();
    for (name, mutate) in mutants {
        let binary = workspace.build(name, mutate);
        let ran = run(&binary, &model, &[]);
        if !failed(&ran).contains(&ALREADY_BOOKED) {
            survived.push((name, ran));
        }
    }
    assert!(
        survived.is_empty(),
        "mutants the suite did not kill: {survived:?}"
    );
}

/// A `deletes:` command beside the fixture's create-or-refuse: a slot removed by `FreeSlot` is no
/// longer a record, so booking its identity again creates it. The harness's tombstoning store
/// answers `get` for a removed slot — a server that treats a removed record as still existing.
const FREE_SLOT: &str = "  - name: demo.items.FreeSlot
    input:
      - {name: slot_id, type: demo.items.ItemId}
    outcomes:
      - name: freed
        deletes: demo.items.Slot
        instance: slot_id
        emits: [demo.items.SlotFreed]
        payload:
          demo.items.SlotFreed: {slot_id: input.slot_id}
      - {name: no-such-slot, unknown_instance: true, error: demo.items.NoSuchSlot}
";

#[test]
fn a_server_that_treats_a_removed_record_as_existing_fails_a_scenario() {
    let model = served_model(BOOK_SLOT, FREE_SLOT);
    let workspace = Workspace::new("removed", &model, &[]);
    let served = workspace.build("served", str::to_owned);
    assert_eq!(
        failed(&run(&served, &model, &[])),
        Vec::<&str>::new(),
        "the generated server over a removing store passes"
    );
    let ran = run(&served, &model, &[("HARNESS_TOMBSTONES", "1")]);
    assert!(
        !failed(&ran).is_empty(),
        "no scenario books an identity a `deletes:` removed, so a lookup that still finds it \
         passes every scenario: {ran:?}"
    );
}

/// The same mutant confined to the generated `existing_instance:` lookup: `FreeSlot` remembers the
/// identity it removed, and `BookSlot`'s lookup answers `already-booked` for a remembered one. The
/// store itself removes the row, so `FreeSlot`'s own `unknown_instance:` scenario cannot see it;
/// only a scenario booking an identity a `deletes:` removed can.
#[test]
#[ignore = "pre-existing: filed separately (coordinator)"]
fn a_lookup_that_still_finds_a_removed_record_fails_a_scenario() {
    let model = served_model(BOOK_SLOT, FREE_SLOT);
    let workspace = Workspace::new("removed-lookup", &model, &["freed"]);
    once(&workspace.original, LOOKUP);
    once(
        &workspace.original,
        "SlotStorage::delete(&mut self.ports, &input.slot_id);",
    );
    let mutant = workspace.build("remembers-removed", |source| {
        format!(
            "{source}\n/// Adversary mutant: identities `FreeSlot` removed.\npub static FREED: \
             std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());\n"
        )
        .replace(
            LOOKUP,
            "if SlotStorage::get(&self.ports, &input.slot_id).is_some() || \
             FREED.lock().expect(\"one thread\").contains(&input.slot_id.0) {",
        )
        .replace(
            "SlotStorage::delete(&mut self.ports, &input.slot_id);",
            "SlotStorage::delete(&mut self.ports, &input.slot_id); \
             FREED.lock().expect(\"one thread\").push(input.slot_id.0.clone());",
        )
    });
    let ran = run(&mutant, &model, &[]);
    assert!(
        !failed(&ran).is_empty(),
        "no scenario books an identity a `deletes:` removed, so a lookup that still finds it \
         passes every scenario: {ran:?}"
    );
}

// ---- 4. precedence: input-guarded refusals before either lookup ---------------------------------

/// An input-guarded refusal on both commands. The suite sends each refused input for an identity
/// nothing stored and for one a record carries (precedence step 2 before step 3), so a server that
/// looks the identity up first answers `already-booked` or `updated` instead.
#[test]
fn input_guarded_refusals_answer_before_either_lookup() {
    let refusal =
        "    outcomes:\n      - name: rejected\n        when: label == \"bad\"\n        error: \
         demo.items.SlotTaken\n";
    let book_slot = BOOK_SLOT.replacen("    outcomes:\n", refusal, 1);
    let model = served_model(&book_slot, "").replacen(
        "    outcomes:\n      - name: updated\n",
        &format!("{refusal}      - name: updated\n"),
        1,
    );
    assert_eq!(model.matches("name: rejected").count(), 2, "{model}");
    let workspace = Workspace::new("precedence", &model, &[]);
    let served = workspace.build("served", str::to_owned);
    let ran = run(&served, &model, &[]);
    for refused in [
        "demo.items.BookSlot/outcome/rejected",
        "demo.items.PutItem/outcome/rejected",
    ] {
        assert!(
            ran.iter().any(|(id, _)| id == refused),
            "{refused}: {ran:?}"
        );
    }
    assert_eq!(failed(&ran), Vec::<&str>::new());
}

// ---- the adapter: the suite's requests, over the harness's line protocol -------------------------

struct Served {
    child: RefCell<Child>,
    stdin: RefCell<ChildStdin>,
    stdout: RefCell<BufReader<ChildStdout>>,
    published: RefCell<Vec<ess_conformance::ObservedEvent>>,
    sequence: RefCell<u64>,
}

impl Served {
    fn start(binary: &Path, ir: &EssIr, env: &[(&str, &str)]) -> Self {
        let mut routes = Vec::new();
        for component in ir.components().values() {
            for route in ess_gen::http::routes(ir, component) {
                let name = match route.serves {
                    ess_gen::http::Served::Command(handle) => ir.command(handle).name.to_string(),
                    ess_gen::http::Served::View(handle) => ir.view(handle).name.to_string(),
                };
                routes.extend([name, route.method.as_str().to_owned(), route.path]);
            }
        }
        let mut child = Command::new(binary)
            .args(routes)
            .envs(env.iter().copied())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("the harness starts");
        let stdin = child.stdin.take().expect("piped");
        let stdout = BufReader::new(child.stdout.take().expect("piped"));
        Self {
            child: RefCell::new(child),
            stdin: RefCell::new(stdin),
            stdout: RefCell::new(stdout),
            published: RefCell::default(),
            sequence: RefCell::new(0),
        }
    }

    fn ask(&self, request: &serde_json::Value) -> serde_json::Value {
        let mut stdin = self.stdin.borrow_mut();
        writeln!(stdin, "{request}").expect("the harness reads");
        stdin.flush().expect("the harness reads");
        let mut line = String::new();
        self.stdout
            .borrow_mut()
            .read_line(&mut line)
            .expect("the harness answers");
        serde_json::from_str(&line).unwrap_or_else(|error| panic!("`{line}`: {error}"))
    }

    fn tick(&self) -> u64 {
        let mut sequence = self.sequence.borrow_mut();
        *sequence += 1;
        *sequence
    }
}

impl Drop for Served {
    fn drop(&mut self) {
        let _ = self.child.borrow_mut().kill();
        let _ = self.child.borrow_mut().wait();
    }
}

fn json_of(node: &Node) -> serde_json::Value {
    match node {
        Node::Null => serde_json::Value::Null,
        Node::Bool(value) => serde_json::Value::Bool(*value),
        Node::Number(number) => {
            serde_json::from_str(&number.to_string()).expect("a number's spelling is JSON")
        }
        Node::Text(text) => serde_json::Value::String(text.clone()),
        Node::Seq(items) => serde_json::Value::Array(items.iter().map(json_of).collect()),
        Node::Map(members) => serde_json::Value::Object(
            members
                .iter()
                .map(|(name, value)| (name.clone(), json_of(value)))
                .collect(),
        ),
    }
}

fn node_of(value: &serde_json::Value) -> Node {
    serde_json::from_value(value.clone()).expect("the wire writes JSON a node reads")
}

fn failure(observation: &str, answer: &serde_json::Value) -> ess_conformance::TargetError {
    ess_conformance::TargetError::unavailable(observation, answer.to_string())
}

impl ess_conformance::ConformanceTarget for Served {
    fn identity(
        &self,
    ) -> Result<ess_conformance::ImplementationIdentity, ess_conformance::TargetError> {
        Ok(ess_conformance::ImplementationIdentity::new(
            "upsert-by-existence-adversary",
            "1",
        ))
    }

    fn begin_scenario(
        &self,
        _: &ess_conformance::ScenarioContext,
    ) -> Result<(), ess_conformance::TargetError> {
        let answer = self.ask(&serde_json::json!({"op": "reset"}));
        if answer["ok"] != serde_json::json!(true) {
            return Err(failure("opening a scenario", &answer));
        }
        self.published.borrow_mut().clear();
        *self.sequence.borrow_mut() = 0;
        Ok(())
    }

    fn end_scenario(
        &self,
        _: &ess_conformance::ScenarioContext,
    ) -> Result<(), ess_conformance::TargetError> {
        self.published.borrow_mut().clear();
        Ok(())
    }

    fn execute_command(
        &self,
        request: ess_conformance::SemanticCommandRequest,
    ) -> Result<ess_conformance::SemanticCommandResult, ess_conformance::TargetError> {
        use ess_conformance::scenario::{CommandRef, ErrorRef, EventRef, OutcomeRef};
        let observation = format!("invoking `{}`", request.command);
        let body = serde_json::Value::Object(
            request
                .input
                .iter()
                .map(|(name, value)| (name.clone(), json_of(value)))
                .collect(),
        );
        let answer = self.ask(&serde_json::json!({
            "op": "command",
            "command": request.command.to_string(),
            "actor": request.actor.as_ref().map(ToString::to_string),
            "body": body.to_string(),
        }));
        let status = answer["status"]
            .as_u64()
            .ok_or_else(|| failure(&observation, &answer))?;
        let served = &answer["answer"];
        if status == 403 && served["refused"] == "not granted" {
            return Err(ess_conformance::TargetError::not_granted(
                served["actor"].as_str(),
            ));
        }
        if status == 501 {
            return Ok(ess_conformance::SemanticCommandResult::undeclared());
        }
        let outcome = served["outcome"]
            .as_str()
            .ok_or_else(|| failure(&observation, &answer))?;
        let outcome = OutcomeRef::new(
            CommandRef::new(request.command.name().clone()),
            outcome
                .parse()
                .map_err(|_| failure(&observation, &answer))?,
        );
        let error = match served.get("error") {
            Some(error) => {
                let name: ErrorRef = error
                    .as_str()
                    .and_then(|name| name.parse().ok())
                    .ok_or_else(|| failure(&observation, &answer))?;
                let mut declared = ess_conformance::DeclaredErrorValue::new(name);
                if let Some(serde_json::Value::Object(payload)) = served.get("payload") {
                    for (field, value) in payload {
                        if !value.is_null() {
                            declared = declared.with(field.clone(), node_of(value));
                        }
                    }
                }
                Some(declared)
            }
            None => None,
        };
        let mut direct_events = Vec::new();
        for published in served["published"].as_array().into_iter().flatten() {
            let reference: EventRef = published["event"]
                .as_str()
                .and_then(|name| name.parse().ok())
                .ok_or_else(|| failure(&observation, &answer))?;
            let mut event = ess_conformance::ObservedEvent::new(reference);
            if let Some(serde_json::Value::Object(payload)) = published.get("payload") {
                for (field, value) in payload {
                    event = event.with(field.clone(), node_of(value));
                }
            }
            let event = event
                .in_activity(request.correlation.clone())
                .at(self.tick());
            self.published.borrow_mut().push(event.clone());
            direct_events.push(event);
        }
        let consistency = Some(
            ess_primitives::consistency::ConsistencyToken::new(format!("seq:{}", self.tick()))
                .map_err(|why| {
                    ess_conformance::TargetError::unavailable(observation.clone(), why.to_string())
                })?,
        );
        Ok(ess_conformance::SemanticCommandResult {
            outcome: Some(outcome),
            error,
            consistency,
            direct_events,
            response: None,
        })
    }

    fn query_view(
        &self,
        request: ess_conformance::SemanticViewRequest,
    ) -> Result<ess_conformance::SemanticViewResult, ess_conformance::TargetError> {
        let observation = format!("reading `{}`", request.view);
        let answer = self.ask(&serde_json::json!({"op": "view", "view": request.view.to_string()}));
        let Some(rows) = answer["answer"]["rows"].as_array() else {
            return Err(failure(&observation, &answer));
        };
        Ok(ess_conformance::SemanticViewResult::of(rows.iter().map(
            |row| {
                row.as_object()
                    .into_iter()
                    .flatten()
                    .map(|(field, value)| (field.clone(), node_of(value)))
                    .collect::<ess_conformance::target::ViewRow>()
            },
        )))
    }

    fn observe_events(
        &self,
        request: ess_conformance::EventObservationRequest,
    ) -> Result<Vec<ess_conformance::ObservedEvent>, ess_conformance::TargetError> {
        Ok(self
            .published
            .borrow()
            .iter()
            .filter(|event| event.event == request.event)
            .cloned()
            .collect())
    }

    fn configure_external_outcome(
        &self,
        _: ess_conformance::ExternalOutcomeControl,
    ) -> Result<(), ess_conformance::TargetError> {
        Err(ess_conformance::TargetError::unsupported(
            "forcing an outcome",
            "none is external",
        ))
    }

    fn redeliver_event(
        &self,
        _: ess_conformance::RedeliveryRequest,
    ) -> Result<(), ess_conformance::TargetError> {
        Err(ess_conformance::TargetError::unsupported(
            "redelivering",
            "the model declares no binding",
        ))
    }
}
