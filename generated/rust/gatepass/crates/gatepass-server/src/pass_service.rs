// generated from gatepass v1
// model digest 7d021b6ebe1c4715096f165d6564389be0f46311f67d791ed748f627314d611c
// contract digest 2668f3034afb388a33d7add462e15a830b6010fbfe83101f1dd2526fa18d52ed
// do not edit: regenerate with `ess synthesize`

//! The `pass-service` component of `gatepass` v1, on the wire.
//!
//! The specification says this component's callers are not deployed with it, so its surface
//! exists on a wire. Which wire is derived rather than chosen: the one contract this model
//! projects for a command surface is the `OpenAPI` document, and an `OpenAPI` document is an
//! HTTP contract. The document is beside this file, served verbatim at `/openapi.json`.

use crate::{entry, http, json, wire};

/// The contract this surface answers, byte for byte as `generated/` commits it.
///
/// Embedded rather than rebuilt at run time: a server that regenerated its own contract could
/// publish one the repository never reviewed.
pub const OPENAPI: &str = include_str!("pass-service.openapi.json");

/// The prose the same model produced, byte for byte as the documentation projection wrote it.
pub const DOCS: &str = include_str!("pass-service.docs.md");

/// Every route this surface answers, in path order.
///
/// The same set the `OpenAPI` document declares, plus the two documents about the surface
/// itself, which no specification construct names and nothing can therefore derive. A path
/// absent from this table is answered with `404`, including one the document declares and this
/// table forgot — which is the failure a table computed twice would hide.
pub const ROUTES: &[(&str, &str)] = &[
    ("GET", "/docs"),
    ("GET", "/openapi.json"),
    ("POST", "/visits/commands/admit-visitor"),
    ("POST", "/visits/commands/register-visit"),
    ("POST", "/visits/commands/sign-out-visitor"),
    ("GET", "/visits/views/by-id"),
    ("GET", "/visits/views/expected"),
];

/// What this process says about itself as it starts, before it answers anything.
///
/// Three lines of JSON on standard output, in this order, every member of them derived from the
/// specification — except `runtime`, which is appended by the emitted code below and holds what
/// is true of *this process*: the language it was synthesised into, and the address it bound.
/// Everything outside `runtime` is the same in every language this plan is emitted into, and
/// `cargo xtask synth --check` starts both and compares them.
pub const STARTUP: &[&str] = &[
    "{\"log\":\"ess/1\",\"event\":\"system.starting\",\"system\":\"gatepass\",\"version\":\"v1\",\"model_digest\":\"7d021b6ebe1c4715096f165d6564389be0f46311f67d791ed748f627314d611c\",\"contract_digest\":\"2668f3034afb388a33d7add462e15a830b6010fbfe83101f1dd2526fa18d52ed\",\"components\":[\"pass-service\"],\"capabilities\":{\"generated\":28,\"obligations\":1,\"refused\":0}",
    "{\"log\":\"ess/1\",\"event\":\"surface.serving\",\"component\":\"pass-service\",\"reached_by\":\"network\",\"transport\":\"http/1.1\",\"routes\":7,\"paths\":[{\"method\":\"GET\",\"path\":\"/docs\",\"serves\":\"documentation\",\"name\":\"docs\"},{\"method\":\"GET\",\"path\":\"/openapi.json\",\"serves\":\"contract\",\"name\":\"openapi\"},{\"method\":\"POST\",\"path\":\"/visits/commands/admit-visitor\",\"serves\":\"command\",\"name\":\"gatepass.visit.AdmitVisitor\"},{\"method\":\"POST\",\"path\":\"/visits/commands/register-visit\",\"serves\":\"command\",\"name\":\"gatepass.visit.RegisterVisit\"},{\"method\":\"POST\",\"path\":\"/visits/commands/sign-out-visitor\",\"serves\":\"command\",\"name\":\"gatepass.visit.SignOutVisitor\"},{\"method\":\"GET\",\"path\":\"/visits/views/by-id\",\"serves\":\"view\",\"name\":\"gatepass.visit.VisitById\"},{\"method\":\"GET\",\"path\":\"/visits/views/expected\",\"serves\":\"view\",\"name\":\"gatepass.visit.ExpectedVisits\"}]",
    "{\"log\":\"ess/1\",\"event\":\"system.ready\",\"system\":\"gatepass\",\"surfaces\":1",
];

