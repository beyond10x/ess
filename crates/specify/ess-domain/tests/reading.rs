//! Clock attachments preserve distinct representation and observation authority.
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

fn model(representation: &str, encoding: &str, offset: &str) -> String {
    format!(
        "format: ess/3\nsystem: chronology\nversion: v1\ndomain: chronology.reading\ntypes:\n  - name: chronology.reading.ClockValue\n    kind: newtype\n    of: {representation}\n    reading:\n      encoding: {encoding}\n      origins:\n        - role: producer_process\n          offset: {offset}\n"
    )
}
fn assemble(text: &str) -> Result<Specification, String> {
    let raw = RawSpecFile::parse(text).map_err(|e| e.to_string())?;
    Specification::assemble([(Source::new("reading.yaml"), raw)]).map_err(|e| e.to_string())
}

#[test]
fn reading_contract_admits_supported_scalar_representations() {
    for (representation, encoding, offset) in [
        ("String", "offset_date_time_text", "encoded_offset"),
        (
            "String",
            "local_date_time_millis_literal_z",
            "requires_observation",
        ),
        ("Integer", "unix_seconds", "encoding_defined_epoch"),
    ] {
        assemble(&model(representation, encoding, offset)).unwrap();
    }
}

#[test]
fn reading_contract_refuses_representation_and_authority_contradictions() {
    for (representation, encoding, offset) in [
        ("Timestamp", "offset_date_time_text", "encoded_offset"),
        ("Integer", "offset_date_time_text", "encoded_offset"),
        ("String", "unix_seconds", "encoding_defined_epoch"),
        (
            "String",
            "local_date_time_millis_literal_z",
            "encoded_offset",
        ),
        ("Integer", "unix_seconds", "requires_observation"),
    ] {
        assert!(assemble(&model(representation, encoding, offset)).is_err());
    }
}

#[test]
fn reading_contract_refuses_empty_duplicate_and_unknown_attachments() {
    let text = model("String", "offset_date_time_text", "encoded_offset");
    for changed in [
        text.replace("origins:\n        - role: producer_process\n          offset: encoded_offset", "origins: []"),
        text.replace("offset: encoded_offset", "offset: encoded_offset\n        - role: producer_process\n          offset: encoded_offset"),
        text.replace("      encoding:", "      trusted: true\n      encoding:"),
    ] {
        assert!(assemble(&changed).is_err());
    }
}

fn facts(formatter: &str) -> ess_domain::reading::ClockReadingEvidence {
    ess_domain::reading::ClockReadingEvidence {
        correlation: "scenario-1".into(),
        occurrence: "sample#0.value".into(),
        process_instance: "process-1".into(),
        epoch: "epoch-1".into(),
        origin: "producer_process".into(),
        formatter: formatter.into(),
        offset_minutes: None,
    }
}

#[test]
fn known_offset_epoch_and_fixed_local_readings_resolve_to_the_same_coordinate() {
    use ess_domain::reading::{compare_clock_readings, resolve_clock_reading};
    let resolve = |encoding, authority, value, evidence| {
        resolve_clock_reading(
            encoding,
            &[("producer_process", authority)],
            value,
            &evidence,
            "scenario-1",
            "sample#0.value",
        )
        .unwrap()
    };
    let offset = resolve(
        "offset_date_time_text",
        "encoded_offset",
        "2026-09-11T12:00:00+02:00",
        facts("encoded_offset"),
    );
    let epoch = resolve(
        "unix_seconds",
        "encoding_defined_epoch",
        "1789120800",
        facts("unix_seconds"),
    );
    let mut local_facts = facts("fixed_offset");
    local_facts.offset_minutes = Some(120);
    let local = resolve(
        "local_date_time_millis_literal_z",
        "requires_observation",
        "2026-09-11T12:00:00.000Z",
        local_facts,
    );
    assert_eq!(offset.unix_millis, 1_789_120_800_000);
    assert_eq!(
        compare_clock_readings(&offset, &epoch).unwrap(),
        std::cmp::Ordering::Equal
    );
    assert_eq!(
        compare_clock_readings(&local, &epoch).unwrap(),
        std::cmp::Ordering::Equal
    );
}

