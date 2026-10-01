//! The binding contract between an `ess-ui/1` document and the HTTP surface ESS synthesizes for a
//! served component (beyond10x/ess#311).
//!
//! Two halves, both plain data:
//!
//! - [`Binding`]: the route table a renderer reads instead of deriving a path. It is computed from
//!   the model by `ess_ui_check::binding` through `ess_gen::http::routes` — the one derivation the
//!   synthesized servers answer — and covers only the views and commands the document names. It
//!   serialises to JSON for a generator that is not Rust.
//! - [`classify`]: the one reading of a command's answer. A served command answers a declared
//!   branch (`outcome`, `published`, and `error`/`payload` when it refused) or a refusal the surface
//!   makes itself (`refused`, plus `actor` on a `403` for a caller no grant admits and `committed`
//!   on a `501`). The two are told apart by the body's members, not by the status alone: a `403`
//!   is both a declared branch decided by the caller and the standard refusal, and a `400` is both
//!   a declared `input_absent:` answer and a body the server could not read.
//!
//! This module depends on nothing of the model: a renderer holding a [`Binding`] needs no compiler.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Every route a document's views and commands bind to, by served component.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Binding {
    /// The ESS system the routes were computed from.
    pub system: String,
    /// Each `reached_by: network` component that serves something the document names, by name.
    pub components: BTreeMap<String, ServedComponent>,
    /// Every view and command name as the document writes it, to its qualified name.
    pub names: BTreeMap<String, String>,
}

/// What one served component answers for the document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ServedComponent {
    /// The views the document reads from it, by qualified name.
    pub views: BTreeMap<String, ViewRoute>,
    /// The commands the document sends it, by qualified name.
    pub commands: BTreeMap<String, CommandRoute>,
}

/// How one view is read: a `GET` on `path`, with its parameters in the query.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ViewRoute {
    /// The path, as the served surface answers it.
    pub path: String,
    /// Its declared parameters, in declaration order.
    pub params: Vec<QueryParam>,
}

/// One view parameter, carried as a query parameter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryParam {
    /// The name a document binds it by (`reads.params`).
    pub name: String,
    /// The query key a request carries.
    pub wire: String,
    /// `false` for an `Optional` parameter and a paging parameter.
    pub required: bool,
    /// The primitive the value is written as (`String`, `Integer`, `Uuid`, …): the primitive at
    /// the root of its type, through any newtype; `String` for an enum.
    pub scalar: String,
}

/// How one command is sent: a `POST` on `path`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandRoute {
    /// The path, as the served surface answers it.
    pub path: String,
    /// Whether a request must carry a body (`ess_gen::http::body_required`).
    pub body_required: bool,
    /// Every declared error the command can answer, by its wire code (the `error` member of a
    /// [`Answer::Refused`]).
    pub errors: BTreeMap<String, ErrorRoute>,
}

/// One declared error a command can answer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorRoute {
    /// The status its branch answers with (`ess_gen::http::status`); the first branch in
    /// declaration order when two branches report one error.
    pub status: u16,
    /// What it is shown as.
    pub display: String,
}

/// What a command's answer means to the user who sent it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "answer", rename_all = "snake_case", deny_unknown_fields)]
pub enum Answer {
    /// A declared branch that refused nothing was taken.
    Accepted,
    /// A declared branch refused: its error's wire code and, where the error has fields, their
    /// values. Shown where the user acted.
    Refused {
        /// The error's wire code.
        error: String,
        /// The error's fields, when it declares any.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        payload: Option<serde_json::Value>,
    },
    /// The standard refusal for a caller no grant admits: the actor the request was
    /// authenticated as, or `None` when it was authenticated as nobody.
    NotGranted {
        /// The actor's qualified name.
        actor: Option<String>,
    },
    /// A refusal the surface made about the request rather than the command: a body it could not
    /// read, a path it does not declare, a method the path does not answer.
    Malformed {
        /// Why, as the surface wrote it.
        refused: String,
    },
    /// The realization is unfinished (`501`). A client must not retry it when `committed`: the
    /// effect and its events stand, and a retry would perform the command twice.
    Unfinished {
        /// Whether the command's effect was committed before delivering what it published failed.
        committed: bool,
    },
    /// Not an answer the served surface gives: a body that is not a JSON object, or one whose
    /// members match none of the shapes above (a proxy's error page, a gateway timeout).
    Transport,
}

/// Classifies one answer to a command by its body's members, with the status deciding only
/// between the shapes that share members.
///
/// - `outcome` without `error`, under a `2xx`: [`Answer::Accepted`].
/// - `outcome` with a string `error`, under anything but a `2xx`: [`Answer::Refused`].
/// - `refused` and `actor` (a string or `null`) under `403`: [`Answer::NotGranted`].
/// - `refused` and a boolean `committed` under `501`: [`Answer::Unfinished`].
/// - any other string `refused`: [`Answer::Malformed`].
/// - anything else: [`Answer::Transport`].
pub fn classify(status: u16, body: &str) -> Answer {
    let Ok(serde_json::Value::Object(members)) = serde_json::from_str::<serde_json::Value>(body)
    else {
        return Answer::Transport;
    };
    let success = (200..300).contains(&status);
    if let Some(outcome) = members.get("outcome") {
        if !outcome.is_string() {
            return Answer::Transport;
        }
        return match members.get("error") {
            None if success => Answer::Accepted,
            Some(serde_json::Value::String(error)) if !success => Answer::Refused {
                error: error.clone(),
                payload: members
                    .get("payload")
                    .filter(|payload| !payload.is_null())
                    .cloned(),
            },
            _ => Answer::Transport,
        };
    }
    let Some(serde_json::Value::String(refused)) = members.get("refused") else {
        return Answer::Transport;
    };
    match (status, members.get("actor"), members.get("committed")) {
        (403, Some(serde_json::Value::String(actor)), _) => Answer::NotGranted {
            actor: Some(actor.clone()),
        },
        (403, Some(serde_json::Value::Null), _) => Answer::NotGranted { actor: None },
        (501, _, Some(serde_json::Value::Bool(committed))) => Answer::Unfinished {
            committed: *committed,
        },
        _ => Answer::Malformed {
            refused: refused.clone(),
        },
    }
}