/// Writes the startup record, with this process's own facts closing each line.
fn announce(address: &std::net::SocketAddr) {
    for facts in STARTUP {
        let mut line = String::from(*facts);
        line.push_str(",\"runtime\":{\"address\":");
        json::push_text(&mut line, &address.to_string());
        line.push_str(",\"language\":\"rust\",\"port\":");
        json::push_integer(&mut line, i64::from(address.port()));
        line.push_str("}}");
        println!("{line}");
    }
}

/// Serves `pass-service` at `address`, and does not return while it can answer.
///
/// `address` may name port `0`, which binds an ephemeral port; the startup record says which one
/// was taken, because a caller that cannot learn the port cannot make a request.
///
/// It chooses no realization. Every command reaches the port, and a port over unimplemented
/// obligations answers the typed refusal this surface reports as `501` — the honest empty
/// state rather than a server that pretends.
///
/// `authenticate` is the realization's: it says who each request was sent by, or `None`, and
/// [`dispatch`] checks that caller's grant before the command runs. Nothing here reads an
/// actor from the request itself.
///
/// One connection at a time: each is dropped after [`http::READ_TIMEOUT`] without a byte, or
/// [`http::WRITE_TIMEOUT`] of a stalled write, and whatever fails on one connection — a caller
/// that hung up before reading its answer, a reset, a failed accept — ends that connection only.
///
/// # Errors
///
/// What binding the address refuses: the address is taken, or the port is privileged.
pub fn serve<PassServiceBehaviors>(system: &mut gatepass_system::System<PassServiceBehaviors>, address: &str, authenticate: impl Fn(&http::Request) -> Option<gatepass_types::actor::Caller>) -> std::io::Result<()>
where
    PassServiceBehaviors: gatepass_types::visit::obligations::AdmitVisitorBehavior + gatepass_types::visit::obligations::RegisterVisitBehavior + gatepass_types::visit::obligations::SignOutVisitorBehavior + gatepass_types::visit::obligations::ExpectedVisitsQuery + gatepass_types::visit::obligations::VisitByIdQuery,
{
    let listener = std::net::TcpListener::bind(address)?;
    announce(&listener.local_addr()?);
    for connection in listener.incoming() {
        // What fails on one connection ends that connection, never this loop: a caller that
        // gave up before it was accepted, or before it read its answer, is that caller's affair.
        let Ok(connection) = connection else {
            // An accept the listener itself failed (no descriptor left) would fail again at
            // once: pause before the next.
            std::thread::sleep(std::time::Duration::from_millis(10));
            continue;
        };
        let _ = connection.set_read_timeout(Some(http::READ_TIMEOUT));
        let _ = connection.set_write_timeout(Some(http::WRITE_TIMEOUT));
        let mut reader = std::io::BufReader::new(connection);
        let (answer, refused) = match http::read(&mut reader) {
            Ok(request) => (dispatch(system, authenticate(&request).as_ref(), &request), false),
            Err(refusal) => (refusal, true),
        };
        let mut stream = reader.into_inner();
        if http::write(&mut stream, &answer).is_ok() && refused {
            http::linger(&mut stream);
        }
    }
    Ok(())
}

