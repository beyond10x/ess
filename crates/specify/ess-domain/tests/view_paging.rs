//! A view declares paging (`paging:`, ess/16; beyond10x/ess#174, `docs/design/view-paging.md`).
//!
//! Two declared parameters slice the view's declared order: `size` of the rows the filter admits,
//! starting at `(page - first_page) * size`, with the filtered count beside them where `total:
//! true`. The parameters `paging:` names are what it reads, so the `unobservable_fact` refusal of a
//! parameter no filter reads does not apply to them. The caller-supplied filter expression #174 also
//! asks for is not part of this construct.

use ess_domain::Specification;
use ess_primitives::error::{ValidationCode, ValidationErrors};

fn assemble(body: &str) -> Result<Specification, ValidationErrors> {
    let raw = ess_domain::spec::RawSpecFile::parse(body)
        .unwrap_or_else(|error| panic!("the document parses: {error}\n{body}"));
    Specification::assemble([(ess_domain::system::Source::new("jobs.yaml"), raw)])
}

fn admitted(body: &str) -> Specification {
    assemble(body).unwrap_or_else(|errors| panic!("the model is admitted: {errors}\n{body}"))
}

fn refused(body: &str) -> ValidationErrors {
    match assemble(body) {
        Ok(_) => panic!("the model is refused:\n{body}"),
        Err(errors) => errors,
    }
}

fn assert_code(errors: &ValidationErrors, code: ValidationCode, needle: &str) {
    assert!(
        errors
            .as_slice()
            .iter()
            .any(|error| error.code == code && error.to_string().contains(needle)),
        "expected {code:?} mentioning `{needle}`, got:\n{errors}"
    );
}

/// The #174 repro at `format`, with `params`, `tail` (what follows `filter:`) as the view's
/// declaration.
fn jobs(format: u32, params: &str, tail: &str) -> String {
    format!(
        "format: ess/{format}
system: demo
version: v1
domain: demo.jobs
summary: A job list the caller filters and pages.
types:
  - {{name: demo.jobs.JobId, kind: newtype, of: Uuid}}
  - {{name: demo.jobs.JobType, kind: newtype, of: String}}
  - {{name: demo.jobs.PageNumber, kind: newtype, of: Integer}}
entities:
  - name: demo.jobs.Job
    identity: {{name: job_id, type: demo.jobs.JobId}}
    fields:
      - {{name: type, type: demo.jobs.JobType}}
    lifecycle: {{initial: Open, states: [Open], terminal: [Open], transitions: []}}
actors:
  - {{name: demo.jobs.Clerk, may: [demo.jobs.CreateJob]}}
commands:
  - name: demo.jobs.CreateJob
    input:
      - {{name: type, type: demo.jobs.JobType}}
    outcomes:
      - name: created
        creates: demo.jobs.Job
        instance: job_id
        sets: {{type: input.type}}
        emits: [demo.jobs.JobCreated]
        payload:
          demo.jobs.JobCreated: {{job_id: {{generated: true}}, type: input.type}}
events:
  - name: demo.jobs.JobCreated
    fields:
      - {{name: job_id, type: demo.jobs.JobId}}
      - {{name: type, type: demo.jobs.JobType}}
views:
  - name: demo.jobs.JobList
    source: demo.jobs.Job
    consistency: read_your_writes
    params:
{params}    filter: type == param.type
{tail}    fields:
      - {{name: job_id, type: demo.jobs.JobId}}
      - {{name: type, type: demo.jobs.JobType}}
"
    )
}

const PARAMS: &str = "      - {name: type, type: Optional<demo.jobs.JobType>}
      - {name: page, type: Integer}
      - {name: size, type: Integer}
";

const ORDERED: &str = "    order_by: [job_id asc]\n";

fn paged(paging: &str) -> String {
    format!("{ORDERED}    paging: {paging}\n")
}

#[test]
fn the_174_repro_with_a_paging_block_is_admitted() {
    let spec = admitted(&jobs(
        16,
        PARAMS,
        &paged("{page: page, size: size, total: true}"),
    ));
    let view = spec.views().values().next().expect("the view is declared");
    let paging = view.paging.as_ref().expect("the view is paged");
    assert_eq!(paging.page, "page");
    assert_eq!(paging.size, "size");
    assert!(paging.total);
    assert_eq!(paging.first_page, 0);
}

#[test]
fn without_a_paging_block_page_and_size_are_still_unobservable() {
    let errors = refused(&jobs(16, PARAMS, ORDERED));
    assert_code(
        &errors,
        ValidationCode::UnobservableFact,
        "declares the parameter `page` and no filter reads it",
    );
    assert_code(
        &errors,
        ValidationCode::UnobservableFact,
        "declares the parameter `size` and no filter reads it",
    );
}

