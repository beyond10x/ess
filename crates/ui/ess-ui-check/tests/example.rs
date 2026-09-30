//! The partner-portal example passes every check, with one finding: the placeholder its forecast
//! chart reads while the view is designed before the model has it.

use std::path::Path;

use ess_ui_check::{check, Severity};

#[test]
fn the_partner_portal_example_passes_with_only_its_placeholder_warning() {
    let file =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../examples/partner-portal/ui.yaml");
    let report = check(&file, None).unwrap_or_else(|error| panic!("{error}"));
    let found: Vec<(&str, &str, Severity)> = report
        .findings
        .iter()
        .map(|finding| {
            (
                finding.path.as_str(),
                finding.check.as_str(),
                finding.severity,
            )
        })
        .collect();
    assert_eq!(
        found,
        [(
            "pages/forecast.quarterly/sections/chart/reads",
            "unbound_placeholder",
            Severity::Warning
        )],
        "{:#?}",
        report.findings
    );
    assert!(!report.has_errors());
}