/// Answers one request.
///
/// A path this table does not hold is a `404` naming where the whole table is published; a
/// path it holds under a different method is a `405` naming the one it answers. Neither is a
/// status the contract declares, and neither should be: both are facts about a transport rather
/// than about any command.
///
/// Public so a caller can hand it a request it built itself: [`serve`] is this function behind a
/// socket, and nothing else.
///
/// `caller` is who the realization authenticated the request as, or `None`. Every command
/// checks its grant before it runs, and answers the standard refusal when the caller is none
/// or is an actor the specification does not grant the command.
pub fn dispatch<PassServiceBehaviors>(system: &mut gatepass_system::System<PassServiceBehaviors>, caller: Option<&gatepass_types::actor::Caller>, request: &http::Request) -> http::Response
where
    PassServiceBehaviors: gatepass_types::visit::obligations::AdmitVisitorBehavior + gatepass_types::visit::obligations::RegisterVisitBehavior + gatepass_types::visit::obligations::SignOutVisitorBehavior + gatepass_types::visit::obligations::ExpectedVisitsQuery + gatepass_types::visit::obligations::VisitByIdQuery,
{
    match request.path.as_str() {
        "/docs" => {
            if request.method != "GET" {
                return http::method_not_allowed("GET");
            }
            http::Response::new(200, http::MARKDOWN, DOCS)
        }
        "/openapi.json" => {
            if request.method != "GET" {
                return http::method_not_allowed("GET");
            }
            http::Response::new(200, http::JSON, OPENAPI)
        }
        "/visits/commands/admit-visitor" => {
            if request.method != "POST" {
                return http::method_not_allowed("POST");
            }
            if let Err(actor) = admit(caller, "gatepass.visit.AdmitVisitor") {
                return not_granted(actor);
            }
            serve_gatepass_visit_admit_visitor(system, &request.body)
        }
        "/visits/commands/register-visit" => {
            if request.method != "POST" {
                return http::method_not_allowed("POST");
            }
            if let Err(actor) = admit(caller, "gatepass.visit.RegisterVisit") {
                return not_granted(actor);
            }
            serve_gatepass_visit_register_visit(system, &request.body)
        }
        "/visits/commands/sign-out-visitor" => {
            if request.method != "POST" {
                return http::method_not_allowed("POST");
            }
            if let Err(actor) = admit(caller, "gatepass.visit.SignOutVisitor") {
                return not_granted(actor);
            }
            serve_gatepass_visit_sign_out_visitor(system, &request.body)
        }
        "/visits/views/by-id" => {
            if request.method != "GET" {
                return http::method_not_allowed("GET");
            }
            http::answer(run_gatepass_visit_visit_by_id(system))
        }
        "/visits/views/expected" => {
            if request.method != "GET" {
                return http::method_not_allowed("GET");
            }
            http::answer(run_gatepass_visit_expected_visits(system))
        }
        other => http::Response::refusal(
            404,
            &format!("`{other}` is not a path this surface declares; `GET /openapi.json` publishes every one that is"),
        ),
    }
}

/// Runs one command or view of `pass-service` by its qualified name, with no transport.
///
/// The same decoding, refusals and rendering the HTTP routes use — each route and this function call
/// one `run_*` function — so a conformance runner or an in-process caller drives the system
/// without a socket and without a dispatch table of its own. `Ok` is the declared outcome, as the
/// route's body renders it; a view ignores `input`, as its `GET` route ignores a body.
///
/// # Errors
///
/// [`entry::Refused::Unknown`] naming `name` when this surface declares no command or view
/// by it; [`entry::Refused::Input`] when `input` is not the command's declared input (the
/// route's `400`); [`entry::Refused::Unmet`] when the port reports an unmet obligation, and
/// [`entry::Refused::Undelivered`] when the command took effect and delivering what it published
/// failed (the route's `501`, with `committed` `false` and `true`).
/// [`entry::Refused::NotGranted`] when `caller` — who the realization authenticated the call
/// as — is none, or is an actor the specification does not grant the command; checked before
/// the command runs (the route's `403`).
pub fn handle<PassServiceBehaviors>(system: &mut gatepass_system::System<PassServiceBehaviors>, caller: Option<&gatepass_types::actor::Caller>, name: &str, input: json::Value) -> Result<json::Value, entry::Refused>
where
    PassServiceBehaviors: gatepass_types::visit::obligations::AdmitVisitorBehavior + gatepass_types::visit::obligations::RegisterVisitBehavior + gatepass_types::visit::obligations::SignOutVisitorBehavior + gatepass_types::visit::obligations::ExpectedVisitsQuery + gatepass_types::visit::obligations::VisitByIdQuery,
{
    let answered = match name {
        "gatepass.visit.AdmitVisitor" => match admit(caller, "gatepass.visit.AdmitVisitor") {
            Ok(()) => run_gatepass_visit_admit_visitor(system, &input),
            Err(actor) => Err(entry::Refused::NotGranted(actor.map(str::to_owned))),
        },
        "gatepass.visit.ExpectedVisits" => run_gatepass_visit_expected_visits(system),
        "gatepass.visit.RegisterVisit" => match admit(caller, "gatepass.visit.RegisterVisit") {
            Ok(()) => run_gatepass_visit_register_visit(system, &input),
            Err(actor) => Err(entry::Refused::NotGranted(actor.map(str::to_owned))),
        },
        "gatepass.visit.SignOutVisitor" => match admit(caller, "gatepass.visit.SignOutVisitor") {
            Ok(()) => run_gatepass_visit_sign_out_visitor(system, &input),
            Err(actor) => Err(entry::Refused::NotGranted(actor.map(str::to_owned))),
        },
        "gatepass.visit.VisitById" => run_gatepass_visit_visit_by_id(system),
        other => return Err(entry::Refused::Unknown(other.to_owned())),
    };
    let (_, body) = answered?;
    Ok(entry::read(&body))
}

