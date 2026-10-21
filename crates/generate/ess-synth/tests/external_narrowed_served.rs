//! An `external:` branch narrowed to the rows a subject guard selects (`ess/29`,
//! `story:external-branch-narrowed-by-subject-guard`), executed against the generated Rust and Go
//! behaviour.
//!
//! `Redeem` refuses a hashed grant when the outside service says so (`hashed-mismatch`, declared
//! first), a wrong redirect and a wrong plain verifier. Its behaviour is generated, and a harness
//! supplies the ports the specification leaves to the implementor — where a grant is stored and
//! which external branch the provider takes — recording each port call. The provider is asked only
//! for a hashed grant, after the row is read, and before the redirect is compared; on a plain grant
//! it is never asked, so forcing it changes nothing there.

#[allow(dead_code)]
mod related_guard_served;
use related_guard_served as served;

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use ess_synth::{CapabilityKind, SynthesisDisposition, Target};

const MODEL: &str = include_str!(
    "../../../verify/ess-conformance/tests/fixtures/external-narrowed-by-subject.yaml"
);

fn model() -> ess_compiler::ir::EssIr {
    served::model(&served::served(MODEL, "vault"))
}

/// A fresh directory for one target's tree and harness, kept on failure for review.
fn fresh_root(label: &str) -> PathBuf {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("external-narrowed-{label}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("a fresh root");
    root
}

fn write(synthesis: &ess_synth::Synthesis, directory: &Path) {
    for (relative, artifact) in &synthesis.artifacts {
        let file = directory.join(relative);
        std::fs::create_dir_all(file.parent().expect("a parent")).expect("mkdir");
        std::fs::write(file, &artifact.contents).expect("write artifact");
    }
}

fn printed(output: &Output) -> String {
    format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn assert_generated(synthesis: &ess_synth::Synthesis, target: Target) {
    assert_eq!(
        synthesis
            .plan
            .disposition_of(CapabilityKind::CommandBehavior, "demo.vault.Redeem"),
        Some(&SynthesisDisposition::Generated),
        "{target:?}:\n{}",
        synthesis.plan.to_markdown()
    );
}

/// Each case: the grant's method, the redirect sent, the verifier sent, whether the provider says
/// the verifier is wrong, the outcome required, and the port calls required in order.
const CASES: [(&str, &str, &str, &str, bool, &str, &str); 6] = [
    // On a hashed grant the provider is asked after the row is read, before the redirect.
    (
        "hashed-forced",
        "hashed",
        "r",
        "v",
        true,
        "hashed-mismatch",
        "get,external",
    ),
    (
        "hashed-forced-wrong-redirect",
        "hashed",
        "elsewhere",
        "v",
        true,
        "hashed-mismatch",
        "get,external",
    ),
    (
        "hashed-declined",
        "hashed",
        "r",
        "v",
        false,
        "redeemed",
        "get,external,get,put",
    ),
    (
        "hashed-declined-wrong-redirect",
        "hashed",
        "elsewhere",
        "v",
        false,
        "redirect-mismatch",
        "get,external",
    ),
    // On a plain grant the provider is never asked, whatever it would say.
    (
        "plain-forced",
        "plain",
        "r",
        "c",
        true,
        "redeemed",
        "get,get,put",
    ),
    (
        "plain-forced-wrong-verifier",
        "plain",
        "r",
        "v",
        true,
        "plain-mismatch",
        "get",
    ),
];

const RUST_HARNESS: &str = r#"use std::cell::RefCell;
use std::rc::Rc;
use demo_types::behaviour::{Context, ExternalCommand, Generated, GrantStorage};
use demo_types::primitives::Uuid;
use demo_types::vault::{self, obligations::RedeemBehavior};

struct State {
    held: Option<vault::GrantSnapshot>,
    refuse: bool,
    calls: Vec<&'static str>,
}
#[derive(Clone)]
struct Ports(Rc<RefCell<State>>);
impl GrantStorage for Ports {
    fn get(&self, id: &vault::GrantId) -> Option<vault::GrantSnapshot> {
        let mut s = self.0.borrow_mut();
        s.calls.push("get");
        s.held.clone().filter(|held| held.data.grant_id == *id)
    }
    fn put(&mut self, held: vault::GrantSnapshot) {
        let mut s = self.0.borrow_mut();
        s.calls.push("put");
        s.held = Some(held);
    }
    fn delete(&mut self, _: &vault::GrantId) { panic!("redeem never deletes"); }
    fn list(&self) -> Vec<vault::GrantSnapshot> { self.0.borrow().held.clone().into_iter().collect() }
}
impl Context for Ports {
    fn generate_demo_vault_grant_id(&mut self) -> vault::GrantId { panic!("redeem creates nothing") }
    fn external(&mut self, command: ExternalCommand<'_>, outcome: &'static str) -> bool {
        assert_eq!(command.name(), "demo.vault.Redeem");
        assert_eq!(outcome, "hashed-mismatch");
        let mut s = self.0.borrow_mut();
        s.calls.push("external");
        s.refuse
    }
}
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let (case, method, redirect, verifier, refuse, expected, calls) =
        (&args[1], &args[2], &args[3], &args[4], args[5] == "true", &args[6], &args[7]);
    let id = vault::GrantId(Uuid("00000000-0000-4000-8000-000000000001".to_owned()));
    let held = vault::GrantSnapshot {
        state: vault::GrantState::Issued,
        data: vault::GrantData {
            grant_id: id.clone(),
            method: if method == "hashed" { vault::Method::Hashed } else { vault::Method::Plain },
            challenge: "c".to_owned(),
            redirect: "r".to_owned(),
        },
    };
    let ports = Ports(Rc::new(RefCell::new(State { held: Some(held), refuse, calls: Vec::new() })));
    let input = vault::Redeem { grant_id: id, verifier: verifier.to_owned(), redirect: redirect.to_owned() };
    let outcome = match Generated::new(ports.clone()).redeem(input).expect("a declared outcome") {
        vault::RedeemOutcome::HashedMismatch { .. } => "hashed-mismatch",
        vault::RedeemOutcome::RedirectMismatch { .. } => "redirect-mismatch",
        vault::RedeemOutcome::PlainMismatch { .. } => "plain-mismatch",
        vault::RedeemOutcome::Redeemed { .. } => "redeemed",
        vault::RedeemOutcome::AlreadyRedeemed { .. } => "already-redeemed",
    };
    let s = ports.0.borrow();
    let made = s.calls.join(",");
    println!("{case}\t{outcome}\t{made}");
    assert_eq!(outcome, expected.as_str(), "{case}");
    assert_eq!(made, calls.as_str(), "{case}");
}
"#;

const GO_HARNESS: &str = r#"package main

import (
	"fmt"
	"os"
	"strings"

	"example.invalid/demo/types/behaviour"
	"example.invalid/demo/types/primitives"
	"example.invalid/demo/types/vault"
)

type ports struct {
	held   *vault.GrantSnapshot
	refuse bool
	calls  []string
}

func (p *ports) Get(id vault.GrantId) (vault.GrantSnapshot, bool) {
	p.calls = append(p.calls, "get")
	if p.held == nil || p.held.Data.GrantId != id {
		return vault.GrantSnapshot{}, false
	}
	return *p.held, true
}
func (p *ports) Put(row vault.GrantSnapshot) { p.calls = append(p.calls, "put"); p.held = &row }
func (p *ports) Delete(_ vault.GrantId)       { panic("redeem never deletes") }
func (p *ports) List() []vault.GrantSnapshot {
	if p.held == nil {
		return nil
	}
	return []vault.GrantSnapshot{*p.held}
}
func (p *ports) GenerateDemoVaultGrantId() vault.GrantId { panic("redeem creates nothing") }
func (p *ports) External(command behaviour.ExternalCommand, outcome string) bool {
	if command.Name() != "demo.vault.Redeem" || outcome != "hashed-mismatch" {
		panic("unexpected external branch")
	}
	p.calls = append(p.calls, "external")
	return p.refuse
}
func main() {
	c, method, redirect, verifier, refuse, expected, calls := os.Args[1], os.Args[2], os.Args[3], os.Args[4], os.Args[5] == "true", os.Args[6], os.Args[7]
	id := vault.NewGrantId(primitives.NewUuid("00000000-0000-4000-8000-000000000001"))
	var held vault.GrantSnapshot
	held.State = vault.GrantStateIssued{}
	held.Data.GrantId = id
	held.Data.Challenge = "c"
	held.Data.Redirect = "r"
	if method == "hashed" {
		held.Data.Method = vault.MethodHashed{}
	} else {
		held.Data.Method = vault.MethodPlain{}
	}
	p := &ports{held: &held, refuse: refuse}
	generated := behaviour.New(behaviour.Ports{GrantStorage: p, Context: p})
	result, err := generated.Redeem(vault.Redeem{GrantId: id, Verifier: verifier, Redirect: redirect})
	if err != nil {
		panic("a declared outcome")
	}
	outcome := ""
	switch result.(type) {
	case vault.RedeemOutcomeHashedMismatch:
		outcome = "hashed-mismatch"
	case vault.RedeemOutcomeRedirectMismatch:
		outcome = "redirect-mismatch"
	case vault.RedeemOutcomePlainMismatch:
		outcome = "plain-mismatch"
	case vault.RedeemOutcomeRedeemed:
		outcome = "redeemed"
	case vault.RedeemOutcomeAlreadyRedeemed:
		outcome = "already-redeemed"
	default:
		panic("unexpected outcome type")
	}
	made := strings.Join(p.calls, ",")
	fmt.Printf("%s\t%s\t%s\n", c, outcome, made)
	if outcome != expected || made != calls {
		fmt.Fprintf(os.Stderr, "%s: want %s %s\n", c, expected, calls)
		os.Exit(1)
	}
}
"#;

/// Runs every case of the built harness `binary`, each a separate process.
fn run_cases(binary: &Path, root: &Path) {
    for (case, method, redirect, verifier, refuse, expected, calls) in CASES {
        let output = Command::new(binary)
            .args([
                case,
                method,
                redirect,
                verifier,
                if refuse { "true" } else { "false" },
                expected,
                calls,
            ])
            .output()
            .expect("the harness runs");
        assert!(
            output.status.success(),
            "`{case}` failed; tree kept at {}:\n{}",
            root.display(),
            printed(&output)
        );
    }
}

#[test]
fn narrowed_external_runs_alike_on_the_generated_rust_target() {
    let ir = model();
    let synthesis = ess_synth::synthesize_for(&ir, Target::Rust).expect("the model synthesizes");
    assert_generated(&synthesis, Target::Rust);
    let root = fresh_root("rust");
    write(&synthesis, &root.join("demo"));
    let harness = root.join("harness");
    std::fs::create_dir_all(harness.join("src")).expect("mkdir");
    std::fs::write(
        harness.join("Cargo.toml"),
        "[package]\nname = \"narrowed-harness\"\nversion = \"0.0.0\"\nedition = \"2021\"\n\
         [dependencies]\ndemo-types = { path = \"../demo/crates/demo-types\" }\n[workspace]\n",
    )
    .expect("manifest");
    std::fs::write(harness.join("src/main.rs"), RUST_HARNESS).expect("harness");
    let target = root.join("target");
    let built = Command::new(std::env::var_os("CARGO").expect("cargo executable"))
        .args(["build", "--offline", "--quiet", "--target-dir"])
        .arg(&target)
        .current_dir(&harness)
        .env_remove("CARGO_TARGET_DIR")
        .env_remove("CARGO_ENCODED_RUSTFLAGS")
        .env_remove("RUSTC_WRAPPER")
        .env("RUSTFLAGS", "-D warnings")
        .env("CARGO_INCREMENTAL", "0")
        .output()
        .expect("cargo runs");
    assert!(
        built.status.success(),
        "the harness builds against the generated ports; tree kept at {}:\n{}",
        root.display(),
        printed(&built)
    );
    run_cases(&target.join("debug/narrowed-harness"), &root);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn narrowed_external_runs_alike_on_the_generated_go_target() {
    assert!(
        served::go_available(),
        "Go is required: the generated Go target is unchecked without it"
    );
    let ir = model();
    let synthesis = ess_synth::synthesize_for(&ir, Target::Go).expect("the model synthesizes");
    assert_generated(&synthesis, Target::Go);
    let root = fresh_root("go");
    write(&synthesis, &root.join("demo"));
    let harness = root.join("harness");
    std::fs::create_dir_all(&harness).expect("mkdir");
    std::fs::write(
        harness.join("go.mod"),
        "module narrowedharness\n\ngo 1.21\n\nrequire example.invalid/demo v0.0.0\n\nreplace \
         example.invalid/demo => ../demo\n",
    )
    .expect("Go manifest");
    std::fs::write(harness.join("main.go"), GO_HARNESS).expect("Go harness");
    let binary = root.join("go-harness");
    let built = Command::new("go")
        .args(["build", "-o"])
        .arg(&binary)
        .arg(".")
        .current_dir(&harness)
        .env("GOFLAGS", "-mod=mod -buildvcs=false")
        .env("GOPROXY", "off")
        .env("GOWORK", "off")
        .output()
        .expect("the Go toolchain runs");
    assert!(
        built.status.success(),
        "the Go harness builds against the generated ports; tree kept at {}:\n{}",
        root.display(),
        printed(&built)
    );
    run_cases(&binary, &root);
    let _ = std::fs::remove_dir_all(&root);
}
