//! A literal after `else:` (`{input: f, else: <literal>}`, source format `ess/16`, beyond10x/ess#163,
//! `docs/design/value-expressions.md` E4).

use ess_domain::command::{PayloadSource, RawOutcome, ScalarKind};

fn spec(body: &str) -> Result<ess_domain::Specification, String> {
    let raw = ess_domain::spec::RawSpecFile::parse(body).map_err(|e| e.to_string())?;
    ess_domain::Specification::assemble([(ess_domain::system::Source::new("orders.yaml"), raw)])
        .map_err(|e| e.to_string())
}

/// One creating command with an optional `tier` and an optional `rank`; `{payload}` and `{sets}`
/// are the sources of the event's and the entity's `tier` and `rank`.
fn model(format: &str, payload: &str, sets: &str) -> String {
    format!(
        "format: {format}
system: demo
version: v1
domain: demo.orders
types:
  - {{name: demo.orders.OrderId, kind: newtype, of: String}}
  - {{name: demo.orders.Tier, kind: enum, variants: [Standard, Express]}}
entities:
  - name: demo.orders.Order
    identity: {{name: order_id, type: demo.orders.OrderId}}
    fields:
      - {{name: tier, type: demo.orders.Tier}}
      - {{name: rank, type: Integer}}
      - {{name: label, type: String}}
    lifecycle: {{initial: Open, states: [Open], terminal: [Open]}}
events:
  - name: demo.orders.Opened
    fields:
      - {{name: order_id, type: demo.orders.OrderId}}
      - {{name: tier, type: demo.orders.Tier}}
actors:
  - {{name: demo.orders.Clerk, may: [demo.orders.Open]}}
commands:
  - name: demo.orders.Open
    input:
      - {{name: tier, type: Optional<demo.orders.Tier>}}
      - {{name: rank, type: Optional<Integer>}}
      - {{name: label, type: Optional<String>}}
    outcomes:
      - name: opened
        creates: demo.orders.Order
        instance: order_id
        emits: [demo.orders.Opened]
        payload:
          demo.orders.Opened:
            order_id: {{generated: true}}
            tier: {payload}
        sets:
{sets}
"
    )
}

const SETS: &str = "          tier: {input: tier, else: Express}
          rank: {input: rank, else: 3}
          label: {input: label, else: 'none yet'}";

fn refused(body: &str, expected: &[&str]) {
    let error = spec(body)
        .err()
        .unwrap_or_else(|| panic!("must not compile:\n{body}"));
    for needle in expected {
        assert!(error.contains(needle), "expected {needle:?} in:\n{error}");
    }
}

fn opened_sources(
    spec: &ess_domain::Specification,
) -> (
    std::collections::BTreeMap<String, PayloadSource>,
    PayloadSource,
) {
    let command = spec
        .commands()
        .values()
        .find(|command| command.name.to_string() == "demo.orders.Open")
        .expect("the command is declared");
    let outcome = &command.outcomes[0];
    let sets = outcome
        .sets
        .iter()
        .map(|(target, source)| (target.clone(), source.clone()))
        .collect();
    let payload = outcome
        .payload
        .values()
        .next()
        .and_then(|fields| fields.get("tier"))
        .expect("the payload fills `tier`")
        .clone();
    (sets, payload)
}

#[test]
fn a_literal_fallback_compiles_under_ess_16_in_a_payload_and_in_sets() {
    let body = model("ess/16", "{input: tier, else: Standard}", SETS);
    let spec = spec(&body).unwrap_or_else(|error| panic!("{error}"));
    let (sets, payload) = opened_sources(&spec);
    assert_eq!(
        payload,
        PayloadSource::InputOrGenerated {
            field: "tier".to_owned(),
            otherwise: Some(Box::new(PayloadSource::Literal {
                value: "Standard".to_owned()
            })),
        }
    );
    assert_eq!(
        sets.get("rank"),
        Some(&PayloadSource::InputOrGenerated {
            field: "rank".to_owned(),
            otherwise: Some(Box::new(PayloadSource::Scalar {
                value: "3".to_owned(),
                scalar: ScalarKind::Integer,
            })),
        })
    );
    assert_eq!(
        sets.get("label"),
        Some(&PayloadSource::InputOrGenerated {
            field: "label".to_owned(),
            otherwise: Some(Box::new(PayloadSource::Literal {
                value: "none yet".to_owned()
            })),
        })
    );
}

#[test]
fn a_generated_fallback_is_unchanged() {
    let body = model(
        "ess/14",
        "{input: tier, else: {generated: true}}",
        "          tier: {input: tier, else: {generated: true}}
          rank: 0
          label: 'x'",
    );
    let spec = spec(&body).unwrap_or_else(|error| panic!("{error}"));
    let (_, payload) = opened_sources(&spec);
    assert_eq!(
        payload,
        PayloadSource::InputOrGenerated {
            field: "tier".to_owned(),
            otherwise: None,
        }
    );
}

#[test]
fn a_literal_fallback_needs_ess_16() {
    for format in ["ess/14", "ess/15"] {
        let body = model(format, "{input: tier, else: Standard}", SETS);
        refused(
            &body,
            &[
                "unsupported_format_version",
                "a literal after `else:` requires specification format ess/16",
            ],
        );
    }
}