/// Nothing, where `caller` may invoke `command`; otherwise the actor the standard refusal names,
/// `None` where the request was authenticated as no actor.
///
/// Checked before the command runs, on the caller the realization authenticated the request
/// as and never on anything the request says about itself. Public, so code that drives the system
/// in process checks the grant exactly as every route does.
pub fn admit(caller: Option<&gatepass_types::actor::Caller>, command: &str) -> Result<(), Option<&'static str>> {
    match caller {
        Some(caller) if caller.may(command) => Ok(()),
        Some(caller) => Err(Some(caller.actor.name())),
        None => Err(None),
    }
}

/// The standard refusal for an actor no grant admits, as the contract declares it: `403`,
/// `{"refused": "not granted", "actor": <name or null>}`.
fn not_granted(actor: Option<&str>) -> http::Response {
    let mut body = String::from("{");
    json::member(&mut body, "refused");
    json::push_text(&mut body, "not granted");
    json::member(&mut body, "actor");
    match actor {
        Some(actor) => json::push_text(&mut body, actor),
        None => body.push_str("null"),
    }
    body.push('}');
    http::Response::new(403, http::JSON, body)
}

/// `POST` `gatepass.visit.AdmitVisitor`: reads the declared input, runs the port, answers the declared outcome.
fn serve_gatepass_visit_admit_visitor<PassServiceBehaviors>(system: &mut gatepass_system::System<PassServiceBehaviors>, body: &[u8]) -> http::Response
where
    PassServiceBehaviors: gatepass_types::visit::obligations::AdmitVisitorBehavior + gatepass_types::visit::obligations::RegisterVisitBehavior + gatepass_types::visit::obligations::SignOutVisitorBehavior + gatepass_types::visit::obligations::ExpectedVisitsQuery + gatepass_types::visit::obligations::VisitByIdQuery,
{
    let text = match std::str::from_utf8(body) {
        Ok(text) => text,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not UTF-8: {error}"));
        }
    };
    let value = match json::parse(text) {
        Ok(value) => value,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not JSON: {error}"));
        }
    };
    http::answer(run_gatepass_visit_admit_visitor(system, &value))
}

/// `gatepass.visit.AdmitVisitor` from its input as a JSON value: decode, run the port, render the declared outcome.
///
/// The one path the `POST` route and [`handle`] share. A decoding failure is located under
/// `body`, as the route reports it.
fn run_gatepass_visit_admit_visitor<PassServiceBehaviors>(system: &mut gatepass_system::System<PassServiceBehaviors>, value: &json::Value) -> Result<(u16, String), entry::Refused>
where
    PassServiceBehaviors: gatepass_types::visit::obligations::AdmitVisitorBehavior + gatepass_types::visit::obligations::RegisterVisitBehavior + gatepass_types::visit::obligations::SignOutVisitorBehavior + gatepass_types::visit::obligations::ExpectedVisitsQuery + gatepass_types::visit::obligations::VisitByIdQuery,
{
    let input = match wire::decode_command_gatepass_visit_admit_visitor(value, "body") {
        Ok(input) => input,
        Err(error) => {
            // `400` and not `422`: this is a body the schema decides, which is the difference
            // between fixing a value and fixing a serialiser.
            return Err(entry::Refused::Input(format!("{error}")));
        }
    };
    let outcome = match system.pass_service.admit_visitor(input) {
        Ok(outcome) => outcome,
        Err(unmet) => return Err(entry::Refused::Unmet(format!("{unmet}"))),
    };
    // Deliver what this command published to every binding that reacts to it, then take it
    // off the log: a long-running server keeps nothing from one request to the next.
    let delivered = system.pump();
    let _ = system.take_published();
    if let Err(failure) = delivered {
        return Err(entry::Refused::Undelivered(format!("delivering what the command published: {failure}")));
    }
    Ok(answer_gatepass_visit_admit_visitor(&outcome))
}

