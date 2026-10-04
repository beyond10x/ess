//! An event expectation comparing captured identities (beyond10x/ess#273) replays in actual Firefox,
//! in the declaration player and in the coverage replay, each rebuilt from the browser product
//! `ess verify conform web` writes (ordinary and `--suite-format 5`) after the product's own
//! bytes are admitted.
#[path = "support/browser.rs"]
mod browser;

use std::{fs, path::PathBuf, process::Command};

const MODEL: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/captured-identities.yaml");

/// A queue opened and a ticket filed into it, both captured; closing the ticket names both.
const SCENARIO: &str = r"type: ess-scenario/1
domain: desk.tickets
scenario: close-a-filed-ticket
summary: Closing a filed ticket names the ticket and its queue.
arrange:
  - instance: q
    entity: desk.tickets.Queue
  - instance: t
    entity: desk.tickets.Ticket
timeline:
  - at: 2026-01-05T09:00:00Z
    command: desk.tickets.OpenQueue
    input: {label: front}
    outcome: opened
    capture: {instance: q, event: desk.tickets.QueueOpened, field: queue_id}
  - at: 2026-01-05T09:00:01Z
    command: desk.tickets.FileTicket
    input: {queue_id: {$instance: q}}
    outcome: filed
    capture: {instance: t, event: desk.tickets.TicketFiled, field: ticket_id}
  - at: 2026-01-05T09:00:02Z
    command: desk.tickets.CloseTicket
    input: {ticket_id: {$instance: t}}
    outcome: closed
    events:
      - event: desk.tickets.TicketClosed
        payload: {ticket_id: {$instance: t}, closed: {$instance: t}, queue_id: {$instance: q}}
";

/// Every expected event each scenario's acts show, as the player built them.
const EVENTS: &str = r"(async () => {
  try {
    const {default: player} = await import('./player.js');
    const events = player.scenarios.flatMap(s => s.acts.flatMap(a => a.events.map(e =>
      ({scenario: s.name, step: e.step, event: e.event, payload: e.payload ?? null}))));
    return JSON.stringify({admitted: true, events});
  } catch (error) { return JSON.stringify({admitted: false, error: String(error)}); }
})()";

#[test]
fn actual_firefox_replays_captured_identities_in_both_players() {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "captured-identities-browser-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("scenarios")).unwrap();
    fs::write(root.join("desk.yaml"), MODEL).unwrap();
    fs::write(root.join("scenarios/close.yaml"), SCENARIO).unwrap();
    let mut browser = browser::Browser::new(&root);
    for (label, format) in [("declaration", None), ("coverage", Some("5"))] {
        let out = root.join(label);
        let mut command = Command::new(env!("CARGO_BIN_EXE_ess"));
        command
            .args(["verify", "conform", "web", "--path"])
            .arg(root.join("desk.yaml"))
            .arg("--scenarios")
            .arg(root.join("scenarios/close.yaml"));
        if let Some(format) = format {
            command.args(["--suite-format", format]);
        }
        let output = command.arg("--out").arg(&out).output().unwrap();
        assert!(
            output.status.success(),
            "{label}: {}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        browser::legacy_replay_fixture(&out);
        let server = browser::Server::new(&out);
        let context = browser.open(&format!("{}/index.html", server.url));
        let result = browser.evaluate(&context, EVENTS);
        fs::write(
            root.join(format!("{label}.json")),
            serde_json::to_string_pretty(&result).unwrap(),
        )
        .unwrap();
        assert_eq!(result["admitted"], true, "{label}: {result}");
        let events = result["events"].as_array().unwrap();
        let closed = events
            .iter()
            .find(|event| {
                event["scenario"] == "desk.tickets/authored/close-a-filed-ticket"
                    && event["event"] == "desk.tickets.TicketClosed"
            })
            .unwrap_or_else(|| panic!("{label}: the closing event is shown: {result}"));
        assert_eq!(closed["step"], "expect_event_values", "{label}: {closed}");
        for (field, instance) in [("ticket_id", "t"), ("closed", "t"), ("queue_id", "q")] {
            assert_eq!(
                closed["payload"][field],
                serde_json::json!({"kind": "instance", "instance": instance}),
                "{label} {field}: {closed}"
            );
        }
    }
    drop(browser);
    let _ = fs::remove_dir_all(&root);
}
