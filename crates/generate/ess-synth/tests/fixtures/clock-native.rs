#[cfg(test)]
mod clock_reading_checks {
    use super::*;
    use crate::primitives::clock_reading::*;
    fn evidence(formatter: &str) -> ClockReadingEvidence {
        ClockReadingEvidence {
            correlation: "scenario-1".into(),
            occurrence: "sample#0.value".into(),
            process_instance: "process-1".into(),
            epoch: "epoch-1".into(),
            origin: "producer_process".into(),
            formatter: formatter.into(),
            offset_minutes: None,
        }
    }
    fn epoch() -> ClockCoordinate {
        EpochSeconds(1_789_120_800)
            .resolve_reading(&evidence("unix_seconds"), "scenario-1", "sample#0.value")
            .unwrap()
    }
    #[test]
    fn offset_and_epoch_agree() {
        let coordinate = OffsetText("2026-09-11T12:00:00+02:00".into())
            .resolve_reading(&evidence("encoded_offset"), "scenario-1", "sample#0.value")
            .unwrap();
        assert_eq!(
            compare_clock_readings(&coordinate, &epoch()).unwrap(),
            std::cmp::Ordering::Equal
        );
    }
    #[test]
    fn local_offset_is_observed_and_wrong_offset_changes_order() {
        let value = LocalText("2026-09-11T12:00:00.000Z".into());
        let mut facts = evidence("fixed_offset");
        facts.offset_minutes = Some(120);
        assert_eq!(
            compare_clock_readings(
                &value
                    .resolve_reading(&facts, "scenario-1", "sample#0.value")
                    .unwrap(),
                &epoch()
            )
            .unwrap(),
            std::cmp::Ordering::Equal
        );
        facts.offset_minutes = Some(0);
        assert_eq!(
            compare_clock_readings(
                &value
                    .resolve_reading(&facts, "scenario-1", "sample#0.value")
                    .unwrap(),
                &epoch()
            )
            .unwrap(),
            std::cmp::Ordering::Greater
        );
    }
    #[test]
    fn literal_z_has_no_utc_authority() {
        assert_eq!(
            LocalText("2026-09-11T12:00:00.000Z".into()).resolve_reading(
                &evidence("unknown"),
                "scenario-1",
                "sample#0.value"
            ),
            Err(ReadingError::UnknownEvidence)
        );
    }
    #[test]
    fn stale_evidence_refuses() {
        let value = EpochSeconds(1);
        for (correlation, occurrence) in [("old", "sample#0.value"), ("scenario-1", "old")] {
            assert_eq!(
                value.resolve_reading(&evidence("unix_seconds"), correlation, occurrence),
                Err(ReadingError::MismatchedOccurrence)
            );
        }
    }
    #[test]
    fn cross_source_and_epoch_refuse() {
        for changed in ["source", "epoch"] {
            let mut facts = evidence("unix_seconds");
            if changed == "source" {
                facts.process_instance = "other".into()
            } else {
                facts.epoch = "other".into()
            }
            let result = EpochSeconds(1)
                .resolve_reading(&facts, "scenario-1", "sample#0.value")
                .unwrap();
            assert_eq!(
                compare_clock_readings(&result, &epoch()),
                Err(ReadingError::DifferentClock)
            );
        }
    }
    #[test]
    fn invalid_dates_ranges_and_erased_origin_refuse() {
        for text in [
            "2026-02-29T00:00:00Z",
            "2026-09-11T12:00:60Z",
            "2026-09-11T12:00:00+14:01",
            "1970-01-01T00:00:00+01:00",
        ] {
            assert!(OffsetText(text.into())
                .resolve_reading(&evidence("encoded_offset"), "scenario-1", "sample#0.value")
                .is_err())
        }
        let mut facts = evidence("unix_seconds");
        facts.origin = "unknown".into();
        assert_eq!(
            EpochSeconds(1).resolve_reading(&facts, "scenario-1", "sample#0.value"),
            Err(ReadingError::UnknownEvidence)
        );
        assert!(EpochSeconds(i64::MAX)
            .resolve_reading(&evidence("unix_seconds"), "scenario-1", "sample#0.value")
            .is_err());
    }
}