/// One declared outcome of `gatepass.visit.AdmitVisitor`: the branch that was taken, every event it published in
/// publication order, the declared error where there is one, and that error's own payload —
/// with the status the contract declares for that branch.
fn answer_gatepass_visit_admit_visitor(outcome: &gatepass_types::visit::AdmitVisitorOutcome) -> (u16, String) {
    let mut body = String::from("{");
    let status = match outcome {
        gatepass_types::visit::AdmitVisitorOutcome::Admitted { visitor_admitted, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "admitted");
            json::member(&mut body, "published");
            body.push('[');
            body.push('{');
            json::member(&mut body, "event");
            json::push_text(&mut body, "gatepass.visit.VisitorAdmitted");
            json::member(&mut body, "payload");
            wire::encode_event_gatepass_visit_visitor_admitted(visitor_admitted, &mut body);
            body.push('}');
            body.push(']');
            202
        }
        gatepass_types::visit::AdmitVisitorOutcome::WrongState { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "wrong-state");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "gatepass.visit.VisitStateConflict");
            json::member(&mut body, "payload");
            wire::encode_error_gatepass_visit_visit_state_conflict(error, &mut body);
            409
        }
        gatepass_types::visit::AdmitVisitorOutcome::WrongStateUnknownInstance => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "wrong-state");
            json::member(&mut body, "published");
            body.push_str("[]");
            json::member(&mut body, "error");
            json::push_text(&mut body, "gatepass.visit.VisitStateConflict");
            409
        }
    };
    body.push('}');
    (status, body)
}

/// `POST` `gatepass.visit.RegisterVisit`: reads the declared input, runs the port, answers the declared outcome.
fn serve_gatepass_visit_register_visit<PassServiceBehaviors>(system: &mut gatepass_system::System<PassServiceBehaviors>, body: &[u8]) -> http::Response
where
    PassServiceBehaviors: gatepass_types::visit::obligations::AdmitVisitorBehavior + gatepass_types::visit::obligations::RegisterVisitBehavior + gatepass_types::visit::obligations::SignOutVisitorBehavior + gatepass_types::visit::obligations::ExpectedVisitsQuery + gatepass_types::visit::obligations::VisitByIdQuery,
{
    let text = match std::str::from_utf8(body) {
        Ok(text) => text,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not UTF-8: {error}"));
        }
    };
    let value = match json::parse(text) {
        Ok(value) => value,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not JSON: {error}"));
        }
    };
    http::answer(run_gatepass_visit_register_visit(system, &value))
}

/// `gatepass.visit.RegisterVisit` from its input as a JSON value: decode, run the port, render the declared outcome.
///
/// The one path the `POST` route and [`handle`] share. A decoding failure is located under
/// `body`, as the route reports it.
fn run_gatepass_visit_register_visit<PassServiceBehaviors>(system: &mut gatepass_system::System<PassServiceBehaviors>, value: &json::Value) -> Result<(u16, String), entry::Refused>
where
    PassServiceBehaviors: gatepass_types::visit::obligations::AdmitVisitorBehavior + gatepass_types::visit::obligations::RegisterVisitBehavior + gatepass_types::visit::obligations::SignOutVisitorBehavior + gatepass_types::visit::obligations::ExpectedVisitsQuery + gatepass_types::visit::obligations::VisitByIdQuery,
{
    let input = match wire::decode_command_gatepass_visit_register_visit(value, "body") {
        Ok(input) => input,
        Err(error) => {
            // `400` and not `422`: this is a body the schema decides, which is the difference
            // between fixing a value and fixing a serialiser.
            return Err(entry::Refused::Input(format!("{error}")));
        }
    };
    let outcome = match system.pass_service.register_visit(input) {
        Ok(outcome) => outcome,
        Err(unmet) => return Err(entry::Refused::Unmet(format!("{unmet}"))),
    };
    // Deliver what this command published to every binding that reacts to it, then take it
    // off the log: a long-running server keeps nothing from one request to the next.
    let delivered = system.pump();
    let _ = system.take_published();
    if let Err(failure) = delivered {
        return Err(entry::Refused::Undelivered(format!("delivering what the command published: {failure}")));
    }
    Ok(answer_gatepass_visit_register_visit(&outcome))
}

