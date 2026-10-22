//! `sets: {field: []}` — a branch storing the empty list in a required `List<…>` field (`ess/31`).
//!
//! A required list field a view publishes had no writer a creating command could name: a list
//! literal in `sets:` was refused, so the row was created without the field and the view could not
//! be filled. From `ess/31` exactly the empty list is a `sets:` source over a `List<…>` field.
//! Below `ess/31` it is refused naming `ess/31`; a list holding anything stays refused at every
//! format, and so does `[]` over a field that is no list.

fn spec(body: &str) -> Result<ess_domain::Specification, String> {
    let raw = ess_domain::spec::RawSpecFile::parse(body).map_err(|e| e.to_string())?;
    ess_domain::Specification::assemble([(ess_domain::system::Source::new("jobs.yaml"), raw)])
        .map_err(|e| e.to_string())
}

/// A job holding a required list of tags, created by a command that sets `target` to `value`.
fn model(format: &str, target: &str, value: &str) -> String {
    format!(
        "format: {format}
system: demo
version: v1
domain: demo.jobs
types:
  - {{name: demo.jobs.JobId, kind: newtype, of: Uuid}}
entities:
  - name: demo.jobs.Job
    identity: {{name: job_id, type: demo.jobs.JobId}}
    fields:
      - {{name: label, type: String}}
      - {{name: title, type: String}}
      - {{name: tags, type: List<String>}}
      - {{name: notes, type: Optional<List<String>>}}
    lifecycle: {{initial: Open, states: [Open], terminal: [Open], transitions: []}}
events:
  - name: demo.jobs.JobAdded
    fields: [{{name: job_id, type: demo.jobs.JobId}}]
commands:
  - name: demo.jobs.AddJob
    input: [{{name: label, type: String}}]
    outcomes:
      - name: added
        creates: demo.jobs.Job
        instance: job_id
        sets:
          label: input.label
          {target}: {value}
        emits: [demo.jobs.JobAdded]
        payload: {{demo.jobs.JobAdded: {{job_id: {{generated: true}}}}}}
"
    )
}

#[test]
fn an_empty_list_literal_sets_a_required_list() {
    let parsed = spec(&model("ess/31", "tags", "[]")).expect("`[]` fills a required list");
    let command = parsed
        .commands()
        .values()
        .find(|c| c.name.to_string() == "demo.jobs.AddJob")
        .expect("the command");
    let source = &command.outcomes[0].sets["tags"];
    assert_eq!(
        format!("{source:?}"),
        "EmptyList",
        "the source is the empty list"
    );
    let encoded = ess_yaml::to_string(command).unwrap();
    assert!(encoded.contains("tags: []"), "{encoded}");
    let raw: ess_domain::command::RawCommandSpec = ess_yaml::from_str(&encoded).unwrap();
    let roundtrip = ess_domain::command::CommandSpec::try_from(raw).unwrap();
    assert_eq!(&roundtrip, command, "the construct survives a round trip");
}

#[test]
fn an_empty_list_literal_below_ess_31_names_ess_31() {
    for format in ["ess/1", "ess/27", "ess/30"] {
        let error = spec(&model(format, "tags", "[]"))
            .expect_err("below ess/31 the empty list is no source");
        for required in ["unsupported_format_version", "tags", "ess/31"] {
            assert!(
                error.contains(required),
                "{format}: {required:?} missing from:\n{error}"
            );
        }
    }
}

#[test]
fn an_empty_list_literal_on_a_non_list_field_is_refused() {
    for (target, held) in [("title", "String"), ("notes", "Optional<List<String>>")] {
        let error = spec(&model("ess/31", target, "[]"))
            .expect_err("only a required list holds the empty list");
        for required in [
            "type_mismatch",
            &format!("demo.jobs.Job.{target}"),
            held,
            "List<",
        ] {
            assert!(
                error.contains(required),
                "{target}: {required:?} missing from:\n{error}"
            );
        }
    }
}

#[test]
fn a_non_empty_list_literal_stays_refused() {
    for format in ["ess/30", "ess/31"] {
        for value in ["[urgent]", "[a, b]"] {
            let error = spec(&model(format, "tags", value))
                .expect_err("a list holding anything is no source");
            assert!(
                error.contains("a list is read only as the `via:`"),
                "{format} {value}: the refusal is the one a list always had:\n{error}"
            );
        }
    }
}

#[test]
fn an_empty_list_literal_in_an_event_payload_is_refused() {
    let text = model("ess/31", "tags", "[]").replace(
        "{demo.jobs.JobAdded: {job_id: {generated: true}}}",
        "{demo.jobs.JobAdded: {job_id: []}}",
    );
    let error = spec(&text).expect_err("`[]` is a `sets:` source only");
    for required in ["job_id", "[]", "sets:"] {
        assert!(
            error.contains(required),
            "{required:?} missing from:\n{error}"
        );
    }
}
