//! Adversarial cases for `story:concurrent-history-lanes`: what the page draws, held to the story's
//! outcome ("one lane per client", "for a violation the operation where the search failed") and to
//! the guide's "For a violation, the page marks the call where the search failed".

use std::path::{Path, PathBuf};

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::faulty::{self, Fault};
use ess_conformance::history::{self, History, Verdict};
use ess_conformance::lanes;
use ess_conformance::linearize::{self, DEFAULT_BUDGET};
use ess_conformance::record::{self, Atomic};
use ess_conformance::reference::Billing;
use ess_conformance::scenario::SuiteProvenance;
use ess_conformance::sessions;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

fn billing_model() -> EssIr {
    let base = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../examples/billing")
        .canonicalize()
        .expect("the billing example exists");
    let mut found: Vec<PathBuf> = Vec::new();
    let mut pending = vec![base.clone()];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).expect("readable") {
            let path = entry.expect("an entry").path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|it| it == "yaml") {
                found.push(path);
            }
        }
    }
    found.sort();
    let mut sources = SourceMap::new();
    let mut parsed = Vec::new();
    for path in found {
        let label = path
            .strip_prefix(&base)
            .expect("inside")
            .display()
            .to_string();
        let text = std::fs::read_to_string(&path).expect("readable");
        let raw = RawSpecFile::parse(&text).expect("well formed");
        sources.insert(label.clone(), text);
        parsed.push((Source::new(label), raw));
    }
    let specification = Specification::assemble(parsed).expect("billing validates");
    compile(&specification, &sources).expect("billing resolves")
}

fn through_the_reader(ir: &EssIr, recorded: &History) -> History {
    let bytes = serde_json::to_vec(recorded).expect("a history serializes");
    history::read(&bytes, &SuiteProvenance::of(ir).spec_digest)
        .unwrap_or_else(|refusal| panic!("the history is admitted: {refusal}"))
}

fn page(ir: &EssIr, history: &History) -> String {
    lanes::render(ir, history, DEFAULT_BUDGET)
        .unwrap_or_else(|refusal| panic!("the history renders: {refusal}"))
}

/// The first `<section class="history">` of the page: the history as given, not the shrunk one.
fn first_section(page: &str) -> &str {
    let start = page
        .find("<section class=\"history\"")
        .expect("the page draws the history");
    let rest = &page[start..];
    &rest[..rest.find("</section>").expect("a closed section")]
}

#[test]
fn a_client_that_made_no_calls_still_has_its_lane() {
    // `ess-history/1` admits `clients` above every operation's `client` (history.rs: an operation's
    // client is only required to be below `clients`); a runner whose third client timed out before
    // its first call writes exactly that. The story's outcome is "one lane per client".
    let model = billing_model();
    let reference = Billing::new();
    let mut recorded = record::record(
        &model,
        &Atomic(&reference),
        &faulty::lost_update_workload(),
        0,
    )
    .expect("recorded");
    recorded.clients += 1;
    let history = through_the_reader(&model, &recorded);
    let idle = history.clients - 1;
    assert!(
        history
            .operations
            .iter()
            .all(|operation| operation.client != idle),
        "client {idle} made no call"
    );

    let page = page(&model, &history);
    let section = first_section(&page);
    assert_eq!(
        section.matches("<g class=\"lane\"").count() as u64,
        history.clients,
        "the header says {} client(s); one lane per client:\n{section}",
        history.clients
    );
    assert!(
        section.contains(&format!("<g class=\"lane\" data-client=\"{idle}\"")),
        "client {idle} has a lane"
    );
}

#[test]
fn a_read_violation_marks_the_read_that_decided_it() {
    // `Fault::StaleReadUnderReadYourWrites` is the fault matrix's read-violation row. Its page says
    // "Violation"; the guide says "For a violation, the page marks the call where the search
    // failed", and for a read violation that call is the read `Checked::read` names.
    let model = billing_model();
    let target = faulty::billing(Fault::StaleReadUnderReadYourWrites);
    let (history, read) = (0..24)
        .find_map(|seed| {
            let recorded = sessions::record(
                &model,
                &target,
                &target,
                &faulty::stale_read_workload(),
                seed,
            )
            .expect("the workload names what billing declares");
            let history = through_the_reader(&model, &recorded);
            let checked = linearize::check(&model, &history, DEFAULT_BUDGET).expect("checked");
            (checked.verdict == Verdict::Violation)
                .then_some(())
                .and(checked.read)
                .map(|read| (history, read))
        })
        .expect("some seed records the stale read");

    let page = page(&model, &history);
    let section = first_section(&page);
    assert!(section.contains("Read violation"), "{section}");
    let marked: Vec<&str> = section
        .split("<rect class=\"op failing")
        .skip(1)
        .map(|rest| rest.split('>').next().expect("a closed tag"))
        .collect();
    assert_eq!(
        marked.len(),
        1,
        "the read {} is marked where the check failed:\n{section}",
        read.operation_id
    );
    assert!(
        marked[0].contains(&format!("data-operation=\"{}\"", read.operation_id)),
        "{}",
        marked[0]
    );
}
