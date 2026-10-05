//! Adversary pass 2 on `story:go-generated-behaviour`, after correction 1 (`910ebc38d`).
//!
//! Each case holds the Go emission to what the unit says of itself:
//!
//! - helpers are renamed out of a package name's way "in code only", so a domain named `truth`
//!   still yields a module that builds where a guard reads `any:`;
//! - `behaviour.go` imports standard-library packages for its helpers, so a domain named like one
//!   (`sort`, `big`, `strings`, `fmt`) still yields a module that builds;
//! - every behaviour and query the plan marks generated is generated in Go, and every one it owes
//!   is forwarded and stubbed — for the committed trees and for a model of this file;
//! - a method-name collision leaves a module that builds and a stub for every colliding seam;
//! - a stored field `broken_invariant` keeps its wire name, and a plain field where the entity
//!   declares no invariant;
//! - the committed gatepass servers answer every view alike after every command (badge excepted).

use std::io::{BufRead as _, Read as _, Write as _};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile_locating;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_synth::{synthesize_for, CapabilityKind, Synthesis, Target};
use serde_json::{json, Value};

// ---- shared ----------------------------------------------------------------------------------

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root")
}

fn scratch(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("adversary-go-pass2-{name}-{}", std::process::id()))
}

fn fixture(documents: &[(&str, &str)]) -> EssIr {
    let mut sources = SourceMap::new();
    let mut labels = Vec::new();
    let mut parsed = Vec::new();
    for (label, text) in documents {
        let raw = RawSpecFile::parse(text)
            .unwrap_or_else(|error| panic!("the fixture `{label}` is well formed: {error}"));
        sources.insert((*label).to_owned(), (*text).to_owned());
        labels.push((*label).to_owned());
        parsed.push((Source::new((*label).to_owned()), raw));
    }
    let specification = Specification::assemble(parsed)
        .unwrap_or_else(|errors| panic!("the fixture validates:\n{errors}"));
    compile_locating(&specification, &sources, &labels)
        .unwrap_or_else(|diagnostics| panic!("the fixture resolves:\n{diagnostics}"))
}

fn write_tree(directory: &Path, synthesis: &Synthesis) {
    let _ = std::fs::remove_dir_all(directory);
    for artifact in synthesis.artifacts.values() {
        let path = directory.join(&artifact.path);
        std::fs::create_dir_all(path.parent().expect("a file has a parent")).expect("mkdir");
        std::fs::write(&path, &artifact.contents).expect("write");
    }
}

fn go_tool(directory: &Path, arguments: &[&str]) -> Output {
    Command::new("go")
        .args(arguments)
        .current_dir(directory)
        .env("GOFLAGS", "-mod=mod")
        .env("GOPROXY", "off")
        .env("GOWORK", "off")
        .env("CGO_ENABLED", "0")
        .output()
        .expect("the Go tool runs")
}

fn go_available() -> bool {
    Command::new("go")
        .arg("version")
        .output()
        .is_ok_and(|output| output.status.success())
}

/// Builds and vets every package of a synthesized Go tree; `Err` carries the compiler's words.
fn go_builds(label: &str, synthesis: &Synthesis) -> Result<(), String> {
    let tree = scratch(label);
    write_tree(&tree, synthesis);
    let mut log = String::new();
    let mut ok = true;
    for arguments in [&["build", "./..."][..], &["vet", "./..."][..]] {
        let run = go_tool(&tree, arguments);
        if !run.status.success() {
            ok = false;
            log.push_str(&String::from_utf8_lossy(&run.stdout));
            log.push_str(&String::from_utf8_lossy(&run.stderr));
            break;
        }
    }
    let _ = std::fs::remove_dir_all(&tree);
    if ok {
        Ok(())
    } else {
        Err(log)
    }
}

// ---- helper renaming -------------------------------------------------------------------------

/// One bounded context named `domain`, whose one command reads an `any:` guard: Go spells that
/// `anyOf(...)`, whose parameter is `readings ...truth`.
fn guarded_model(domain: &str) -> String {
    let d = format!("probe.{domain}");
    format!(
        "format: ess/18
system: probe
version: v1
domain: {d}
errors:
  - name: {d}.OutOfRange
    summary: The hours are out of range.
events:
  - name: {d}.Logged
    fields:
      - {{name: hours, type: Integer}}
commands:
  - name: {d}.LogHours
    input:
      - {{name: hours, type: Integer}}
    outcomes:
      - name: out-of-range
        when: {{any: [hours > 10, hours < 0]}}
        error: {d}.OutOfRange
      - name: logged
        emits: [{d}.Logged]
        payload:
          {d}.Logged:
            hours: input.hours
components:
  - component: probe-service
    summary: Logs hours.
    owns: {{domains: [{d}]}}
    accepts: {{commands: [{d}.LogHours]}}
    publishes: {{events: [{d}.Logged]}}
"
    )
}

