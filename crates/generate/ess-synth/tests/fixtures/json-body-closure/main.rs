use shelf_server::http;
use shelf_server::shelf_service as surface;
use shelf_types::actor::{Actor, Caller};
use shelf_types::behaviour::Generated;

fn main() {
    let requests = std::fs::read_to_string(std::env::args().nth(1).expect("the requests")).expect("read");
    let mut system = shelf_system::System::new(shelf_service::ShelfService::new(Generated::new(
        shelf_server::memory::MemoryPorts::default(),
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
