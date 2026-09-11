//! Selection needs a real event; periodic host causes cannot silently drop it.
use ess_domain::{system::Source, RawSpecFile, Specification};

#[test]
fn periodic_selection_declarations_and_mappings_are_refused() {
    let source = include_str!("fixtures/periodic.yaml");
    for changed in [
        source.replace("    mapping:", "    selection_inputs:\n      - name: items\n        from: event.status\n        as: List<String>\n    mapping:"),
        source.replace("status: host_read.status", "status: {selection: selected, path: []}"),
    ] {
        let raw = RawSpecFile::parse(&changed).expect("valid source syntax");
        let error = Specification::assemble([(Source::new("periodic-selection.yaml"), raw)])
            .expect_err("periodic selection has no event authority").to_string();
        assert!(error.contains("periodic causes do not supply event"), "{error}");
    }
}