/// `rename_helpers` renames every helper "in code only" and treats a word beside a `.` as a
/// package qualifier — so the type in `readings ...truth`, preceded by the variadic `...`, is
/// never renamed while `type truth int` is. A domain named `truth` then leaves `...truth` naming
/// the domain package.
#[test]
fn adversary_a_domain_named_truth_still_builds_where_a_guard_reads_any() {
    if !go_available() {
        eprintln!("no Go toolchain on this machine; the variadic helper is unchecked");
        return;
    }
    let control = guarded_model("work");
    let ir = fixture(&[("model.yaml", &control)]);
    let go = synthesize_for(&ir, Target::Go).expect("the control synthesizes to Go");
    assert!(
        go.plan
            .is_generated(CapabilityKind::CommandBehavior, "probe.work.LogHours"),
        "the guarded command is generated"
    );
    assert!(
        go.artifacts["types/behaviour/behaviour.go"]
            .contents
            .contains("func anyOf(readings ...truth) truth {"),
        "the control reads its guard through anyOf"
    );
    go_builds("truth-control", &go)
        .unwrap_or_else(|log| panic!("the control builds under a neutral name:\n{log}"));

    let model = guarded_model("truth");
    let ir = fixture(&[("model.yaml", &model)]);
    let go = synthesize_for(&ir, Target::Go).expect("the probe synthesizes to Go");
    if let Err(log) = go_builds("truth", &go) {
        panic!(
            "a domain named `truth` does not build:\n{log}\n--- behaviour.go helpers ---\n{}",
            go.artifacts["types/behaviour/behaviour.go"]
                .contents
                .lines()
                .filter(|line| line.contains("truth"))
                .collect::<Vec<_>>()
                .join("\n")
        );
    }
}

/// `behaviour.go` imports `sort`, `math/big`, `strings` and `fmt` for its helpers, beside every
/// domain package it spells, and nothing reserves those names among the module's packages. The
/// generated-views fixture uses all four (an order, decimal sums and averages, number comparison,
/// and a distinct count over timestamps).
#[test]
fn adversary_a_domain_named_like_a_standard_library_import_still_builds() {
    if !go_available() {
        eprintln!("no Go toolchain on this machine; the import collision is unchecked");
        return;
    }
    let text = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/generated-views.yaml"),
    )
    .expect("the fixture is readable");
    let control = fixture(&[("model.yaml", &text)]);
    let go = synthesize_for(&control, Target::Go).expect("the control synthesizes to Go");
    let behaviour = &go.artifacts["types/behaviour/behaviour.go"].contents;
    for import in ["\"sort\"", "\"math/big\"", "\"strings\"", "\"fmt\""] {
        assert!(
            behaviour.contains(import),
            "the control imports {import}:\n{behaviour}"
        );
    }
    go_builds("stdlib-control", &go)
        .unwrap_or_else(|log| panic!("the control builds under `ledger.work`:\n{log}"));
    let mut broken = Vec::new();
    for domain in ["sort", "big", "strings", "fmt"] {
        let renamed = text.replace("ledger.work", &format!("ledger.{domain}"));
        let ir = fixture(&[("model.yaml", &renamed)]);
        let go = synthesize_for(&ir, Target::Go).expect("the probe synthesizes to Go");
        if let Err(log) = go_builds(&format!("stdlib-{domain}"), &go) {
            broken.push(format!("domain `ledger.{domain}`:\n{log}"));
        }
    }
    assert!(
        broken.is_empty(),
        "a domain named like a standard-library package does not build:\n{}",
        broken.join("\n")
    );
}

// ---- the generated-ness boundary ------------------------------------------------------------

/// The body of `func (b *Generated) <method>(`, up to its closing brace at column 0.
fn generated_method<'a>(behaviour: &'a str, method: &str) -> Option<&'a str> {
    let head = format!(" *Generated) {method}(");
    let start = behaviour.find(&head)?;
    let rest = &behaviour[start..];
    let end = rest.find("\n}\n").map_or(rest.len(), |at| at + 3);
    Some(&rest[..end])
}

