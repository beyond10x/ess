//! A command route whose `ess-http/1` binding takes one input field as its bare request body
//! (`story:a-command-route-takes-one-input-as-its-bare-request-body`, beyond10x/ess#748): the
//! terminal's adapter sends that field's value alone as the body, the path carrying the rest.

use std::collections::BTreeMap;
use std::io::{Read as _, Write as _};
use std::net::TcpListener;

use ess_ui::binding::{Answer, Binding};
use ess_ui_tui::{DataAdapter as _, HttpAdapter};
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

/// A server on a free port that answers its one request with `answer` (a whole HTTP answer).
fn serving(answer: &'static str) -> (String, std::thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("a free port");
    let base = format!("http://{}", listener.local_addr().expect("an address"));
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("one connection");
        let mut chunk = [0_u8; 4096];
        let mut request = Vec::new();
        loop {
            let read = stream.read(&mut chunk).expect("the request reads");
            request.extend_from_slice(&chunk[..read]);
            let text = String::from_utf8_lossy(&request).into_owned();
            if let Some((head, body)) = text.split_once("\r\n\r\n") {
                let length = head
                    .lines()
                    .find_map(|line| line.strip_prefix("Content-Length: "))
                    .map_or(0, |length| length.trim().parse().expect("a length"));
                if body.len() >= length {
                    break;
                }
            }
            if read == 0 {
                break;
            }
        }
        stream
            .write_all(answer.as_bytes())
            .expect("the answer writes");
    });
    (base, server)
}

/// One view and one command whose answers an `ess-http/1` binding declares (beyond10x/ess#723):
/// the view's rows under `items`, the command `204` with no body or its refusal as
/// `{code: <wire name>, ...}` at `409`.
const DECLARED: &str = r#"{
  "system": "catalog",
  "components": {
    "board-service": {
      "views": {
        "catalog.board.Cards": {"path": "/cards", "params": [], "answer": "array", "member": "items"},
        "catalog.board.Card": {"path": "/card", "params": [], "answer": "single", "empty": 404}
      },
      "commands": {
        "catalog.board.ArchiveCard": {
          "path": "/cards/archive",
          "body_required": false,
          "errors": {"catalog.board.CardLocked": {"status": 409, "display": "The card is locked"}},
          "answers": [
            {"status": 204, "outcome": "archived", "body": "none"},
            {"status": 409, "outcome": "locked", "body": "error", "member": "code",
             "error": {"wire": "card_locked", "code": "catalog.board.CardLocked"}}
          ]
        }
      }
    }
  },
  "names": {
    "board.Cards": "catalog.board.Cards",
    "board.Card": "catalog.board.Card",
    "board.ArchiveCard": "catalog.board.ArchiveCard"
  }
}"#;

fn declared(answer: &'static str) -> (HttpAdapter, std::thread::JoinHandle<()>) {
    let (base, server) = serving(answer);
    let binding: Binding = serde_json::from_str(DECLARED).expect("the binding reads");
    let bases = BTreeMap::from([("board-service".to_owned(), base)]);
    (
        HttpAdapter::new(binding, &bases, None).expect("the adapter is built"),
        server,
    )
}

fn read(view: &str, answer: &'static str) -> Result<Vec<Value>, String> {
    let (adapter, server) = declared(answer);
    let rows = adapter
        .read(&ess_ui_tui::ReadRequest {
            view: view.to_owned(),
            fixture: None,
            params: BTreeMap::new(),
        })
        .map(|read| read.rows);
    server.join().expect("the server ran");
    rows
}

fn run(answer: &'static str) -> Answer {
    let (mut adapter, server) = declared(answer);
    let answer = adapter.run("board.ArchiveCard", &BTreeMap::new());
    server.join().expect("the server ran");
    answer
}

#[test]
fn a_view_answer_is_read_as_its_binding_declares_it() {
    let rows = read(
        "board.Cards",
        "HTTP/1.1 200 OK\r\nContent-Length: 30\r\n\r\n{\"items\":[{\"id\":\"c-1\"}],\"n\":1}",
    )
    .expect("the wrapped array reads");
    assert_eq!(rows.len(), 1, "{rows:?}");
    let refused = read(
        "board.Cards",
        "HTTP/1.1 200 OK\r\nContent-Length: 23\r\n\r\n{\"rows\":[{\"id\":\"c-1\"}]}",
    );
    assert!(refused.is_err(), "{refused:?}");
    let none = read(
        "board.Card",
        "HTTP/1.1 404 Not Found\r\nContent-Length: 2\r\n\r\n{}",
    )
    .expect("the empty answer reads");
    assert!(none.is_empty(), "{none:?}");
    let one = read(
        "board.Card",
        "HTTP/1.1 200 OK\r\nContent-Length: 12\r\n\r\n{\"id\":\"c-1\"}",
    )
    .expect("the one row reads");
    assert_eq!(one.len(), 1, "{one:?}");
}

#[test]
fn a_command_answer_is_read_as_its_binding_declares_it() {
    assert_eq!(
        run("HTTP/1.1 204 No Content\r\nContent-Length: 0\r\n\r\n"),
        Answer::Accepted
    );
    assert_eq!(
        run("HTTP/1.1 409 Conflict\r\nContent-Length: 34\r\n\r\n{\"code\":\"card_locked\",\"since\":\"x\"}"),
        Answer::Refused {
            error: "catalog.board.CardLocked".to_owned(),
            payload: Some(serde_json::json!({"since": "x"})),
        }
    );
    assert_eq!(
        run("HTTP/1.1 401 Unauthorized\r\nContent-Length: 0\r\n\r\n"),
        Answer::NotGranted { actor: None }
    );
}
