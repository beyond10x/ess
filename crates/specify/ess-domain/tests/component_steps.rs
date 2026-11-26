//! A component's `steps:` (ess/38): the work it starts itself, a level step running a command for
//! each row a view answers and a time step running a command every period, and what validation
//! checks of them (story:a-component-declares-level-and-time-steps-that-validate-checks-and-the-interpreter-runs).

/// A queue a worker leases from, and a sweep run every minute.
pub const JOBS: &str = "\
format: ess/38
system: jobs
version: v1
domain: demo.jobs
summary: A queue worked by its own worker.
types:
  - {name: demo.jobs.JobId, kind: newtype, of: Uuid}
  - {name: demo.jobs.SweepId, kind: newtype, of: Uuid}
entities:
  - name: demo.jobs.Job
    identity: {name: job_id, type: demo.jobs.JobId}
    fields:
      - {name: title, type: String}
    lifecycle:
      initial: Pending
      states: [Pending, Leased]
      terminal: [Leased]
      transitions:
        - {name: lease, from: [Pending], to: Leased}
  - name: demo.jobs.Sweep
    identity: {name: sweep_id, type: demo.jobs.SweepId}
    fields: []
    lifecycle: {initial: Done, states: [Done], terminal: [Done]}
errors:
  - {name: demo.jobs.NotPending, summary: The job is not pending., fields: []}
events:
  - name: demo.jobs.JobSubmitted
    fields:
      - {name: job_id, type: demo.jobs.JobId}
  - name: demo.jobs.JobLeased
    fields:
      - {name: job_id, type: demo.jobs.JobId}
  - name: demo.jobs.Swept
    fields:
      - {name: sweep_id, type: demo.jobs.SweepId}
actors:
  - name: demo.jobs.Client
    may: [demo.jobs.Submit, demo.jobs.PendingJobs]
  - name: demo.jobs.Worker
    system: true
    may: [demo.jobs.Lease, demo.jobs.RecordSweep, demo.jobs.PendingJobs]
components:
  - component: worker
    owns: {domains: [demo.jobs]}
    accepts:
      commands: [demo.jobs.Submit, demo.jobs.Lease, demo.jobs.RecordSweep]
    publishes:
      events: [demo.jobs.JobSubmitted, demo.jobs.JobLeased, demo.jobs.Swept]
    reached_by: network
    steps:
      - {form: level, view: demo.jobs.PendingJobs, command: demo.jobs.Lease, every: PT5S}
      - {form: time, command: demo.jobs.RecordSweep, every: PT60S}
commands:
  - name: demo.jobs.Submit
    input:
      - {name: title, type: String}
    outcomes:
      - name: submitted
        creates: demo.jobs.Job
        instance: job_id
        sets: {title: input.title}
        emits: [demo.jobs.JobSubmitted]
        payload:
          demo.jobs.JobSubmitted: {job_id: {generated: true}}
  - name: demo.jobs.Lease
    input:
      - {name: job_id, type: demo.jobs.JobId}
    outcomes:
      - name: leased
        moves: demo.jobs.Job.lease
        instance: job_id
        emits: [demo.jobs.JobLeased]
        payload:
          demo.jobs.JobLeased: {job_id: input.job_id}
      - {name: not-pending, wrong_state: true, error: demo.jobs.NotPending}
  - name: demo.jobs.RecordSweep
    input: []
    outcomes:
      - name: swept
        creates: demo.jobs.Sweep
        instance: sweep_id
        emits: [demo.jobs.Swept]
        payload:
          demo.jobs.Swept: {sweep_id: {generated: true}}
views:
  - name: demo.jobs.PendingJobs
    source: demo.jobs.Job
    consistency: read_your_writes
    filter: state == Pending
    fields:
      - {name: job_id, type: demo.jobs.JobId}
      - {name: title, type: String}
";

const LEVEL: &str =
    "      - {form: level, view: demo.jobs.PendingJobs, command: demo.jobs.Lease, every: PT5S}\n";
const TIME: &str = "      - {form: time, command: demo.jobs.RecordSweep, every: PT60S}\n";

/// The specification, or every refusal as its display, one per line.
fn assembled(text: &str) -> Result<ess_domain::Specification, String> {
    let raw = ess_domain::spec::RawSpecFile::parse(text).map_err(|error| error.to_string())?;
    ess_domain::Specification::assemble([(ess_domain::system::Source::new("jobs.yaml"), raw)])
        .map_err(|errors| errors.to_string())
}

fn refused(text: &str) -> String {
    match assembled(text) {
        Ok(_) => panic!("must refuse:\n{text}"),
        Err(errors) => errors,
    }
}

fn with(text: &str, from: &str, to: &str) -> String {
    assert!(text.contains(from), "the model holds `{from}`");
    text.replacen(from, to, 1)
}