/// One declared outcome of `gatepass.visit.RegisterVisit`: the branch that was taken, every event it published in
/// publication order, the declared error where there is one, and that error's own payload —
/// with the status the contract declares for that branch.
fn answer_gatepass_visit_register_visit(outcome: &gatepass_types::visit::RegisterVisitOutcome) -> (u16, String) {
    let mut body = String::from("{");
    let status = match outcome {
        gatepass_types::visit::RegisterVisitOutcome::Registered { visit_registered, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "registered");
            json::member(&mut body, "published");
            body.push('[');
            body.push('{');
            json::member(&mut body, "event");
            json::push_text(&mut body, "gatepass.visit.VisitRegistered");
            json::member(&mut body, "payload");
            wire::encode_event_gatepass_visit_visit_registered(visit_registered, &mut body);
            body.push('}');
            body.push(']');
            202
        }
        gatepass_types::visit::RegisterVisitOutcome::Refused { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "refused");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "gatepass.visit.InvalidVisitLength");
            json::member(&mut body, "payload");
            wire::encode_error_gatepass_visit_invalid_visit_length(error, &mut body);
            422
        }
    };
    body.push('}');
    (status, body)
}

/// `POST` `gatepass.visit.SignOutVisitor`: reads the declared input, runs the port, answers the declared outcome.
fn serve_gatepass_visit_sign_out_visitor<PassServiceBehaviors>(system: &mut gatepass_system::System<PassServiceBehaviors>, body: &[u8]) -> http::Response
where
    PassServiceBehaviors: gatepass_types::visit::obligations::AdmitVisitorBehavior + gatepass_types::visit::obligations::RegisterVisitBehavior + gatepass_types::visit::obligations::SignOutVisitorBehavior + gatepass_types::visit::obligations::ExpectedVisitsQuery + gatepass_types::visit::obligations::VisitByIdQuery,
{
    let text = match std::str::from_utf8(body) {
        Ok(text) => text,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not UTF-8: {error}"));
        }
    };
    let value = match json::parse(text) {
        Ok(value) => value,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not JSON: {error}"));
        }
    };
    http::answer(run_gatepass_visit_sign_out_visitor(system, &value))
}

/// `gatepass.visit.SignOutVisitor` from its input as a JSON value: decode, run the port, render the declared outcome.
///
/// The one path the `POST` route and [`handle`] share. A decoding failure is located under
/// `body`, as the route reports it.
fn run_gatepass_visit_sign_out_visitor<PassServiceBehaviors>(system: &mut gatepass_system::System<PassServiceBehaviors>, value: &json::Value) -> Result<(u16, String), entry::Refused>
where
    PassServiceBehaviors: gatepass_types::visit::obligations::AdmitVisitorBehavior + gatepass_types::visit::obligations::RegisterVisitBehavior + gatepass_types::visit::obligations::SignOutVisitorBehavior + gatepass_types::visit::obligations::ExpectedVisitsQuery + gatepass_types::visit::obligations::VisitByIdQuery,
{
    let input = match wire::decode_command_gatepass_visit_sign_out_visitor(value, "body") {
        Ok(input) => input,
        Err(error) => {
            // `400` and not `422`: this is a body the schema decides, which is the difference
            // between fixing a value and fixing a serialiser.
            return Err(entry::Refused::Input(format!("{error}")));
        }
    };
    let outcome = match system.pass_service.sign_out_visitor(input) {
        Ok(outcome) => outcome,
        Err(unmet) => return Err(entry::Refused::Unmet(format!("{unmet}"))),
    };
    // Deliver what this command published to every binding that reacts to it, then take it
    // off the log: a long-running server keeps nothing from one request to the next.
    let delivered = system.pump();
    let _ = system.take_published();
    if let Err(failure) = delivered {
        return Err(entry::Refused::Undelivered(format!("delivering what the command published: {failure}")));
    }
    Ok(answer_gatepass_visit_sign_out_visitor(&outcome))
}

