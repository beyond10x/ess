use std::fmt::Write as _;

use rack_server::{http, json};
use rack_server::rack_service as surface;
use rack_types::actor::{Actor, Caller};
use rack_types::behaviour::Generated;

fn main() {
    let requests = std::fs::read_to_string(std::env::args().nth(1).expect("the requests")).expect("read");
    let corpus = std::fs::read_to_string(std::env::args().nth(2).expect("the corpus")).expect("read");
    for (index, text) in corpus.lines().enumerate() {
        let held = json::Value::Text(text.to_owned());
        let value = json::timestamp_at(&held, "body.t", "an RFC 3339 instant");
        let key = json::key_timestamp(text, "body.k");
        let at = value.as_ref().err().map(|error| error.at.clone()).unwrap_or_default();
        println!("stamp\t{index}\t{}\t{}\t{at}", value.is_ok(), key.is_ok());
    }
    let started = std::time::Instant::now();
    let long = format!("2026-10-29T12:00:00.{}Z", "9".repeat(4_000_000));
    let long_ok = json::timestamp_at(&json::Value::Text(long.clone()), "t", "x").is_ok();
    let cut = format!("{}X", &long[..long.len() - 1]);
    let cut_ok = json::timestamp_at(&json::Value::Text(cut), "t", "x").is_ok();
    let declared = ["a", "b", "c"];
    let mut members = vec![("a".to_owned(), json::Value::Null); 200_000];
    let many_ok = json::closed_at(&json::Value::Object(members.clone()), "body", &declared).is_ok();
    members.push(("d".to_owned(), json::Value::Null));
    let last_at = json::closed_at(&json::Value::Object(members), "body", &declared)
        .err()
        .map(|error| error.at)
        .unwrap_or_default();
    let mut text = String::from("{");
    for index in 0..100_000 {
        let _ = write!(text, "\"a\":{index},");
    }
    text.push_str("\"zz\":1}");
    let parsed = json::parse(&text).expect("JSON");
    let parsed_at = json::closed_at(&parsed, "body", &declared)
        .err()
        .map(|error| error.at)
        .unwrap_or_default();
    println!(
        "hostile\t{long_ok}\t{cut_ok}\t{many_ok}\t{last_at}\t{parsed_at}\t{}",
        started.elapsed().as_millis()
    );
    let first_bad = json::parse(r#"{"t":"yesterday","t":"2026-10-29T12:00:00Z"}"#).expect("JSON");
    let first = json::timestamp_at(json::member_at(&first_bad, "body", "t").expect("present"), "body.t", "x");
    let last_bad = json::parse(r#"{"t":"2026-10-29T12:00:00Z","t":"yesterday"}"#).expect("JSON");
    let last = json::timestamp_at(json::member_at(&last_bad, "body", "t").expect("present"), "body.t", "x");
    println!("duplicate\t{}\t{}", first.is_ok(), last == Ok("2026-10-29T12:00:00Z"));
    let mut system = rack_system::System::new(rack_service::RackService::new(Generated::new(
        rack_server::memory::MemoryPorts::default(),
    )));
    let clerk = Caller { actor: Actor::ALL[0] };
    for line in requests.lines() {
        let fields: Vec<&str> = line.split('\t').collect();
        let [label, method, path, body] = fields[..] else {
            panic!("a request line has four fields: {line}");
        };
        let request = http::Request {
            method: method.to_owned(),
            path: path.to_owned(),
            query: String::new(),
            headers: vec![("content-type".to_owned(), http::JSON.to_owned())],
            body: body.as_bytes().to_vec(),
        };
        let answered = surface::dispatch(&mut system, Some(&clerk), &request);
        println!("{label}\t{}\t{}", answered.status, answered.body.replace('\n', " "));
    }
}
