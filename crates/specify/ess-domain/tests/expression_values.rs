//! What validation decides about an expression value, `{expr: "<expression>"}`, in `sets:`, an
//! event's `payload:` and a computed view field (`ess/27`, `story:expression-arithmetic-and-time`,
//! beyond10x/ess#560, `docs/design/generic-language-features.md` § G1).
//!
//! The expression is typed by the declared types it reads: `Timestamp - Timestamp` is a `Duration`,
//! `/` an `Optional<Decimal>` that a required field takes only through `coalesce(…)`. A position
//! reads its own roots: a command outcome `input.` and, over an existing subject, `subject.`; a
//! view field the source row's fields. Below `ess/27` the mapping keeps the nested-mapping reading
//! it had, and a refusal of that reading names `ess/27`. Below `ess/30` `now` stays refused in a
//! view filter, naming the `from`/`to` parameter idiom; from `ess/30` a view filter reads it.

fn spec(body: &str) -> Result<ess_domain::Specification, String> {
    let raw = ess_domain::spec::RawSpecFile::parse(body).map_err(|e| e.to_string())?;
    ess_domain::Specification::assemble([(ess_domain::system::Source::new("ledger.yaml"), raw)])
        .map_err(|e| e.to_string())
}

fn refused(body: &str) -> String {
    match spec(body) {
        Ok(_) => panic!("the document validates:\n{body}"),
        Err(errors) => errors,
    }
}

