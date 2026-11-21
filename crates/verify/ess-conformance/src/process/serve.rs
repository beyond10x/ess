//! The protocol's other end: request lines in, any [`ConformanceTarget`]'s answers out.

use std::fmt;
use std::io::{BufRead, Read, Write};

use serde::Serialize;

use super::exact::{self, Bad, ExactMap, Value};
use super::{wire, Op, MAX_LINE, TARGET_FORMAT};
use crate::target::{ConformanceTarget, TargetError};

/// Why [`serve`] stopped before its input ended, by kind, line number and byte offset only.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServeError {
    /// Reading a request or writing an answer failed.
    Io(std::io::ErrorKind),
    /// A request line longer than [`MAX_LINE`] bytes.
    LineTooLong {
        /// Which line, counting from 1.
        line: u64,
    },
    /// A request line that is not UTF-8.
    NotUtf8 {
        /// Which line, counting from 1.
        line: u64,
        /// The first byte that is not.
        at: usize,
    },
    /// A request line that is not one JSON value.
    NotJson {
        /// Which line, counting from 1.
        line: u64,
        /// Where reading stopped.
        at: usize,
    },
    /// A request whose envelope or known operation's members do not decode.
    NotARequest {
        /// Which line, counting from 1.
        line: u64,
        /// The value that does not match.
        at: usize,
    },
    /// A number the exact reader refuses.
    NumberOutOfRange {
        /// Which line, counting from 1.
        line: u64,
        /// Where the number starts.
        at: usize,
    },
}

impl ServeError {
    /// The kind's name: `io`, `line-too-long`, `not-utf8`, `not-json`, `not-a-request` or
    /// `number-out-of-range`.
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::Io(_) => "io",
            Self::LineTooLong { .. } => "line-too-long",
            Self::NotUtf8 { .. } => "not-utf8",
            Self::NotJson { .. } => "not-json",
            Self::NotARequest { .. } => "not-a-request",
            Self::NumberOutOfRange { .. } => "number-out-of-range",
        }
    }
}

impl fmt::Display for ServeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(kind) => write!(
                f,
                "reading a request or writing an answer failed ({kind:?})"
            ),
            Self::LineTooLong { line } => {
                write!(f, "request line {line} exceeds {MAX_LINE} bytes")
            }
            Self::NotUtf8 { line, at } => {
                write!(f, "request line {line} is not UTF-8 at byte {at}")
            }
            Self::NotJson { line, at } => write!(f, "request line {line} is not JSON at byte {at}"),
            Self::NotARequest { line, at } => write!(
                f,
                "request line {line} is not a request of ess-conformance-target/1 at byte {at}"
            ),
            Self::NumberOutOfRange { line, at } => write!(
                f,
                "request line {line} holds a number outside the domain at byte {at}"
            ),
        }
    }
}

impl std::error::Error for ServeError {}

/// Answers request lines from `input` with `target`, one answer line per request on `output`, until
/// `input` ends.
///
/// The inverse of [`ProcessTarget`](super::ProcessTarget): the handshake is answered with
/// `target.identity()` speaking [`TARGET_FORMAT`], each operation by the trait method it names, and
/// an `op` this build does not know with `unsupported`. Each answer is flushed before the next
/// request is read. Returns `Ok` at end of input, and an error, naming no byte of the line, at the
/// first request it cannot decode.
pub fn serve<T: ConformanceTarget + ?Sized>(
    target: &T,
    mut input: impl BufRead,
    mut output: impl Write,
) -> Result<(), ServeError> {
    let mut number = 0_u64;
    let mut bytes = Vec::new();
    loop {
        bytes.clear();
        let limit = u64::try_from(MAX_LINE)
            .unwrap_or(u64::MAX)
            .saturating_add(1);
        let read = input
            .by_ref()
            .take(limit)
            .read_until(b'\n', &mut bytes)
            .map_err(|e| ServeError::Io(e.kind()))?;
        if read == 0 {
            return Ok(());
        }
        number += 1;
        let line = number;
        if bytes.last() == Some(&b'\n') {
            bytes.pop();
        } else if bytes.len() > MAX_LINE {
            return Err(ServeError::LineTooLong { line });
        }
        let text = std::str::from_utf8(&bytes).map_err(|e| ServeError::NotUtf8 {
            line,
            at: e.valid_up_to(),
        })?;
        let bad = |bad: Bad| match bad {
            Bad::NotJson(at) => ServeError::NotJson { line, at },
            Bad::Number(at) => ServeError::NumberOutOfRange { line, at },
            Bad::Shape(at) => ServeError::NotARequest { line, at },
        };
        let value = exact::parse(text).map_err(bad)?;
        let answer = answer(target, &value).map_err(bad)?;
        output
            .write_all(answer.as_bytes())
            .and_then(|()| output.write_all(b"\n"))
            .and_then(|()| output.flush())
            .map_err(|e| ServeError::Io(e.kind()))?;
    }
}

#[derive(Serialize)]
struct Answer<'a, T: Serialize> {
    id: u64,
    ok: &'a T,
}

#[derive(Serialize)]
struct Refusal<'a> {
    id: u64,
    target_error: &'a TargetError,
}

