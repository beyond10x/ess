
#[cfg(test)]
mod retained_result_checks {
    use super::*;
    #[test]
    fn both_native_branches_return_the_actual_typed_result() {
        let response = SeedResponse {
            revision_id: crate::primitives::Uuid("00000000-0000-4000-8000-000000000037".into()),
            stamp: crate::primitives::Timestamp("2026-09-22T01:02:03Z".into()),
            number: 9_007_199_254_740_993, optional: None,
            values: vec![i64::MIN, i64::MAX],
        };
        let original = SeedOutcome::Seeded { response: response.clone(), seeded: Seeded { record_id: response.revision_id.clone() } };
        let replay = SeedOutcome::Replayed { response };
        let SeedOutcome::Seeded { response: original, .. } = original else { panic!("original") };
        let SeedOutcome::Replayed { response: replay } = replay else { panic!("replay") };
        assert_eq!(original, replay);
        assert_eq!(replay.number, 9_007_199_254_740_993);
    }
}
