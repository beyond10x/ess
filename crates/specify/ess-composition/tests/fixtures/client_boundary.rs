//! Executed against the emitted Todo/Usage client as a separate downstream crate.
//! The title types below are established by two-components/domains/todo.yaml;
//! the byte transport does not admit either request against that model.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use composition_fixture::{
    service_todo, AuthorityProvider, Client, ClientError, EndpointProvider, Operation,
    OperationKind, Service, Transport,
};

#[derive(Clone, Debug, PartialEq, Eq)]
struct FixtureAuthority {
    subject: &'static str,
}

struct Authority {
    value: FixtureAuthority,
    reads: Rc<Cell<usize>>,
}

impl AuthorityProvider for Authority {
    type Authority = FixtureAuthority;

    fn authority(&self) -> &FixtureAuthority {
        self.reads.set(self.reads.get() + 1);
        &self.value
    }
}

struct Endpoints {
    endpoint: Option<&'static str>,
    lookups: Rc<RefCell<Vec<Service>>>,
}

impl EndpointProvider for Endpoints {
    fn endpoint(&self, service: &Service) -> Option<&str> {
        self.lookups.borrow_mut().push(*service);
        self.endpoint
    }
}

#[derive(Debug)]
struct Call {
    endpoint: String,
    authority: FixtureAuthority,
    operation: Operation,
    payload: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum TransportError {
    Offline,
}

struct RecordingTransport {
    calls: Rc<RefCell<Vec<Call>>>,
    response: Result<Vec<u8>, TransportError>,
}

impl Transport<FixtureAuthority> for RecordingTransport {
    type Error = TransportError;

    fn execute(
        &self,
        endpoint: &str,
        authority: &FixtureAuthority,
        operation: Operation,
        payload: &[u8],
    ) -> Result<Vec<u8>, Self::Error> {
        self.calls.borrow_mut().push(Call {
            endpoint: endpoint.to_owned(),
            authority: authority.clone(),
            operation,
            payload: payload.to_vec(),
        });
        self.response.clone()
    }
}

fn assert_todo_identity(service: Service) {
    // Independent fixture expectations, not values copied from the descriptor
    // supplied to Client::execute or from the plan produced by the same emitter.
    assert_eq!(service.key(), "todo");
    assert_eq!(service.system(), "workbench");
    assert_eq!(service.version(), "v1");
    assert_eq!(service.component(), "todo-component");
    assert_eq!(
        service.source_digest(),
        "29fd772e3abb87d8042f8155577c3df9549219ace80c9e595d7da9783f7b7dda"
    );
}

#[test]
fn compatible_and_incompatible_titles_reach_the_same_selected_operation_unchanged() {
    let calls = Rc::new(RefCell::new(Vec::new()));
    let lookups = Rc::new(RefCell::new(Vec::new()));
    let authority_reads = Rc::new(Cell::new(0));
    let client = Client::new(
        Endpoints {
            endpoint: Some("recording://todo"),
            lookups: lookups.clone(),
        },
        Authority {
            value: FixtureAuthority {
                subject: "fixture-operator",
            },
            reads: authority_reads.clone(),
        },
        RecordingTransport {
            calls: calls.clone(),
            response: Ok(vec![0, 255, b'R', b'\n']),
        },
    );
    assert!(calls.borrow().is_empty());
    assert!(lookups.borrow().is_empty());
    assert_eq!(authority_reads.get(), 0);

    // Title is a String newtype. The second JSON is incompatible with that
    // declaration but both requests are simply forwarded by this client.
    assert_eq!(
        client.execute(
            service_todo::COMMAND_CREATE_LIST,
            br#"{"details":{"title":"Inbox"}}"#
        ),
        Ok(vec![0, 255, b'R', b'\n'])
    );
    assert_eq!(
        client.execute(
            service_todo::COMMAND_CREATE_LIST,
            br#"{"details":{"title":7}}"#
        ),
        Ok(vec![0, 255, b'R', b'\n'])
    );

    let observed = calls.borrow();
    assert_eq!(observed.len(), 2);
    assert_eq!(observed[0].payload, br#"{"details":{"title":"Inbox"}}"#);
    assert_eq!(observed[1].payload, br#"{"details":{"title":7}}"#);
    for call in observed.iter() {
        assert_eq!(call.endpoint, "recording://todo");
        assert_eq!(
            call.authority,
            FixtureAuthority {
                subject: "fixture-operator"
            }
        );
        assert_eq!(call.operation.service_key(), "todo");
        assert_eq!(call.operation.semantic(), "workbench.todo.CreateList");
        assert_eq!(call.operation.kind(), OperationKind::Command);
    }
    assert_eq!(lookups.borrow().len(), 2);
    for service in lookups.borrow().iter() {
        assert_todo_identity(*service);
    }
    assert_eq!(authority_reads.get(), 2);
}

#[test]
fn missing_endpoint_prevents_authority_lookup_and_transport() {
    let calls = Rc::new(RefCell::new(Vec::new()));
    let lookups = Rc::new(RefCell::new(Vec::new()));
    let authority_reads = Rc::new(Cell::new(0));
    let client = Client::new(
        Endpoints {
            endpoint: None,
            lookups: lookups.clone(),
        },
        Authority {
            value: FixtureAuthority {
                subject: "fixture-operator",
            },
            reads: authority_reads.clone(),
        },
        RecordingTransport {
            calls: calls.clone(),
            response: Ok(vec![42]),
        },
    );
    assert_eq!(
        client.execute(service_todo::COMMAND_CREATE_LIST, b"anything"),
        Err(ClientError::MissingEndpoint("todo"))
    );
    assert!(calls.borrow().is_empty());
    assert_eq!(authority_reads.get(), 0);
    assert_eq!(lookups.borrow().len(), 1);
    assert_todo_identity(lookups.borrow()[0]);
}

#[test]
fn transport_error_is_returned_without_reinterpretation() {
    let calls = Rc::new(RefCell::new(Vec::new()));
    let client = Client::new(
        Endpoints {
            endpoint: Some("recording://todo"),
            lookups: Rc::default(),
        },
        Authority {
            value: FixtureAuthority {
                subject: "fixture-operator",
            },
            reads: Rc::default(),
        },
        RecordingTransport {
            calls: calls.clone(),
            response: Err(TransportError::Offline),
        },
    );
    assert_eq!(
        client.execute(
            service_todo::COMMAND_CREATE_LIST,
            br#"{"details":{"title":7}}"#
        ),
        Err(ClientError::Transport(TransportError::Offline))
    );
    assert_eq!(calls.borrow().len(), 1);
    assert_eq!(calls.borrow()[0].payload, br#"{"details":{"title":7}}"#);
}
