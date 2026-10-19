//! A branch the held state selects is declared before every accepting branch of its command, and
//! Entity Runtime then answers as the precedence order says (beyond10x/ess#486).
//!
//! The lowering orders every non-refusal guarded branch by declaration, so with `rushed: when:
//! rush == true` declared before `unreminded: when_subject: {predicate: reminder_count == 0}`, a
//! `Draft` invoice with no reminder sent `rush: true` was answered `rushed`, where the design's
//! order (`docs/design/cross-record-and-stored-field-guards.md`, step 4 before step 6) and the Rust
//! and Go targets answer `unreminded`. Validation refuses that declaration order; declared the
//! other way round, Entity Runtime answers `unreminded`.
//!
//! The lowering takes its branch order from the command's precedence plan
//! (`docs/design/selection-plan.md`, `story:entity-runtime-lowering-reads-selection-plan`): with
//! the held-state and accepting phases exchanged through the plan's one test seam, the lowered
//! order and Entity Runtime's answers exchange with them, and the default and the wrong-state
//! branch keep the places entity-core requires of them in every phase order.
//!
//! A held-state branch may still be declared after an accepting branch whose input guard the finite
//! prover shows disjoint from its own: the epic's one named exception. The plan lowers it first
//! where the base lowering kept declaration order, so the bytes move and, the guards never holding
//! together, the answers do not (`tests/fixtures/held-state-after-disjoint-accepting.yaml`).
use std::collections::BTreeMap;
use std::num::NonZeroU32;
use std::path::Path;

use entity_core::{
    identity, EntityInstance, Evaluation, FieldKind, LoadedDecision, PreloadDecision, Registry,
    Runtime,
};
use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile_locating;
use ess_compiler::source::SourceMap;
use ess_domain::command::precedence::{with_phase_order, Phase};
use ess_domain::component::ComponentName;
use ess_domain::name::QualifiedName;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_entity_runtime::{lower, LoweredService, LoweringOptions};
use ess_service_contract::{extract, ServiceIr};
use ess_synth::SynthesisPlan;
use serde_json::{json, Value};

const CANCEL: &str = "    input:\n      - name: invoice_id\n        type: billing.invoice.InvoiceId\n\n    outcomes:\n      - name: cancelled\n        moves: billing.invoice.Invoice.cancel\n";
const INPUT: &str = "    input:\n      - name: invoice_id\n        type: billing.invoice.InvoiceId\n      - name: rush\n        type: Boolean\n\n    outcomes:\n";
const RUSHED: &str = "      - name: rushed\n        when: rush == true\n        moves: billing.invoice.Invoice.cancel\n        instance: invoice_id\n        emits:\n          - billing.invoice.InvoiceCancelled\n        payload:\n          billing.invoice.InvoiceCancelled:\n            invoice_id: input.invoice_id\n";
const UNREMINDED: &str = "      - name: unreminded\n        when_subject:\n          predicate: reminder_count == 0\n        moves: billing.invoice.Invoice.cancel\n        instance: invoice_id\n        emits:\n          - billing.invoice.InvoiceCancelled\n        payload:\n          billing.invoice.InvoiceCancelled:\n            invoice_id: input.invoice_id\n";
const DEFAULT: &str = "      - name: cancelled\n        moves: billing.invoice.Invoice.cancel\n";

