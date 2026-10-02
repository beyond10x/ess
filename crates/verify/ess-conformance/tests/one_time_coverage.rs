//! Actual source-produced coverage/35 input, kept identical for every runtime.
#[allow(dead_code)]
mod support_one_time;
use ess_conformance::{
    coverage::{Origins, Scope},
    coverage_build,
    report::Status,
    Runner,
};
use support_one_time::{Mode, Service};

const SOURCE: &str = "format: ess/21\nsystem: credentials\nversion: v1\ndomain: credentials.api\ncommands:\n  - name: credentials.api.Issue\n    response: [{name: secret, type: String}]\n    outcomes: [{name: issued, returns: true, one_time_response: [secret]}]\n";
const AUTHORED: &str = "type: ess-scenario/4\ndomain: credentials.api\nscenario: protected-rotation\nsummary: Issue then rotate without redisclosing the old value.\ntimeline:\n  - at: 2026-10-02T00:00:00Z\n    command: credentials.api.Issue\n    outcome: issued\n  - at: 2026-10-02T00:00:01Z\n    command: credentials.api.Issue\n    outcome: issued\n";

#[test]
fn actual_coverage35_producer_execution_is_frozen() {
    let spec = ess_domain::Specification::assemble([(
        ess_domain::system::Source::new("one-time-coverage.yaml"),
        ess_domain::spec::RawSpecFile::parse(SOURCE).unwrap(),
    )])
    .unwrap();
    let ir =
        ess_compiler::resolve::compile(&spec, &ess_compiler::source::SourceMap::new()).unwrap();
    let input = coverage_build::build(
        &ir,
        &[coverage_build::CoverageSource::new("protected-rotation.yaml", AUTHORED).unwrap()],
        Scope::System,
        Origins::GeneratedAndAuthored,
    )
    .unwrap();
    let id = "credentials.api/authored/protected-rotation"
        .parse()
        .unwrap();
    let input = input.select(&[id]).unwrap();
    assert_eq!(
        input.selected().suite().provenance.suite_version.major(),
        35
    );
    assert_eq!(input.parents().len(), 1);
    assert_eq!(input.selected().suite().len(), 1);
    let mut manifest = Vec::new();
    for mode in [Mode::Healthy, Mode::Retry, Mode::Error, Mode::Unsupported] {
        let target = Service::new(mode);
        let report =
            Runner::for_suite(input.selected().suite()).run_admitted(input.selected(), &target);
        let (status, code) = mode.expected();
        assert_eq!(report.scenarios[0].status, status);
        let counts =
            ess_conformance::counts::CountReport::from_run(&report, input.selected()).unwrap();
        let actual_counts = serde_json::to_value(counts.counts()).unwrap();
        assert_eq!(
            actual_counts,
            serde_json::json!({"total":1,"passed":usize::from(status==Status::Passed),"failed":usize::from(status==Status::Failed),"skipped":0,"unsupported":usize::from(status==Status::Unsupported),"error":usize::from(status==Status::Error)})
        );
        let evidence = serde_json::to_string(&report.scenarios).unwrap();
        assert!(evidence.contains(code));
        let count_bytes = counts.to_canonical_json().unwrap();
        for value in target
            .returned_plaintexts()
            .into_iter()
            .chain([support_one_time::FIRST.into()])
        {
            assert!(!evidence.contains(&value));
            assert!(!count_bytes.contains(&value));
        }
        manifest.push(serde_json::json!({"case":mode,"status":status,"required_code":code,"counts":actual_counts,"callback_trace":target.trace()}));
    }
    let root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/one-time-coverage");
    let output = std::env::var_os("ESS_ONE_TIME_COVERAGE_OUT").map(std::path::PathBuf::from);
    for (name, text) in [
        ("model.yaml", SOURCE.to_owned()),
        ("scenario.yaml", AUTHORED.to_owned()),
        (
            "input.json",
            format!(
                "{}\n",
                serde_json::to_string_pretty(&input.document()).unwrap()
            ),
        ),
        (
            "manifest.json",
            format!("{}\n", serde_json::to_string_pretty(&manifest).unwrap()),
        ),
    ] {
        if let Some(output) = &output {
            std::fs::create_dir_all(output).unwrap();
            std::fs::write(output.join(name), text).unwrap();
        } else {
            assert_eq!(
                std::fs::read_to_string(root.join(name)).unwrap(),
                text,
                "coverage fixture drift: {name}"
            );
        }
    }
}
