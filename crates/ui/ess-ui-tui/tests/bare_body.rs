//! A command route whose `ess-http/1` binding takes one input field as its bare request body
//! (`story:a-command-route-takes-one-input-as-its-bare-request-body`, beyond10x/ess#748): the
//! terminal's adapter sends that field's value alone as the body, the path carrying the rest.

use std::collections::BTreeMap;
use std::io::{Read as _, Write as _};
use std::net::TcpListener;

use ess_ui::binding::Binding;
use ess_ui_tui::HttpAdapter;
use ess_yaml::Value;

/// One command, `PUT /boards/{board}/cards/priority`, sending `priorities` bare.
const BINDING: &str = r#"{
  "system": "catalog",
  "components": {
    "board-service": {
      "views": {},
      "commands": {
        "catalog.board.SetCardPriorities": {
          "path": "/boards/{board}/cards/priority",
          "method": "PUT",
          "path_params": {"board": "board_id"},
          "body": "priorities",
          "body_required": true,
          "errors": {}
        }
      }
    }
  },
  "names": {"board.SetCardPriorities": "catalog.board.SetCardPriorities"}
}"#;

/// The one request a server on a free port reads, answered `202`, and the adapter's answer.
fn exchanged(input: &str) -> (String, Result<(u16, String), String>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("a free port");
    let base = format!("http://{}", listener.local_addr().expect("an address"));
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("one connection");
        let mut request = Vec::new();
        let mut chunk = [0_u8; 4096];
        loop {
            let read = stream.read(&mut chunk).expect("the request reads");
            request.extend_from_slice(&chunk[..read]);
            let text = String::from_utf8_lossy(&request).into_owned();
            if let Some((head, body)) = text.split_once("\r\n\r\n") {
                let length = head
                    .lines()
                    .find_map(|line| line.strip_prefix("Content-Length: "))
                    .map_or(0, |length| length.trim().parse().expect("a length"));
                if body.len() >= length || read == 0 {
                    break;
                }
            }
            if read == 0 {
                break;
            }
        }
        stream
            .write_all(b"HTTP/1.1 202 Accepted\r\nContent-Length: 2\r\n\r\n{}")
            .expect("the answer writes");
        String::from_utf8_lossy(&request).into_owned()
    });
    let binding: Binding = serde_json::from_str(BINDING).expect("the binding reads");
    let bases = BTreeMap::from([("board-service".to_owned(), base)]);
    let adapter = HttpAdapter::new(binding, &bases, None).expect("the adapter is built");
    let input: BTreeMap<String, Value> = ess_yaml::from_str(input).expect("the input parses");
    let answer = adapter.post("board.SetCardPriorities", &input);
    (server.join().expect("the server ran"), answer)
}

#[test]
fn a_bare_body_route_sends_the_fields_value_as_the_body() {
    let (request, answer) =
        exchanged("{board_id: b-1, priorities: [{id: c-1, priority: 1}, {id: c-2, priority: 2}]}");
    assert_eq!(answer, Ok((202, "{}".to_owned())));
    let (head, body) = request.split_once("\r\n\r\n").expect("a head and a body");
    assert!(
        head.starts_with("PUT /boards/b-1/cards/priority HTTP/1.1\r\n"),
        "{head}"
    );
    assert!(head.contains("Content-Type: application/json"), "{head}");
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(body).expect("the body is JSON"),
        serde_json::json!([{"id": "c-1", "priority": 1}, {"id": "c-2", "priority": 2}]),
        "{body}"
    );
}