/// The billing example at `ess/20`, with `CancelInvoice` declaring `branches` before its default.
fn assembled(branches: &[&str]) -> (Result<Specification, String>, SourceMap, Vec<String>) {
    let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../examples/billing");
    let cancel = format!("{INPUT}{}{DEFAULT}", branches.concat());
    let changes: [(&str, &str, &str); 4] = [
        ("system.yaml", "format: ess/1\n", "format: ess/20\n"),
        (
            "domains/email.yaml",
            "            recipient: input.recipient\n",
            "            recipient: input.recipient\n            message_id: {generated: true}\n",
        ),
        (
            "domains/invoice.yaml",
            "          billing.invoice.InvoiceCreated:\n",
            "          billing.invoice.InvoiceCreated:\n            invoice_id: {generated: true}\n",
        ),
        ("domains/invoice.yaml", CANCEL, &cancel),
    ];
    let mut pending = vec![base.clone()];
    let mut paths = Vec::new();
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).expect("fixture directory is readable") {
            let path = entry.expect("fixture entry").path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|it| it == "yaml") {
                paths.push(path);
            }
        }
    }
    paths.sort();
    let mut parsed = Vec::new();
    let mut labels = Vec::new();
    let mut sources = SourceMap::new();
    for path in paths {
        let label = path.strip_prefix(&base).unwrap().display().to_string();
        let mut text = std::fs::read_to_string(&path).unwrap();
        for (target, before, after) in &changes {
            if label == *target {
                assert!(text.contains(before), "{label}: `{before}` is present");
                text = text.replacen(before, after, 1);
            }
        }
        sources.insert(label.clone(), text.clone());
        parsed.push((
            Source::new(label.clone()),
            RawSpecFile::parse(&text).unwrap(),
        ));
        labels.push(label);
    }
    (
        Specification::assemble(parsed).map_err(|errors| errors.to_string()),
        sources,
        labels,
    )
}

fn compiled(branches: &[&str]) -> EssIr {
    let (specification, sources, labels) = assembled(branches);
    let specification = specification.unwrap_or_else(|errors| panic!("{errors}"));
    compile_locating(&specification, &sources, &labels).expect("fixture compiles")
}

fn registry_of(ir: &EssIr) -> Registry {
    let plan = SynthesisPlan::of(ir);
    let service = extract(ir, &plan, &ComponentName::new("invoice-service").unwrap())
        .expect("service extracts");
    registry(&lowered(&service))
}

fn lowered(service: &ServiceIr<'_>) -> LoweredService {
    lower(
        service,
        &LoweringOptions {
            definition_versions: ["billing.invoice.Account", "billing.invoice.Invoice"]
                .iter()
                .map(|entry| (QualifiedName::new(entry).unwrap(), NonZeroU32::MIN))
                .collect(),
            scales: BTreeMap::new(),
        },
    )
    .unwrap_or_else(|diagnostics| panic!("{diagnostics:?}"))
}

/// `CancelInvoice`'s branches in their lowered order.
fn lowered_order(lowered: &LoweredService) -> Vec<String> {
    lowered.definitions()[&QualifiedName::new("billing.invoice.Invoice").unwrap()]
        .as_definition()
        .operations["billing.invoice.CancelInvoice"]
        .outcomes
        .iter()
        .map(|outcome| outcome.name.clone())
        .collect()
}

fn registry(lowered: &LoweredService) -> Registry {
    let mut registry = Registry::new();
    for definition in lowered.definitions().values() {
        registry
            .register(definition.as_definition().clone())
            .expect("definition registers");
    }
    registry.validate_all().expect("closure validates");
    registry
}

const INVOICE: &str = "b404a1e8-9360-4af5-a0ac-8a483adfa225";

fn invoice(state: &str, reminders: u32) -> EntityInstance {
    let fields: serde_json::Map<String, Value> = serde_json::from_value(json!({
        "invoice_id": INVOICE,
        "account_id": "ae861ea5-96d8-48cf-a843-c9a0baf11389",
        "total": {"amount": 25, "currency": "EUR"},
        "payee": {"kind": "person", "value": "buyer@example.test"},
        "channel": "Email",
        "lines": [],
        "metadata": {},
        "settlement_window": "PT1H",
        "is_recurring": false,
        "signature": "AA==",
        "reminder_count": reminders
    }))
    .expect("invoice fields");
    EntityInstance {
        entity: "billing.invoice.Invoice".to_owned(),
        version: 1,
        id: identity::address(FieldKind::String, &json!(INVOICE)).expect("identity address"),
        lifecycle_state: state.to_owned(),
        revision: 1,
        fields,
    }
}