/// `true` where one refusal line carries `code` and every fragment.
fn names(errors: &str, code: &str, fragments: &[&str]) -> bool {
    errors.lines().any(|line| {
        line.contains(&format!("[{code}]")) && fragments.iter().all(|it| line.contains(it))
    })
}

#[test]
fn a_component_with_a_level_and_a_time_step_validates_at_ess_38() {
    if let Err(errors) = assembled(JOBS) {
        panic!("{errors}");
    }
}

#[test]
fn below_ess_38_steps_are_refused_naming_the_key_and_the_format() {
    let errors = refused(&with(JOBS, "format: ess/38\n", "format: ess/37\n"));
    assert!(
        names(
            &errors,
            "unsupported_format_version",
            &["`steps`", "ess/38"]
        ),
        "{errors}"
    );
}

#[test]
fn a_step_naming_an_undeclared_view_or_command_is_refused_with_the_declared_names() {
    let errors = refused(&with(
        JOBS,
        "view: demo.jobs.PendingJobs, command: demo.jobs.Lease",
        "view: demo.jobs.OpenJobs, command: demo.jobs.Lease",
    ));
    assert!(
        names(
            &errors,
            "undeclared_reference",
            &["demo.jobs.OpenJobs", "demo.jobs.PendingJobs"]
        ),
        "{errors}"
    );
    let errors = refused(&with(
        JOBS,
        "command: demo.jobs.RecordSweep, every",
        "command: demo.jobs.Sweep, every",
    ));
    assert!(
        names(
            &errors,
            "undeclared_reference",
            &["demo.jobs.Sweep`", "demo.jobs.RecordSweep"]
        ),
        "{errors}"
    );
}

#[test]
fn an_actor_that_may_not_run_the_command_or_read_the_view_is_refused_naming_the_grant() {
    let named = with(
        JOBS,
        "command: demo.jobs.Lease, every",
        "command: demo.jobs.Lease, actor: demo.jobs.Client, every",
    );
    let errors = refused(&named);
    assert!(
        names(
            &errors,
            "conflicting_declaration",
            &["demo.jobs.Client", "may run `demo.jobs.Lease`"]
        ),
        "{errors}"
    );
    let unread = with(
        JOBS,
        "may: [demo.jobs.Lease, demo.jobs.RecordSweep, demo.jobs.PendingJobs]",
        "may: [demo.jobs.Lease, demo.jobs.RecordSweep]",
    );
    let errors = refused(&unread);
    assert!(
        names(
            &errors,
            "conflicting_declaration",
            &["demo.jobs.Worker", "may read `demo.jobs.PendingJobs`"]
        ),
        "{errors}"
    );
}

#[test]
fn a_command_granted_to_no_system_actor_needs_its_actor_named() {
    let open = with(
        JOBS,
        "    may: [demo.jobs.Submit, demo.jobs.PendingJobs]\n",
        "    may: [demo.jobs.Submit, demo.jobs.PendingJobs, demo.jobs.RecordSweep]\n",
    );
    let errors = refused(&open);
    assert!(
        names(
            &errors,
            "missing_declaration",
            &["demo.jobs.RecordSweep", "actor"]
        ),
        "{errors}"
    );
    let named = with(
        &open,
        "command: demo.jobs.RecordSweep, every",
        "command: demo.jobs.RecordSweep, actor: demo.jobs.Client, every",
    );
    if let Err(errors) = assembled(&named) {
        panic!("{errors}");
    }
}

#[test]
fn a_level_step_over_a_command_with_an_input_no_row_field_fills_is_refused_naming_it() {
    let errors = refused(&with(
        JOBS,
        "      - {name: job_id, type: demo.jobs.JobId}\n      - {name: title, type: String}\n",
        "      - {name: title, type: String}\n",
    ));
    assert!(
        names(
            &errors,
            "missing_declaration",
            &["`job_id`", "demo.jobs.PendingJobs"]
        ),
        "{errors}"
    );
}

#[test]
fn a_time_step_needs_a_period_and_reads_it_as_a_period() {
    let errors = refused(&with(
        JOBS,
        TIME,
        "      - {form: time, command: demo.jobs.RecordSweep}\n",
    ));
    assert!(
        names(&errors, "missing_declaration", &["`every`"]),
        "{errors}"
    );
    let errors = refused(&with(
        JOBS,
        TIME,
        "      - {form: time, command: demo.jobs.RecordSweep, every: PT0S}\n",
    ));
    assert!(errors.contains("PT0S"), "{errors}");
    let errors = refused(&with(
        JOBS,
        LEVEL,
        "      - {form: level, command: demo.jobs.Lease}\n",
    ));
    assert!(
        names(&errors, "missing_declaration", &["`view`"]),
        "{errors}"
    );
}
