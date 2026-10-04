//! A `when_related:` guard reading a stored field of the addressed subject (ess/22,
//! beyond10x/ess#304) keeps its command a hand-written obligation in every generated target, and
//! the obligation's contract states the order the guard is answered in.
//!
//! Generating the guard is story:related-guard-behaviour (beyond10x/ess#319), which removes this
//! file's single case when it lands: until then the plan owes the behaviour, and says what is owed.

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile_locating;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_synth::{synthesize_for, CapabilityKind, SynthesisDisposition, Target};

const STORED_REFERENCE: &str = include_str!(
    "../../../verify/ess-conformance/tests/fixtures/related-guard-stored-reference.yaml"
);
const COMMAND: &str = "demo.tasks.CompleteTask";

fn compile_text(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("well formed: {error}"));
    let specification = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("validates:\n{errors}"));
    let mut sources = SourceMap::new();
    sources.insert("model.yaml".to_owned(), text.to_owned());
    compile_locating(&specification, &sources, &["model.yaml".to_owned()])
        .unwrap_or_else(|diagnostics| panic!("resolves:\n{diagnostics}"))
}

#[test]
fn a_stored_reference_guard_stays_an_obligation_with_its_contract() {
    let ir = compile_text(STORED_REFERENCE);
    for target in [Target::Rust, Target::Go, Target::Web, Target::Clap] {
        let synthesis = synthesize_for(&ir, target)
            .unwrap_or_else(|failure| panic!("{target:?} emits the model: {failure:?}"));
        let obligation = match synthesis
            .plan
            .disposition_of(CapabilityKind::CommandBehavior, COMMAND)
        {
            Some(SynthesisDisposition::Obligation(obligation)) => obligation,
            other => panic!("{target:?}: `{COMMAND}` stays an obligation, not {other:?}"),
        };
        let why = obligation.reason.describes();
        assert!(
            why.contains("`when_related:`"),
            "{target:?}: the guard is what keeps it owed: {why}"
        );
        let contract = &obligation.contract;
        // The branch phrase names the stored field it reads, on the addressed subject.
        assert!(
            contract
                .contains("`blocked` when the `demo.tasks.Task` that `subject.blocked_by` names"),
            "{target:?}: {contract}"
        );
        // The order: the addressed row's existence and held state, then the stored reference
        // as the subject held it before the branch, then acceptance.
        let order = [
            "input refusal",
            "addressed-row existence",
            "held state",
            "then read the stored reference",
            "`exists: false`",
            "accepting and external branches",
        ];
        let at: Vec<usize> = order
            .iter()
            .map(|phrase| {
                contract
                    .find(phrase)
                    .unwrap_or_else(|| panic!("{target:?}: `{phrase}` is stated: {contract}"))
            })
            .collect();
        assert!(
            at.windows(2).all(|pair| pair[0] < pair[1]),
            "{target:?}: the contract states {order:?} in that order: {contract}"
        );
        assert!(
            contract.contains("absent, it selects no `when_related:` branch"),
            "{target:?}: an absent stored reference is stated: {contract}"
        );
        assert!(
            !contract.contains("`exists: false` before input-guarded refusals"),
            "{target:?}: the input-reference order is not claimed for a stored reference: \
             {contract}"
        );
    }
}