/// Which branch `CancelInvoice` takes with `rush` on a `Draft` invoice holding `reminders`.
fn taken(registry: &Registry, rush: bool, reminders: u32) -> String {
    let runtime = Runtime::new(registry);
    let row = invoice("Draft", reminders);
    let arguments = json!({"input": {"invoice_id": INVOICE, "rush": rush}, "bound": {}});
    match runtime.decide_before_load(
        &row.entity,
        row.version,
        row.id.clone(),
        "billing.invoice.CancelInvoice",
        arguments,
    ) {
        Ok(PreloadDecision::Refused(refusal)) => format!("before load: {}", refusal.outcome),
        Ok(PreloadDecision::Load(prepared)) => match prepared.select_with(&row) {
            Ok(LoadedDecision::NeedsFulfillment(prepared)) => prepared.outcome().to_string(),
            Ok(LoadedDecision::Complete(evaluation)) => match evaluation.into_decision() {
                Ok(_) => "accepted".to_owned(),
                Err(refusal) => format!("refused {refusal:?}"),
            },
            Err(error) => format!("error {error:?}"),
        },
        Err(error) => format!("error before load: {error:?}"),
    }
}

#[test]
fn an_accepting_branch_declared_before_a_held_state_branch_is_refused() {
    let (specification, _, _) = assembled(&[RUSHED, UNREMINDED]);
    let errors = specification.map_or_else(|errors| errors, |_| panic!("the model validates"));
    assert!(
        errors.contains(
            "command.billing.invoice.CancelInvoice.outcomes.unreminded: `unreminded` is selected \
             by the held state, which answers before the accepting branch `rushed` declared above \
             it"
        ),
        "{errors}"
    );
}

#[test]
fn declared_first_the_held_state_branch_answers_on_entity_runtime() {
    let registry = registry_of(&compiled(&[UNREMINDED, RUSHED]));
    assert_eq!(taken(&registry, true, 0), "unreminded");
    assert_eq!(taken(&registry, false, 0), "unreminded");
    assert_eq!(taken(&registry, true, 1), "rushed");
    assert_eq!(taken(&registry, false, 1), "cancelled");
}

/// The precedence order with the held-state phase (step 4) and the accepting phase (step 6)
/// exchanged.
const EXCHANGED: [Phase; 8] = [
    Phase::InputAbsent,
    Phase::RelatedRow,
    Phase::InputRefusal,
    Phase::Existence,
    Phase::Accepting,
    Phase::PresentRelated,
    Phase::HeldState,
    Phase::Default,
];

#[test]
fn exchanged_held_state_and_accepting_phases_exchange_the_lowered_order_on_entity_runtime() {
    let ir = compiled(&[UNREMINDED, RUSHED]);
    let plan = SynthesisPlan::of(&ir);
    let service = extract(&ir, &plan, &ComponentName::new("invoice-service").unwrap())
        .expect("service extracts");

    let declared = lowered(&service);
    assert_eq!(
        lowered_order(&declared),
        ["unreminded", "rushed", "cancelled", "wrong-state"]
    );

    let exchanged = with_phase_order(EXCHANGED, || lowered(&service));
    assert_eq!(
        lowered_order(&exchanged),
        ["rushed", "unreminded", "cancelled", "wrong-state"]
    );
    let registry = registry(&exchanged);
    assert_eq!(taken(&registry, true, 0), "rushed");
    assert_eq!(taken(&registry, false, 0), "unreminded");
    assert_eq!(taken(&registry, true, 1), "rushed");
    assert_eq!(taken(&registry, false, 1), "cancelled");
}