#[test]
fn only_the_parameters_paging_names_are_exempt() {
    let params = format!("{PARAMS}      - {{name: sort, type: String}}\n");
    let errors = refused(&jobs(16, &params, &paged("{page: page, size: size}")));
    assert_code(
        &errors,
        ValidationCode::UnobservableFact,
        "declares the parameter `sort` and no filter reads it",
    );
    assert!(
        !errors.to_string().contains("parameter `page`"),
        "`page` is read by `paging:`:\n{errors}"
    );
}

#[test]
fn paging_below_ess_16_is_refused_by_version() {
    let errors = refused(&jobs(15, PARAMS, &paged("{page: page, size: size}")));
    assert_code(
        &errors,
        ValidationCode::UnsupportedFormatVersion,
        "`paging:` requires specification format ess/16",
    );
    assert!(
        errors
            .as_slice()
            .iter()
            .any(|error| error.to_string().contains("JobList.paging")),
        "refused at the key the author wrote:\n{errors}"
    );
}

#[test]
fn paging_needs_a_declared_order() {
    let errors = refused(&jobs(16, PARAMS, "    paging: {page: page, size: size}\n"));
    assert_code(
        &errors,
        ValidationCode::MissingDeclaration,
        "declares `paging:` and no `order_by:`",
    );
}

#[test]
fn paging_names_declared_parameters() {
    let errors = refused(&jobs(16, PARAMS, &paged("{page: offset, size: size}")));
    assert_code(
        &errors,
        ValidationCode::UndeclaredReference,
        "`paging.page` names `offset`, which `params:` does not declare",
    );
}

#[test]
fn a_paging_parameter_is_an_integer() {
    let params = "      - {name: type, type: Optional<demo.jobs.JobType>}
      - {name: page, type: String}
      - {name: size, type: Integer}
";
    let errors = refused(&jobs(16, params, &paged("{page: page, size: size}")));
    assert_code(&errors, ValidationCode::TypeMismatch, "`page` is `String`");
}

#[test]
fn a_paging_parameter_may_be_an_integer_newtype_or_optional() {
    let params = "      - {name: type, type: Optional<demo.jobs.JobType>}
      - {name: page, type: demo.jobs.PageNumber}
      - {name: size, type: Optional<Integer>}
";
    admitted(&jobs(16, params, &paged("{page: page, size: size}")));
}

#[test]
fn page_and_size_are_two_parameters() {
    let errors = refused(&jobs(16, PARAMS, &paged("{page: size, size: size}")));
    assert_code(
        &errors,
        ValidationCode::ConflictingDeclaration,
        "names `size` as both the page and the size",
    );
}

#[test]
fn a_filter_does_not_read_a_paging_parameter() {
    let params = "      - {name: type, type: Optional<demo.jobs.JobType>}
      - {name: page, type: Integer}
      - {name: size, type: Integer}
";
    let body = jobs(16, params, &paged("{page: page, size: size}")).replace(
        "filter: type == param.type",
        "filter: {all: ['type == param.type', 'param.size > 0']}",
    );
    let errors = refused(&body);
    assert_code(
        &errors,
        ValidationCode::ConflictingDeclaration,
        "reads the paging parameter `size` in `filter:`",
    );
}

#[test]
fn the_first_page_is_zero_or_one() {
    admitted(&jobs(
        16,
        PARAMS,
        &paged("{page: page, size: size, first_page: 1}"),
    ));
    let errors = refused(&jobs(
        16,
        PARAMS,
        &paged("{page: page, size: size, first_page: 2}"),
    ));
    assert_code(
        &errors,
        ValidationCode::UnsupportedConstruct,
        "`first_page` is 2; pages are numbered from 0 or from 1",
    );
}

#[test]
fn a_paged_view_round_trips_through_yaml() {
    let spec = admitted(&jobs(
        16,
        PARAMS,
        &paged("{page: page, size: size, first_page: 1, total: true}"),
    ));
    let view = spec.views().values().next().expect("declared").clone();
    let written = serde_yaml::to_string(&view).expect("serializes");
    assert!(written.contains("paging:"), "{written}");
    let read: ess_domain::view::RawViewSpec = serde_yaml::from_str(&written).expect("parses");
    let back = ess_domain::view::ViewSpec::try_from(read).expect("valid");
    assert_eq!(back.paging, view.paging);
}

#[test]
fn an_unknown_key_in_paging_is_refused() {
    let body = jobs(16, PARAMS, &paged("{page: page, size: size, cursor: next}"));
    assert!(
        ess_domain::spec::RawSpecFile::parse(&body).is_err(),
        "`cursor` is not a paging key"
    );
}
