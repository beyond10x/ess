//! The authenticated caller as a value source and a guard operand (source format `ess/16`,
//! beyond10x/ess#168, `docs/design/caller-values.md`).
//!
//! An actor declares `attributes:` that come with the caller's credential; `{caller: <attribute>}`
//! reads one in `payload:` and `sets:`, and `caller.<attribute>` compares one with an input field in
//! `when:` or with a stored field in `when_subject:`.

fn spec(body: &str) -> Result<ess_domain::Specification, String> {
    let raw = ess_domain::spec::RawSpecFile::parse(body).map_err(|e| e.to_string())?;
    ess_domain::Specification::assemble([(ess_domain::system::Source::new("notes.yaml"), raw)])
        .map_err(|e| e.to_string())
}

fn accepted(body: &str) -> ess_domain::Specification {
    spec(body).unwrap_or_else(|errors| panic!("must compile:\n{errors}\n{body}"))
}

fn refused(body: &str, expected: &[&str]) {
    let error = spec(body)
        .err()
        .unwrap_or_else(|| panic!("must not compile:\n{body}"));
    for needle in expected {
        assert!(error.contains(needle), "expected {needle:?} in:\n{error}");
    }
}

/// The #168 repro, grown by the command that refuses a caller who is not the note's agent.
///
/// `{format}` is the header, `{actors}` the `actors:` block, `{created}` the `sets:` of `created`,
/// `{payload}` the `account_id` source of `NoteCreated`, `{guard}` the refusal's condition and
/// `{edit_input}` extra inputs of `EditNote`.
struct Notes<'a> {
    format: &'a str,
    actors: &'a str,
    created: &'a str,
    payload: &'a str,
    guard: &'a str,
    edit_input: &'a str,
}

const ACTORS: &str = "  - name: demo.notes.AccountUser
    attributes:
      - {name: account_id, type: demo.notes.AccountId}
      - {name: agent_id, type: demo.notes.AgentId}
    may: [demo.notes.CreateNote, demo.notes.EditNote]
";

impl Default for Notes<'_> {
    fn default() -> Self {
        Self {
            format: "ess/16",
            actors: ACTORS,
            created:
                "{account_id: {caller: account_id}, agent_id: {caller: agent_id}, text: input.text}",
            payload: "{caller: account_id}",
            guard: "when_subject: {predicate: agent_id != caller.agent_id}",
            edit_input: "",
        }
    }
}

impl Notes<'_> {
    fn render(&self) -> String {
        let Notes {
            format,
            actors,
            created,
            payload,
            guard,
            edit_input,
        } = self;
        format!(
            "format: {format}
system: demo
version: v1
summary: Minimal repro.
domains: [demo.notes]
domain: demo.notes
types:
  - {{name: demo.notes.NoteId, kind: newtype, of: Uuid}}
  - {{name: demo.notes.AccountId, kind: newtype, of: Uuid}}
  - {{name: demo.notes.AgentId, kind: newtype, of: Uuid}}
entities:
  - name: demo.notes.Note
    identity: {{name: note_id, type: demo.notes.NoteId}}
    fields:
      - {{name: account_id, type: demo.notes.AccountId}}
      - {{name: agent_id, type: demo.notes.AgentId}}
      - {{name: text, type: String}}
    lifecycle: {{initial: Open, states: [Open], terminal: [Open], transitions: []}}
actors:
{actors}
commands:
  - name: demo.notes.CreateNote
    input:
      - {{name: text, type: String}}
    outcomes:
      - name: created
        creates: demo.notes.Note
        instance: note_id
        sets: {created}
        emits: [demo.notes.NoteCreated]
        payload:
          demo.notes.NoteCreated: {{note_id: {{generated: true}}, account_id: {payload}, text: input.text}}
  - name: demo.notes.EditNote
    input:
      - {{name: note_id, type: demo.notes.NoteId}}
      - {{name: text, type: String}}
{edit_input}
    outcomes:
      - name: forbidden
        {guard}
        error: demo.notes.NotYourNote
      - name: edited
        updates: demo.notes.Note
        instance: note_id
        sets: {{text: input.text}}
        emits: [demo.notes.NoteEdited]
        payload:
          demo.notes.NoteEdited: {{note_id: input.note_id, text: input.text}}
errors:
  - name: demo.notes.NotYourNote
    summary: The caller is not the note's agent.
events:
  - name: demo.notes.NoteCreated
    fields:
      - {{name: note_id, type: demo.notes.NoteId}}
      - {{name: account_id, type: demo.notes.AccountId}}
      - {{name: text, type: String}}
  - name: demo.notes.NoteEdited
    fields:
      - {{name: note_id, type: demo.notes.NoteId}}
      - {{name: text, type: String}}
views:
  - name: demo.notes.NoteDetails
    source: demo.notes.Note
    consistency: read_your_writes
    fields:
      - {{name: note_id, type: demo.notes.NoteId}}
      - {{name: account_id, type: demo.notes.AccountId}}
      - {{name: agent_id, type: demo.notes.AgentId}}
      - {{name: text, type: String}}
"
        )
    }
}

