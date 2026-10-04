//! An authored act whose expected outcome the command's own guards contradict (beyond10x/ess#222).
//!
//! `validate` reads the act's literal input against the command's `when:` guards under the
//! precedence order (`docs/design/input-guard-overlap-precedence.md`): input-guarded refusals
//! first, the first declared answering; accepting `when:` branches in declaration order; the
//! default only where no `when:` holds. An act expecting a branch that order decidedly does not
//! take — by `outcome:`, or by `error:` with no `outcome:` — is refused (`ESS-AUTHOR-041`). Where
//! the input does not decide it — a guard reading an `{$instance}`, the held state, a related row,
//! or an external answer — the act is accepted as before.

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::authored::{compile as compile_authored, Authoring, Source};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source as SpecSource;

/// `closed: open == false`, the default `reopened`, and `id-required: ticket_id == ""` declared
/// last and taken first (beyond10x/ess#178).
const TICKETS: &str = include_str!("fixtures/input-guard-overlap.yaml");
/// Three input-guarded refusals, a `wrong_state:` refusal and a default move.
const VAULT: &str = include_str!("fixtures/arrangement-input-refusal.yaml");
/// Two refusals decided by a related row, and a default.
const SIGN_IN: &str = include_str!("fixtures/related-guard-sign-in.yaml");
/// A `when:` beside `when_subject_state:` on three branches, and a default.
const CALLS: &str = include_str!("fixtures/subject-state.yaml");

/// An input refusal, an accepting `when:` declared before an external branch, and a default.
const COURIER: &str = r"
format: ess/16
system: courier
version: v1
domain: courier.mail
events:
  - {name: courier.mail.Kept, fields: []}
  - {name: courier.mail.Sent, fields: []}
errors:
  - {name: courier.mail.Throttled, fields: []}
  - {name: courier.mail.Undeliverable, fields: []}
commands:
  - name: courier.mail.SendMail
    input:
      - {name: recipient, type: String}
      - {name: attempt, type: Integer}
    outcomes:
      - {name: slowed, when: attempt > 10, error: courier.mail.Throttled}
      - {name: local, when: recipient == postmaster, emits: [courier.mail.Kept]}
      - {name: failed, external: the provider rejects the address, error: courier.mail.Undeliverable}
      - {name: sent, emits: [courier.mail.Sent]}
";

