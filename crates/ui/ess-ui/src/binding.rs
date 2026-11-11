//! The binding contract between an `ess-ui/1` document and the HTTP surface ESS synthesizes for a
//! served component (beyond10x/ess#311).
//!
//! Two halves, both plain data:
//!
//! - [`Binding`]: the route table a renderer reads instead of deriving a path. It is computed from
//!   the model by `ess_ui_check::binding` through `ess_gen::http::routes` — the one derivation the
//!   synthesized servers answer — and covers only the views and commands the document names. With
//!   an `ess-http/1` binding (`ess_ui_check::binding_over`) each route is the one that binding
//!   declares, through `ess_gen::http::bound_routes`, with its method and path variables
//!   (beyond10x/ess#713). It serialises to JSON for a generator that is not Rust.
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
    /// Every view and command name as the document writes it, to its qualified name; and every
    /// model enum a choice's `options` name.
    pub names: BTreeMap<String, String>,
    /// Every model enum a choice's `options` name, by qualified name: its variants in declaration
    /// order (beyond10x/ess#330). Absent from the JSON when the document names none, so a binding
    /// computed before it keeps its bytes.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub enums: BTreeMap<String, Vec<EnumVariant>>,
}

impl Binding {
    /// The route of the view the document writes as `name`, or names by its qualified name.
    pub fn view(&self, name: &str) -> Option<&ViewRoute> {
        let qualified = self.names.get(name).map_or(name, String::as_str);
        self.components
            .values()
            .find_map(|served| served.views.get(qualified))
    }

    /// Whether any route leaves the derived surface's method, carries a path parameter or takes
    /// one input field as its bare body: what an `ess-http/1` binding moves (beyond10x/ess#713,
    /// #748). A renderer that reads only paths needs nothing more when this is `false`.
    pub fn routed(&self) -> bool {
        self.components.values().any(|served| {
            served
                .views
                .values()
                .any(|route| route.method.is_some() || !route.path_params.is_empty())
                || served.commands.values().any(|route| {
                    route.method.is_some() || !route.path_params.is_empty() || route.body.is_some()
                })
        })
    }
}

/// One variant of a model enum, as a choice offers it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnumVariant {
    /// What is sent: the variant's wire spelling.
    pub value: String,
    /// What is shown: the variant's display name, else its name.
    pub label: String,
}

/// What a name a choice's `options` write means in a model (beyond10x/ess#330).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnumLookup {
    /// An enum: its qualified name and its variants in declaration order.
    Enum {
        /// The qualified name.
        name: String,
        /// The variants, in declaration order.
        variants: Vec<EnumVariant>,
    },
    /// A type, entity or view of the model that is not an enum.
    NotEnum {
        /// The qualified name.
        name: String,
    },
    /// Several names of the model, which the written name could each mean.
    Ambiguous(Vec<String>),
    /// Nothing of the model.
    Unknown,
}

/// The enums a document's `options` may name besides its own `types`: the model's, given to
/// [`crate::load_str_with`]. `ess_ui_check::Model` resolves them in a compiled model; a
/// [`Binding`] resolves the ones it carries, so a renderer holding only the binding lists them.
pub trait ModelEnums {
    /// The model's system, as a refusal names it.
    fn system(&self) -> &str;
    /// What `name`, written as a choice's `options`, means in the model.
    fn lookup(&self, name: &str) -> EnumLookup;
    /// [`Self::lookup`] for the `options` at `at`; a caller that defers the model, as `ess ui
    /// test` without `--model` does, records where each name was written.
    fn lookup_at(&self, name: &str, at: &crate::NodePath) -> EnumLookup {
        let _ = at;
        self.lookup(name)
    }
    /// What the commands `document` names do to the fixture rows of the views it names, when the
    /// model can say (`ess ui test --model`, `ess generate ui --model --fixtures`). `None` for a
    /// holder of no compiled model, as a [`Binding`].
    fn fixture_effects(&self, document: &crate::Document) -> Option<FixtureEffects> {
        let _ = document;
        None
    }
}

/// What a confirmed command does to the fixture rows the views read, computed from the compiled
/// model and keyed by the names the document writes. Fixture mode applies it in memory.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct FixtureEffects {
    /// Each command the document names whose success changes a row, or whose change fixture mode
    /// cannot apply.
    pub commands: BTreeMap<String, CommandEffect>,
    /// Each view the document names that lists one row per entity, with what it takes to place a
    /// changed row in it.
    pub views: BTreeMap<String, ViewShape>,
}

/// What one command does to a fixture row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CommandEffect {
    /// Fixture mode cannot apply it: the reason, shown where the user acted.
    NotApplied {
        /// Why the effect is not applied.
        unapplied: String,
    },
    /// The row the input names changes as the success outcome says.
    Applies(RowChange),
}

/// The change a command's one success outcome makes to a row of its entity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RowChange {
    /// The entity, qualified.
    pub entity: String,
    /// The input field naming the instance.
    pub instance: String,
    /// The state a `moves:` ends in.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
    /// Each entity field the outcome sets, by name, and where its value comes from.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub sets: BTreeMap<String, SetValue>,
    /// Whether the row is removed (`deletes:`).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub delete: bool,
}

