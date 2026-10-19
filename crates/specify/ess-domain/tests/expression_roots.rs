//! A value reads what its branch established (`ess/27`,
//! `story:expression-roots-a-branch-establishes-are-readable`, beyond10x/ess#555, #556,
//! `docs/design/generic-language-features.md` § G1, "Narrowing").
//!
//! A refusal guarded by a related row the addressed record's stored reference names has read that
//! record, so its payload reads the record's stored fields. A value its own guard proves present
//! reads as the inner type: an `Optional` input in a `when_related:` branch over it (an absent
//! reference selects no such branch), a stored `Optional` field under `defined(field)` in
//! `when_subject:`, and `{related: {via, field}}` under `when_related: {predicate:
//! defined(field)}`. Everything else keeps today's refusal, and below `ess/27` so do these.

fn spec(body: &str) -> Result<ess_domain::Specification, String> {
    let raw = ess_domain::spec::RawSpecFile::parse(body).map_err(|e| e.to_string())?;
    ess_domain::Specification::assemble([(ess_domain::system::Source::new("review.yaml"), raw)])
        .map_err(|e| e.to_string())
}

fn refused(body: &str) -> String {
    match spec(body) {
        Ok(_) => panic!("the document validates:\n{body}"),
        Err(errors) => errors,
    }
}

/// The review model at `format`, with each command's outcomes as given.
#[allow(clippy::too_many_lines, reason = "the whole model, written out once")]
fn model(format: &str, close: &str, complain: &str, settle: &str) -> String {
    format!(
        "format: {format}
system: demo
version: v1
domain: demo.review
types:
  - {{name: demo.review.ReviewId, kind: newtype, of: Uuid}}
  - {{name: demo.review.IncidentId, kind: newtype, of: Uuid}}
  - {{name: demo.review.CustomerId, kind: newtype, of: Uuid}}
  - {{name: demo.review.ComplaintId, kind: newtype, of: Uuid}}
entities:
  - name: demo.review.Incident
    identity: {{name: incident_id, type: demo.review.IncidentId}}
    fields:
      - {{name: approved_by, type: Optional<demo.review.ReviewId>}}
    lifecycle:
      initial: Detected
      states: [Detected, Settled]
      terminal: [Settled]
      transitions:
        - {{name: settle, from: [Detected], to: Settled}}
  - name: demo.review.Customer
    identity: {{name: customer_id, type: demo.review.CustomerId}}
    fields: []
    lifecycle: {{initial: Active, states: [Active], terminal: [Active], transitions: []}}
  - name: demo.review.Review
    identity: {{name: review_id, type: demo.review.ReviewId}}
    fields:
      - {{name: incident_id, type: demo.review.IncidentId}}
    lifecycle:
      initial: Open
      states: [Open, Closed]
      terminal: [Closed]
      transitions:
        - {{name: close, from: [Open], to: Closed}}
  - name: demo.review.Complaint
    identity: {{name: complaint_id, type: demo.review.ComplaintId}}
    fields:
      - {{name: customer_id, type: Optional<demo.review.CustomerId>}}
    lifecycle: {{initial: Filed, states: [Filed], terminal: [Filed], transitions: []}}
errors:
  - {{name: demo.review.IncidentMissing, summary: No incident carries the identity., fields: []}}
  - name: demo.review.IncidentNotSettled
    summary: The incident is not settled.
    fields:
      - {{name: incident_id, type: demo.review.IncidentId}}
  - name: demo.review.NoSuchCustomer
    summary: No customer carries the identity.
    fields:
      - {{name: customer_id, type: demo.review.CustomerId}}
  - name: demo.review.IncidentLocked
    summary: The incident is approved.
    fields:
      - {{name: approved_by, type: demo.review.ReviewId}}
events:
  - name: demo.review.IncidentOpened
    fields:
      - {{name: incident_id, type: demo.review.IncidentId}}
  - name: demo.review.IncidentSettled
    fields:
      - {{name: incident_id, type: demo.review.IncidentId}}
  - name: demo.review.ReviewOpened
    fields:
      - {{name: review_id, type: demo.review.ReviewId}}
  - name: demo.review.ReviewClosed
    fields:
      - {{name: review_id, type: demo.review.ReviewId}}
  - name: demo.review.ComplaintFiled
    fields:
      - {{name: complaint_id, type: demo.review.ComplaintId}}
  - name: demo.review.CustomerJoined
    fields:
      - {{name: customer_id, type: demo.review.CustomerId}}
commands:
  - name: demo.review.OpenIncident
    input:
      - {{name: incident_id, type: demo.review.IncidentId}}
      - {{name: approved_by, type: Optional<demo.review.ReviewId>}}
    outcomes:
      - name: opened
        creates: demo.review.Incident
        instance: incident_id
        sets: {{approved_by: input.approved_by}}
        emits: [demo.review.IncidentOpened]
        payload:
          demo.review.IncidentOpened: {{incident_id: input.incident_id}}
  - name: demo.review.JoinCustomer
    input:
      - {{name: customer_id, type: demo.review.CustomerId}}
    outcomes:
      - name: joined
        creates: demo.review.Customer
        instance: customer_id
        emits: [demo.review.CustomerJoined]
        payload:
          demo.review.CustomerJoined: {{customer_id: input.customer_id}}
  - name: demo.review.OpenReview
    input:
      - {{name: review_id, type: demo.review.ReviewId}}
      - {{name: incident_id, type: demo.review.IncidentId}}
    outcomes:
      - name: opened
        creates: demo.review.Review
        instance: review_id
        sets: {{incident_id: input.incident_id}}
        emits: [demo.review.ReviewOpened]
        payload:
          demo.review.ReviewOpened: {{review_id: input.review_id}}
  - name: demo.review.CloseReview
    input:
      - {{name: review_id, type: demo.review.ReviewId}}
    outcomes:
{close}      - name: closed
        moves: demo.review.Review.close
        instance: review_id
        emits: [demo.review.ReviewClosed]
        payload:
          demo.review.ReviewClosed: {{review_id: input.review_id}}
  - name: demo.review.FileComplaint
    input:
      - {{name: complaint_id, type: demo.review.ComplaintId}}
      - {{name: customer_id, type: Optional<demo.review.CustomerId>}}
    outcomes:
{complain}      - name: filed
        creates: demo.review.Complaint
        instance: complaint_id
        sets: {{customer_id: input.customer_id}}
        emits: [demo.review.ComplaintFiled]
        payload:
          demo.review.ComplaintFiled: {{complaint_id: input.complaint_id}}
  - name: demo.review.SettleIncident
    input:
      - {{name: incident_id, type: demo.review.IncidentId}}
    outcomes:
{settle}      - name: settled
        moves: demo.review.Incident.settle
        instance: incident_id
        emits: [demo.review.IncidentSettled]
        payload:
          demo.review.IncidentSettled: {{incident_id: input.incident_id}}
"
    )
}

