//! beyond10x/ess#468, run inside the package generated from `json-cli.yaml`: a `Json` result
//! field reaches stdout as JSON, and a reply outside the declared shape is `cli_result`.
use cli_contract::wire::Shape;
use cli_contract::*;
use serde_json::{json, Value};
use std::ffi::OsString;

struct Reply(Option<Value>);
impl Handler for Reply {
    fn call(&mut self, _: &Invocation<'_>) -> HandlerReply {
        HandlerReply::Success(self.0.take().expect("one call"))
    }
}

struct NoSources;
impl Sources for NoSources {
    fn acquire(&mut self, _: ProtectedSource) -> Result<String, AcquireError> {
        unreachable!("describe reads no protected or document source")
    }
}

fn describe(output: &str, reply: &Value) -> ProcessOutput {
    run(
        [
            "json-demo",
            "describe",
            "--profile",
            "p",
            "--output",
            output,
        ]
        .map(OsString::from)
        .to_vec(),
        &mut NoSources,
        &mut Reply(Some(reply.clone())),
        None,
    )
}

#[test]
fn the_package_reads_the_json_shape_from_its_plan() {
    let Shape::Struct { fields } = plan().callables["describe"].result.shape.clone() else {
        panic!("a struct result")
    };
    assert_eq!(fields["payload"], Shape::Json);
    assert_eq!(
        fields["items"],
        Shape::List {
            of: Box::new(Shape::Json)
        }
    );
}

#[test]
fn a_json_object_is_written_as_json_not_as_json_text() {
    let document =
        json!({"schema": {"type": "object", "required": ["id"]}, "rows": [1, null, "x"]});
    let reply = json!({"payload": document, "items": [document, 2], "keyed": {"k": document}});
    let output = describe("json", &reply);
    assert_eq!(output.exit_code, 0, "{output:?}");
    assert_eq!(output.stderr, "");
    let written: Value = serde_json::from_str(&output.stdout).unwrap();
    assert_eq!(written, json!({"ok": true, "result": reply}));
    assert_eq!(written["result"]["payload"], document);
    assert!(!output.stdout.contains("\\\""), "{}", output.stdout);
    let human = describe("human", &reply);
    assert_eq!(human.exit_code, 0, "{human:?}");
    assert_eq!(human.stdout, format!("{reply}\n"));
}

#[test]
fn a_reply_outside_the_shape_around_json_is_cli_result() {
    for reply in [
        json!({"items": [], "keyed": {}}),
        json!({"payload": {}, "items": {}, "keyed": {}}),
        json!({"payload": {}, "items": [], "keyed": []}),
        json!({"payload": {}, "items": [], "keyed": {}, "extra": true}),
        json!("{\"payload\": {}, \"items\": [], \"keyed\": {}}"),
    ] {
        let output = describe("json", &reply);
        assert_eq!(output.exit_code, 1, "{reply}");
        assert_eq!(output.stdout, "", "{reply}");
        assert_eq!(
            serde_json::from_str::<Value>(&output.stderr).unwrap(),
            json!({"ok": false, "error": {"code": "cli_result", "data": {}}}),
            "{reply}"
        );
    }
}