/// Every command behaviour and view query of a plan, with whether it is generated, read from a
/// canonical `plan.json`.
fn planned_seams(plan: &Value) -> Vec<(String, bool)> {
    plan["capabilities"]
        .as_array()
        .expect("capabilities")
        .iter()
        .filter(|capability| {
            matches!(
                capability["kind"].as_str(),
                Some("command_behavior" | "view_query")
            )
        })
        .map(|capability| {
            (
                capability["source"].as_str().expect("a source").to_owned(),
                capability["disposition"]["disposition"] == "generated",
            )
        })
        .collect()
}

/// Every way a Go tree lets the plan's line and the emitted code disagree: a generated seam that
/// forwards or refuses, or has no stub-free method; an owed seam that is computed, or has no stub.
fn boundary_problems(
    label: &str,
    plan: &Value,
    behaviour: &str,
    domain_sources: &str,
    target: &str,
) -> Vec<String> {
    let mut problems = Vec::new();
    for (source, generated) in planned_seams(plan) {
        let method = source.rsplit('.').next().expect("a name");
        let stub = domain_sources.contains(&format!("func (Unimplemented) {method}("));
        let named_collision = target.contains(&format!("`{method}`"));
        match generated_method(behaviour, method) {
            None if named_collision => {}
            None => problems.push(format!(
                "{label}: `{source}` is not a method of Generated, and TARGET.md is silent"
            )),
            Some(body) if generated => {
                if body.contains(".Owed.") || body.contains("Unimplemented") {
                    problems.push(format!(
                        "{label}: `{source}` is generated by the plan but forwarded:\n{body}"
                    ));
                }
                if stub {
                    problems.push(format!(
                        "{label}: `{source}` is generated by the plan but stubbed as owed"
                    ));
                }
            }
            Some(body) => {
                if !body.contains(&format!(".Owed.{method}(")) {
                    problems.push(format!(
                        "{label}: `{source}` is owed by the plan but computed in Go:\n{body}"
                    ));
                }
                if !stub {
                    problems.push(format!("{label}: `{source}` is owed but has no stub"));
                }
            }
        }
    }
    problems
}

fn committed_tree(name: &str) -> (Value, String, String, String) {
    let tree = root().join("generated/go").join(name);
    let plan: Value =
        serde_json::from_str(&std::fs::read_to_string(tree.join("plan.json")).expect("plan.json"))
            .expect("plan.json is JSON");
    let behaviour =
        std::fs::read_to_string(tree.join("types/behaviour/behaviour.go")).expect("behaviour.go");
    let mut domains = String::new();
    for entry in std::fs::read_dir(tree.join("types")).expect("types") {
        let directory = entry.expect("an entry").path();
        for file in std::fs::read_dir(&directory).expect("a package") {
            let file = file.expect("a file").path();
            if file.extension().is_some_and(|it| it == "go") {
                domains.push_str(&std::fs::read_to_string(&file).expect("readable"));
            }
        }
    }
    let target = std::fs::read_to_string(tree.join("TARGET.md")).expect("TARGET.md");
    (plan, behaviour, domains, target)
}

fn synthesized_tree(synthesis: &Synthesis) -> (Value, String, String, String) {
    let plan: Value =
        serde_json::from_str(&synthesis.plan.to_canonical_json()).expect("the plan is JSON");
    let behaviour = synthesis
        .artifacts
        .get("types/behaviour/behaviour.go")
        .map(|artifact| artifact.contents.clone())
        .unwrap_or_default();
    let domains: String = synthesis
        .artifacts
        .iter()
        .filter(|(path, _)| {
            path.starts_with("types/")
                && Path::new(path.as_str()).extension() == Some("go".as_ref())
        })
        .map(|(_, artifact)| artifact.contents.as_str())
        .collect();
    let target = synthesis.artifacts["TARGET.md"].contents.clone();
    (plan, behaviour, domains, target)
}

/// One bounded context using existence selection (update-or-create and an existing-instance
/// refusal), a held-state guard, an aggregate view and invariants together, beside one command a
/// `when_related:` guard beside an `external:` branch keeps owed.
const MIXED: &str = "format: ess/18
system: mix
version: v1
domain: mix.shop
types:
  - {name: mix.shop.OrderId, kind: newtype, of: String}
  - {name: mix.shop.SlotId, kind: newtype, of: String}
entities:
  - name: mix.shop.Order
    identity: {name: order_id, type: mix.shop.OrderId}
    fields:
      - {name: shop, type: String}
      - {name: quantity, type: Integer}
      - {name: price, type: Decimal}
    invariants:
      - quantity >= 0
      - price >= 0
    lifecycle:
      initial: Placed
      states: [Placed, Shipped]
      terminal: [Shipped]
      transitions:
        - {name: ship, from: [Placed], to: Shipped}
  - name: mix.shop.Slot
    identity: {name: slot_id, type: mix.shop.SlotId}
    fields:
      - {name: label, type: String}
    lifecycle: {initial: Open, states: [Open], terminal: [Open], transitions: []}