/// The incident ledger at `format`, with `sets:` entries, payload entries, a view field line and a
/// view filter line as given.
fn model(format: &str, sets: &str, payload: &str, view_field: &str, filter: &str) -> String {
    format!(
        "format: {format}
system: demo
version: v1
domain: demo.ledger
types:
  - {{name: demo.ledger.IncidentId, kind: newtype, of: Uuid}}
entities:
  - name: demo.ledger.Incident
    identity: {{name: incident_id, type: demo.ledger.IncidentId}}
    fields:
      - {{name: started_at, type: Timestamp}}
      - {{name: ended_at, type: Timestamp}}
      - {{name: duration, type: Optional<Duration>}}
      - {{name: lag, type: Optional<Integer>}}
      - {{name: affected, type: Integer}}
      - {{name: total, type: Integer}}
      - {{name: share, type: Optional<Decimal>}}
      - {{name: note, type: Optional<String>}}
    lifecycle:
      initial: Recorded
      states: [Recorded, Closed]
      terminal: [Closed]
      transitions:
        - {{name: close, from: [Recorded], to: Closed}}
events:
  - name: demo.ledger.IncidentRecorded
    fields:
      - {{name: incident_id, type: demo.ledger.IncidentId}}
      - {{name: duration, type: Duration}}
  - name: demo.ledger.IncidentClosed
    fields:
      - {{name: incident_id, type: demo.ledger.IncidentId}}
commands:
  - name: demo.ledger.RecordPastIncident
    input:
      - {{name: started_at, type: Timestamp}}
      - {{name: ended_at, type: Timestamp}}
      - {{name: affected, type: Integer}}
      - {{name: total, type: Integer}}
      - {{name: code, type: String}}
    outcomes:
      - name: recorded
        creates: demo.ledger.Incident
        instance: incident_id
        emits: [demo.ledger.IncidentRecorded]
        payload:
          demo.ledger.IncidentRecorded:
            incident_id: {{generated: true}}
{payload}        sets:
          started_at: input.started_at
          ended_at: input.ended_at
          affected: input.affected
          total: input.total
{sets}  - name: demo.ledger.CloseIncident
    input:
      - {{name: incident_id, type: demo.ledger.IncidentId}}
      - {{name: closed_at, type: Timestamp}}
    outcomes:
      - name: closed
        moves: demo.ledger.Incident.close
        instance: incident_id
        emits: [demo.ledger.IncidentClosed]
        payload:
          demo.ledger.IncidentClosed: {{incident_id: input.incident_id}}
        sets:
          duration: {{expr: \"input.closed_at - subject.started_at\"}}
views:
  - name: demo.ledger.Incidents
    source: demo.ledger.Incident
{filter}    fields:
      - {{name: incident_id, type: demo.ledger.IncidentId}}
{view_field}"
    )
}

const DURATION: &str = "            duration: {expr: \"input.ended_at - input.started_at\"}\n";

fn sets(line: &str) -> String {
    format!("          {line}\n")
}

#[test]
fn the_issue_shapes_validate_at_ess_27() {
    let body = model(
        "ess/27",
        &sets("share: {expr: \"coalesce(round(input.affected / input.total, 2, half_even), 0)\"}"),
        DURATION,
        "      - {name: elapsed, type: Duration, expr: \"ended_at - started_at\"}\n",
        "",
    );
    spec(&body).unwrap_or_else(|errors| panic!("{errors}"));
}

#[test]
fn an_expression_below_ess_27_is_refused_naming_it() {
    let body = model("ess/26", "", DURATION, "", "");
    let errors = refused(&body);
    assert!(
        errors.contains("unsupported_format_version") && errors.contains("ess/27"),
        "{errors}"
    );
    let view = model(
        "ess/26",
        "",
        "",
        "      - {name: elapsed, type: Duration, expr: \"ended_at - started_at\"}\n",
        "",
    );
    let errors = refused(&view);
    assert!(
        errors.contains("unsupported_format_version") && errors.contains("ess/27"),
        "{errors}"
    );
}

#[test]
fn an_expression_of_another_type_than_its_field_is_refused() {
    let body = model(
        "ess/27",
        &sets("lag: {expr: \"input.ended_at - input.started_at\"}"),
        DURATION,
        "",
        "",
    );
    let errors = refused(&body);
    assert!(
        errors.contains("type_mismatch") && errors.contains("Duration") && errors.contains("lag"),
        "{errors}"
    );
}

#[test]
fn a_quotient_into_a_required_field_is_refused_naming_coalesce() {
    let body = model(
        "ess/27",
        "",
        "            duration: {expr: \"input.ended_at - input.started_at\"}\n",
        "",
        "",
    )
    .replace(
        "      - {name: share, type: Optional<Decimal>}",
        "      - {name: share, type: Decimal}",
    )
    .replace(
        "          total: input.total\n",
        "          total: input.total\n          share: {expr: \"input.affected / input.total\"}\n",
    );
    let errors = refused(&body);
    assert!(
        errors.contains("type_mismatch") && errors.contains("coalesce"),
        "{errors}"
    );
}

#[test]
fn a_misspelled_or_unread_root_is_refused_by_name() {
    for (line, says) in [
        ("note: {expr: \"input.missing + \\\"x\\\"\"}", "missing"),
        ("note: {expr: \"subject.note\"}", "subject"),
        ("note: {expr: \"started_at\"}", "input."),
        (
            "lag: {expr: \"input.affected * 2 + len(input.code)\"}",
            "len",
        ),
    ] {
        let body = model("ess/27", &sets(line), DURATION, "", "");
        let errors = refused(&body);
        assert!(errors.contains(says), "{line}: {errors}");
    }
}

#[test]
fn a_value_read_at_the_current_time_is_refused_by_name() {
    let body = model(
        "ess/27",
        &sets("duration: {expr: \"now - input.started_at\"}"),
        DURATION,
        "",
        "",
    );
    let errors = refused(&body);
    assert!(errors.contains("now"), "{errors}");
}

#[test]
fn a_computed_view_field_reads_its_source_row_and_is_typed() {
    let unknown = model(
        "ess/27",
        "",
        DURATION,
        "      - {name: elapsed, type: Duration, expr: \"ended_at - opened_at\"}\n",
        "",
    );
    assert!(refused(&unknown).contains("opened_at"));
    let mistyped = model(
        "ess/27",
        "",
        DURATION,
        "      - {name: elapsed, type: Integer, expr: \"ended_at - started_at\"}\n",
        "",
    );
    let errors = refused(&mistyped);
    assert!(
        errors.contains("type_mismatch") && errors.contains("Duration"),
        "{errors}"
    );
}

/// `story:expression-arithmetic-and-time`, `now_relative_view_filter`: below `ess/30` a view read
/// has no clock a suite sets, so `now` in a view filter stays refused, and the refusal names the
/// idiom that keeps the bound in the specification: `from`/`to` parameters the caller sends.
#[test]
fn now_relative_view_filter() {
    for format in ["ess/27", "ess/29"] {
        let body = model(
            format,
            "",
            DURATION,
            "",
            "    filter: \"started_at >= now - 720h\"\n",
        );
        let errors = refused(&body);
        assert!(
            errors.contains("view filter")
                && errors.contains("param.from")
                && errors.contains("to"),
            "{format}: {errors}"
        );
    }
}

/// `story:expression-arithmetic-and-time`, `now_relative_view_filter`: from `ess/30` a view filter
/// reads `now`, the instant of the view read, as a lower or an upper bound on a `Timestamp` field.
#[test]
fn now_relative_view_filter_is_admitted_from_ess_30() {
    for filter in [
        "    filter: \"started_at >= now - 720h\"\n",
        "    filter: {all: [\"started_at >= now - 720h\", \"ended_at < now\"]}\n",
    ] {
        let body = model("ess/30", "", DURATION, "", filter);
        if let Err(errors) = spec(&body) {
            panic!("{filter}: {errors}");
        }
    }
}
