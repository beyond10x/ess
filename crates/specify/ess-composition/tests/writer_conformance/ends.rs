//! An `event` end and a `command` end with `part: response` follow the same presence rules.

use crate::common::Pair;

fn event(name: &str, field: &str) -> String {
    format!("  - name: {name}\n    fields:\n{field}")
}

fn command(name: &str, field: &str) -> String {
    format!(
        "  - name: {name}\n    input:\n      - {{ name: channel, type: String }}\n    \
         response:\n{field}    outcomes:\n      - name: done\n        returns: true\n"
    )
}

fn field(spelling: &str) -> &'static str {
    match spelling {
        "null_when_absent" => {
            "      - name: codec\n        type: Optional<String>\n        presence: null_when_absent\n"
        }
        "omitted_or_null" => {
            "      - name: codec\n        type: Optional<String>\n        presence: omitted_or_null\n"
        }
        "unstated" => "      - name: codec\n        type: Optional<String>\n",
        _ => "      - { name: codec, type: String }\n",
    }
}

fn pair() -> Pair {
    let relay = format!(
        "domain: relay.session\n\nevents:\n{}{}{}\ncommands:\n{}{}{}",
        event("relay.session.Opened", field("null_when_absent")),
        event("relay.session.Closed", field("unstated")),
        event("relay.session.Sent", field("required")),
        command("relay.session.Open", field("null_when_absent")),
        command("relay.session.Close", field("unstated")),
        command("relay.session.Send", field("required")),
    );
    let wire = format!(
        "domain: wire.session\n\nevents:\n{}{}\ncommands:\n{}{}",
        event("wire.session.Opened", field("omitted_or_null")),
        event("wire.session.Closed", field("required")),
        command("wire.session.Open", field("omitted_or_null")),
        command("wire.session.Close", field("required")),
    );
    Pair::with_surface(
        &relay,
        (
            &[
                "relay.session.Open",
                "relay.session.Close",
                "relay.session.Send",
            ],
            &[
                "relay.session.Opened",
                "relay.session.Closed",
                "relay.session.Sent",
            ],
        ),
        &wire,
        (
            &["wire.session.Open", "wire.session.Close"],
            &["wire.session.Opened", "wire.session.Closed"],
        ),
    )
}

fn entries(local: &str, imported: &str, extra: &str) -> String {
    Pair::entry(local, imported, extra)
}

const OPTIONAL_AGAINST_REQUIRED: &str =
    "field `codec`: `Optional<String>` where the imported type has `String`";

#[test]
fn an_event_end_applies_the_same_rules() {
    let models = pair();
    let writer = models.document(
        "ess-composition/6",
        &entries(
            "event: relay.session.Opened",
            "event: wire.session.Opened",
            "    writer: true\n",
        ),
    );
    let ir = models.composes(&writer);
    let json: serde_json::Value =
        serde_json::from_str(&ir.to_canonical_json()).expect("composition IR is JSON");
    assert_eq!(json["conformances"][0]["writer"], true, "{json:#}");

    models.drift(&models.document(
        "ess-composition/6",
        &entries(
            "event: relay.session.Opened",
            "event: wire.session.Opened",
            "",
        ),
    ));
    models.composes(&models.document(
        "ess-composition/6",
        &entries(
            "event: relay.session.Sent",
            "event: wire.session.Opened",
            "    writer: true\n",
        ),
    ));
    let detail = models.drift(&models.document(
        "ess-composition/6",
        &entries(
            "event: relay.session.Closed",
            "event: wire.session.Closed",
            "    writer: true\n",
        ),
    ));
    assert!(detail.contains(OPTIONAL_AGAINST_REQUIRED), "{detail}");
}

#[test]
fn a_command_response_end_applies_the_same_rules() {
    let models = pair();
    let response = |local: &str, imported: &str, extra: &str| {
        models.document(
            "ess-composition/6",
            &entries(
                &format!("command: relay.session.{local}, part: response"),
                &format!("command: wire.session.{imported}, part: response"),
                extra,
            ),
        )
    };
    let ir = models.composes(&response("Open", "Open", "    writer: true\n"));
    let json: serde_json::Value =
        serde_json::from_str(&ir.to_canonical_json()).expect("composition IR is JSON");
    assert_eq!(json["conformances"][0]["writer"], true, "{json:#}");

    models.drift(&response("Open", "Open", ""));
    models.composes(&response("Send", "Open", "    writer: true\n"));
    let detail = models.drift(&response("Close", "Close", "    writer: true\n"));
    assert!(detail.contains(OPTIONAL_AGAINST_REQUIRED), "{detail}");
}