events:
  - name: mix.shop.OrderStored
    fields: [{name: order_id, type: mix.shop.OrderId}]
  - name: mix.shop.OrderShipped
    fields: [{name: order_id, type: mix.shop.OrderId}]
  - name: mix.shop.SlotBooked
    fields: [{name: slot_id, type: mix.shop.SlotId}]
errors:
  - name: mix.shop.Conflict
  - name: mix.shop.Taken
  - name: mix.shop.NoSlot
actors:
  - {name: mix.shop.Clerk, may: [mix.shop.PutOrder, mix.shop.BookSlot, mix.shop.ShipOrder, mix.shop.HoldSlot]}
commands:
  - name: mix.shop.PutOrder
    input:
      - {name: order_id, type: mix.shop.OrderId}
      - {name: shop, type: String}
      - {name: quantity, type: Integer}
      - {name: price, type: Decimal}
    outcomes:
      - name: updated
        updates: mix.shop.Order
        instance: order_id
        sets: {quantity: input.quantity, price: input.price}
        emits: [mix.shop.OrderStored]
        payload:
          mix.shop.OrderStored: {order_id: input.order_id}
      - name: created
        unknown_instance: true
        creates: mix.shop.Order
        instance: order_id
        sets: {shop: input.shop, quantity: input.quantity, price: input.price}
        emits: [mix.shop.OrderStored]
        payload:
          mix.shop.OrderStored: {order_id: input.order_id}
  - name: mix.shop.BookSlot
    input:
      - {name: slot_id, type: mix.shop.SlotId}
      - {name: label, type: String}
    outcomes:
      - name: booked
        creates: mix.shop.Slot
        instance: slot_id
        sets: {label: input.label}
        emits: [mix.shop.SlotBooked]
        payload:
          mix.shop.SlotBooked: {slot_id: input.slot_id}
      - {name: already-booked, existing_instance: true, error: mix.shop.Taken}
  - name: mix.shop.ShipOrder
    input: [{name: order_id, type: mix.shop.OrderId}]
    outcomes:
      - name: already-shipped
        when_subject_state: Shipped
        error: mix.shop.Conflict
      - name: shipped
        moves: mix.shop.Order.ship
        instance: order_id
        emits: [mix.shop.OrderShipped]
        payload:
          mix.shop.OrderShipped: {order_id: input.order_id}
      - {name: no-such-order, unknown_instance: true, error: mix.shop.Conflict}
  - name: mix.shop.HoldSlot
    input:
      - {name: slot_id, type: mix.shop.SlotId}
      - {name: label, type: String}
    outcomes:
      - name: no-slot
        when_related: {via: input.slot_id, exists: false}
        error: mix.shop.NoSlot
      - {name: provider-refused, external: the slot provider refuses the hold, error: mix.shop.NoSlot}
      - name: held
        emits: [mix.shop.SlotBooked]
        payload:
          mix.shop.SlotBooked: {slot_id: input.slot_id}
views:
  - name: mix.shop.Orders
    source: mix.shop.Order
    consistency: read_your_writes
    fields:
      - {name: order_id, type: mix.shop.OrderId}
      - {name: state, type: mix.shop.Order.State}
      - {name: quantity, type: Integer}
  - name: mix.shop.ByShop
    source: mix.shop.Order
    consistency: read_your_writes
    filter: state == Placed
    group_by: [shop]
    fields:
      - {name: shop, type: String}
      - {name: orders, type: Integer, aggregate: {count: {}}}
      - {name: units, type: Integer, aggregate: {sum: quantity}}
      - {name: mean_price, type: Optional<Decimal>, aggregate: {avg: price}}
      - {name: dearest, type: Optional<Decimal>, aggregate: {max: price}}
components:
  - component: mix-service
    summary: Holds every order and slot.
    owns: {domains: [mix.shop]}
    accepts: {commands: [mix.shop.PutOrder, mix.shop.BookSlot, mix.shop.ShipOrder, mix.shop.HoldSlot]}
    publishes: {events: [mix.shop.OrderStored, mix.shop.OrderShipped, mix.shop.SlotBooked]}
    reached_by: network
";