/// The plan read backwards puts the default first and the wrong-state branch (step 4) before the
/// default; entity-core requires the default last among the branches it selects and drops the
/// wrong-state branch from selection, so the lowering keeps both after every other branch.
#[test]
fn the_default_and_the_wrong_state_branch_stay_last_in_any_phase_order() {
    let ir = compiled(&[UNREMINDED, RUSHED]);
    let plan = SynthesisPlan::of(&ir);
    let service = extract(&ir, &plan, &ComponentName::new("invoice-service").unwrap())
        .expect("service extracts");
    let mut reversed = Phase::PRECEDENCE;
    reversed.reverse();

    let lowered = with_phase_order(reversed, || lowered(&service));
    assert_eq!(
        lowered_order(&lowered),
        ["rushed", "unreminded", "cancelled", "wrong-state"]
    );
    let registry = registry(&lowered);
    assert_eq!(taken(&registry, true, 0), "rushed");
    assert_eq!(taken(&registry, false, 0), "unreminded");
    assert_eq!(taken(&registry, false, 1), "cancelled");
}

/// `tests/fixtures/held-state-after-disjoint-accepting.yaml`, whose `demo.ticket.Close` declares
/// the held-state branch `first-close` (`when: rush == false`) after the accepting `rushed`
/// (`when: rush == true`).
fn ticket_model() -> EssIr {
    let label = "held-state-after-disjoint-accepting.yaml";
    let text = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(label),
    )
    .expect("fixture is readable");
    let mut sources = SourceMap::new();
    sources.insert(label.to_owned(), text.clone());
    let specification = Specification::assemble(vec![(
        Source::new(label),
        RawSpecFile::parse(&text).unwrap(),
    )])
    .unwrap_or_else(|errors| panic!("the fixture validates: {errors}"));
    compile_locating(&specification, &sources, &[label.to_owned()]).expect("fixture compiles")
}

fn ticket_lowered(service: &ServiceIr<'_>) -> LoweredService {
    lower(
        service,
        &LoweringOptions {
            definition_versions: BTreeMap::from([(
                QualifiedName::new("demo.ticket.Ticket").unwrap(),
                NonZeroU32::MIN,
            )]),
            scales: BTreeMap::new(),
        },
    )
    .unwrap_or_else(|diagnostics| panic!("{diagnostics:?}"))
}

/// `demo.ticket.Close`'s branches in their lowered order.
fn close_order(lowered: &LoweredService) -> Vec<String> {
    lowered.definitions()[&QualifiedName::new("demo.ticket.Ticket").unwrap()]
        .as_definition()
        .operations["demo.ticket.Close"]
        .outcomes
        .iter()
        .map(|outcome| outcome.name.clone())
        .collect()
}

const TICKET: &str = "5d3f0c1e-8a4b-4e2f-9b6a-1c7d2e3f4a5b";

/// Which branch `demo.ticket.Close` takes with `note` and `rush` on a ticket in `state` reopened
/// `reopened` times.
fn closed_by(registry: &Registry, state: &str, reopened: u32, note: &str, rush: bool) -> String {
    let runtime = Runtime::new(registry);
    let row = EntityInstance {
        entity: "demo.ticket.Ticket".to_owned(),
        version: 1,
        id: identity::address(FieldKind::String, &json!(TICKET)).expect("identity address"),
        lifecycle_state: state.to_owned(),
        revision: 1,
        fields: serde_json::from_value(json!({
            "ticket_id": TICKET, "note": "", "reopened": reopened
        }))
        .expect("ticket fields"),
    };
    let arguments =
        json!({"input": {"ticket_id": TICKET, "note": note, "rush": rush}, "bound": {}});
    match runtime.decide_before_load(
        &row.entity,
        row.version,
        row.id.clone(),
        "demo.ticket.Close",
        arguments,
    ) {
        Ok(PreloadDecision::Refused(refusal)) => format!("before load: {}", refusal.outcome),
        Ok(PreloadDecision::Load(prepared)) => match prepared.select_with(&row) {
            Ok(LoadedDecision::NeedsFulfillment(prepared)) => prepared.outcome().to_string(),
            Ok(LoadedDecision::Complete(Evaluation::Accepted(decision))) => {
                format!("accepted {:?}", decision.record.outcome)
            }
            Ok(LoadedDecision::Complete(Evaluation::Refused(refusal))) => {
                format!("refused {}", refusal.outcome)
            }
            Err(error) => format!("error {error:?}"),
        },
        Err(error) => format!("error before load: {error:?}"),
    }
}