#[test]
fn a_literal_fallback_is_type_checked_against_the_target() {
    // Not a variant of the enum the payload field is.
    let body = model("ess/16", "{input: tier, else: Premium}", SETS);
    refused(&body, &["type_mismatch", "Premium"]);
    // Not a variant of the enum the entity field is.
    let body = model(
        "ess/16",
        "{input: tier, else: Standard}",
        "          tier: {input: tier, else: Premium}
          rank: {input: rank, else: 3}
          label: {input: label, else: 'none yet'}",
    );
    refused(&body, &["type_mismatch", "Premium"]);
    // Text over an Integer.
    let body = model(
        "ess/16",
        "{input: tier, else: Standard}",
        "          tier: {input: tier, else: Express}
          rank: {input: rank, else: three}
          label: {input: label, else: 'none yet'}",
    );
    refused(&body, &["type_mismatch", "three"]);
    // An unquoted number over a String, whose repair is the quoted spelling.
    let body = model(
        "ess/16",
        "{input: tier, else: Standard}",
        "          tier: {input: tier, else: Express}
          rank: {input: rank, else: 3}
          label: {input: label, else: 0}",
    );
    refused(&body, &["type_mismatch", "'0'"]);
}

#[test]
fn a_payload_fallback_literal_is_refused_wherever_a_bare_payload_literal_is() {
    // Every misspelling the bare payload literal rule refuses (`check_payload_literal`): an input
    // named without its prefix, a near miss of `input.`, and the binding's `event.` carried over.
    for written in ["rank", "inptu.rank", "event.rank", "subject.tier"] {
        let bare = model("ess/16", written, SETS);
        refused(&bare, &["misspelled_reference"]);
        let fallback = model("ess/16", &format!("{{input: tier, else: {written}}}"), SETS);
        refused(&fallback, &["misspelled_reference"]);
    }
    // And a literal the target cannot spell is one refusal, not two.
    let body = model("ess/16", "{input: tier, else: Premium}", SETS);
    let error = spec(&body).expect_err("`Premium` is no variant");
    assert_eq!(error.matches("Premium").count(), 1, "{error}");
}

#[test]
fn a_fallback_that_reads_something_is_still_refused() {
    for otherwise in [
        "input.label",
        "{subject: tier}",
        "{cleared: true}",
        "{input: tier, else: Standard}",
    ] {
        let body = model(
            "ess/16",
            &format!("{{input: tier, else: {otherwise}}}"),
            SETS,
        );
        refused(
            &body,
            &["`else:` admits `{generated: true}` only, or a literal"],
        );
    }
    // `subject.<field>` after `else:` is the misspelling E2 refuses everywhere.
    let body = model("ess/16", "{input: tier, else: subject.tier}", SETS);
    refused(&body, &["misspelled_reference", "{subject: tier}"]);
}

/// The refusal of a literal after `else:` reads as the bare literal's does — code, path, owner and
/// reason — except the hint, which must be a repair `else:` admits.
#[test]
fn a_fallback_refusal_reads_as_the_bare_one_with_a_hint_else_admits() {
    let first_line = |body: String| {
        let error = spec(&body).expect_err("refused");
        error.lines().next().unwrap_or_default().to_owned()
    };
    let without_hint = |line: &str| line.split(" (hint: ").next().unwrap_or_default().to_owned();
    let sets = |label: &str| {
        model(
            "ess/16",
            "{input: tier, else: Standard}",
            &format!(
                "          tier: {{input: tier, else: Express}}
          rank: {{input: rank, else: 3}}
          label: {label}"
            ),
        )
    };
    let bare = first_line(sets("0"));
    let fallback = first_line(sets("{input: label, else: 0}"));
    assert_eq!(
        without_hint(&bare),
        without_hint(&fallback),
        "{bare}\n{fallback}"
    );
    assert!(fallback.contains("demo.orders.Order.label"), "{fallback}");
    assert!(
        fallback.contains("quote it: `label: {input: label, else: '0'}`"),
        "{fallback}"
    );
    let bare = first_line(sets("2.5"));
    let fallback = first_line(sets("{input: label, else: 2.5}"));
    assert_eq!(
        without_hint(&bare),
        without_hint(&fallback),
        "{bare}\n{fallback}"
    );

    for written in ["rank", "inptu.rank", "event.rank"] {
        let bare = first_line(model("ess/16", written, SETS));
        let fallback = first_line(model(
            "ess/16",
            &format!("{{input: tier, else: {written}}}"),
            SETS,
        ));
        assert_eq!(
            without_hint(&bare),
            without_hint(&fallback),
            "{bare}\n{fallback}"
        );
        assert!(!fallback.contains("write `input."), "{fallback}");
        assert!(fallback.contains("else: {generated: true}"), "{fallback}");
    }
    // The repair the hint names compiles.
    spec(&model(
        "ess/16",
        "{input: tier, else: {generated: true}}",
        SETS,
    ))
    .unwrap_or_else(|error| panic!("{error}"));
}

#[test]
fn a_literal_fallback_round_trips_through_the_raw_document() {
    let written = "name: opened
creates: demo.orders.Order
instance: order_id
sets:
  tier: {input: tier, else: Express}
  rank: {input: rank, else: 3}
  label: {input: label, else: 'none yet'}
  flag: {input: flag, else: {generated: true}}
";
    let raw: RawOutcome = serde_yaml::from_str(written).unwrap_or_else(|error| panic!("{error}"));
    let again = serde_yaml::to_string(&raw).expect("an outcome serializes");
    let reread: RawOutcome =
        serde_yaml::from_str(&again).unwrap_or_else(|error| panic!("{error}\n{again}"));
    assert_eq!(raw.sets, reread.sets, "{again}");
    assert!(again.contains("else: 3"), "{again}");
    assert!(again.contains("else: Express"), "{again}");
    assert!(again.contains("generated: true"), "{again}");
}
