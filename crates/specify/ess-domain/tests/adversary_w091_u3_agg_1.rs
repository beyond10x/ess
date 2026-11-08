//! Adversary cases for `story:aggregate-measures-over-expressions` (`ess/36`): the format gate the
//! changelog entry and `FORMAT_HISTORY` row 36 promise for a Duration measure.
//!
//! Both say: "Below `ess/36` the key, an expression measure and a Duration measure are refused as
//! `unsupported_format_version` naming `ess/36`."
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::error::{ValidationCode, ValidationErrors};

const LEDGER: &str = include_str!(
    "../../../verify/ess-conformance/tests/fixtures/aggregate-measures-over-expressions.yaml"
);

fn listed(errors: &ValidationErrors) -> String {
    errors
        .as_slice()
        .iter()
        .map(|error| {
            format!(
                "{:?} {} {} (hint: {})",
                error.code,
                error.location,
                error.message,
                error.hint.clone().unwrap_or_default()
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// The ledger at `format`, without `ordered:`, with a `Duration` field `elapsed` on the window,
/// and one view `demo.ledger.V` grouped by `incident` whose second field is `field`.
fn model(format: &str, field: &str) -> String {
    let head = LEDGER
        .replace("format: ess/36", &format!("format: {format}"))
        .replace("    ordered: true\n", "")
        .replace(
            "      - {name: level, type: demo.ledger.ImpactLevel}\n",
            "      - {name: level, type: demo.ledger.ImpactLevel}\n      - {name: elapsed, type: Duration}\n",
        )
        .replace(
            "          level: input.level\n",
            "          level: input.level\n          elapsed: input.elapsed\n",
        );
    let head = head.split_once("views:\n").unwrap().0;
    format!(
        "{head}views:\n  - name: demo.ledger.V\n    source: demo.ledger.Window\n    \
         group_by: [incident]\n    fields:\n      - {{name: incident, type: String}}\n      - \
         {field}\n"
    )
}

fn assemble(text: &str) -> Result<Specification, ValidationErrors> {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    Specification::assemble([(Source::new("ledger.yaml"), raw)])
}

/// At `ess/36` a `sum` of a Duration field is admitted (the control), and at `ess/35` it is refused
/// as `unsupported_format_version` naming `ess/36`, as the changelog and the format history say.
#[test]
fn a_duration_field_measure_below_ess_36_is_refused_as_unsupported_format_version() {
    for field in [
        "{name: total, type: Duration, aggregate: {sum: elapsed}}",
        "{name: longest, type: Optional<Duration>, aggregate: {max: elapsed}}",
        "{name: mean, type: Optional<Decimal>, aggregate: {avg: elapsed}}",
    ] {
        let current = model("ess/36", field);
        if let Err(errors) = assemble(&current) {
            panic!("admitted at ess/36: {field}\n{}", listed(&errors));
        }
        let old = model("ess/35", field);
        let errors = assemble(&old)
            .err()
            .unwrap_or_else(|| panic!("must refuse at ess/35: {field}"));
        let site = "view.demo.ledger.V.fields[1].aggregate";
        let found = errors
            .as_slice()
            .iter()
            .find(|error| error.location == site)
            .unwrap_or_else(|| panic!("a refusal at {site}: {}", listed(&errors)));
        assert_eq!(
            found.code,
            ValidationCode::UnsupportedFormatVersion,
            "{field}: the changelog promises `unsupported_format_version` naming ess/36:\n{}",
            listed(&errors)
        );
    }
}