fn fixture(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("the fixture is well formed");
    let specification = Specification::assemble([(SpecSource::new("fixture.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the fixture validates:\n{errors}"));
    compile(&specification, &SourceMap::new())
        .unwrap_or_else(|diagnostics| panic!("the fixture resolves:\n{diagnostics}"))
}

fn authoring(ir: &EssIr, text: &str) -> Authoring {
    compile_authored(ir, &[Source::new("scenario.yaml", text)])
}

/// A document in `domain`, with `arrange` before a timeline of `acts`.
fn document(domain: &str, arrange: &str, acts: &str) -> String {
    format!(
        "type: ess-scenario/1\ndomain: {domain}\nscenario: a-scenario\n\
         summary: What this scenario proves, in one line.\n{arrange}timeline:\n{acts}"
    )
}

/// One act at second `at` sending `command` the `input` flow mapping, with `rest` under it.
fn act(at: u32, command: &str, input: &str, rest: &str) -> String {
    format!("  - at: 2026-01-05T09:00:{at:02}Z\n    command: {command}\n    input: {input}\n{rest}")
}

fn set_open(input: &str, rest: &str) -> String {
    document(
        "demo.tickets",
        "",
        &act(0, "demo.tickets.SetTicketOpen", input, rest),
    )
}

fn configure(input: &str, rest: &str) -> String {
    document(
        "vault.acct",
        "",
        &act(0, "vault.acct.Configure", input, rest),
    )
}

fn send_mail(input: &str, rest: &str) -> String {
    document(
        "courier.mail",
        "",
        &act(0, "courier.mail.SendMail", input, rest),
    )
}

/// Every refusal of `text`, as `(code, text)`.
fn refusals(ir: &EssIr, text: &str) -> Vec<(String, String)> {
    authoring(ir, text)
        .refusals
        .iter()
        .map(|refusal| (refusal.code().to_string(), refusal.to_string()))
        .collect()
}

/// The one refusal of `text`, which is `ESS-AUTHOR-041`; its text.
fn contradiction(ir: &EssIr, text: &str) -> String {
    let compiled = authoring(ir, text);
    assert!(
        compiled.scenarios.is_empty(),
        "a refused document produced a scenario"
    );
    let refused: Vec<(String, String)> = compiled
        .refusals
        .iter()
        .map(|refusal| (refusal.code().to_string(), refusal.to_string()))
        .collect();
    assert_eq!(refused.len(), 1, "exactly one refusal: {refused:?}");
    assert_eq!(refused[0].0, "ESS-AUTHOR-041", "{refused:?}");
    refused[0].1.clone()
}

fn accepted(ir: &EssIr, text: &str) {
    let compiled = authoring(ir, text);
    assert!(
        compiled.is_complete(),
        "accepted: {:?}",
        compiled
            .refusals
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
    );
    assert_eq!(compiled.scenarios.len(), 1);
}

fn names(text: &str, named: &[&str]) {
    for name in named {
        assert!(text.contains(name), "names {name}: {text}");
    }
}

#[test]
fn an_act_expecting_an_accepting_branch_an_input_refusal_answers_first_is_refused() {
    let ir = fixture(TICKETS);
    let text = contradiction(
        &ir,
        &set_open("{ticket_id: '', open: false}", "    outcome: closed\n"),
    );
    names(
        &text,
        &[
            "`demo.tickets.SetTicketOpen`",
            "`closed`",
            "`id-required`",
            "ticket_id == \"\"",
            "open: false",
        ],
    );
}

#[test]
fn the_refusal_names_the_act_the_branch_taken_first_its_guard_and_the_repair() {
    let ir = fixture(TICKETS);
    let text = contradiction(
        &ir,
        &set_open("{ticket_id: '', open: false}", "    outcome: closed\n"),
    );
    assert_eq!(
        text,
        "refusal[ESS-AUTHOR-041]: `demo.tickets/authored/a-scenario` in scenario.yaml\n  \
         `demo.tickets.SetTicketOpen` is expected to answer `closed` for {open: false, \
         ticket_id: \"\"}, which its guards decide otherwise: `id-required` (ticket_id == \"\") \
         answers it first, so `closed` is not taken\n  \
         help: send an input the expected branch's `when:` admits and no branch answered before \
         it claims, or expect the branch that input takes: input-guarded refusals answer first, \
         the first declared of them, then accepting `when:` and external branches in declaration \
         order, and the default only where no `when:` holds"
    );
}

#[test]
fn an_act_expecting_a_branch_its_own_guard_refutes_is_refused() {
    let ir = fixture(TICKETS);
    let text = contradiction(
        &ir,
        &set_open("{ticket_id: t-1, open: true}", "    outcome: closed\n"),
    );
    names(&text, &["`closed`", "open == false", "open: true"]);
}

#[test]
fn an_act_expecting_the_default_where_a_guarded_branch_holds_is_refused() {
    let ir = fixture(TICKETS);
    let accepting = contradiction(
        &ir,
        &set_open("{ticket_id: t-1, open: false}", "    outcome: reopened\n"),
    );
    names(&accepting, &["`reopened`", "`closed`", "open == false"]);
    let refusing = contradiction(
        &ir,
        &set_open("{ticket_id: '', open: true}", "    outcome: reopened\n"),
    );
    names(&refusing, &["`reopened`", "`id-required`"]);
}

#[test]
fn an_act_whose_expected_outcome_its_input_selects_is_accepted() {
    let ir = fixture(TICKETS);
    for (input, outcome) in [
        ("{ticket_id: '', open: false}", "id-required"),
        ("{ticket_id: '', open: true}", "id-required"),
        ("{ticket_id: t-1, open: false}", "closed"),
        ("{ticket_id: t-1, open: true}", "reopened"),
    ] {
        accepted(&ir, &set_open(input, &format!("    outcome: {outcome}\n")));
    }
}

#[test]
fn an_error_claim_without_an_outcome_is_held_to_the_branches_that_report_it() {
    let ir = fixture(TICKETS);
    let claim = "    error: {name: demo.tickets.TicketIdRequired}\n";
    let text = contradiction(&ir, &set_open("{ticket_id: t-1, open: false}", claim));
    names(
        &text,
        &["error `demo.tickets.TicketIdRequired`", "`id-required`"],
    );
    accepted(&ir, &set_open("{ticket_id: '', open: false}", claim));
}

#[test]
fn of_two_input_refusals_the_first_declared_answers() {
    let ir = fixture(VAULT);
    let text = contradiction(
        &ir,
        &configure(
            "{id: '', secret: long-enough-secret, issuer: ''}",
            "    outcome: missing-configuration\n",
        ),
    );
    names(
        &text,
        &["`missing-configuration`", "`id-required`", "id == \"\""],
    );
    accepted(
        &ir,
        &configure(
            "{id: '', secret: long-enough-secret, issuer: ''}",
            "    outcome: id-required\n",
        ),
    );
}

#[test]
fn an_input_refusal_answers_before_the_held_state_and_the_state_is_not_guessed() {
    let ir = fixture(VAULT);
    let text = contradiction(
        &ir,
        &configure(
            "{id: a-1, secret: long-enough-secret, issuer: ''}",
            "    outcome: already-configured\n",
        ),
    );
    names(&text, &["`already-configured`", "`missing-configuration`"]);
    for outcome in ["already-configured", "configured"] {
        accepted(
            &ir,
            &configure(
                "{id: a-1, secret: long-enough-secret, issuer: acme}",
                &format!("    outcome: {outcome}\n"),
            ),
        );
    }
    accepted(
        &ir,
        &configure(
            "{id: a-1, secret: long-enough-secret, issuer: acme}",
            "    error: {name: vault.acct.AlreadyConfigured}\n",
        ),
    );
}

#[test]
fn a_branch_beside_a_held_state_is_held_to_its_input_guard_and_not_to_the_state() {
    let ir = fixture(CALLS);
    let call = "00000000-0000-4000-8000-000000000001";
    let report = |incoming: &str, outcome: &str| {
        document(
            "calls.core",
            "",
            &act(
                0,
                "calls.core.Report",
                &format!("{{call_id: {call}, incoming: {incoming}, note: n}}"),
                &format!("    outcome: {outcome}\n"),
            ),
        )
    };
    let text = contradiction(&ir, &report("Unspecified", "ringing"));
    names(&text, &["`ringing`", "incoming == Ringing"]);
    // Which state the row holds is the arrangement's to say, not the input's.
    for outcome in ["ringing", "refreshed", "preserved", "enriched"] {
        accepted(&ir, &report("Ringing", outcome));
    }
    accepted(&ir, &report("Unspecified", "enriched"));
}

#[test]
fn a_branch_a_related_row_decides_is_not_guessed() {
    let ir = fixture(SIGN_IN);
    for outcome in ["no-configuration", "no-redirect-entry", "initiated"] {
        accepted(
            &ir,
            &document(
                "demo.signin",
                "",
                &act(
                    0,
                    "demo.signin.InitiateSignIn",
                    "{tenant: 00000000-0000-4000-8000-000000000001, client: web}",
                    &format!("    outcome: {outcome}\n"),
                ),
            ),
        );
    }
}

#[test]
fn an_external_branch_is_held_to_what_answers_before_its_provider_and_no_further() {
    let ir = fixture(COURIER);
    let refused = contradiction(
        &ir,
        &send_mail("{recipient: someone, attempt: 11}", "    outcome: failed\n"),
    );
    names(&refused, &["`failed`", "`slowed`", "attempt > 10"]);
    let preceded = contradiction(
        &ir,
        &send_mail(
            "{recipient: postmaster, attempt: 1}",
            "    outcome: failed\n",
        ),
    );
    names(&preceded, &["`failed`", "`local`"]);
    // Whether the provider rejects the address is its answer, not the input's.
    accepted(
        &ir,
        &send_mail("{recipient: someone, attempt: 1}", "    outcome: failed\n"),
    );
    accepted(
        &ir,
        &send_mail(
            "{recipient: someone, attempt: 1}",
            "    outcome: failed\n    error: {name: courier.mail.Undeliverable}\n",
        ),
    );
    accepted(
        &ir,
        &send_mail("{recipient: someone, attempt: 1}", "    outcome: sent\n"),
    );
}

/// Two input refusals over an enum and an optional's presence, and a default (beyond10x/ess#280).
const QUOTA: &str = r"
format: ess/19
system: mini
version: v1
domain: mini.q
types:
  - {name: mini.q.QuotaId, kind: newtype, of: Uuid}
  - {name: mini.q.Scope, kind: enum, variants: [Budget, Provider]}
entities:
  - name: mini.q.Quota
    identity: {name: quota_id, type: mini.q.QuotaId}
    fields:
      - {name: scope, type: mini.q.Scope}
      - {name: provider, type: Optional<String>}
    lifecycle: {initial: Active, states: [Active], terminal: [Active]}
errors:
  - {name: mini.q.InvalidScope, fields: []}
events:
  - name: mini.q.QuotaSet
    fields:
      - {name: quota_id, type: mini.q.QuotaId}
commands:
  - name: mini.q.SetQuota
    input:
      - {name: scope, type: mini.q.Scope}
      - {name: provider, type: Optional<String>}
    outcomes:
      - {name: provider-not-allowed, when: {scope: Budget, provider: {exists: true}}, error: mini.q.InvalidScope}
      - {name: provider-missing, when: {scope: Provider, provider: {exists: false}}, error: mini.q.InvalidScope}
      - name: set
        creates: mini.q.Quota
        instance: quota_id
        sets: {scope: input.scope, provider: input.provider}
        emits: [mini.q.QuotaSet]
        payload:
          mini.q.QuotaSet: {quota_id: {generated: true}}
";

#[test]
fn an_optional_left_out_of_the_act_is_absent_and_decides_a_presence_guard() {
    let ir = fixture(QUOTA);
    let quota = |input: &str, outcome: &str| {
        document(
            "mini.q",
            "",
            &act(
                0,
                "mini.q.SetQuota",
                input,
                &format!("    outcome: {outcome}\n"),
            ),
        )
    };
    accepted(&ir, &quota("{scope: Budget}", "set"));
    accepted(&ir, &quota("{scope: Provider, provider: p-1}", "set"));
    accepted(&ir, &quota("{scope: Provider}", "provider-missing"));
    let present = contradiction(&ir, &quota("{scope: Budget, provider: p-1}", "set"));
    names(&present, &["`set`", "`provider-not-allowed`"]);
    let absent = contradiction(&ir, &quota("{scope: Provider}", "set"));
    names(&absent, &["`set`", "`provider-missing`"]);
}

/// `OpenTicket` creates the ticket `t`, captured from its event.
const OPENED: &str = "  - at: 2026-01-05T09:00:00Z\n    command: demo.tickets.OpenTicket\n    \
     input: {}\n    outcome: opened\n    events:\n      - event: demo.tickets.TicketOpened\n    \
     capture: {instance: t, event: demo.tickets.TicketOpened, field: ticket_id}\n";

fn after_opening(input: &str, outcome: &str) -> String {
    document(
        "demo.tickets",
        "arrange:\n  - instance: t\n    entity: demo.tickets.Ticket\n",
        &format!(
            "{OPENED}{}",
            act(
                1,
                "demo.tickets.SetTicketOpen",
                input,
                &format!("    outcome: {outcome}\n")
            )
        ),
    )
}

#[test]
fn a_guard_reading_an_instance_reference_is_not_decided() {
    let ir = fixture(TICKETS);
    // `id-required` reads `ticket_id`, which an `{$instance}` hides: neither branch is refused.
    accepted(
        &ir,
        &after_opening("{ticket_id: {$instance: t}, open: false}", "closed"),
    );
    accepted(
        &ir,
        &after_opening("{ticket_id: {$instance: t}, open: false}", "id-required"),
    );
    // `closed` reads only `open`, which the act sends literally.
    let text = contradiction(
        &ir,
        &after_opening("{ticket_id: {$instance: t}, open: true}", "closed"),
    );
    names(
        &text,
        &["`closed`", "open == false", "ticket_id: <reference>"],
    );
}

#[test]
fn a_contradiction_is_one_refusal_beside_the_others_of_its_file() {
    let ir = fixture(TICKETS);
    let text = document(
        "demo.tickets",
        "",
        &format!(
            "{}{}",
            act(
                0,
                "demo.tickets.SetTicketOpen",
                "{ticket_id: '', open: false}",
                "    outcome: closed\n"
            ),
            act(
                1,
                "demo.tickets.SetTicketOpen",
                "{ticket_id: t-1, open: false}",
                "    outcome: nowhere\n"
            ),
        ),
    );
    let codes: Vec<String> = refusals(&ir, &text)
        .into_iter()
        .map(|(code, _)| code)
        .collect();
    assert_eq!(codes, vec!["ESS-AUTHOR-041", "ESS-AUTHOR-007"]);
}

#[test]
fn a_coverage_inventory_carries_the_refusal_and_is_admitted() {
    use ess_conformance::{
        coverage::{Origins, Scope},
        coverage_build::{build, CoverageSource},
    };
    let ir = fixture(TICKETS);
    let refused = set_open("{ticket_id: '', open: false}", "    outcome: closed\n");
    let fine = set_open("{ticket_id: t-1, open: false}", "    outcome: closed\n")
        .replace("a-scenario", "b-scenario");
    let input = build(
        &ir,
        &[
            CoverageSource::new("a.yaml", refused).unwrap(),
            CoverageSource::new("b.yaml", fine).unwrap(),
        ],
        Scope::System,
        Origins::Authored,
    )
    .expect("an inventory with an ESS-AUTHOR-041 refusal is admitted");
    let inventory = input.selected().coverage().unwrap();
    let codes: Vec<&str> = inventory.refused.iter().map(|r| r.code.as_str()).collect();
    assert_eq!(codes, vec!["ESS-AUTHOR-041"]);
    ess_conformance::AdmittedSuite::from_json(input.selected().original_json())
        .expect("the written inventory reads back");
}

/// `related-guard-sign-in.yaml` with an input-guarded refusal over `region` declared before the
/// related branches.
fn sign_in_with_a_region_refusal() -> String {
    SIGN_IN
        .replace(
            "errors:\n",
            "errors:\n  - {name: demo.signin.RegionClosed, fields: []}\n",
        )
        .replace(
            "      - {name: client, type: demo.signin.ClientId}\n    outcomes:\n",
            "      - {name: client, type: demo.signin.ClientId}\n      - {name: region, type: String}\n    \
             outcomes:\n      - {name: region-closed, when: region == north, error: demo.signin.RegionClosed}\n",
        )
}

#[test]
fn a_missing_related_row_answers_before_the_input_refusal_its_input_selects() {
    let ir = fixture(&sign_in_with_a_region_refusal());
    let sign_in = |outcome: &str| {
        document(
            "demo.signin",
            "",
            &act(
                0,
                "demo.signin.InitiateSignIn",
                "{tenant: 00000000-0000-4000-8000-000000000001, client: web, region: north}",
                &format!("    outcome: {outcome}\n"),
            ),
        )
    };
    // Step 1 of the precedence order: `exists: false` is read before any input refusal.
    accepted(&ir, &sign_in("no-configuration"));
    // The predicate refusal on a present row comes after the input refusal, as does the default.
    for outcome in ["no-redirect-entry", "initiated"] {
        let text = contradiction(&ir, &sign_in(outcome));
        names(&text, &[&format!("`{outcome}`"), "`region-closed`"]);
    }
}

/// An admitted coverage input whose only refusal is `ESS-AUTHOR-041`.
fn inventory_with_the_refusal() -> ess_conformance::coverage::AdmittedInput {
    use ess_conformance::{
        coverage::{Origins, Scope},
        coverage_build::{build, CoverageSource},
    };
    let ir = fixture(TICKETS);
    let refused = set_open("{ticket_id: '', open: false}", "    outcome: closed\n");
    let fine = set_open("{ticket_id: t-1, open: false}", "    outcome: closed\n")
        .replace("a-scenario", "b-scenario");
    build(
        &ir,
        &[
            CoverageSource::new("a.yaml", refused).unwrap(),
            CoverageSource::new("b.yaml", fine).unwrap(),
        ],
        Scope::System,
        Origins::Authored,
    )
    .expect("an inventory with an ESS-AUTHOR-041 refusal is admitted")
}

fn write_all(directory: &std::path::Path, artifacts: impl IntoIterator<Item = (String, String)>) {
    for (path, contents) in artifacts {
        let path = directory.join(path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, contents).unwrap();
    }
}

fn succeeded(output: &std::process::Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

/// The Rust, Go and TypeScript coverage readers each admit an input carrying `ESS-AUTHOR-041` and
/// refuse one carrying `ESS-AUTHOR-042`, a code no cause numbers.
#[test]
fn every_coverage_reader_admits_the_refusal_and_no_code_after_it() {
    use std::process::Command;
    let input = inventory_with_the_refusal();
    let admitted = input.document().to_canonical_json().unwrap();
    assert!(admitted.contains("ESS-AUTHOR-041"));
    let unknown = admitted.replace("ESS-AUTHOR-041", "ESS-AUTHOR-042");
    assert!(ess_conformance::coverage::AdmittedInput::from_json(&admitted).is_ok());
    assert!(ess_conformance::coverage::AdmittedInput::from_json(&unknown).is_err());

    let root = std::path::Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("authored-041-readers-{}", std::process::id()));

    let go = root.join("go");
    write_all(
        &go,
        ess_conformance::go::emit_input(&input)
            .unwrap()
            .into_iter()
            .map(|artifact| (artifact.path, artifact.contents)),
    );
    std::fs::write(go.join("go.mod"), "module authored041\n\ngo 1.24\n").unwrap();
    let package = go.join("essconform");
    std::fs::write(package.join("admitted.txt"), &admitted).unwrap();
    std::fs::write(package.join("unknown.txt"), &unknown).unwrap();
    std::fs::write(
        package.join("authored_041_test.go"),
        r#"package essconform

import (
    "os"
    "testing"
)

func TestAuthored041(t *testing.T) {
    for file, want := range map[string]bool{"admitted.txt": true, "unknown.txt": false} {
        raw, err := os.ReadFile(file)
        if err != nil { t.Fatal(err) }
        _, err = admitRunInput(string(raw))
        if (err == nil) != want { t.Fatalf("%s: admitted=%v want=%v: %v", file, err == nil, want, err) }
    }
}
"#,
    )
    .unwrap();
    let output = Command::new("go")
        .args(["test", "-count=1", "-run", "TestAuthored041", "./..."])
        .current_dir(&go)
        .output()
        .expect("go is required for the Go reader");
    assert!(output.status.success(), "Go: {}", succeeded(&output));

    let ts = root.join("ts");
    write_all(
        &ts,
        ess_conformance::ts::emit_input(&input)
            .unwrap()
            .into_iter()
            .map(|artifact| (artifact.path, artifact.contents)),
    );
    let package = ts.join("essconform");
    let mut compile = Command::new("tsc");
    if let Some(modules) = std::env::var_os("ESS_TYPES_NODE") {
        compile
            .arg("--typeRoots")
            .arg(std::path::Path::new(&modules).join("@types"));
    }
    let output = compile
        .args(["--project", "tsconfig.json", "--noCheck"])
        .current_dir(&package)
        .output()
        .expect("tsc is required for the TypeScript reader");
    assert!(output.status.success(), "tsc: {}", succeeded(&output));
    std::fs::write(package.join("admitted.txt"), &admitted).unwrap();
    std::fs::write(package.join("unknown.txt"), &unknown).unwrap();
    std::fs::write(
        package.join("authored041.mjs"),
        r"import {readFileSync} from 'node:fs';
import {admitRunInput} from './dist/runtime.js';
const read = (file) => { try { admitRunInput(readFileSync(file, 'utf8')); return true; } catch { return false; } };
const result = {admitted: read('admitted.txt'), unknown: read('unknown.txt')};
console.log(JSON.stringify(result));
if (!result.admitted || result.unknown) process.exitCode = 1;
",
    )
    .unwrap();
    let output = Command::new("node")
        .arg("authored041.mjs")
        .current_dir(&package)
        .output()
        .expect("node is required for the TypeScript reader");
    assert!(
        output.status.success(),
        "TypeScript: {}",
        succeeded(&output)
    );
    std::fs::remove_dir_all(&root).ok();
}

/// A refusal over a start already past, and one over a start within an hour of the current time,
/// which no single moment's reading decides for every run.
const CLOCK: &str = r#"
format: ess/16
system: clock
version: v1
domain: clock.c
errors:
  - {name: clock.c.Refused, fields: []}
events:
  - {name: clock.c.Done, fields: []}
commands:
  - name: clock.c.Book
    input:
      - {name: at, type: Timestamp}
    outcomes:
      - {name: past, when: at < now - 60s, error: clock.c.Refused}
      - {name: booked, emits: [clock.c.Done]}
  - name: clock.c.Hold
    input:
      - {name: at, type: Timestamp}
    outcomes:
      - {name: near, when: {all: ["at > now - 1h", "at < now + 1h"]}, error: clock.c.Refused}
      - {name: held, emits: [clock.c.Done]}
"#;

fn clocked(command: &str, at: &str, outcome: &str) -> String {
    document(
        "clock.c",
        "",
        &act(
            0,
            command,
            &format!("{{at: '{at}'}}"),
            &format!("    outcome: {outcome}\n"),
        ),
    )
}

#[test]
fn a_guard_over_the_current_time_is_decided_only_where_every_run_reads_it_alike() {
    let ir = fixture(CLOCK);
    // Past at every run since the operand exists: the refusal answers first.
    let text = contradiction(
        &ir,
        &clocked("clock.c.Book", "2025-01-01T00:00:00Z", "booked"),
    );
    names(&text, &["`booked`", "`past`"]);
    accepted(
        &ir,
        &clocked("clock.c.Book", "2025-01-01T00:00:00Z", "past"),
    );
    // Ahead of the earliest run and past later: which one a target sees is its clock's to say.
    for outcome in ["past", "booked"] {
        accepted(
            &ir,
            &clocked("clock.c.Book", "2026-12-01T00:00:00Z", outcome),
        );
    }
    // Within an hour of `now` at some run and not at the first or the last: not decided.
    for outcome in ["near", "held"] {
        accepted(
            &ir,
            &clocked("clock.c.Hold", "2030-01-01T00:00:00Z", outcome),
        );
    }
    // Never within an hour of any run: decided.
    let text = contradiction(
        &ir,
        &clocked("clock.c.Hold", "2020-01-01T00:00:00Z", "near"),
    );
    names(&text, &["`near`", "refutes it"]);
}