/// beyond10x/ess#555: the refusal reads the review's stored reference.
const NOT_SETTLED: &str = "      - name: missing
        when_related: {via: incident_id, exists: false}
        error: demo.review.IncidentMissing
      - name: not-settled
        when_related: {via: incident_id, predicate: state == Detected}
        error: demo.review.IncidentNotSettled
        payload:
          demo.review.IncidentNotSettled: {incident_id: {subject: incident_id}}
";

/// beyond10x/ess#556 (a): the Optional input its own `exists: false` branch reads.
const NO_CUSTOMER: &str = "      - name: no-customer
        when_related: {via: input.customer_id, exists: false}
        error: demo.review.NoSuchCustomer
        payload:
          demo.review.NoSuchCustomer: {customer_id: input.customer_id}
";

/// beyond10x/ess#556 (b): the stored Optional its own `defined()` guard reads.
const SEALED: &str = "      - name: sealed
        when_subject: {predicate: defined(approved_by)}
        error: demo.review.IncidentLocked
        payload:
          demo.review.IncidentLocked: {approved_by: {subject: approved_by}}
";

/// The `{related:}` sibling of (b): the related row's Optional field under `defined()`.
const LOCKED: &str = "      - name: missing
        when_related: {via: incident_id, exists: false}
        error: demo.review.IncidentMissing
      - name: locked
        when_related: {via: incident_id, predicate: defined(approved_by)}
        error: demo.review.IncidentLocked
        payload:
          demo.review.IncidentLocked: {approved_by: {related: {via: incident_id, field: approved_by}}}
";

#[test]
fn subject_field_readable_on_a_refusal_after_existence() {
    spec(&model("ess/27", NOT_SETTLED, "", "")).unwrap_or_else(|errors| panic!("{errors}"));
}

#[test]
fn optional_input_narrowed_in_its_own_exists_false_branch() {
    let specification =
        spec(&model("ess/27", "", NO_CUSTOMER, "")).unwrap_or_else(|errors| panic!("{errors}"));
    let error = specification
        .errors()
        .values()
        .find(|error| error.name.to_string() == "demo.review.NoSuchCustomer")
        .expect("the error is declared");
    assert_eq!(
        error.fields[0].type_ref.to_string(),
        "demo.review.CustomerId",
        "the error field is the inner type, not Optional"
    );
}

#[test]
fn stored_optional_narrowed_under_defined() {
    spec(&model("ess/27", "", "", SEALED)).unwrap_or_else(|errors| panic!("{errors}"));
    spec(&model("ess/27", LOCKED, "", "")).unwrap_or_else(|errors| panic!("{errors}"));
}

#[test]
fn unnarrowed_reads_stay_refused() {
    // The same reads on a branch whose guard does not establish them.
    let unguarded_input = NO_CUSTOMER.replace(
        "when_related: {via: input.customer_id, exists: false}",
        "when: complaint_id == \"x\"",
    );
    let errors = refused(&model("ess/27", "", &unguarded_input, ""));
    assert!(
        errors.contains("has type `Optional<demo.review.CustomerId>`")
            && errors.contains("no conversion is declared"),
        "{errors}"
    );
    let unguarded_stored = SEALED.replace("defined(approved_by)", "not defined(approved_by)");
    let errors = refused(&model("ess/27", "", "", &unguarded_stored));
    assert!(
        errors.contains("Optional<demo.review.ReviewId>"),
        "{errors}"
    );
    let unguarded_related = LOCKED.replace("defined(approved_by)", "state == Detected");
    let errors = refused(&model("ess/27", &unguarded_related, "", ""));
    assert!(
        errors.contains("Optional<demo.review.ReviewId>"),
        "{errors}"
    );
    // A refusal answered before any row is read still reads no subject.
    let before_row = NOT_SETTLED.replace(
        "when_related: {via: incident_id, predicate: state == Detected}",
        "when: review_id == \"x\"",
    );
    let errors = refused(&model("ess/27", &before_row, "", ""));
    assert!(errors.contains("acts on no entity"), "{errors}");
}

#[test]
fn below_ess_27_each_read_keeps_its_refusal() {
    for (close, complain, settle) in [
        (NOT_SETTLED, "", ""),
        ("", NO_CUSTOMER, ""),
        ("", "", SEALED),
        (LOCKED, "", ""),
    ] {
        let errors = refused(&model("ess/26", close, complain, settle));
        assert!(errors.contains("ess/27"), "{errors}");
    }
}