/// One declared outcome of `gatepass.visit.SignOutVisitor`: the branch that was taken, every event it published in
/// publication order, the declared error where there is one, and that error's own payload —
/// with the status the contract declares for that branch.
fn answer_gatepass_visit_sign_out_visitor(outcome: &gatepass_types::visit::SignOutVisitorOutcome) -> (u16, String) {
    let mut body = String::from("{");
    let status = match outcome {
        gatepass_types::visit::SignOutVisitorOutcome::SignedOut { visitor_departed, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "signed-out");
            json::member(&mut body, "published");
            body.push('[');
            body.push('{');
            json::member(&mut body, "event");
            json::push_text(&mut body, "gatepass.visit.VisitorDeparted");
            json::member(&mut body, "payload");
            wire::encode_event_gatepass_visit_visitor_departed(visitor_departed, &mut body);
            body.push('}');
            body.push(']');
            202
        }
        gatepass_types::visit::SignOutVisitorOutcome::WrongState { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "wrong-state");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "gatepass.visit.VisitStateConflict");
            json::member(&mut body, "payload");
            wire::encode_error_gatepass_visit_visit_state_conflict(error, &mut body);
            409
        }
        gatepass_types::visit::SignOutVisitorOutcome::WrongStateUnknownInstance => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "wrong-state");
            json::member(&mut body, "published");
            body.push_str("[]");
            json::member(&mut body, "error");
            json::push_text(&mut body, "gatepass.visit.VisitStateConflict");
            409
        }
    };
    body.push('}');
    (status, body)
}

/// `GET` `gatepass.visit.VisitById` at `eventual` consistency: every row the owed projection holds.
///
/// The one path the `GET` route and [`handle`] share.
fn run_gatepass_visit_visit_by_id<PassServiceBehaviors>(system: &gatepass_system::System<PassServiceBehaviors>) -> Result<(u16, String), entry::Refused>
where
    PassServiceBehaviors: gatepass_types::visit::obligations::AdmitVisitorBehavior + gatepass_types::visit::obligations::RegisterVisitBehavior + gatepass_types::visit::obligations::SignOutVisitorBehavior + gatepass_types::visit::obligations::ExpectedVisitsQuery + gatepass_types::visit::obligations::VisitByIdQuery,
{
    match system.pass_service.visit_by_id() {
        Ok(rows) => {
            let mut body = String::from("{");
            json::member(&mut body, "rows");
            body.push('[');
            for (position, row) in rows.iter().enumerate() {
                if position > 0 {
                    body.push(',');
                }
                wire::encode_view_gatepass_visit_visit_by_id(row, &mut body);
            }
            body.push(']');
            body.push('}');
            Ok((200, body))
        }
        Err(unmet) => Err(entry::Refused::Unmet(format!("{unmet}"))),
    }
}

/// `GET` `gatepass.visit.ExpectedVisits` at `read_your_writes` consistency: every row the owed projection holds.
///
/// The one path the `GET` route and [`handle`] share.
fn run_gatepass_visit_expected_visits<PassServiceBehaviors>(system: &gatepass_system::System<PassServiceBehaviors>) -> Result<(u16, String), entry::Refused>
where
    PassServiceBehaviors: gatepass_types::visit::obligations::AdmitVisitorBehavior + gatepass_types::visit::obligations::RegisterVisitBehavior + gatepass_types::visit::obligations::SignOutVisitorBehavior + gatepass_types::visit::obligations::ExpectedVisitsQuery + gatepass_types::visit::obligations::VisitByIdQuery,
{
    match system.pass_service.expected_visits() {
        Ok(rows) => {
            let mut body = String::from("{");
            json::member(&mut body, "rows");
            body.push('[');
            for (position, row) in rows.iter().enumerate() {
                if position > 0 {
                    body.push(',');
                }
                wire::encode_view_gatepass_visit_expected_visits(row, &mut body);
            }
            body.push(']');
            body.push('}');
            Ok((200, body))
        }
        Err(unmet) => Err(entry::Refused::Unmet(format!("{unmet}"))),
    }
}
