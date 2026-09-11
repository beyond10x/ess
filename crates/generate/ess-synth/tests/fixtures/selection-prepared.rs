use selection_system::choose_from_prepared;
use selection_types::core::{Arrived, Domain, Leg};
#[test]
fn host_prepares_once_then_generated_code_selects() {
    let event = Arrived { data: "host-owned raw representation".into() };
    let calls = std::cell::Cell::new(0);
    let prepare = |_: &str| { calls.set(calls.get()+1); vec![
        Some(Leg { id: "external".into(), from: "remote".into(), domain: Some(Domain::External), source: None }),
        Some(Leg { id: "agent".into(), from: "remote".into(), domain: Some(Domain::Internal), source: None }),
    ] };
    let prepared = prepare(&event.data);
    let result = choose_from_prepared(&event, &prepared).unwrap();
    assert_eq!(calls.get(), 1);
    assert_eq!(result.agent_id.as_deref(), Some("agent"));
    assert_eq!(result.external_id.as_deref(), Some("external"));
}
