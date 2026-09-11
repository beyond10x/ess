//! Periodic binding declarations preserve their real cause and required input authority.
use ess_domain::{system::Source, RawSpecFile, Specification};

#[test]
fn periodic_poll_has_no_fabricated_event_and_keeps_required_host_inputs() {
    let text = include_str!("fixtures/periodic.yaml");
    let raw = RawSpecFile::parse(text).expect("source/3 periodic cause must parse");
    let spec = Specification::assemble([(Source::new("periodic.yaml"), raw)])
        .expect("typed authenticated host inputs must satisfy the actual command");
    let value = serde_json::to_value(spec.bindings().values().next().unwrap()).unwrap();
    assert!(value.get("event").is_none());
    assert_eq!(value["periodic"]["every"], "PT2S");
    assert_eq!(value["mapping"]["agent_id"]["kind"], "host_context");
    assert_eq!(value["mapping"]["status"]["kind"], "host_read");
}

fn admit(text: &str) -> Result<Specification, String> {
    let raw = RawSpecFile::parse(text).map_err(|error| error.to_string())?;
    Specification::assemble([(Source::new("periodic.yaml"), raw)])
        .map_err(|error| error.to_string())
}

#[test]
fn periodic_period_and_profile_are_closed_and_bounded() {
    let source = include_str!("fixtures/periodic.yaml");
    for bad in [
        "PT0S",
        "PT01S",
        "PT+2S",
        "PT-1S",
        "PT1.5S",
        "PT1M",
        "PT4294967296S",
        "NaN",
        ".inf",
    ] {
        assert!(
            admit(&source.replace("PT2S", bad)).is_err(),
            "accepted {bad}"
        );
    }
    assert!(admit(&source.replace("PT2S", "PT4294967295S")).is_ok());
    for (valid, bad) in [
        ("fixed_rate", "fixed_delay"),
        ("after_period", "immediate"),
        ("one_pending_drop_excess", "catch_up"),
        ("stop_acknowledged", "cancel_requested"),
    ] {
        assert!(
            admit(&source.replace(valid, bad)).is_err(),
            "accepted {bad}"
        );
    }
}

#[test]
fn periodic_host_authority_and_required_fields_cannot_be_fabricated() {
    let source = include_str!("fixtures/periodic.yaml");
    for changed in [
        source.replace("ess/3", "ess/2"),
        source.replace("owner: poll-service", "owner: unknown-service"),
        source.replace("      status: host_read.status\n", ""),
        source.replace("host_read.status", "host_read.missing"),
        source.replace("host_context.agent_id", "event.agent_id"),
        source.replace("host_context.agent_id", "host_context.agent_id.nested"),
        source.replace(
            "type: String\n    outcomes:",
            "type: Integer\n    outcomes:",
        ),
        source.replace(
            "    when:\n",
            "    when:\n      event: example.poll.Updated\n",
        ),
        source.replace("delivery: at_most_once", "delivery: at_least_once"),
    ] {
        assert!(
            admit(&changed).is_err(),
            "accepted invalid host/cause:\n{changed}"
        );
    }
}

#[test]
fn periodic_deadlines_use_checked_positive_ordinals() {
    let period = ess_domain::binding::periodic::Period::parse("PT2S").unwrap();
    assert_eq!(period.deadline(10, 1), Some(2010));
    assert_eq!(period.deadline(10, 0), None);
    assert_eq!(period.deadline(u64::MAX, 1), None);
    assert_eq!(period.deadline(0, u64::MAX), None);
}