#[test]
fn the_repro_validates_at_ess_16() {
    let spec = accepted(&Notes::default().render());
    let actor = spec
        .actors()
        .values()
        .next()
        .expect("the repro declares one actor");
    let names: Vec<&str> = actor
        .attributes
        .iter()
        .map(|field| field.name.as_str())
        .collect();
    assert_eq!(names, ["account_id", "agent_id"]);
    let created = &spec.commands().values().next().unwrap().outcomes[0];
    assert_eq!(
        created.sets.get("account_id"),
        Some(&ess_domain::command::PayloadSource::CallerAttribute {
            attribute: "account_id".to_owned()
        }),
        "`{{caller: account_id}}` is a source, not a nested mapping"
    );
}

#[test]
fn attributes_below_ess_16_are_refused_by_format() {
    refused(
        &Notes {
            format: "ess/15",
            created: "{account_id: input.text, agent_id: input.text, text: input.text}",
            payload: "input.text",
            guard: "when: text == \"\"",
            ..Notes::default()
        }
        .render(),
        &["unsupported_format_version", "attributes", "ess/16"],
    );
}

#[test]
fn a_caller_source_below_ess_16_stays_the_nested_mapping_it_was() {
    // No `attributes:` and the old format: `{caller: account_id}` over a newtype is a nested
    // mapping, refused as the issue quotes it — not read as a caller.
    refused(
        &Notes {
            format: "ess/15",
            actors: "  - {name: demo.notes.AccountUser, may: [demo.notes.CreateNote, demo.notes.EditNote]}\n",
            created: "{account_id: {caller: account_id}, agent_id: {generated: true}, text: input.text}",
            payload: "{generated: true}",
            guard: "when: text == \"\"",
            ..Notes::default()
        }
        .render(),
        &["type_mismatch", "a nested mapping fills a struct's fields"],
    );
}

#[test]
fn a_caller_guard_below_ess_16_is_refused_by_format() {
    refused(
        &Notes {
            format: "ess/15",
            actors: "  - {name: demo.notes.AccountUser, may: [demo.notes.CreateNote, demo.notes.EditNote]}\n",
            created: "{account_id: {generated: true}, agent_id: {generated: true}, text: input.text}",
            payload: "{generated: true}",
            ..Notes::default()
        }
        .render(),
        &["unsupported_format_version", "caller.", "ess/16"],
    );
}

#[test]
fn an_attribute_no_actor_declares_is_refused() {
    refused(
        &Notes {
            payload: "{caller: tenant_id}",
            ..Notes::default()
        }
        .render(),
        &[
            "undeclared_reference",
            "tenant_id",
            "demo.notes.AccountUser",
        ],
    );
}

#[test]
fn a_caller_attribute_of_another_type_is_refused() {
    refused(
        &Notes {
            created:
                "{account_id: {caller: agent_id}, agent_id: {caller: agent_id}, text: input.text}",
            ..Notes::default()
        }
        .render(),
        &["type_mismatch", "agent_id"],
    );
}

#[test]
fn mixing_actors_with_and_without_the_attribute_is_refused_where_it_is_read() {
    let actors = format!("{ACTORS}  - {{name: demo.notes.Robot, may: [demo.notes.CreateNote]}}\n");
    refused(
        &Notes {
            actors: &actors,
            ..Notes::default()
        }
        .render(),
        &[
            "conflicting_declaration",
            "demo.notes.Robot",
            "account_id",
            "CreateNote",
        ],
    );
}

#[test]
fn a_command_no_actor_may_invoke_has_no_caller_to_read() {
    refused(
        &Notes {
            actors: "  - name: demo.notes.AccountUser\n    attributes:\n      - {name: account_id, type: demo.notes.AccountId}\n      - {name: agent_id, type: demo.notes.AgentId}\n    may: [demo.notes.EditNote]\n",
            ..Notes::default()
        }
        .render(),
        &["undeclared_reference", "no actor may invoke", "CreateNote"],
    );
}

#[test]
fn a_when_guard_compares_the_caller_with_an_input_field() {
    accepted(
        &Notes {
            guard: "when: input_account != caller.account_id",
            edit_input: "      - {name: input_account, type: demo.notes.AccountId}\n",
            ..Notes::default()
        }
        .render(),
    );
}

#[test]
fn a_when_guard_comparing_the_caller_with_another_type_is_refused() {
    refused(
        &Notes {
            guard: "when: text == caller.account_id",
            ..Notes::default()
        }
        .render(),
        &["type_mismatch"],
    );
}

#[test]
fn a_caller_operand_is_compared_with_a_field_and_nothing_else() {
    refused(
        &Notes {
            guard: "when: caller.account_id == \"x\"",
            ..Notes::default()
        }
        .render(),
        &["caller.account_id", "compared"],
    );
}

#[test]
fn a_guard_reading_an_undeclared_attribute_is_refused() {
    refused(
        &Notes {
            guard: "when_subject: {predicate: agent_id != caller.owner_id}",
            ..Notes::default()
        }
        .render(),
        &["owner_id"],
    );
}

#[test]
fn an_input_named_caller_keeps_being_read_as_the_input() {
    // `caller.<member>` under a struct input named `caller` is the input's member, as it was.
    let body = Notes {
        guard: "when: caller.flag == \"on\"",
        edit_input: "      - {name: caller, type: demo.notes.Flags}\n",
        ..Notes::default()
    }
    .render()
    .replace(
        "  - {name: demo.notes.AgentId, kind: newtype, of: Uuid}\n",
        "  - {name: demo.notes.AgentId, kind: newtype, of: Uuid}\n  - {name: demo.notes.Flags, kind: struct, fields: [{name: flag, type: String}]}\n",
    );
    accepted(&body);
}
