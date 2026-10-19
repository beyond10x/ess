//! `shorthand: <field>` through the `ess` binary (`ess/27`, beyond10x/ess#594): the projected JSON
//! Schema of a struct with a shorthand is `oneOf` the field's schema and the closed object, and
//! `ess generate schema validate` against it admits the short and the long form and refuses a
//! value of any other JSON type.

use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::{json, Value};

const MODEL: &str = "\
format: ess/27
system: tracker
version: v1
summary: A tracker whose operations may be written by name alone.
domain: tracker.ops
types:
  - name: tracker.ops.Operation
    kind: struct
    shorthand: op
    fields:
      - name: op
        type: String
      - name: paging
        type: Optional<String>
actors:
  - name: tracker.ops.Operator
    may:
      - tracker.ops.Run
commands:
  - name: tracker.ops.Run
    input:
      - name: operation
        type: tracker.ops.Operation
    response:
      - name: ticket
        type: String
    outcomes:
      - name: ran
        summary: The operation runs.
        returns: true
components:
  - component: ops
    owns: {domains: [tracker.ops]}
    accepts: {commands: [tracker.ops.Run]}
    reached_by: network
";

/// The `$id` the test gives the projected command schema: the projection writes none of its own
/// yet, and the registry `ess generate schema validate` reads is keyed by `$id`.
const PROJECTED_ID: &str = "urn:ess-test:tracker.ops.Run";

/// The `$id` of the one-property envelope an instance selects with its `schema` member, whose
/// `value` is the projected `Operation` definition.
const ENVELOPE_ID: &str = "urn:ess-test:operation";

fn scratch() -> PathBuf {
    let directory = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("shorthand-schema-{}", std::process::id()));
    if directory.exists() {
        std::fs::remove_dir_all(&directory).expect("a stale scratch directory is removed");
    }
    std::fs::create_dir_all(directory.join("model")).expect("a scratch directory");
    directory
}

fn ess(directory: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_ess"))
        .current_dir(directory)
        .args(args)
        .output()
        .expect("the `ess` binary runs")
}

fn text(output: &std::process::Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

#[test]
fn schema_projects_shorthand_as_one_of_scalar_and_object() {
    let directory = scratch();
    std::fs::write(directory.join("model/system.yaml"), MODEL).expect("the model is written");
    let generated = ess(
        &directory,
        &[
            "generate", "--path", "model", "--kind", "schema", "--out", "out",
        ],
    );
    assert!(generated.status.success(), "{}", text(&generated));
    let path = directory.join("out/schema/commands/tracker.ops.Run.schema.json");
    let mut projected: Value =
        serde_json::from_str(&std::fs::read_to_string(&path).expect("the projected schema"))
            .expect("the projected schema is JSON");

    let definition = &projected["$defs"]["tracker.ops.Operation"];
    assert_eq!(definition["x-ess-shorthand"], json!("op"), "{definition}");
    assert_eq!(
        definition["oneOf"][0],
        json!({"type": "string"}),
        "{definition}"
    );
    assert_eq!(
        definition["oneOf"][1]["type"],
        json!("object"),
        "{definition}"
    );
    assert_eq!(
        definition["oneOf"][1]["additionalProperties"],
        json!(false),
        "the long form is the closed object: {definition}"
    );

    let registry = directory.join("registry");
    std::fs::create_dir_all(&registry).expect("a registry directory");
    projected["$id"] = json!(PROJECTED_ID);
    std::fs::write(
        registry.join("run.schema.json"),
        serde_json::to_string_pretty(&projected).expect("JSON"),
    )
    .expect("the projected schema is written");
    let envelope = json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": ENVELOPE_ID,
        "type": "object",
        "properties": {
            "schema": {"const": ENVELOPE_ID},
            "value": {"$ref": format!("{PROJECTED_ID}#/$defs/tracker.ops.Operation")},
        },
        "required": ["schema", "value"],
        "additionalProperties": false,
    });
    std::fs::write(
        registry.join("envelope.schema.json"),
        serde_json::to_string_pretty(&envelope).expect("JSON"),
    )
    .expect("the envelope is written");

    let cases = [
        ("short", json!("getIssue"), true),
        ("long", json!({"op": "getIssue", "paging": "token"}), true),
        ("number", json!(7), false),
        ("list", json!(["getIssue"]), false),
    ];
    for (label, value, admitted) in cases {
        let instance = directory.join(format!("{label}.json"));
        std::fs::write(
            &instance,
            serde_json::to_string(&json!({"schema": ENVELOPE_ID, "value": value})).expect("JSON"),
        )
        .expect("the instance is written");
        let validated = ess(
            &directory,
            &[
                "generate",
                "schema",
                "validate",
                "--schemas",
                "registry",
                &format!("{label}.json"),
            ],
        );
        assert_eq!(
            validated.status.success(),
            admitted,
            "`{value}` is {}:\n{}",
            if admitted { "admitted" } else { "refused" },
            text(&validated)
        );
    }
}