/// The epic's named exception, pinned in the lowered-definitions table: the plan lowers
/// `first-close` before `rushed`, where the base lowering kept their declaration order, and the
/// two orders answer every request alike. Exchanging the held-state and accepting phases gives the
/// base lowering's order for this command.
#[test]
fn a_held_state_branch_after_a_disjoint_accepting_one_moves_bytes_and_no_answer() {
    let ir = ticket_model();
    let plan = SynthesisPlan::of(&ir);
    let service =
        extract(&ir, &plan, &ComponentName::new("tickets").unwrap()).expect("service extracts");
    let planned = ticket_lowered(&service);
    let declared = with_phase_order(EXCHANGED, || ticket_lowered(&service));
    assert_eq!(
        close_order(&planned),
        [
            "invalid-note",
            "first-close",
            "rushed",
            "closed",
            "not-open"
        ]
    );
    assert_eq!(
        close_order(&declared),
        [
            "invalid-note",
            "rushed",
            "first-close",
            "closed",
            "not-open"
        ]
    );

    let (planned, declared) = (registry(&planned), registry(&declared));
    let mut answers = Vec::new();
    for state in ["Open", "Closed"] {
        for reopened in [0, 1] {
            for note in ["", "late"] {
                for rush in [true, false] {
                    let here = closed_by(&planned, state, reopened, note, rush);
                    let base = closed_by(&declared, state, reopened, note, rush);
                    assert_eq!(
                        here, base,
                        "{state}, reopened {reopened}, note {note:?}, rush {rush}"
                    );
                    answers.push(here);
                }
            }
        }
    }
    let refused = "before load: invalid-note";
    assert_eq!(
        answers,
        [
            // Open, reopened 0: `note: ""` refused, then `rush` picks `rushed` or `first-close`.
            refused,
            refused,
            "rushed",
            "first-close",
            // Open, reopened 1: `first-close` does not hold, so `rush: false` takes the default.
            refused,
            refused,
            "rushed",
            "closed",
            // Closed: no move starts there, so entity-core answers the wrong-state branch.
            refused,
            refused,
            "refused not-open",
            "refused not-open",
            refused,
            refused,
            "refused not-open",
            "refused not-open",
        ],
        "every branch answers some request, in both orders"
    );
}

// ---- the one precedence rule (G4) ----------------------------------------------------------

const RULE: &str =
    include_str!("../../../specify/ess-compiler/tests/fixtures/precedence-rule-format-a.yaml");

/// Under the one rule a missing row the input names answers at step 4, after the addressed
/// record's existence and held state. Entity Runtime reads no row but the one its request names,
/// so it lowers no `when_related:` branch in any order: the command is refused by name at each such
/// branch (`RelatedGuardUnsupported`, answer 10 of the record), never lowered without it.
#[test]
fn format_a_related_row_existence_lowers_in_plan_order_or_is_refused_by_name() {
    let text = format!(
        "{}\ncomponents:\n  - component: seat-service\n    reached_by: network\n    owns: {{domains: [demo.rule]}}\n    accepts: {{commands: [demo.rule.DefineTier, demo.rule.Issue, demo.rule.Release, demo.rule.ChangeTier]}}\n    publishes: {{events: [demo.rule.TierDefined, demo.rule.Issued, demo.rule.Released, demo.rule.TierChanged]}}\n",
        RULE.trim_end()
    );
    let raw = RawSpecFile::parse(&text).unwrap_or_else(|error| panic!("{error}"));
    let mut sources = SourceMap::new();
    sources.insert("model.yaml".to_owned(), text.clone());
    let specification = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    let ir = compile_locating(&specification, &sources, &["model.yaml".to_owned()])
        .expect("fixture compiles");
    let plan = SynthesisPlan::of(&ir);
    let service = extract(&ir, &plan, &ComponentName::new("seat-service").unwrap())
        .expect("service extracts");
    let options = LoweringOptions {
        definition_versions: ["demo.rule.Seat", "demo.rule.Tier"]
            .iter()
            .map(|entry| (QualifiedName::new(entry).unwrap(), NonZeroU32::MIN))
            .collect(),
        scales: BTreeMap::new(),
    };
    let Err(diagnostics) = lower(&service, &options) else {
        panic!("a command guarded by a related row is not lowered");
    };
    let related: Vec<String> = diagnostics
        .into_vec()
        .into_iter()
        .filter(|diagnostic| {
            diagnostic.code == ess_entity_runtime::LoweringCode::RelatedGuardUnsupported
        })
        .map(|diagnostic| diagnostic.path)
        .collect();
    assert_eq!(
        related,
        [
            "demo.rule.ChangeTier.unknown-tier.when_related",
            "demo.rule.ChangeTier.tier-closed.when_related"
        ],
    );
}