#[test]
fn literal_z_cannot_supply_missing_offset_and_wrong_offset_changes_the_coordinate() {
    use ess_domain::reading::{resolve_clock_reading, ReadingError};
    let resolve = |evidence| {
        resolve_clock_reading(
            "local_date_time_millis_literal_z",
            &[("producer_process", "requires_observation")],
            "2026-09-11T12:00:00.000Z",
            &evidence,
            "scenario-1",
            "sample#0.value",
        )
    };
    assert_eq!(
        resolve(facts("unknown")),
        Err(ReadingError::UnknownEvidence)
    );
    assert_eq!(
        resolve(facts("encoded_offset")),
        Err(ReadingError::IncompatibleFormatter)
    );
    let mut evidence = facts("fixed_offset");
    evidence.offset_minutes = Some(0);
    assert_eq!(resolve(evidence).unwrap().unix_millis, 1_789_128_000_000);
}

#[test]
fn erased_origin_source_epoch_and_stale_occurrence_refuse_authority() {
    use ess_domain::reading::{compare_clock_readings, resolve_clock_reading, ReadingError};
    let resolve = |evidence| {
        resolve_clock_reading(
            "unix_seconds",
            &[("producer_process", "encoding_defined_epoch")],
            "1",
            &evidence,
            "scenario-1",
            "sample#0.value",
        )
    };
    for member in ["source", "epoch", "origin", "correlation", "occurrence"] {
        let mut evidence = facts("unix_seconds");
        match member {
            "source" => evidence.process_instance.clear(),
            "epoch" => evidence.epoch.clear(),
            "origin" => evidence.origin = "unknown".into(),
            "correlation" => evidence.correlation = "old".into(),
            _ => evidence.occurrence = "old".into(),
        }
        assert!(resolve(evidence).is_err(), "{member}");
    }
    let first = resolve(facts("unix_seconds")).unwrap();
    for member in ["source", "epoch"] {
        let mut evidence = facts("unix_seconds");
        if member == "source" {
            evidence.process_instance = "process-2".into();
        } else {
            evidence.epoch = "epoch-2".into();
        }
        assert_eq!(
            compare_clock_readings(&first, &resolve(evidence).unwrap()),
            Err(ReadingError::DifferentClock)
        );
    }
}

#[test]
fn bounded_reading_grammar_refuses_dates_fractions_leaps_offsets_and_overflow() {
    use ess_domain::reading::{normalize_text, normalize_unix};
    for value in [
        "1969-12-31T23:59:59Z",
        "2026-02-29T00:00:00Z",
        "2024-02-30T00:00:00Z",
        "2026-09-11T12:00:60Z",
        "2026-09-11T24:00:00Z",
        "2026-09-11T12:00:00.1Z",
        "2026-09-11T12:00:00.0000Z",
        "2026-09-11t12:00:00Z",
        "2026-09-11T12:00:00+14:01",
        "9999-12-31T23:59:59-00:01",
        "1970-01-01T00:00:00+00:01",
    ] {
        assert!(normalize_text(value, None).is_err(), "{value}");
    }
    assert!(normalize_unix(i64::MAX).is_err());
    assert!(normalize_unix(-1).is_err());
    assert_eq!(
        normalize_unix(253_402_300_799).unwrap(),
        253_402_300_799_000
    );
    assert_eq!(
        normalize_text("2000-02-29T00:00:00.000Z", None).unwrap(),
        951_782_400_000
    );
    assert!(normalize_text("1970-01-01T00:00:00.000Z", Some(1)).is_err());
}

#[test]
fn attached_readings_require_ess3_and_unattached_legacy_wrappers_remain_ess1() {
    let text = model("String", "offset_date_time_text", "encoded_offset");
    for old in [1, 2] {
        let error = assemble(&text.replace("ess/3", &format!("ess/{old}"))).unwrap_err();
        assert!(error.contains("ess/3"), "{error}");
    }
    assemble(&text).unwrap();
    let plain = text
        .split("    reading:")
        .next()
        .unwrap()
        .replace("ess/3", "ess/1");
    assemble(&plain).unwrap();
}