/// Where a `sets:` value comes from.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SetValue {
    /// The command's input field of this name.
    Input(String),
    /// A value written in the outcome.
    Value(serde_json::Value),
}

/// How a view places a changed row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ViewShape {
    /// The entity it lists, qualified.
    pub entity: String,
    /// The row key carrying the identity.
    pub identity: String,
    /// The row key carrying the lifecycle state.
    pub state: String,
    /// Each entity field to the row key it is carried under, where that is another name.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub wires: BTreeMap<String, String>,
    /// What a row must hold to be listed, by row key; absent for a view that lists every row.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filter: Option<RowFilter>,
    /// Why a changed row cannot be placed in it: its filter is not one fixture mode evaluates.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unapplied: Option<String>,
}

/// A view's `filter:` over row keys, in the forms fixture mode evaluates.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum RowFilter {
    /// Every part holds.
    All {
        /// The parts.
        of: Vec<RowFilter>,
    },
    /// Some part holds.
    Any {
        /// The parts.
        of: Vec<RowFilter>,
    },
    /// The part does not hold.
    Not {
        /// The part.
        of: Box<RowFilter>,
    },
    /// The field equals the value.
    Eq {
        /// The row key.
        field: String,
        /// The value.
        value: serde_json::Value,
    },
    /// The field differs from the value.
    Ne {
        /// The row key.
        field: String,
        /// The value.
        value: serde_json::Value,
    },
    /// The field equals one of the values.
    In {
        /// The row key.
        field: String,
        /// The values.
        values: Vec<serde_json::Value>,
    },
    /// The field equals none of the values.
    NotIn {
        /// The row key.
        field: String,
        /// The values.
        values: Vec<serde_json::Value>,
    },
}

impl ModelEnums for Binding {
    fn system(&self) -> &str {
        &self.system
    }

    fn lookup(&self, name: &str) -> EnumLookup {
        self.names
            .get(name)
            .and_then(|qualified| {
                self.enums.get(qualified).map(|variants| EnumLookup::Enum {
                    name: qualified.clone(),
                    variants: variants.clone(),
                })
            })
            .unwrap_or(EnumLookup::Unknown)
    }
}

/// What one served component answers for the document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ServedComponent {
    /// The views the document reads from it, by qualified name.
    pub views: BTreeMap<String, ViewRoute>,
    /// The commands the document sends it, by qualified name.
    pub commands: BTreeMap<String, CommandRoute>,
}

/// How one view is read: a `GET` on `path`, with its parameters in the query, unless an
/// `ess-http/1` binding names another method or carries a parameter in the path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ViewRoute {
    /// The path, as the served surface answers it.
    pub path: String,
    /// Its declared parameters, in declaration order.
    pub params: Vec<QueryParam>,
    /// The row field, by wire name, carrying the identity of the entity the view projects: the
    /// value a choice over the view sends when it names no field (beyond10x/ess#328). Present only
    /// for a view a choice reads, and absent from the JSON otherwise, so a binding without choices
    /// keeps its bytes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub identity: Option<String>,
    /// The method, where an `ess-http/1` binding serves the view on another than `GET`; absent
    /// from the JSON at `GET`, so a route table computed without a binding keeps its bytes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    /// Each variable of [`Self::path`] (`{name}`) to the parameter, by [`QueryParam::name`], whose
    /// value it carries; a carried parameter leaves the query. Absent from the JSON when empty.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub path_params: BTreeMap<String, String>,
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

/// How one command is sent: a `POST` on `path`, unless an `ess-http/1` binding names another
/// method or carries an input field in the path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandRoute {
    /// The path, as the served surface answers it.
    pub path: String,
    /// The method, where an `ess-http/1` binding serves the command on another than `POST`;
    /// absent from the JSON at `POST`, so a route table computed without a binding keeps its
    /// bytes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    /// Each variable of [`Self::path`] (`{name}`) to the input field, by the name the body
    /// carries it under, whose value it carries; a carried field leaves the body. Absent from the
    /// JSON when empty.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub path_params: BTreeMap<String, String>,
    /// Each variable of [`Self::path_params`] whose field is renamed on the wire, to the field's
    /// model name: a `bind` may send the field under either name, so the variable is filled from
    /// the other when the wire name holds no value, and both leave the body. Absent from the JSON
    /// when empty.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub path_aliases: BTreeMap<String, String>,
    /// The input field whose value is the whole request body, by the name the input carries it
    /// under, where an `ess-http/1` binding takes one bare (`request: {body: input.<field>}`);
    /// the path carries every other input field. Absent from the JSON otherwise.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    /// [`Self::body`]'s field by its model name, where it is renamed on the wire: a `bind` may
    /// send the field under either name, and the body is the other's value when the wire name
    /// holds none. Absent from the JSON otherwise.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body_alias: Option<String>,
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
    /// Accepted in fixture mode, and the command's effect was applied to the fixture rows: the
    /// views read again.
    FixtureApplied,
    /// Accepted in fixture mode, which could not apply the command's effect: the rows are as they
    /// were, and the reason is shown where the user acted.
    FixtureNotApplied {
        /// Why the effect was not applied.
        reason: String,
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