const PAIRS: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/guard-pair-corpus.yaml");

/// The lowering diagnostics of the guard-pair corpus served by one component, by path.
fn pair_diagnostics() -> Vec<(String, String)> {
    let ir = served_ir(PAIRS, "pairs-service");
    let plan = SynthesisPlan::of(&ir);
    let service = extract(&ir, &plan, &ComponentName::new("pairs-service").unwrap())
        .expect("service extracts");
    let options = LoweringOptions {
        definition_versions: ["demo.pairs.Seat", "demo.pairs.Tier"]
            .iter()
            .map(|entry| (QualifiedName::new(entry).unwrap(), NonZeroU32::MIN))
            .collect(),
        scales: BTreeMap::new(),
    };
    match lower(&service, &options) {
        Ok(_) => Vec::new(),
        Err(diagnostics) => diagnostics
            .into_vec()
            .into_iter()
            .map(|diagnostic| (diagnostic.path, format!("{:?}", diagnostic.code)))
            .collect(),
    }
}

/// `text` served by one network component `component`, owning its domain and accepting every
/// command, compiled.
fn served_ir(text: &str, component: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let probe = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    let domain = text
        .lines()
        .find_map(|line| line.strip_prefix("domain: "))
        .expect("one domain");
    let commands: Vec<String> = probe.commands().keys().map(ToString::to_string).collect();
    let events: Vec<String> = probe.events().keys().map(ToString::to_string).collect();
    let text = format!(
        "{}\ncomponents:\n  - component: {component}\n    reached_by: network\n    owns: {{domains: [{domain}]}}\n    accepts: {{commands: [{}]}}\n    publishes: {{events: [{}]}}\n",
        text.trim_end(),
        commands.join(", "),
        events.join(", ")
    );
    let raw = RawSpecFile::parse(&text).unwrap_or_else(|error| panic!("{error}"));
    let mut sources = SourceMap::new();
    sources.insert("model.yaml".to_owned(), text.clone());
    let specification = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile_locating(&specification, &sources, &["model.yaml".to_owned()]).expect("compiles")
}

/// `text` without the command `name`: its block, the comment above it, and the grant naming it.
fn without_command(text: &str, name: &str) -> String {
    let declared = text
        .find(&format!("  - name: {name}\n    input:"))
        .unwrap_or_else(|| panic!("{name} is declared"));
    let start = text[..declared]
        .rfind("\n  # ")
        .filter(|comment| !text[comment + 1..declared].contains("\n  - name: "))
        .map_or(declared, |comment| comment + 1);
    let rest = &text[declared + 1..];
    let end = declared
        + 1
        + ["\n  # ", "\n  - name: demo.pairs.", "\nviews:"]
            .iter()
            .filter_map(|next| rest.find(next))
            .min()
            .expect("something follows the command")
        + 1;
    format!("{}{}", &text[..start], &text[end..]).replacen(&format!("      - {name}\n"), "", 1)
}

