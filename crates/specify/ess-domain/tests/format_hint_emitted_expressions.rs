//! The format hints of expression refusals, and the header refusal, name the source that
//! declares the system
//! (`story:every-format-version-hint-names-the-source-that-declares-the-system`).

use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::error::{ValidationCode, ValidationError};

const STORED_NOW: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/now-stored-rows.yaml");
const VIEW_NOW: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/now-view-filter.yaml");

/// A job scheduler guarding on the current time (`ess/16`) and a text length (`ess/11`).
const JOBS: &str = "format: ess/16
system: demo
version: v1
domain: demo.jobs
entities:
  - name: demo.jobs.Job
    identity: {name: job_id, type: Uuid}
    fields: []
    lifecycle:
      initial: Scheduled
      states: [Scheduled]
      terminal: [Scheduled]
events:
  - name: demo.jobs.JobScheduled
    fields: [{name: job_id, type: Uuid}]
errors:
  - {name: demo.jobs.InPast, fields: []}
commands:
  - name: demo.jobs.Schedule
    input:
      - {name: starts_at, type: Timestamp}
      - {name: label, type: String}
    outcomes:
      - name: start-in-past
        when: starts_at < now
        error: demo.jobs.InPast
      - name: label-too-long
        when: label.count > 40
        error: demo.jobs.InPast
      - name: scheduled
        creates: demo.jobs.Job
        instance: job_id
        emits: [demo.jobs.JobScheduled]
        payload: {demo.jobs.JobScheduled: {job_id: {generated: true}}}
";

const SYSTEM: &str = "format: ess/16\nsystem: demo\nversion: v1\ndomains: [demo.jobs]\n";

fn refusals(files: &[(&str, &str)]) -> Vec<ValidationError> {
    let parsed = files.iter().map(|(name, text)| {
        let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{name}: {error}"));
        (Source::new(*name), raw)
    });
    Specification::assemble(parsed.collect::<Vec<_>>())
        .expect_err("refused")
        .as_slice()
        .to_vec()
}

fn at<'a>(errors: &'a [ValidationError], location: &str) -> &'a ValidationError {
    errors
        .iter()
        .find(|error| error.location == location)
        .unwrap_or_else(|| panic!("no refusal at {location}: {errors:#?}"))
}

fn domain_file() -> String {
    format!(
        "format: ess/16\n{}",
        &JOBS[JOBS.find("domain: demo.jobs").expect("a domain")..]
    )
}

#[test]
fn the_current_time_and_a_text_length_below_their_formats_name_the_source() {
    let errors = refusals(&[(
        "model.yaml",
        &JOBS.replacen("format: ess/16", "format: ess/10", 1),
    )]);
    let now = at(
        &errors,
        "command.demo.jobs.Schedule.outcomes.start-in-past.when",
    );
    assert_eq!(now.code, ValidationCode::UnsupportedFormatVersion);
    assert_eq!(
        now.message,
        "`starts_at < now` orders the Timestamp `starts_at` against the current time, `now`, which requires specification format ess/16"
    );
    assert_eq!(
        now.hint.as_deref(),
        Some("write `format: ess/16` on the source that declares the system")
    );
    let length = at(
        &errors,
        "command.demo.jobs.Schedule.outcomes.label-too-long.when",
    );
    assert_eq!(
        length.message,
        "`label.count`: the length of a String requires specification format ess/11"
    );
    assert_eq!(
        length.hint.as_deref(),
        Some("write `format: ess/11` on the source that declares the system")
    );
}

#[test]
fn the_stored_current_time_below_ess_22_names_the_source() {
    let errors = refusals(&[(
        "model.yaml",
        &STORED_NOW.replacen("format: ess/22", "format: ess/21", 1),
    )]);
    let error = at(
        &errors,
        "command.demo.leases.RenewLease.outcomes.renewed.when_subject",
    );
    assert_eq!(
        error.message,
        "`expires_at >= now - 1h` orders the stored Timestamp `expires_at` against the current time, `now - 1h`, which a `when_subject:` or `when_related:` predicate requires specification format ess/22 for"
    );
    assert_eq!(
        error.hint.as_deref(),
        Some("write `format: ess/22` on the source that declares the system")
    );
}

#[test]
fn a_view_filter_over_now_below_ess_30_names_the_source() {
    let errors = refusals(&[(
        "model.yaml",
        &VIEW_NOW.replacen("format: ess/30", "format: ess/29", 1),
    )]);
    let error = at(&errors, "view.demo.ledger.RecentIncidents.filter");
    assert_eq!(error.code, ValidationCode::TypeMismatch);
    assert!(
        error.message.ends_with("; or write `format: ess/30` on the source that declares the system, from which a view filter reads `now` at the instant of the read"),
        "{error:?}"
    );
}

#[test]
fn a_header_key_on_another_source_names_the_source_of_the_system() {
    let domain = domain_file();
    let errors = refusals(&[("system.yaml", SYSTEM), ("domains/jobs.yaml", &domain)]);
    let error = at(&errors, "domains/jobs.yaml.system");
    assert_eq!(error.code, ValidationCode::MissingDeclaration);
    assert_eq!(
        error.hint.as_deref(),
        Some(
            "the source that carries the format, the version and the domain list is the one that names the system: move these keys to `system.yaml`"
        )
    );
}

#[test]
fn with_no_source_naming_the_system_the_header_hint_keeps_its_text() {
    let domain = domain_file();
    let errors = refusals(&[("domains/jobs.yaml", &domain)]);
    let error = at(&errors, "domains/jobs.yaml.system");
    assert_eq!(
        error.hint.as_deref(),
        Some(
            "the source that carries the format, the version and the domain list is the one that names the system"
        )
    );
}
