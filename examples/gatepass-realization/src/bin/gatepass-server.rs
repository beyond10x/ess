//! The `gatepass` system, realized, on the wire.
//!
//! Forty lines, and the arrow points the way it always points here: this binary links the
//! hand-written realization into the generated surface, and the generated surface knows nothing
//! about it. `gatepass_server::pass_service::serve` binds, writes the startup record and answers
//! the routes the committed `OpenAPI` document declares; everything it answers *with* comes
//! through the port from [`gatepass_realization::linker`].
//!
//! # The port comes from the environment, not from an argument
//!
//! `PORT` unset or `0` binds an ephemeral port, which is what makes the gate's demonstration
//! deterministic: two of these run side by side without agreeing about a number in advance, and
//! each says in its startup record which port it took. There is no argument parsing here at all —
//! a synthesised surface takes no options, so there is nothing to parse.

use std::process::ExitCode;

use gatepass_types::actor::{Actor, Caller};

/// Who a request was sent by, from a demonstration credential: `authorization: Actor <qualified
/// actor name>`.
///
/// How a request proves who sent it is the realization's, never the contract's; a deployment
/// verifies a session, a token or a certificate here. This one trusts the header, which is exactly
/// what a real deployment must not do, and it is here so the demonstration can be driven as any
/// declared actor. A request naming none, or naming no declared actor, is authenticated as nobody
/// and the served surface refuses it.
fn authenticate(request: &gatepass_server::http::Request) -> Option<Caller> {
    let (_, value) = request
        .headers
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case("authorization"))?;
    let name = value.strip_prefix("Actor ")?;
    Actor::ALL
        .iter()
        .find(|actor| actor.name() == name)
        .map(|actor| Caller { actor: *actor })
}

fn main() -> ExitCode {
    let port = std::env::var("PORT").unwrap_or_else(|_| "0".to_owned());
    let address = format!("127.0.0.1:{port}");
    let mut assembled = gatepass_realization::linker::honest();
    match gatepass_server::pass_service::serve(&mut assembled.system, &address, authenticate) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{{\"log\":\"ess/1\",\"event\":\"system.stopped\",\"reason\":\"{error}\"}}");
            ExitCode::FAILURE
        }
    }
}