/// Every command of the guard-pair corpus is lowered in its plan's order or refused by name: each
/// one guarded by a related row at every `when_related:` branch (`RelatedGuardUnsupported`), one
/// answering a request with no input document at that branch, and the rest at the
/// `unknown_instance:` branch Entity Runtime has no shape for; without that branch,
/// `when_subject:` beside `when_subject_state:` lowers, its branches in the plan's order (record
/// answer 10).
#[test]
fn guard_pair_corpus_lowers_or_refuses_each_command_by_name() {
    let diagnostics = pair_diagnostics();
    for (command, refused) in [
        ("demo.pairs.StateBesideRelated", "RelatedGuardUnsupported"),
        ("demo.pairs.SubjectBesideRelated", "RelatedGuardUnsupported"),
        ("demo.pairs.AbsentBesideRelated", "RelatedGuardUnsupported"),
        ("demo.pairs.AbsentBesideRelated", "InputAbsentUnsupported"),
        (
            "demo.pairs.WrongStateBesideRelatedMove",
            "RelatedGuardUnsupported",
        ),
        (
            "demo.pairs.BeforeRowBesideRelated",
            "RelatedGuardUnsupported",
        ),
        (
            "demo.pairs.StateBesideWrongState",
            "OutcomeShapeUnsupported",
        ),
        ("demo.pairs.SubjectBesideState", "OutcomeShapeUnsupported"),
        ("demo.pairs.TwoRefusalsOneRow", "RelatedGuardUnsupported"),
    ] {
        assert!(
            diagnostics
                .iter()
                .any(|(path, code)| path.starts_with(&format!("{command}.")) && code == refused),
            "{command} is refused as {refused}: {diagnostics:#?}"
        );
    }
    assert_eq!(
        diagnostics
            .iter()
            .filter(|(path, _)| path.starts_with("demo.pairs.SubjectBesideState."))
            .map(|(path, _)| path.as_str())
            .collect::<Vec<_>>(),
        ["demo.pairs.SubjectBesideState.unknown-seat"],
        "{diagnostics:#?}"
    );

    let mut text = PAIRS.to_owned();
    for command in [
        "demo.pairs.StateBesideRelated",
        "demo.pairs.SubjectBesideRelated",
        "demo.pairs.AbsentBesideRelated",
        "demo.pairs.WrongStateBesideRelatedMove",
        "demo.pairs.BeforeRowBesideRelated",
        "demo.pairs.StateBesideWrongState",
        "demo.pairs.TwoRefusalsOneRow",
    ] {
        text = without_command(&text, command);
    }
    for unknown in ["release-unknown", "retire-unknown", "unknown-seat"] {
        let at = text.find(&format!("      - name: {unknown}\n")).unwrap();
        let end = at + text[at + 1..].find("\n      - name: ").unwrap() + 1;
        text = format!("{}{}", &text[..at], &text[end..]);
    }
    let ir = served_ir(&text, "pairs-service");
    let plan = SynthesisPlan::of(&ir);
    let service = extract(&ir, &plan, &ComponentName::new("pairs-service").unwrap())
        .expect("service extracts");
    let lowered = lower(
        &service,
        &LoweringOptions {
            definition_versions: ["demo.pairs.Seat", "demo.pairs.Tier"]
                .iter()
                .map(|entry| (QualifiedName::new(entry).unwrap(), NonZeroU32::MIN))
                .collect(),
            scales: BTreeMap::new(),
        },
    )
    .unwrap_or_else(|diagnostics| panic!("{diagnostics:?}"));
    let order: Vec<String> = lowered.definitions()[&QualifiedName::new("demo.pairs.Seat").unwrap()]
        .as_definition()
        .operations["demo.pairs.SubjectBesideState"]
        .outcomes
        .iter()
        .map(|outcome| outcome.name.clone())
        .collect();
    assert_eq!(order, ["released", "vip", "changed"]);
}
