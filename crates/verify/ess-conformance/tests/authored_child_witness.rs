//! An admitted authored scenario whose input lets one child of a connective alone decide it, and
//! which expects the branch that child decides, withdraws the `ESS-SYNTH-022` refusal naming that
//! child (<https://github.com/beyond10x/ess/issues/619>): the refusal's help says to cover the
//! child with such a scenario, so the scenario has to count.
//!
//! The coverage inventory (suite format 5) and the ordinary synthesis path are held to one rule:
//! the refusal list, the counts and the inventory's completeness read the same withdrawal.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::coverage::{Inventory, Origins, Scope};
use ess_conformance::coverage_build::{self, CoverageSource};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

/// `all: [limit > 0, amount > 0, amount <= limit]` over one struct input: the default's row where
/// `amount <= limit` alone fails compares two facts, which the bounded search does not solve.
const MODEL: &str = r"format: ess/23
system: demo
version: v1
domain: demo.terms

types:
  - name: demo.terms.Terms
    kind: struct
    fields:
      - {name: limit, type: Integer}
      - {name: amount, type: Integer}

errors:
  - name: demo.terms.Invalid

commands:
  - name: demo.terms.Check
    input:
      - {name: terms, type: demo.terms.Terms}
    outcomes:
      - name: valid
        when:
          all:
            - terms.limit > 0
            - terms.amount > 0
            - terms.amount <= terms.limit
        accepts: nothing
      - name: invalid
        error: demo.terms.Invalid
";

/// The child the search leaves unwitnessed, as the refusal prints it.
const CHILD: &str = "not (terms.amount <= terms.limit)";

fn ir() -> EssIr {
    let raw = RawSpecFile::parse(MODEL).unwrap_or_else(|error| panic!("{error}\n{MODEL}"));
    let spec = Specification::assemble([(Source::new("terms.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).expect("the fixture compiles")
}

/// One authored scenario sending `limit` and `amount` and expecting `outcome`.
fn authored(name: &str, limit: i64, amount: i64, outcome: &str) -> String {
    let error = if outcome == "invalid" {
        "\n    error: {name: demo.terms.Invalid}"
    } else {
        ""
    };
    format!(
        "type: ess-scenario/1
domain: demo.terms
scenario: {name}
summary: Terms with limit {limit} and amount {amount}.

timeline:
  - at: 2026-01-01T00:00:00Z
    command: demo.terms.Check
    input:
      terms: {{limit: {limit}, amount: {amount}}}
    outcome: {outcome}{error}
"
    )
}

fn inventory(sources: &[(&str, String)]) -> Inventory {
    let sources: Vec<CoverageSource> = sources
        .iter()
        .map(|(identity, text)| CoverageSource::new(*identity, text.clone()).expect("a source"))
        .collect();
    let origins = if sources.is_empty() {
        Origins::Generated
    } else {
        Origins::GeneratedAndAuthored
    };
    coverage_build::build(&ir(), &sources, Scope::System, origins)
        .expect("the inventory builds")
        .selected()
        .coverage()
        .expect("a coverage suite")
        .clone()
}

fn child_refusals(inventory: &Inventory) -> Vec<&str> {
    inventory
        .refused
        .iter()
        .filter(|refusal| refusal.code == "ESS-SYNTH-022" && refusal.message.contains(CHILD))
        .map(|refusal| refusal.message.as_str())
        .collect()
}

fn counted(inventory: &Inventory) -> usize {
    usize::try_from(inventory.counts.refused).expect("a count")
}

/// The precondition: without an authored scenario, the child is refused beside the default's
/// scenario.
#[test]
fn without_an_authored_scenario_the_child_is_refused() {
    let inventory = inventory(&[]);
    assert_eq!(
        child_refusals(&inventory).len(),
        1,
        "every refusal: {:#?}",
        inventory.refused
    );
    assert_eq!(counted(&inventory), inventory.refused.len());
}

/// The issue's case: an amount above a positive limit isolates `amount <= limit` and takes the
/// default branch, so the refusal for that child is withdrawn and the count falls by one.
#[test]
fn an_admitted_authored_scenario_that_isolates_the_child_withdraws_its_refusal() {
    let before = inventory(&[]);
    let after = inventory(&[(
        "amount-above-limit.yaml",
        authored("amount-above-limit", 5, 6, "invalid"),
    )]);
    assert_eq!(after.authored.len(), 1, "the authored scenario is admitted");
    assert!(
        child_refusals(&after).is_empty(),
        "every refusal: {:#?}",
        after.refused
    );
    assert_eq!(after.refused.len() + 1, before.refused.len());
    assert_eq!(counted(&after), after.refused.len());
    let other = |inventory: &Inventory| -> Vec<String> {
        inventory
            .refused
            .iter()
            .filter(|refusal| !refusal.message.contains(CHILD))
            .map(|refusal| refusal.message.clone())
            .collect()
    };
    assert_eq!(
        other(&after),
        other(&before),
        "only that child's refusal goes"
    );
}

/// An input where two children fail together does not let the child alone decide the connective,
/// so the refusal stands.
#[test]
fn an_authored_scenario_that_does_not_isolate_the_child_leaves_its_refusal() {
    let after = inventory(&[(
        "amount-above-zero-limit.yaml",
        authored("amount-above-zero-limit", 0, 6, "invalid"),
    )]);
    assert_eq!(after.authored.len(), 1, "the authored scenario is admitted");
    assert_eq!(
        child_refusals(&after).len(),
        1,
        "every refusal: {:#?}",
        after.refused
    );
}

/// An authored scenario of the other branch sends no input of the refused row.
#[test]
fn an_authored_scenario_of_the_other_branch_leaves_its_refusal() {
    let after = inventory(&[(
        "amount-within-limit.yaml",
        authored("amount-within-limit", 6, 5, "valid"),
    )]);
    assert_eq!(after.authored.len(), 1, "the authored scenario is admitted");
    assert_eq!(child_refusals(&after).len(), 1);
}

/// The ordinary synthesis path (suite format 4) withdraws the same refusal through the same rule.
#[test]
fn the_ordinary_path_withdraws_the_same_refusal() {
    let ir = ir();
    let mut synthesis = ess_conformance::synthesize::synthesize(&ir);
    let authoring = ess_conformance::authored::compile(
        &ir,
        &[ess_conformance::authored::Source::new(
            "amount-above-limit.yaml",
            authored("amount-above-limit", 5, 6, "invalid"),
        )],
    );
    assert!(authoring.is_complete(), "{:#?}", authoring.refusals);
    let names = |synthesis: &ess_conformance::Synthesis| -> usize {
        synthesis
            .refusals
            .iter()
            .filter(|refusal| refusal.to_string().contains(CHILD))
            .count()
    };
    assert_eq!(names(&synthesis), 1);
    ess_conformance::synthesize::withdraw_witnessed_children(
        &ir,
        &mut synthesis.refusals,
        authoring.scenarios.values(),
    );
    assert_eq!(names(&synthesis), 0);
}