/// The plan is the one source of what is generated (`story:go-generated-behaviour`, Decisions):
/// "Go emits every `CommandBehavior` and `ViewQuery` the plan marks generated", and "`Unimplemented`
/// covers owed seams only". Held for both committed trees and for [`MIXED`], whose Go tree must
/// also build and vet clean.
#[test]
fn adversary_every_seam_the_plan_generates_is_generated_in_go_and_every_owed_one_is_forwarded() {
    let mut problems = Vec::new();
    for name in ["billing", "gatepass"] {
        let (plan, behaviour, domains, target) = committed_tree(name);
        problems.extend(boundary_problems(
            name, &plan, &behaviour, &domains, &target,
        ));
    }
    let ir = fixture(&[("model.yaml", MIXED)]);
    let go = synthesize_for(&ir, Target::Go).expect("the mixed model synthesizes to Go");
    let rust = synthesize_for(&ir, Target::Rust).expect("the mixed model synthesizes to Rust");
    assert_eq!(
        go.artifacts["PLAN.md"].contents, rust.artifacts["PLAN.md"].contents,
        "one plan in both trees"
    );
    let expected = [
        (CapabilityKind::CommandBehavior, "mix.shop.PutOrder", true),
        (CapabilityKind::CommandBehavior, "mix.shop.BookSlot", true),
        (CapabilityKind::CommandBehavior, "mix.shop.ShipOrder", true),
        (CapabilityKind::CommandBehavior, "mix.shop.HoldSlot", false),
        (CapabilityKind::ViewQuery, "mix.shop.Orders", true),
        (CapabilityKind::ViewQuery, "mix.shop.ByShop", true),
    ];
    for (kind, source, generated) in expected {
        assert_eq!(
            go.plan.is_generated(kind, source),
            generated,
            "the plan's line for `{source}`"
        );
    }
    let (plan, behaviour, domains, target) = synthesized_tree(&go);
    problems.extend(boundary_problems(
        "mixed", &plan, &behaviour, &domains, &target,
    ));
    if go_available() {
        if let Err(log) = go_builds("mixed", &go) {
            problems.push(format!("mixed: the Go module does not build:\n{log}"));
        }
    }
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

// ---- the collision row -----------------------------------------------------------------------

/// Two bounded contexts, each with a `Place` that `generated` decides is fully declared or owed,
/// and a generated `Ping<Domain>`.
fn twins(generated: bool) -> EssIr {
    let context = |domain: &str, ping: &str| {
        let place = if generated {
            format!(
                "      - name: done
        emits: [twice.{domain}.Placed]
        payload:
          twice.{domain}.Placed:
            note: input.note
"
            )
        } else {
            format!(
                "      - name: rejected
        when: note == \"no\"
        error: twice.{domain}.Rejected
      - name: done
        emits: [twice.{domain}.Placed]
        payload:
          twice.{domain}.Placed:
            note: input.note
"
            )
        };
        format!(
            "domain: twice.{domain}
errors:
  - name: twice.{domain}.Rejected
    summary: The note is refused.
    fields:
      - {{name: reason, type: String}}
events:
  - name: twice.{domain}.Placed
    fields:
      - {{name: note, type: String}}
commands:
  - name: twice.{domain}.Place
    input:
      - {{name: note, type: String}}
    outcomes:
{place}  - name: twice.{domain}.{ping}
    input:
      - {{name: note, type: String}}
    outcomes:
      - name: done
        emits: [twice.{domain}.Placed]
        payload:
          twice.{domain}.Placed:
            note: input.note
"
        )
    };
    let component = |domain: &str, ping: &str| {
        format!(
            "  - component: {domain}-desk
    owns:
      domains: [twice.{domain}]
    accepts:
      commands: [twice.{domain}.Place, twice.{domain}.{ping}]
    publishes:
      events: [twice.{domain}.Placed]
"
        )
    };
    fixture(&[
        (
            "system.yaml",
            "format: ess/1\nsystem: twice\nversion: v1\ndomains:\n  - twice.alpha\n  - \
             twice.beta\n",
        ),
        ("alpha.yaml", &context("alpha", "PingAlpha")),
        ("beta.yaml", &context("beta", "PingBeta")),
        (
            "wiring.yaml",
            &format!(
                "components:\n{}{}",
                component("alpha", "PingAlpha"),
                component("beta", "PingBeta")
            ),
        ),
    ])
}

/// The collision row says each colliding seam "keeps its seam here, owed, with the same contract
/// and a stub refusing it, exactly as an obligation", and that a component naming one "takes a
/// bundle of your own that has the method and delegates the rest to a `*Generated`". So: the
/// module builds, the row names `Place` and both seams, each domain stubs its `Place`, and the
/// non-colliding seams are still `Generated`'s.
#[test]
fn adversary_a_collision_leaves_a_building_module_and_a_stub_for_every_colliding_seam() {
    let mut problems = Vec::new();
    for generated in [false, true] {
        let label = if generated { "generated" } else { "owed" };
        let ir = twins(generated);
        let go = synthesize_for(&ir, Target::Go).expect("the twins synthesize to Go");
        assert_eq!(
            go.plan
                .is_generated(CapabilityKind::CommandBehavior, "twice.alpha.Place"),
            generated,
            "{label}: the plan's line for Place"
        );
        let target = &go.artifacts["TARGET.md"].contents;
        if !(target.contains("`Place`")
            && target.contains("twice.alpha.Place")
            && target.contains("twice.beta.Place"))
        {
            problems.push(format!("{label}: TARGET.md does not name the collision"));
        }
        for domain in ["alpha", "beta"] {
            let file = &go.artifacts[&format!("types/{domain}/{domain}.go")].contents;
            if !file.contains("func (Unimplemented) Place(") {
                problems.push(format!(
                    "{label}: `twice.{domain}.Place` collides and has no stub in its package"
                ));
            }
        }
        let behaviour = &go.artifacts["types/behaviour/behaviour.go"].contents;
        for ping in ["PingAlpha", "PingBeta"] {
            if generated_method(behaviour, ping).is_none() {
                problems.push(format!("{label}: `{ping}` is no longer generated"));
            }
        }
        if go_available() {
            if let Err(log) = go_builds(&format!("twins-{label}"), &go) {
                problems.push(format!("{label}: the Go module does not build:\n{log}"));
            }
        }
    }
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

// ---- BrokenInvariant reservation -------------------------------------------------------------

fn flagged_model(invariant: bool) -> String {
    let invariants = if invariant {
        "    invariants:\n      - hours >= 0\n"
    } else {
        ""
    };
    format!(
        "format: ess/18
system: flag
version: v1
domain: flag.work
types:
  - {{name: flag.work.TaskId, kind: newtype, of: String}}
events:
  - name: flag.work.TaskLogged
    fields:
      - {{name: task_id, type: flag.work.TaskId}}
      - {{name: broken_invariant, type: Boolean}}
entities:
  - name: flag.work.Task
    identity: {{name: task_id, type: flag.work.TaskId}}
    fields:
      - {{name: hours, type: Integer}}
      - {{name: broken_invariant, type: Boolean}}
{invariants}    lifecycle: {{initial: Open, states: [Open], terminal: [Open], transitions: []}}
actors:
  - {{name: flag.work.Clerk, may: [flag.work.LogTask]}}
commands:
  - name: flag.work.LogTask
    input:
      - {{name: task_id, type: flag.work.TaskId}}
      - {{name: hours, type: Integer}}
      - {{name: broken_invariant, type: Boolean}}
    outcomes:
      - name: logged
        creates: flag.work.Task
        instance: task_id
        sets: {{hours: input.hours, broken_invariant: input.broken_invariant}}
        emits: [flag.work.TaskLogged]
        payload:
          flag.work.TaskLogged: {{task_id: input.task_id, broken_invariant: input.broken_invariant}}
views:
  - name: flag.work.Tasks
    source: flag.work.Task
    consistency: read_your_writes
    filter: broken_invariant == true
    fields:
      - {{name: task_id, type: flag.work.TaskId}}
      - {{name: broken_invariant, type: Boolean}}
  - name: flag.work.Flags
    source: flag.work.Task
    consistency: read_your_writes
    fields:
      - {{name: flagged, type: Integer, aggregate: {{count_distinct: broken_invariant}}}}
components:
  - component: flag-service
    summary: Holds every task.
    owns: {{domains: [flag.work]}}
    accepts: {{commands: [flag.work.LogTask]}}
    publishes: {{events: [flag.work.TaskLogged]}}
    reached_by: network
"
    )
}

/// With an invariant the data field becomes `BrokenInvariant_`, and every generated read of it —
/// a `sets:`, a filter, a projection, an aggregate — must spell that; the wire name stays
/// `broken_invariant` wherever a body carries it. Without an invariant the field is the plain
/// `BrokenInvariant`.
#[test]
fn adversary_a_broken_invariant_field_keeps_its_wire_name_and_every_read_builds() {
    let mut problems = Vec::new();
    for invariant in [true, false] {
        let label = if invariant { "invariant" } else { "plain" };
        let model = flagged_model(invariant);
        let ir = fixture(&[("model.yaml", &model)]);
        let go = synthesize_for(&ir, Target::Go).expect("the model synthesizes to Go");
        let source = "flag.work.LogTask";
        if !go
            .plan
            .is_generated(CapabilityKind::CommandBehavior, source)
        {
            problems.push(format!("{label}: `{source}` is not generated"));
        }
        for source in ["flag.work.Tasks", "flag.work.Flags"] {
            if !go.plan.is_generated(CapabilityKind::ViewQuery, source) {
                problems.push(format!("{label}: `{source}` is not generated"));
            }
        }
        let domain = &go.artifacts["types/work/work.go"].contents;
        let field = if invariant {
            "\tBrokenInvariant_ bool\n"
        } else {
            "\tBrokenInvariant bool\n"
        };
        if !domain.contains(field) {
            problems.push(format!("{label}: the data struct lacks `{}`", field.trim()));
        }
        let wire: String = go
            .artifacts
            .iter()
            .filter(|(path, _)| path.starts_with("server/"))
            .map(|(_, artifact)| artifact.contents.as_str())
            .collect();
        if !wire.contains("\"broken_invariant\"") || wire.contains("BrokenInvariant_\"") {
            problems.push(format!(
                "{label}: the served wire name is not `broken_invariant`"
            ));
        }
        if go_available() {
            if let Err(log) = go_builds(&format!("flag-{label}"), &go) {
                problems.push(format!("{label}: the Go module does not build:\n{log}"));
            }
        }
    }
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

// ---- the served gatepass views ---------------------------------------------------------------

fn rust_server() -> PathBuf {
    let target = scratch("rust-target");
    let built = Command::new(std::env::var_os("CARGO").expect("Cargo supplies its executable"))
        .args([
            "build",
            "--offline",
            "--quiet",
            "--bin",
            "gatepass-server",
            "--manifest-path",
        ])
        .arg(root().join("examples/gatepass-realization/Cargo.toml"))
        .arg("--target-dir")
        .arg(&target)
        .env_remove("CARGO_TARGET_DIR")
        .env("CARGO_INCREMENTAL", "0")
        .env_remove("RUSTC_WRAPPER")
        .output()
        .expect("cargo runs");
    assert!(
        built.status.success(),
        "the Rust gatepass server builds: {}",
        String::from_utf8_lossy(&built.stderr)
    );
    target.join("debug/gatepass-server")
}

fn go_server() -> PathBuf {
    let binary = scratch("go-server");
    let built = Command::new("go")
        .arg("build")
        .arg("-o")
        .arg(&binary)
        .arg("./cmd/gatepass-server")
        .current_dir(root().join("examples/gatepass-go-realization"))
        .env("GOPROXY", "off")
        .env("CGO_ENABLED", "0")
        .output()
        .expect("go build runs");
    assert!(
        built.status.success(),
        "the Go gatepass server builds: {}",
        String::from_utf8_lossy(&built.stderr)
    );
    binary
}

struct Served(Child);

impl Drop for Served {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn serve(binary: &Path) -> (Served, u16) {
    let mut child = Command::new(binary)
        .env("PORT", "0")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the server starts");
    let stdout = child.stdout.take().expect("piped");
    let mut lines = std::io::BufReader::new(stdout).lines();
    let first = lines.next().expect("a startup record").expect("text");
    let record: Value = serde_json::from_str(&first).expect("the record is JSON");
    let port = record["runtime"]["port"]
        .as_u64()
        .and_then(|port| u16::try_from(port).ok())
        .expect("the record names the port");
    std::thread::spawn(move || lines.for_each(drop));
    (Served(child), port)
}

fn request(port: u16, method: &str, path: &str, body: &str) -> (u16, Value) {
    let mut stream = std::net::TcpStream::connect(("127.0.0.1", port)).expect("the server accepts");
    write!(
        stream,
        "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\n\
         Authorization: Actor gatepass.visit.Receptionist\r\nContent-Length: {}\r\n\
         Connection: close\r\n\r\n{body}",
        body.len()
    )
    .expect("the request writes");
    let mut answer = String::new();
    stream
        .read_to_string(&mut answer)
        .expect("the answer reads");
    let (head, payload) = answer.split_once("\r\n\r\n").expect("a head and a body");
    let status = head
        .split(' ')
        .nth(1)
        .and_then(|status| status.parse().ok())
        .expect("a status");
    let payload =
        serde_json::from_str(payload).unwrap_or_else(|_| Value::String(payload.to_owned()));
    (status, payload)
}

fn register(visitor: &str, minutes: i64) -> String {
    json!({
        "visitor": visitor,
        "building": "North",
        "host": {"kind": "employee", "value": "E1"},
        "expected_minutes": minutes,
        "expected_stay": "PT30M",
        "deposit": {"amount": "10.50", "currency": "EUR"},
        "escorts": ["Bo"],
        "notes": {"k": "v"},
        "on_watchlist": false,
    })
    .to_string()
}

/// Every view, after every command of a sequence that walks each visit through each state — every
/// answer with the visit ids replaced by `{V1}` / `{V2}` / `{V3}`, and `badge` dropped from the
/// `by-id` rows (the known difference, pending the model's `sets: {badge}`).
fn walk(port: u16) -> Vec<(String, u16, Value)> {
    let badge = r#"{"serial":"s-1","printed_at":"2026-09-30T08:00:00Z","signature":"AAEC"}"#;
    let steps: Vec<(&str, &str, String)> = vec![
        (
            "register-1",
            "/visits/commands/register-visit",
            register("Ada", 30),
        ),
        (
            "register-2",
            "/visits/commands/register-visit",
            register("Bo", 5),
        ),
        (
            "register-3",
            "/visits/commands/register-visit",
            register("Cy", 90),
        ),
        (
            "admit-2",
            "/visits/commands/admit-visitor",
            format!(r#"{{"visit_id":"{{V2}}","badge":{badge}}}"#),
        ),
        (
            "admit-unknown",
            "/visits/commands/admit-visitor",
            format!(r#"{{"visit_id":"00000000-0000-4000-8000-0000000000ff","badge":{badge}}}"#),
        ),
        (
            "sign-out-3",
            "/visits/commands/sign-out-visitor",
            r#"{"visit_id":"{V3}"}"#.to_owned(),
        ),
        (
            "admit-3-after-departure",
            "/visits/commands/admit-visitor",
            format!(r#"{{"visit_id":"{{V3}}","badge":{badge}}}"#),
        ),
        (
            "sign-out-2",
            "/visits/commands/sign-out-visitor",
            r#"{"visit_id":"{V2}"}"#.to_owned(),
        ),
    ];
    let mut ids: Vec<String> = Vec::new();
    let mut answers = Vec::new();
    let mut record = |label: String, status: u16, answer: &Value, ids: &[String]| {
        let mut text = answer.to_string();
        for (index, id) in ids.iter().enumerate() {
            text = text.replace(id, &format!("{{V{}}}", index + 1));
        }
        let mut value: Value = serde_json::from_str(&text).expect("JSON");
        if label.ends_with("by-id") {
            for row in value["rows"].as_array_mut().into_iter().flatten() {
                if let Some(row) = row.as_object_mut() {
                    row.remove("badge");
                }
            }
        }
        answers.push((label, status, value));
    };
    for (label, path, body) in steps {
        let mut body = body;
        for (index, id) in ids.iter().enumerate() {
            body = body.replace(&format!("{{V{}}}", index + 1), id);
        }
        let (status, answer) = request(port, "POST", path, &body);
        if label.starts_with("register") && status == 202 {
            ids.push(
                answer["published"][0]["payload"]["visit_id"]
                    .as_str()
                    .expect("the created identity is in the payload")
                    .to_owned(),
            );
        }
        record(label.to_owned(), status, &answer, &ids);
        for view in ["expected", "by-id"] {
            let (status, answer) = request(port, "GET", &format!("/visits/views/{view}"), "");
            record(format!("{label} {view}"), status, &answer, &ids);
        }
    }
    answers
}

#[test]
fn adversary_the_gatepass_servers_answer_every_view_alike_after_every_command() {
    if !go_available() {
        eprintln!("no Go toolchain on this machine; the two gatepass servers are not compared");
        return;
    }
    let rust = rust_server();
    let go = go_server();
    let rust_answers = {
        let (_served, port) = serve(&rust);
        walk(port)
    };
    let go_answers = {
        let (_served, port) = serve(&go);
        walk(port)
    };
    let _ = std::fs::remove_file(&go);
    let _ = std::fs::remove_dir_all(scratch("rust-target"));
    assert_eq!(rust_answers.len(), go_answers.len());
    let refused = rust_answers
        .iter()
        .filter(|(_, status, _)| *status == 403)
        .count();
    assert_eq!(refused, 0, "every request is granted: {rust_answers:#?}");
    let mut problems = Vec::new();
    for ((label, rust_status, rust_answer), (_, go_status, go_answer)) in
        rust_answers.iter().zip(&go_answers)
    {
        if (rust_status, rust_answer) != (go_status, go_answer) {
            problems.push(format!(
                "`{label}`: Rust {rust_status} {rust_answer}\n            but Go {go_status} \
                 {go_answer}"
            ));
        }
    }
    assert!(
        problems.is_empty(),
        "the two gatepass servers answer differently:\n{}",
        problems.join("\n")
    );
}