/// The answer line to one decoded request: one arm per operation, which is the catalogue.
#[allow(clippy::too_many_lines)]
fn answer<T: ConformanceTarget + ?Sized>(target: &T, value: &Value) -> Result<String, Bad> {
    let envelope = exact::open_object(value)?;
    let id = exact::unsigned(wire::member(envelope, "id", value)?)?;
    let op = exact::text(wire::member(envelope, "op", value)?)?;
    let request = wire::member(envelope, "request", value)?;
    let Some(known) = Op::from_name(&op) else {
        return Ok(line(
            id,
            Err::<(), _>(TargetError::unsupported(
                op,
                "this target does not implement the operation",
            )),
        ));
    };
    let unit = |answer: Result<(), TargetError>| line(id, answer.map(|()| serde_json::Value::Null));
    Ok(match known {
        Op::Handshake => {
            let identity = target.identity();
            match &identity {
                Ok(implementation) => line(
                    id,
                    Ok(wire::HandshakeAnswer {
                        speaks: TARGET_FORMAT,
                        implementation,
                    }),
                ),
                Err(error) => line(id, Err::<(), _>(error.clone())),
            }
        }
        Op::FixtureValues => {
            let (scenario, contract) = wire::fixture_request(request)?;
            let answer = target.fixture_values(&scenario, &contract);
            line(id, answer.as_ref().map(ExactMap).map_err(Clone::clone))
        }
        Op::OpenPeriodic => {
            let answer = target.open_periodic(exact::serde(request)?);
            line(
                id,
                answer
                    .as_ref()
                    .map(wire::OpenedWire::of)
                    .map_err(Clone::clone),
            )
        }
        Op::ObservePeriodic => {
            let answer = target.observe_periodic(exact::serde(request)?);
            line(
                id,
                answer
                    .as_ref()
                    .map(wire::ObservationWire::of)
                    .map_err(Clone::clone),
            )
        }
        Op::ClosePeriodic => line(id, target.close_periodic(exact::serde(request)?)),
        Op::ObserveClockReading => line(id, target.observe_clock_reading(exact::serde(request)?)),
        Op::EstablishEntity => unit(target.establish_entity(wire::entity_request(request)?)),
        Op::BeginScenario => unit(target.begin_scenario(&exact::serde(request)?)),
        Op::ExecuteCommand => command(id, &target.execute_command(wire::command_request(request)?)),
        Op::ExecuteCommandWithoutInput => command(
            id,
            &target.execute_command_without_input(wire::absent_input_request(request)?),
        ),
        Op::QueryView => view(id, &target.query_view(wire::view_request(request)?)),
        Op::QueryViewAs => view(
            id,
            &target.query_view_as(wire::view_request(request)?, &wire::reader(request)?),
        ),
        Op::QueryViewAnonymous => view(
            id,
            &target.query_view_anonymous(wire::view_request(request)?),
        ),
        Op::ObserveEvents => {
            let answer = target.observe_events(exact::serde(request)?);
            line(
                id,
                answer
                    .as_ref()
                    .map(|events| events.iter().map(wire::EventWire::of).collect::<Vec<_>>())
                    .map_err(Clone::clone),
            )
        }
        Op::ConfigureExternalOutcome => {
            unit(target.configure_external_outcome(exact::serde(request)?))
        }
        Op::ConfigureExternalOutcomeRepeatedly => {
            unit(target.configure_external_outcome_repeatedly(
                exact::serde(request)?,
                wire::times(request)?,
            ))
        }
        Op::RedeliverEvent => unit(target.redeliver_event(exact::serde(request)?)),
        Op::DeliverEvent => unit(target.deliver_event(wire::delivery_request(request)?)),
        Op::ObserveInvocations => {
            let answer = target.observe_invocations(exact::serde(request)?);
            line(
                id,
                answer
                    .as_ref()
                    .map(|items| {
                        items
                            .iter()
                            .map(wire::InvocationWire::of)
                            .collect::<Vec<_>>()
                    })
                    .map_err(Clone::clone),
            )
        }
        Op::MarkInstant => unit(target.mark_instant(exact::serde(request)?)),
        Op::HoldNow => unit(target.hold_now(exact::serde(request)?)),
        Op::ObserveElapsed => line(id, target.observe_elapsed(exact::serde(request)?)),
        Op::ScanView => line(id, target.scan_view(wire::scan_request(request)?)),
        Op::ScanViewAs => line(
            id,
            target.scan_view_as(wire::scan_request(request)?, &wire::reader(request)?),
        ),
        Op::ScanViewAnonymous => line(id, target.scan_view_anonymous(wire::scan_request(request)?)),
        Op::AwaitOutcome => {
            let answer = target.await_outcome(wire::await_request(request)?);
            line(
                id,
                answer
                    .as_ref()
                    .map(wire::AwaitAnswerWire::of)
                    .map_err(Clone::clone),
            )
        }
        Op::EndScenario => unit(target.end_scenario(&exact::serde(request)?)),
    })
}

fn command(id: u64, answer: &Result<crate::target::SemanticCommandResult, TargetError>) -> String {
    line(
        id,
        answer
            .as_ref()
            .map(wire::ResultWire::of)
            .map_err(Clone::clone),
    )
}

fn view(id: u64, answer: &Result<crate::target::SemanticViewResult, TargetError>) -> String {
    line(
        id,
        answer
            .as_ref()
            .map(wire::ViewResultWire::of)
            .map_err(Clone::clone),
    )
}

/// One answer line. A result that cannot be written is answered `unavailable`, which is a correct
/// answer rather than a broken line.
fn line<T: Serialize>(id: u64, answer: Result<T, TargetError>) -> String {
    let written = match answer {
        Ok(ok) => serde_json::to_string(&Answer { id, ok: &ok }),
        Err(error) => serde_json::to_string(&Refusal {
            id,
            target_error: &error,
        }),
    };
    written.unwrap_or_else(|_| {
        let error = TargetError::unavailable("serve", "the answer could not be written as JSON");
        serde_json::to_string(&Refusal {
            id,
            target_error: &error,
        })
        .unwrap_or_default()
    })
}
