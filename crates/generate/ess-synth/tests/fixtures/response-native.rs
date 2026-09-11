#[cfg(test)]
mod response_checks {
    use super::*;
    fn item() -> Item {
        Item {
            remaining: 37,
            created: crate::primitives::Timestamp("2026-09-11T10:00:00Z".into()),
            ended: crate::primitives::Timestamp("2026-09-11T10:01:00Z".into()),
            state: None,
            call_type: "incoming".into(),
            features: std::collections::BTreeMap::from([("enabled".into(), true)]),
        }
    }
    fn returned() -> CancelOutcome {
        CancelOutcome::Cancelled {
            response: CancelResponse { item: item() },
            returned: Returned {
                item: item(),
                receipt: "generated-37".into(),
            },
        }
    }
    #[test]
    fn native_response_values_are_actual() {
        let mut result = returned();
        assert!(result.response_payload_matches());
        let CancelOutcome::Cancelled { response, .. } = &mut result;
        response.item.remaining = 38;
        assert!(!result.response_payload_matches());
    }
    #[test]
    fn native_response_map_mutation_is_not_hidden() {
        let mut result = returned();
        assert!(result.response_payload_matches());
        let CancelOutcome::Cancelled { returned, .. } = &mut result;
        returned.item.features.insert("enabled".into(), false);
        assert!(!result.response_payload_matches());
    }
}
