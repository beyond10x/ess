//! The document-specific files: routes, the app, shells, pages, channels and model types.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{Debug, Write as _};

use ess_ui::{
    Action, ActionConfirm, Body, Carries, ChartKind, Columns, Composite, Direction, Document,
    Field, Header, Live, NavPages, Node, NodeCommon, NodePath, Overlay, Page, PageLayout,
    Primitive, Reads, Region, RegionKind, Section, SectionStates, Selection, Shell, State,
    StateClass, Store, Tab, TabFields, TabForm,
};
use serde_yaml::Value;

use crate::fixtures::millis;
use crate::jsx::{fragment, indent, El};
use crate::ts::{self, Writer};
use crate::types::{default_literal, Types};
use crate::GenerateError;

/// The snake-case name of a unit enum variant; `None` for a retrofit's unmapped marker, which a
/// renderer treats as absent.
pub(crate) fn variant<T: Debug>(value: &T) -> Option<String> {
    let debug = format!("{value:?}");
    if debug.starts_with("Unmapped") {
        return None;
    }
    let mut out = String::new();
    for (index, character) in debug.chars().enumerate() {
        if character.is_ascii_uppercase() {
            if index > 0 {
                out.push('_');
            }
            out.push(character.to_ascii_lowercase());
        } else {
            out.push(character);
        }
    }
    Some(out)
}

fn quoted(value: Option<&String>) -> Option<String> {
    value.map(|value| ts::string(value))
}

fn expr(value: Option<&ess_ui::Expr>) -> Option<String> {
    value.map(|value| ts::string(&value.0))
}

fn strings<'a>(values: impl IntoIterator<Item = &'a String>) -> String {
    ts::array(values.into_iter().map(|value| ts::string(value)))
}

fn exprs<'a>(values: impl IntoIterator<Item = (&'a String, &'a ess_ui::Expr)>) -> String {
    ts::object(
        values
            .into_iter()
            .map(|(name, value)| (name.as_str(), Some(ts::string(&value.0)))),
    )
}

fn duration(value: Option<&String>) -> Option<String> {
    value.and_then(|text| millis(text)).map(|ms| ms.to_string())
}

/// `(name, expression)` pairs: a scope layer's values or setters.
type Entries = Vec<(String, String)>;

/// Where a composite gets its rows.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Rows {
    /// The enclosing section read them.
    FromSection,
    /// The composite reads them itself.
    Own,
}

/// Rendering options passed down to a body.
#[derive(Clone)]
struct Opts {
    rows: Rows,
    binds: Option<String>,
    local_draft: bool,
    selection_state: Option<String>,
    degrades: BTreeMap<String, String>,
    /// The enclosing section or overlay frame already carries this path.
    framed: bool,
    /// The field keying rows: the section's `reads.key`, else its `live.match` (#320).
    row_key: Option<String>,
    /// The row key a choice takes each option's value from (the form field it picks for).
    choice_value: Option<String>,
    /// The local holding the rows a live nested node reads for its composite (#354).
    data: Option<&'static str>,
}

impl Opts {
    fn own() -> Self {
        Self {
            rows: Rows::Own,
            binds: None,
            local_draft: true,
            selection_state: None,
            degrades: BTreeMap::new(),
            framed: false,
            row_key: None,
            choice_value: None,
            data: None,
        }
    }
}

/// Generates the document-specific files, recording which runtime modules they import.
pub(crate) struct Gen<'d> {
    pub(crate) doc: &'d Document,
    pub(crate) types: Types<'d>,
    /// Runtime modules imported anywhere (`runtime/core`, `runtime/composites/form`, …).
    pub(crate) used: BTreeSet<String>,
    imports: BTreeMap<String, BTreeSet<String>>,
    hoisted: Vec<String>,
    prefix: String,
    /// Refusals found while generating; the first fails `render`.
    pub(crate) errors: Vec<GenerateError>,
    /// `PlacementProfile.profiles.<profile>.defaults`, read from the embedded schema.
    profiles: BTreeMap<String, BTreeMap<String, Store>>,
    page_profile: Option<String>,
    /// The page being written, whose params a nested live node keys its channel session by.
    page: Option<String>,
    section_profile: Option<String>,
    /// The locals of each generated component being written, innermost last.
    locals: Vec<BTreeSet<String>>,
    /// Component names of the file being written.
    components: BTreeSet<String>,
    /// The project is bound to a served surface, which streams nothing: a `live:` section polls.
    bound: bool,
    /// The route table a bound project reads: a choice over a view takes the view's identity
    /// from it (beyond10x/ess#328).
    binding: Option<ess_ui::binding::Binding>,
}

/// How often a bound `live:` section reads again when its read declares no `refresh:` duration.
const POLL_MS: u64 = 5000;

/// The shortest `refresh:` a bound `live:` section may poll at; shorter is refused.
const MIN_POLL_MS: u64 = 1000;

/// The longest `refresh:` a bound `live:` section may poll at (24 h), well inside the 2^31 - 1 ms
/// a browser timer holds; longer is refused.
const MAX_POLL_MS: u64 = 24 * 3_600_000;

/// `constructs.PlacementProfile.profiles.<profile>.defaults` of the schema this crate was built
/// with: the store each profile gives each state class.
fn schema_profiles() -> BTreeMap<String, BTreeMap<String, Store>> {
    let schema: Value = serde_yaml::from_str(ess_ui::SCHEMA).unwrap_or(Value::Null);
    let mut out = BTreeMap::new();
    if let Some(profiles) = schema["constructs"]["PlacementProfile"]["profiles"].as_mapping() {
        for (name, profile) in profiles {
            let defaults = profile
                .get("defaults")
                .cloned()
                .and_then(|defaults| {
                    serde_yaml::from_value::<BTreeMap<String, Store>>(defaults).ok()
                })
                .unwrap_or_default();
            out.insert(ts::key_text(name), defaults);
        }
    }
    out
}

/// The end of `src/routes.ts`: `matchPage` over the route table written before it, and
/// `pageAt`.
const MATCH_PAGE: &str = r#"/** A pathname segment decoded; a malformed % sequence stays as written. */
function decodeSegment(segment: string): string {
  try {
    return decodeURIComponent(segment);
  } catch {
    return segment;
  }
}

/** The page a pathname reaches; case-insensitive, a trailing slash optional. */
export function matchPage(pathname: string): PageMatch | undefined {
  const trimmed = pathname.length > 1 && pathname.endsWith("/") ? pathname.slice(0, -1) : pathname;
  const segments = trimmed.split("/").slice(1);
  for (const [pattern, page] of table) {
    const parts = pattern.split("/").slice(1);
    if (parts.length !== segments.length) {
      continue;
    }
    const params: Record<string, string> = {};
    let matches = true;
    for (let index = 0; index < parts.length && matches; index += 1) {
      const part = parts[index];
      const segment = segments[index];
      if (!part.startsWith(":")) {
        matches = part.toLowerCase() === decodeSegment(segment).toLowerCase();
      } else if (segment === "") {
        matches = false;
      } else {
        try {
          params[part.slice(1)] = decodeURIComponent(segment);
        } catch {
          matches = false;
        }
      }
    }
    if (matches) {
      return { page, params };
    }
  }
  return undefined;
}

/** The page whose route matches a pathname. */
export function pageAt(pathname: string): string | undefined {
  return matchPage(pathname)?.page;
}
"#;

/// The path pattern of a page: its name's segments (an alias written as a path loses its
/// leading and trailing `/`), then one `:param` per param.
pub fn route_pattern(name: &str, page: &Page) -> String {
    let mut path = format!("/{}", name.trim_matches('/').replace('.', "/"));
    for param in page.params.keys() {
        path.push_str("/:");
        path.push_str(param);
    }
    path
}

/// The label a page shows in navigation.
fn nav_label(name: &str, page: Option<&Page>) -> String {
    page.and_then(|page| {
        page.nav
            .as_ref()
            .and_then(|nav| nav.label.clone())
            .or_else(|| page.title.clone())
    })
    .unwrap_or_else(|| name.to_owned())
}

impl<'d> Gen<'d> {
    pub(crate) fn new(doc: &'d Document, binding: Option<&ess_ui::binding::Binding>) -> Self {
        Self {
            doc,
            types: Types::new(&doc.types),
            used: BTreeSet::new(),
            imports: BTreeMap::new(),
            hoisted: Vec::new(),
            prefix: String::new(),
            errors: Vec::new(),
            profiles: schema_profiles(),
            page_profile: None,
            page: None,
            section_profile: None,
            locals: Vec::new(),
            components: BTreeSet::new(),
            bound: binding.is_some(),
            binding: binding.cloned(),
        }
    }

    /// Imports `symbol` from `module` (a path under `src/`) into the file being generated.
    fn import(&mut self, module: &str, symbol: &str) -> String {
        if module.starts_with("runtime/") {
            self.used.insert(module.to_owned());
        }
        self.imports
            .entry(module.to_owned())
            .or_default()
            .insert(symbol.to_owned());
        symbol.split(' ').next_back().unwrap_or(symbol).to_owned()
    }

    fn take_imports(&mut self, depth: usize) -> String {
        let up = if depth == 0 {
            "./".to_owned()
        } else {
            "../".repeat(depth)
        };
        let mut out = String::new();
        for (module, symbols) in std::mem::take(&mut self.imports) {
            let specifier = if module.starts_with("runtime/")
                || module == "routes"
                || module == "fixtures"
                || module == "channels"
                || module == "model"
                || module.starts_with("pages/")
                || module.starts_with("shells/")
            {
                format!("{up}{module}")
            } else {
                module.clone()
            };
            if symbols.len() == 1 && symbols.iter().any(|s| s.starts_with("* as")) {
                let symbol = symbols.iter().next().cloned().unwrap_or_default();
                let _ = writeln!(out, "import type {symbol} from \"{specifier}\";");
                continue;
            }
            let list: Vec<&str> = symbols.iter().map(String::as_str).collect();
            let _ = writeln!(
                out,
                "import {{ {} }} from \"{specifier}\";",
                list.join(", ")
            );
        }
        out
    }

    fn model_types(&mut self) {
        self.import("model", "* as M");
    }

    // ── state ────────────────────────────────────────────────────────────────────────────

    /// The profile table of the embedded schema (`PlacementProfile.profiles.<p>.defaults`).
    fn profile_default(&self, profile: Option<&str>, class: &StateClass) -> Option<Store> {
        let class = variant(class)?;
        self.profiles.get(profile?)?.get(&class).cloned()
    }

    /// `PlacementProfile.resolution.order`: explicit store; pinned (profiles ignored); section
    /// profile; page profile; document defaults; the document profile's defaults. The first step
    /// that yields a store wins; nothing yielding one is a refusal.
    fn resolve(&self, state: &State) -> Option<Store> {
        if let Some(store) = state
            .store
            .as_ref()
            .filter(|s| !matches!(s, Store::Unmapped(_)))
        {
            return Some(store.clone());
        }
        let known = |store: &&Store| !matches!(store, Store::Unmapped(_));
        let profiles = if state.pinned {
            None
        } else {
            self.profile_default(self.section_profile.as_deref(), &state.class)
                .or_else(|| self.profile_default(self.page_profile.as_deref(), &state.class))
        };
        profiles
            .or_else(|| {
                self.doc
                    .placement_defaults
                    .get(&state.class)
                    .filter(known)
                    .cloned()
            })
            .or_else(|| {
                let profile = variant(&self.doc.placement_profile);
                self.profile_default(profile.as_deref(), &state.class)
            })
    }

    /// `PlacementProfile.resolution.refusals` for one store of a state.
    fn refusal(state: &State, store: &Store) -> Option<String> {
        let name = variant(store).unwrap_or_default();
        let browser = matches!(
            store,
            Store::Url | Store::SessionStorage | Store::LocalStorage
        );
        if state.class == StateClass::Credential
            && !matches!(store, Store::Memory | Store::ServerSession)
        {
            return Some(format!(
                "a credential is held only in memory or server_session, not {name}"
            ));
        }
        if state.sensitive && browser {
            return Some(format!("sensitive state cannot be placed in {name}"));
        }
        None
    }

    /// The store of a state, or a refusal recorded against its path (generation then fails;
    /// a memory hook stands in so the rest of the file still renders).
    fn store(&mut self, path: &NodePath, state: &State) -> Store {
        let Some(store) = self.resolve(state) else {
            self.refuse(
                path,
                &format!(
                    "no store resolves for class {} (PlacementProfile.resolution.order)",
                    variant(&state.class).unwrap_or_else(|| "unmapped".to_owned())
                ),
            );
            return Store::Memory;
        };
        if let Some(reason) = Self::refusal(state, &store) {
            self.refuse(
                path,
                &format!("{reason} (PlacementProfile.resolution.refusals)"),
            );
            return Store::Memory;
        }
        if let Some(fallback) = &state.fallback {
            if let Some(reason) = Self::refusal(state, &fallback.store) {
                self.refuse(
                    path,
                    &format!("fallback: {reason} (PlacementProfile.resolution.refusals)"),
                );
                return Store::Memory;
            }
        }
        store
    }

    fn refuse(&mut self, path: &NodePath, message: &str) {
        self.errors
            .push(GenerateError::new(format!("{path}: {message}")));
    }

    /// The query key of a url state: its canonical path relative to the page (the route names
    /// the page), `/`-joined; a page's own state is its bare name. Names never hold `/`, so two
    /// distinct paths never share a key.
    fn url_key(path: &NodePath) -> String {
        let segments = path.segments();
        let relative: &[String] =
            if segments.first().is_some_and(|s| s == "pages") && segments.len() > 2 {
                &segments[2..]
            } else {
                segments
            };
        if relative.len() == 2 && relative[0] == "state" {
            relative[1].clone()
        } else {
            relative.join("/")
        }
    }

    /// A component name unique in the file being written.
    fn component_name(&mut self, wanted: &str) -> String {
        let mut suffix = 1;
        loop {
            let name = if suffix == 1 {
                wanted.to_owned()
            } else {
                format!("{wanted}{suffix}")
            };
            if self.components.insert(name.clone()) {
                return name;
            }
            suffix += 1;
        }
    }

    /// Starts a generated component: its locals are checked for collisions until `end_component`.
    fn begin_component(&mut self) {
        self.locals.push(BTreeSet::new());
    }

    fn end_component(&mut self) {
        self.locals.pop();
    }

    /// A `[value, setter]` pair of locals named after `wanted`, suffixed until neither collides
    /// with a local of the current component.
    fn local_pair(&mut self, wanted: &str) -> (String, String) {
        let taken = self.locals.last_mut().expect("inside a component");
        let mut suffix = 1;
        loop {
            let local = if suffix == 1 {
                wanted.to_owned()
            } else {
                format!("{wanted}{suffix}")
            };
            let setter = format!("set{}{}", local[..1].to_ascii_uppercase(), &local[1..]);
            if !taken.contains(&local) && !taken.contains(&setter) {
                taken.insert(local.clone());
                taken.insert(setter.clone());
                return (local, setter);
            }
            suffix += 1;
        }
    }

    /// One placement hook per state; returns the scope values and setters it contributes.
    fn state_hooks(
        &mut self,
        lines: &mut Vec<String>,
        at: &NodePath,
        states: &BTreeMap<String, State>,
        root: &str,
    ) -> (Entries, Entries) {
        let mut values = Vec::new();
        let mut setters = Vec::new();
        for (name, state) in states {
            let path = at.child("state").child(name);
            let (local, setter) = self.local_pair(&ts::camel(name));
            let ty = self.types.lower(&state.ty);
            self.model_types();
            let initial = state
                .default
                .as_ref()
                .map_or_else(|| self.types.initial(&state.ty), default_literal);
            let call = self.placement_call(&path, state, &ty, &initial);
            lines.push(format!("const [{local}, {setter}] = {call};"));
            values.push((name.clone(), local));
            setters.push((format!("{root}.{name}"), setter));
        }
        (values, setters)
    }

    fn placement_call(
        &mut self,
        path: &NodePath,
        state: &State,
        ty: &str,
        initial: &str,
    ) -> String {
        let key = ts::string(&format!("{}:{path}", self.doc.app));
        match self.store(path, state) {
            Store::Url => {
                let hook = self.import("runtime/state/url", "useUrlState");
                format!(
                    "{hook}<{ty}>({}, {initial})",
                    ts::string(&Self::url_key(path))
                )
            }
            Store::SessionStorage => {
                let hook = self.import("runtime/state/session_storage", "useSessionStorageState");
                format!("{hook}<{ty}>({key}, {initial})")
            }
            Store::LocalStorage => {
                let hook = self.import("runtime/state/local_storage", "useLocalStorageState");
                format!("{hook}<{ty}>({key}, {initial})")
            }
            Store::ServerSession => {
                let hook = self.import("runtime/state/server_session", "useServerSessionState");
                format!("{hook}<{ty}>({key}, {initial})")
            }
            Store::Server => {
                let hook = self.import("runtime/state/server", "useServerState");
                let fallback = state.fallback.as_ref().and_then(|fallback| {
                    variant(&fallback.store).map(|store| {
                        ts::object([
                            ("when", Some(ts::string(&fallback.when.0))),
                            ("store", Some(ts::string(&store))),
                        ])
                    })
                });
                let options = ts::object([
                    ("scope", quoted(state.scope.as_ref())),
                    ("fallback", fallback),
                ]);
                format!("{hook}<{ty}>({key}, {initial}, {options})")
            }
            Store::Memory | Store::Unmapped(_) => {
                let hook = self.import("runtime/state/memory", "useMemoryState");
                format!("{hook}<{ty}>({key}, {initial})")
            }
        }
    }

    /// The form's draft, typed and placed as its `draft` state says (memory when it has none).
    /// Claims `draftValue` / `setDraftValue` before any state of the component.
    fn draft_hook(
        &mut self,
        lines: &mut Vec<String>,
        at: &NodePath,
        draft: Option<&State>,
    ) -> (String, String) {
        let (local, setter) = self.local_pair("draftValue");
        let path = at.child("draft");
        let call = if let Some(state) = draft {
            let ty = self.types.lower(&state.ty);
            self.model_types();
            let initial = state.default.as_ref().map_or_else(
                || {
                    let initial = self.types.initial(&state.ty);
                    if initial == "null" {
                        "{}".to_owned()
                    } else {
                        initial
                    }
                },
                default_literal,
            );
            self.placement_call(&path, state, &ty, &initial)
        } else {
            let hook = self.import("runtime/state/memory", "useMemoryState");
            format!(
                "{hook}<Record<string, unknown>>({}, {{}})",
                ts::string(&format!("{}:{path}", self.doc.app))
            )
        };
        lines.push(format!("const [{local}, {setter}] = {call};"));
        (local, setter)
    }

    /// Wraps `inner` in a `ScopeLayer` holding the given state, draft and extra values.
    fn scope_layer(
        &mut self,
        root: &str,
        values: &[(String, String)],
        setters: &[(String, String)],
        extra: &[(String, String)],
        inner: String,
    ) -> String {
        if values.is_empty() && setters.is_empty() && extra.is_empty() {
            return inner;
        }
        let layer = self.import("runtime/core", "ScopeLayer");
        let mut entries: Vec<(String, Option<String>)> = Vec::new();
        if !values.is_empty() {
            let object = ts::object(
                values
                    .iter()
                    .map(|(name, local)| (name.as_str(), Some(local.clone()))),
            );
            entries.push((root.to_owned(), Some(object)));
        }
        for (name, value) in extra {
            entries.push((name.clone(), Some(value.clone())));
        }
        let setters_object = ts::object(
            setters
                .iter()
                .map(|(path, setter)| (path.as_str(), Some(setter.clone()))),
        );
        El::new(&layer)
            .expr(
                "values",
                ts::object(entries.iter().map(|(k, v)| (k.as_str(), v.clone()))),
            )
            .opt("setters", (!setters.is_empty()).then_some(setters_object))
            .child(inner)
            .render()
    }

    // ── specs ────────────────────────────────────────────────────────────────────────────

    fn reads(reads: &Reads) -> Option<String> {
        let view = reads.view.as_ref().or(reads.placeholder.as_ref())?;
        Some(ts::object([
            ("view", Some(ts::string(view))),
            ("filter", expr(reads.filter.as_ref())),
            (
                "params",
                (!reads.params.is_empty()).then(|| exprs(&reads.params)),
            ),
            (
                "paging",
                reads
                    .paging
                    .as_ref()
                    .and_then(variant)
                    .map(|p| ts::string(&p)),
            ),
            ("debounce", duration(reads.debounce.as_ref())),
            ("refresh", expr(reads.refresh.as_ref())),
        ]))
    }

    fn action(&mut self, at: &NodePath, action: &Action) -> String {
        let navigate = action.navigate.as_ref().map(|navigate| {
            ts::object([
                ("to", Some(ts::string(&navigate.to))),
                (
                    "params",
                    (!navigate.params.is_empty()).then(|| exprs(&navigate.params)),
                ),
            ])
        });
        let export = action.export.as_ref().map(|export| {
            ts::object([
                ("reads", Some(ts::string(&export.reads))),
                ("as", Some(ts::string(&export.export_as))),
                ("params", expr(export.params.as_ref())),
            ])
        });
        let upload = action.upload.as_ref().map(|upload| {
            ts::object([
                ("accept", Some(strings(&upload.accept))),
                ("does", Some(ts::string(&upload.does))),
            ])
        });
        let choice = action.choice.as_deref().map(|node| {
            let mut opts = Opts::own();
            opts.binds = None;
            self.node(&at.child("choice"), node, &opts)
        });
        let (confirm, confirm_show, confirm_opens) = match &action.confirm {
            Some(ActionConfirm::Opens(name)) => (None, None, Some(ts::string(name))),
            Some(ActionConfirm::Inline(inline)) => (
                Some(self.overlay_inline(&at.child("confirm").child("overlay"), &inline.overlay)),
                expr(inline.show.as_ref()),
                None,
            ),
            None => (None, None, None),
        };
        let sets = (!action.sets.is_empty()).then(|| exprs(&action.sets));
        ts::object([
            ("data-ui-path", Some(ts::string(&at.to_string()))),
            ("name", Some(ts::string(&action.name))),
            ("label", quoted(action.label.as_ref())),
            (
                "as",
                action
                    .action_as
                    .as_ref()
                    .and_then(variant)
                    .map(|v| ts::string(&v)),
            ),
            ("does", quoted(action.does.as_ref())),
            (
                "bind",
                (!action.bind.is_empty()).then(|| exprs(&action.bind)),
            ),
            ("opens", quoted(action.opens.as_ref())),
            ("navigate", navigate),
            ("export", export),
            ("upload", upload),
            ("copy", expr(action.copy.as_ref())),
            ("sets", sets),
            ("choice", choice),
            ("confirm", confirm),
            ("confirmShow", confirm_show),
            ("confirmOpens", confirm_opens),
            ("optimistic", action.optimistic.then(|| "true".to_owned())),
            ("bulk", action.bulk.then(|| "true".to_owned())),
            ("visible", expr(action.visible.as_ref())),
        ])
    }

    fn actions(&mut self, at: &NodePath, key: &str, actions: &[Action]) -> Option<String> {
        if actions.is_empty() {
            return None;
        }
        self.import("runtime/actions", "type ActionSpec");
        let items: Vec<String> = actions
            .iter()
            .map(|action| self.action(&at.child(key).child(&action.name), action))
            .collect();
        Some(ts::array(items))
    }

    fn action_control(&mut self, at: &NodePath, action: &Action) -> String {
        let control = self.import("runtime/actions", "ActionControl");
        let spec = self.action(at, action);
        El::new(&control).expr("action", spec).render()
    }

    fn field(&mut self, at: &NodePath, field: &Field, in_form: bool) -> String {
        let choice = field.choice.as_deref().map(|node| {
            let mut opts = Opts::own();
            opts.binds = Some(
                field
                    .binds
                    .as_ref()
                    .map_or_else(|| format!("draft.{}", field.field), |binds| binds.0.clone()),
            );
            if !in_form && field.binds.is_none() {
                opts.binds = None;
            }
            opts.choice_value = Some(field.field.clone());
            self.node(&at.child("choice"), node, &opts)
        });
        let label_from = field.label_from.as_ref().map(|label_from| {
            ts::object([
                ("view", Some(ts::string(&label_from.view))),
                ("field", Some(ts::string(&label_from.field))),
                ("key", quoted(label_from.key.as_ref())),
            ])
        });
        ts::object([
            ("data-ui-path", Some(ts::string(&at.to_string()))),
            ("field", Some(ts::string(&field.field))),
            ("name", Some(ts::string(&field.name))),
            ("label", quoted(field.label.as_ref())),
            ("as", quoted(field.field_as.as_ref())),
            ("sortable", field.sortable.then(|| "true".to_owned())),
            ("visible", expr(field.visible.as_ref())),
            ("binds", expr(field.binds.as_ref())),
            ("labelFrom", label_from),
            ("note", quoted(field.note.as_ref())),
            ("choice", choice),
        ])
    }

    fn fields(
        &mut self,
        at: &NodePath,
        key: &str,
        fields: &[Field],
        in_form: bool,
    ) -> Option<String> {
        if fields.is_empty() {
            return None;
        }
        self.import("runtime/fields", "type FieldSpec");
        let items: Vec<String> = fields
            .iter()
            .map(|field| self.field(&at.child(key).child(&field.name), field, in_form))
            .collect();
        Some(ts::array(items))
    }

    fn tabs(&mut self, at: &NodePath, tabs: &[Tab], in_form: bool) -> Option<String> {
        if tabs.is_empty() {
            return None;
        }
        self.import("runtime/fields", "type TabSpec");
        let mut items = Vec::new();
        for tab in tabs {
            let here = at.child("tabs").child(&tab.name);
            let fields = match &tab.fields {
                Some(TabFields::Fields(fields)) => self.fields(&here, "fields", fields, in_form),
                _ => None,
            };
            let content = match &tab.form {
                Some(TabForm::Node(node)) => {
                    Some(self.node(&here.child("form"), node, &Opts::own()))
                }
                Some(TabForm::Action(action)) => {
                    Some(self.action_control(&here.child("form"), action))
                }
                None => None,
            };
            items.push(ts::object([
                ("data-ui-path", Some(ts::string(&here.to_string()))),
                ("name", Some(ts::string(&tab.name))),
                ("label", quoted(tab.label.as_ref())),
                ("visible", expr(tab.visible.as_ref())),
                ("fields", fields),
                ("content", content),
            ]));
        }
        Some(ts::array(items))
    }

    fn node_list(&mut self, at: &NodePath, key: &str, nodes: &[Node]) -> Vec<String> {
        nodes
            .iter()
            .map(|node| {
                let name = node.common.name.as_deref().unwrap_or("");
                self.node(&at.child(key).child(name), node, &Opts::own())
            })
            .collect()
    }

    fn node_fragment(&mut self, at: &NodePath, key: &str, nodes: &[Node]) -> Option<String> {
        if nodes.is_empty() {
            return None;
        }
        let items = self.node_list(at, key, nodes);
        Some(fragment(items))
    }

    // ── nodes ────────────────────────────────────────────────────────────────────────────

    /// A nested node: its body, behind `Visible` when it has a condition, in its own
    /// component when it holds state.
    fn node(&mut self, at: &NodePath, node: &Node, opts: &Opts) -> String {
        let mut opts = opts.clone();
        opts.framed = false;
        opts.row_key = None;
        opts.data = None;
        opts.degrades.clone_from(&node.common.degrades);
        let rendered = match &node.live {
            Some(live) => self.live_node(at, node, live, &opts),
            None => self.body(at, &node.body, &opts),
        };
        let rendered = if node.common.state.is_empty() {
            rendered
        } else {
            let name = self.component_name(&format!(
                "{}Node{}",
                self.prefix,
                ts::pascal(&at.to_string())
            ));
            let mut lines = Vec::new();
            self.begin_component();
            let (values, setters) = self.state_hooks(&mut lines, at, &node.common.state, "state");
            self.end_component();
            let inner = self.scope_layer("state", &values, &setters, &[], rendered);
            self.hoist(&name, &lines, &inner);
            format!("<{name} />")
        };
        self.visible(&node.common, rendered)
    }

    /// A nested node with `live` (beyond10x/ess#354): its own component reads the node's view
    /// and applies the channel's events to it — polls it in a bound project — while it is
    /// mounted, and hands the rows to the composite as a section frame does. A node not shown (an
    /// inactive tab, a collapsed `expand`, a hidden node) is not mounted: it holds no channel and
    /// reads again when it is shown.
    fn live_node(&mut self, at: &NodePath, node: &Node, live: &Live, opts: &Opts) -> String {
        let reads = node.body.live_reads();
        let Some(spec) = reads.and_then(Self::reads) else {
            // The loader admits `live` only beside `reads`; a read naming no view reads nothing.
            return self.body(at, &node.body, opts);
        };
        let name = self.component_name(&format!(
            "{}Live{}",
            self.prefix,
            ts::pascal(&at.to_string())
        ));
        let mut lines = Vec::new();
        self.begin_component();
        let use_scope = self.import("runtime/core", "useScope");
        let use_read = self.import("runtime/data", "useRead");
        lines.push(format!("const __scope = {use_scope}();"));
        lines.push(format!(
            "const __read = {use_read}({spec}, __scope.values, true, true);"
        ));
        let session = if self.bound {
            None
        } else {
            self.node_session(at, reads, &live.channel)
        };
        let data = self.live_rows(
            &mut lines,
            at,
            live,
            reads,
            &node.common.degrades,
            "node",
            session,
        );
        let data = self.filtered_data(&mut lines, data, reads);
        self.end_component();
        let mut inner = opts.clone();
        inner.rows = Rows::FromSection;
        inner.row_key.clone_from(&live.match_field);
        inner.data = Some(data);
        let body = self.body(at, &node.body, &inner);
        let scope = self.import("runtime/core", "DataScope");
        let rendered = El::new(&scope).expr("data", data).child(body).render();
        self.hoist(&name, &lines, &rendered);
        format!("<{name} />")
    }

    /// The session value a nested live node keys a `session: {per}` channel by: its read's param
    /// of that name, else the page's param; refused at `at` when there is neither.
    fn node_session(
        &mut self,
        at: &NodePath,
        reads: Option<&Reads>,
        channel: &str,
    ) -> Option<String> {
        let per = self
            .doc
            .channels
            .get(channel)?
            .session
            .as_ref()?
            .per
            .clone();
        if let Some(value) = reads.and_then(|reads| reads.params.get(&per)) {
            return Some(value.0.clone());
        }
        let page = self.page.as_ref().and_then(|name| self.doc.pages.get(name));
        if page.is_some_and(|page| page.params.contains_key(&per)) {
            return Some(format!("params.{per}"));
        }
        self.refuse(
            &at.child("live"),
            &format!(
                "channel `{channel}` has one instance per `{per}` (session.per), and nothing here \
                 gives a `{per}`: no param of this node's read and no page param of that name"
            ),
        );
        Some(format!("params.{per}"))
    }

    fn visible(&mut self, common: &NodeCommon, rendered: String) -> String {
        match &common.visible {
            Some(condition) => {
                let visible = self.import("runtime/core", "Visible");
                El::new(&visible)
                    .expr("when", ts::string(&condition.0))
                    .child(rendered)
                    .render()
            }
            None => rendered,
        }
    }

    fn hoist(&mut self, name: &str, lines: &[String], rendered: &str) {
        self.import("react", "type ReactNode");
        let mut w = Writer::default();
        w.open(format!("function {name}(): ReactNode {{"));
        for line in lines {
            w.line(line);
        }
        w.line("return (");
        w.line(format!("  {}", indent(rendered, 4)));
        w.line(");");
        w.close("}");
        self.hoisted.push(w.finish());
    }

    fn body(&mut self, at: &NodePath, body: &Body, opts: &Opts) -> String {
        match body {
            Body::Composite(composite) => self.composite(at, composite, opts),
            Body::Widget(instance) => {
                let widget_box = self.import("runtime/widget", "WidgetBox");
                let arrange = self
                    .doc
                    .widgets
                    .get(&instance.component)
                    .and_then(|widget| widget.arrange.as_ref())
                    .and_then(variant);
                let children = self.node_list(at, "body", &instance.body);
                El::new(&widget_box)
                    .path_if(!opts.framed, &at.to_string())
                    .expr("widget", ts::string(&instance.component))
                    .opt("arrange", arrange.map(|a| ts::string(&a)))
                    .children(children)
                    .render()
            }
            Body::Primitive(primitive) => self.primitive(at, primitive),
        }
    }

    fn degrades(opts: &Opts) -> Option<String> {
        (!opts.degrades.is_empty()).then(|| ts::string_map(&opts.degrades))
    }

    fn own_reads(opts: &Opts, reads: Option<&Reads>) -> Option<String> {
        match opts.rows {
            Rows::FromSection => None,
            Rows::Own => reads.and_then(Self::reads),
        }
    }

    #[allow(clippy::too_many_lines)] // one arm per composite kind
    fn composite(&mut self, at: &NodePath, composite: &Composite, opts: &Opts) -> String {
        let path = at.to_string();
        match composite {
            Composite::Collection(c) => {
                let component = self.import("runtime/composites/collection", "Collection");
                let (columns, columns_bind) = match &c.columns {
                    Some(Columns::Fixed(fields)) => {
                        (self.fields(at, "columns", fields, false), None)
                    }
                    Some(Columns::Selectable(selectable)) => (
                        self.fields(&at.child("columns"), "all", &selectable.all, false),
                        Some(ts::string(&selectable.binds.0)),
                    ),
                    Some(Columns::Unmapped(_)) | None => (None, None),
                };
                let sort = c.sort.as_ref().map(|sort| {
                    ts::object([
                        ("by", Some(ts::string(&sort.by))),
                        (
                            "dir",
                            sort.dir.as_ref().and_then(variant).map(|d| ts::string(&d)),
                        ),
                        ("allowed", Some(strings(&sort.allowed))),
                        (
                            "mode",
                            sort.mode.as_ref().and_then(variant).map(|m| ts::string(&m)),
                        ),
                    ])
                });
                let (selection, enabled) = match &c.selection {
                    Some(Selection::Mode(mode)) => (variant(mode), None),
                    Some(Selection::Conditional(conditional)) => (
                        variant(&conditional.mode),
                        Some(ts::string(&conditional.enabled.0)),
                    ),
                    None => (None, None),
                };
                let row_actions = self.actions(at, "row_actions", &c.row_actions);
                let bulk_actions = self.actions(at, "bulk_actions", &c.bulk_actions);
                let actions = self.actions(at, "actions", &c.actions);
                let expand = c
                    .expand
                    .as_deref()
                    .map(|node| self.node(&at.child("expand"), node, &Opts::own()));
                let item = self.node_fragment(at, "item", &c.item);
                let reorder = c
                    .reorder
                    .as_ref()
                    .map(|reorder| ts::object([("does", Some(ts::string(&reorder.does)))]));
                El::new(&component)
                    .path_if(!opts.framed, &path)
                    .opt("reads", Self::own_reads(opts, c.reads.as_ref()))
                    .opt("columns", columns)
                    .opt("columnsBind", columns_bind)
                    .opt("sort", sort)
                    .opt(
                        "style",
                        c.style.as_ref().and_then(variant).map(|s| ts::string(&s)),
                    )
                    .opt("selection", selection.map(|s| ts::string(&s)))
                    .opt("selectionEnabled", enabled)
                    .opt(
                        "selectionState",
                        opts.selection_state.as_ref().map(|s| ts::string(s)),
                    )
                    .opt("rowActions", row_actions)
                    .opt("bulkActions", bulk_actions)
                    .opt("actions", actions)
                    .opt("expand", expand)
                    .opt("item", item)
                    .opt("reorder", reorder)
                    .opt("groupBy", quoted(c.group_by.as_ref()))
                    .opt(
                        "groupOrder",
                        (!c.group_order.is_empty()).then(|| strings(&c.group_order)),
                    )
                    .opt(
                        "showEmptyGroups",
                        c.show_empty_groups.then(|| "true".to_owned()),
                    )
                    .opt(
                        "rowKey",
                        c.reads
                            .as_ref()
                            .and_then(|reads| reads.key.as_ref())
                            .or(opts.row_key.as_ref())
                            .map(|key| ts::string(key)),
                    )
                    .opt("degrades", Self::degrades(opts))
                    .render()
            }
            Composite::Record(r) => {
                let component = self.import("runtime/composites/record", "RecordView");
                let fields = self.fields(at, "fields", &r.fields, false);
                let tabs = self.tabs(at, &r.tabs, false);
                let item = self.node_fragment(at, "item", &r.item);
                let actions = self.actions(at, "actions", &r.actions);
                El::new(&component)
                    .path_if(!opts.framed, &path)
                    .opt("reads", Self::own_reads(opts, r.reads.as_ref()))
                    .opt("fields", fields)
                    .opt("tabs", tabs)
                    .opt("item", item)
                    .opt("actions", actions)
                    .render()
            }
            Composite::Form(f) => {
                let component = self.import("runtime/composites/form", "FormView");
                let fields = self.fields(at, "fields", &f.fields, true);
                let mut groups = Vec::new();
                for group in &f.groups {
                    let here = at.child("groups").child(&group.name);
                    let group_fields = self
                        .fields(&here, "fields", &group.fields, true)
                        .unwrap_or_else(|| "[]".to_owned());
                    let group_actions = self.actions(&here, "actions", &group.actions);
                    groups.push(ts::object([
                        ("data-ui-path", Some(ts::string(&here.to_string()))),
                        ("name", Some(ts::string(&group.name))),
                        ("label", quoted(group.label.as_ref())),
                        ("fields", Some(group_fields)),
                        ("save", quoted(group.save.as_ref())),
                        ("does", quoted(group.does.as_ref())),
                        ("actions", group_actions),
                    ]));
                }
                let groups = (!groups.is_empty()).then(|| {
                    self.import("runtime/composites/form", "type GroupSpec");
                    ts::array(groups)
                });
                let tabs = self.tabs(at, &f.tabs, true);
                let parts = self.node_fragment(at, "parts", &f.parts);
                let actions = self.actions(at, "actions", &f.actions);
                let result = f
                    .result
                    .as_deref()
                    .map(|node| self.node(&at.child("result"), node, &Opts::own()));
                let record = f
                    .record
                    .as_deref()
                    .map(|node| self.node(&at.child("record"), node, &Opts::own()));
                let submit = f.submit.as_ref().map(|submit| {
                    ts::object([
                        ("label", quoted(submit.label.as_ref())),
                        ("closes", submit.closes.map(|v| v.to_string())),
                        ("auto", submit.auto.map(|v| v.to_string())),
                    ])
                });
                let variant_by = f.variant_by.as_ref().map(|variant_by| {
                    ts::object([
                        ("field", Some(ts::string(&variant_by.field))),
                        ("forms", Some(ts::string_map(&variant_by.forms))),
                    ])
                });
                El::new(&component)
                    .path_if(!opts.framed, &path)
                    .expr("does", ts::string(&f.does))
                    .opt("loads", f.loads.as_ref().and_then(Self::reads))
                    .opt("fields", fields)
                    .opt("groups", groups)
                    .opt("tabs", tabs)
                    .opt("parts", parts)
                    .opt("actions", actions)
                    .opt("result", result)
                    .opt("record", record)
                    .opt("submit", submit)
                    .opt(
                        "save",
                        f.save.as_ref().and_then(variant).map(|s| ts::string(&s)),
                    )
                    .opt("variantBy", variant_by)
                    .opt("localDraft", opts.local_draft.then(|| "true".to_owned()))
                    .render()
            }
            Composite::Choice(c) => {
                let component = self.import("runtime/composites/choice", "ChoiceView");
                let options = (!c.options.is_empty()).then(|| {
                    ts::array(c.options.iter().map(|option| {
                        ts::object([
                            ("value", Some(ts::literal(&option.value))),
                            ("label", Some(ts::string(&option.label))),
                        ])
                    }))
                });
                let binds = c
                    .binds
                    .as_ref()
                    .map(|binds| binds.0.clone())
                    .or_else(|| opts.binds.clone());
                // A live choice's node reads for it and hands its rows down (#354).
                let handed = opts.data.filter(|_| c.reads.is_some());
                El::new(&component)
                    .path_if(!opts.framed, &path)
                    .opt(
                        "reads",
                        c.reads
                            .as_ref()
                            .filter(|_| handed.is_none())
                            .and_then(Self::reads),
                    )
                    .opt("data", handed.map(str::to_owned))
                    .opt("options", options)
                    .opt("binds", binds.map(|b| ts::string(&b)))
                    .opt(
                        "valueKey",
                        opts.choice_value.as_ref().map(|key| ts::string(key)),
                    )
                    .opt("valueField", c.value_field().map(ts::string))
                    .opt("labelField", c.label.as_deref().map(ts::string))
                    .opt(
                        "identity",
                        c.reads
                            .as_ref()
                            .and_then(|reads| reads.view.as_deref())
                            .and_then(|view| self.binding.as_ref()?.view(view)?.identity.as_deref())
                            .map(ts::string),
                    )
                    .opt("multiple", c.multiple.then(|| "true".to_owned()))
                    .opt(
                        "style",
                        c.style.as_ref().and_then(variant).map(|s| ts::string(&s)),
                    )
                    .opt(
                        "creatable",
                        c.creatable.as_ref().map(|creatable| {
                            ts::object([("does", Some(ts::string(&creatable.does)))])
                        }),
                    )
                    .opt("note", quoted(c.note.as_ref()))
                    .opt("label", choice_label(at).map(|name| ts::string(&name)))
                    .render()
            }
            Composite::FilterBar(bar) => {
                let component = self.import("runtime/composites/filter_bar", "FilterBar");
                let bound: BTreeSet<&str> =
                    bar.binds.iter().map(|binds| binds.0.as_str()).collect();
                let mut choices = Vec::new();
                for node in &bar.choices {
                    let name = node.common.name.clone().unwrap_or_default();
                    let mut opts = Opts::own();
                    let target = format!("state.{name}");
                    if bound.contains(target.as_str()) {
                        opts.binds = Some(target);
                    }
                    choices.push(self.node(&at.child("choices").child(&name), node, &opts));
                }
                let inputs = self.fields(at, "inputs", &bar.inputs, false);
                let actions = self.actions(at, "actions", &bar.actions);
                El::new(&component)
                    .path_if(!opts.framed, &path)
                    .expr(
                        "binds",
                        ts::array(bar.binds.iter().map(|b| ts::string(&b.0))),
                    )
                    .opt(
                        "search",
                        bar.search.as_ref().map(|search| {
                            ts::object([
                                ("binds", Some(ts::string(&search.binds.0))),
                                ("placeholder", quoted(search.placeholder.as_ref())),
                            ])
                        }),
                    )
                    .opt(
                        "window",
                        bar.window.as_ref().map(|window| {
                            ts::object([
                                ("binds", Some(ts::string(&window.binds.0))),
                                ("time", window.time.map(|t| t.to_string())),
                            ])
                        }),
                    )
                    .opt("choices", (!choices.is_empty()).then(|| fragment(choices)))
                    .opt("inputs", inputs)
                    .opt("actions", actions)
                    .opt("reset", (!bar.reset).then(|| "false".to_owned()))
                    .render()
            }
            Composite::Confirm(c) => {
                let component = self.import("runtime/composites/confirm", "ConfirmView");
                let alternatives = self.actions(at, "alternatives", &c.alternatives);
                El::new(&component)
                    .path_if(!opts.framed, &path)
                    .opt("body", quoted(c.body.as_ref()))
                    .opt("does", quoted(c.does.as_ref()))
                    .opt("references", quoted(c.references.as_ref()))
                    .opt(
                        "consequences",
                        (!c.consequences.is_empty()).then(|| strings(&c.consequences)),
                    )
                    .opt("confirmLabel", quoted(c.confirm_label.as_ref()))
                    .opt("danger", c.danger.map(|d| d.to_string()))
                    .opt(
                        "input",
                        c.input.as_ref().map(|input| {
                            ts::object([
                                ("label", Some(ts::string(&input.label))),
                                ("mustEqual", expr(input.must_equal.as_ref())),
                            ])
                        }),
                    )
                    .opt("alternatives", alternatives)
                    .render()
            }
            Composite::Metric(m) => {
                let component = self.import("runtime/composites/metric", "MetricView");
                El::new(&component)
                    .path_if(!opts.framed, &path)
                    .opt("reads", Self::own_reads(opts, m.reads.as_ref()))
                    .opt("from", expr(m.from.as_ref()))
                    .opt("match", quoted(m.match_field.as_ref()))
                    .opt("window", quoted(m.window.as_ref()))
                    .opt(
                        "format",
                        m.format.as_ref().and_then(variant).map(|f| ts::string(&f)),
                    )
                    .opt("label", quoted(m.label.as_ref()))
                    .opt(
                        "aggregate",
                        m.aggregate.map(|kind| ts::string(kind.as_str())),
                    )
                    .opt("field", quoted(m.field.as_ref()))
                    .render()
            }
            Composite::Chart(c) => {
                let component = self.import("runtime/composites/chart", "ChartView");
                let chart = match &c.chart {
                    ChartKind::Fixed(kind) => ts::string(kind),
                    ChartKind::Chosen(chosen) => ts::object([
                        ("binds", Some(ts::string(&chosen.binds.0))),
                        ("options", Some(strings(&chosen.options))),
                    ]),
                };
                El::new(&component)
                    .path_if(!opts.framed, &path)
                    .opt("reads", Self::own_reads(opts, Some(&c.reads)))
                    .expr("chart", chart)
                    .opt("x", quoted(c.x.as_ref()))
                    .opt("series", (!c.series.is_empty()).then(|| strings(&c.series)))
                    .opt("degrades", Self::degrades(opts))
                    .render()
            }
            Composite::Board(b) => {
                let component = self.import("runtime/composites/board", "BoardView");
                let mut widgets = Vec::new();
                for (kind, node) in &b.widgets {
                    let rendered = self.node(&at.child("widgets").child(kind), node, &Opts::own());
                    widgets.push((kind.clone(), Some(rendered)));
                }
                let item_actions = self.actions(at, "item_actions", &b.item_actions);
                let layout = b.layout.as_ref().map(|layout| {
                    ts::object([
                        ("persistedBy", Some(ts::string(&layout.persisted_by))),
                        ("editableBy", quoted(layout.editable_by.as_ref())),
                        ("state", expr(layout.state.as_ref())),
                    ])
                });
                El::new(&component)
                    .path_if(!opts.framed, &path)
                    .opt("reads", Self::own_reads(opts, Some(&b.reads)))
                    .opt("widgetBy", quoted(b.widget_by.as_ref()))
                    .expr("widgets", multiline_object(&widgets))
                    .opt("itemActions", item_actions)
                    .opt("layout", layout)
                    .opt("degrades", Self::degrades(opts))
                    .render()
            }
            Composite::GraphEditor(g) => {
                let component = self.import("runtime/composites/graph_editor", "GraphEditorView");
                let node_actions = self.actions(at, "node_actions", &g.node_actions);
                let edge_actions = self.actions(at, "edge_actions", &g.edge_actions);
                let toolbar = self.node_fragment(at, "toolbar", &g.toolbar);
                El::new(&component)
                    .path_if(!opts.framed, &path)
                    .opt("reads", Self::own_reads(opts, Some(&g.reads)))
                    .opt(
                        "nodes",
                        g.nodes.as_ref().map(|nodes| {
                            ts::object([
                                ("key", quoted(nodes.key.as_ref())),
                                ("label", quoted(nodes.label.as_ref())),
                                ("kindBy", quoted(nodes.kind_by.as_ref())),
                                ("opens", quoted(nodes.opens.as_ref())),
                            ])
                        }),
                    )
                    .opt(
                        "edges",
                        g.edges.as_ref().map(|edges| {
                            ts::object([
                                ("reads", edges.reads.as_ref().and_then(Self::reads)),
                                ("from", Some(ts::string(&edges.from))),
                                ("to", Some(ts::string(&edges.to))),
                                ("kindBy", quoted(edges.kind_by.as_ref())),
                            ])
                        }),
                    )
                    .opt("nodeActions", node_actions)
                    .opt("edgeActions", edge_actions)
                    .opt("toolbar", toolbar)
                    .opt("degrades", Self::degrades(opts))
                    .render()
            }
            Composite::RichText(r) => {
                let component = self.import("runtime/composites/rich_text", "RichTextView");
                let binds = r
                    .binds
                    .as_ref()
                    .map(|binds| binds.0.clone())
                    .or_else(|| opts.binds.clone());
                El::new(&component)
                    .path_if(!opts.framed, &path)
                    .opt("completes", quoted(r.completes.as_ref()))
                    .opt(
                        "syntax",
                        r.syntax.as_ref().and_then(variant).map(|s| ts::string(&s)),
                    )
                    .opt("binds", binds.map(|b| ts::string(&b)))
                    .render()
            }
            Composite::References(r) => {
                let component = self.import("runtime/composites/references", "ReferencesView");
                let columns = self.fields(at, "columns", &r.columns, false);
                El::new(&component)
                    .path_if(!opts.framed, &path)
                    .opt("reads", Self::own_reads(opts, Some(&r.reads)))
                    .opt("columns", columns)
                    .opt("navigates", r.navigates.map(|n| n.to_string()))
                    .render()
            }
        }
    }

    #[allow(clippy::too_many_lines)] // one arm per primitive
    fn primitive(&mut self, at: &NodePath, primitive: &Primitive) -> String {
        let path = at.to_string();
        match primitive {
            Primitive::Text(t) => {
                let component = self.import("runtime/primitives/text", "TextView");
                El::new(&component)
                    .path(&path)
                    .opt("text", expr(t.text.as_ref()))
                    .opt("field", quoted(t.field.as_ref()))
                    .opt(
                        "style",
                        t.style.as_ref().and_then(variant).map(|s| ts::string(&s)),
                    )
                    .opt(
                        "format",
                        t.format.as_ref().and_then(variant).map(|s| ts::string(&s)),
                    )
                    .opt("currency", expr(t.currency.as_ref()))
                    .render()
            }
            Primitive::Badge(b) => {
                let component = self.import("runtime/primitives/badge", "BadgeView");
                El::new(&component)
                    .path(&path)
                    .opt("text", expr(b.text.as_ref()))
                    .opt("field", quoted(b.field.as_ref()))
                    .opt(
                        "tone",
                        b.tone.as_ref().and_then(variant).map(|s| ts::string(&s)),
                    )
                    .opt(
                        "toneBy",
                        b.tone_by.as_ref().map(|tone_by| {
                            ts::object([
                                ("value", Some(ts::string(&tone_by.value.0))),
                                ("map", Some(ts::literal(&tone_by.map))),
                            ])
                        }),
                    )
                    .render()
            }
            Primitive::Icon(i) => {
                let component = self.import("runtime/primitives/icon", "IconView");
                El::new(&component)
                    .path(&path)
                    .expr("icon", ts::string(&i.icon))
                    .opt(
                        "tone",
                        i.tone.as_ref().and_then(variant).map(|s| ts::string(&s)),
                    )
                    .expr("label", ts::string(&i.label))
                    .render()
            }
            Primitive::Button(b) => {
                let component = self.import("runtime/primitives/button", "ButtonView");
                let action = self.action(&at.child("action"), &b.action);
                El::new(&component)
                    .path(&path)
                    .expr("label", ts::string(&b.label))
                    .expr("action", action)
                    .opt(
                        "tone",
                        b.tone.as_ref().and_then(variant).map(|s| ts::string(&s)),
                    )
                    .opt("icon", quoted(b.icon.as_ref()))
                    .render()
            }
            Primitive::Link(l) => {
                let component = self.import("runtime/primitives/link", "LinkView");
                let to = l.to.as_ref().map(|navigate| {
                    ts::object([
                        ("page", Some(ts::string(&navigate.to))),
                        (
                            "params",
                            (!navigate.params.is_empty()).then(|| exprs(&navigate.params)),
                        ),
                    ])
                });
                El::new(&component)
                    .path(&path)
                    .expr("text", ts::string(&l.text.0))
                    .opt("to", to)
                    .opt("href", expr(l.href.as_ref()))
                    .render()
            }
            Primitive::Input(i) => {
                let component = self.import("runtime/primitives/input", "InputView");
                El::new(&component)
                    .path(&path)
                    .opt("as", quoted(i.input_as.as_ref()))
                    .expr("binds", ts::string(&i.binds.0))
                    .opt("placeholder", quoted(i.placeholder.as_ref()))
                    .render()
            }
            Primitive::Toggle(t) => {
                let component = self.import("runtime/primitives/toggle", "ToggleView");
                let action = t
                    .action
                    .as_ref()
                    .map(|action| self.action(&at.child("action"), action));
                El::new(&component)
                    .path(&path)
                    .expr("label", ts::string(&t.label))
                    .opt("binds", expr(t.binds.as_ref()))
                    .opt("action", action)
                    .render()
            }
            Primitive::Image(i) => {
                let component = self.import("runtime/primitives/image", "ImageView");
                El::new(&component)
                    .path(&path)
                    .expr("src", ts::string(&i.src.0))
                    .expr("alt", ts::string(&i.alt))
                    .opt("fit", quoted(i.fit.as_ref()))
                    .render()
            }
            Primitive::Divider(d) => {
                let component = self.import("runtime/primitives/divider", "DividerView");
                El::new(&component)
                    .path(&path)
                    .opt("orientation", quoted(d.orientation.as_ref()))
                    .render()
            }
        }
    }

    // ── overlays ─────────────────────────────────────────────────────────────────────────

    fn overlay_frame(&mut self, at: &NodePath, overlay: &Overlay, body: String) -> String {
        let frame = self.import("runtime/overlays", "OverlayFrame");
        let kind = variant(&overlay.kind).unwrap_or_else(|| "dialog".to_owned());
        El::new(&frame)
            .path(&at.to_string())
            .expr("kind", ts::string(&kind))
            .opt("title", quoted(overlay.title.as_ref()))
            .child(body)
            .render()
    }

    /// An inline confirm overlay, rendered where its action is.
    fn overlay_inline(&mut self, at: &NodePath, overlay: &Overlay) -> String {
        let mut opts = Opts::own();
        opts.framed = true;
        let body = self.body(at, &overlay.body, &opts);
        let framed = self.overlay_frame(at, overlay, body);
        self.visible(&overlay.common, framed)
    }

    /// An overlay of a page or shell, as its own component.
    fn overlay_component(&mut self, name: &str, at: &NodePath, overlay: &Overlay) {
        let mut lines = Vec::new();
        self.begin_component();
        let mut extra = Vec::new();
        let mut draft_setter = None;
        let mut opts = Opts::own();
        opts.framed = true;
        opts.degrades.clone_from(&overlay.common.degrades);
        if let Body::Composite(Composite::Form(form)) = &overlay.body {
            let (local, setter) = self.draft_hook(&mut lines, at, form.draft.as_ref());
            extra.push(("draft".to_owned(), local));
            draft_setter = Some(setter);
            opts.local_draft = false;
        }
        let (values, mut setters) =
            self.state_hooks(&mut lines, at, &overlay.common.state, "state");
        if let Some(setter) = draft_setter {
            setters.push(("draft".to_owned(), setter));
        }
        self.end_component();
        if !overlay.params.is_empty() {
            let use_scope = self.import("runtime/core", "useScope");
            let evaluate = self.import("runtime/expr", "evaluate");
            lines.push(format!("const __outer = {use_scope}();"));
            // Params are values passed by the opener: `row` is the row the opener ran on.
            lines.push(
                "const __opener = { ...__outer.values, row: __outer.values.opener_row };"
                    .to_owned(),
            );
            let params: Vec<String> = overlay
                .params
                .iter()
                .map(|(key, value)| {
                    format!(
                        "{}: {evaluate}({}, __opener)",
                        ts::key(key),
                        ts::string(&value.0)
                    )
                })
                .collect();
            lines.push(format!(
                "const __overlayParams = {{ ...(__outer.values.params as Record<string, unknown>), {} }};",
                params.join(", ")
            ));
            extra.push(("params".to_owned(), "__overlayParams".to_owned()));
        }
        let body = self.body(at, &overlay.body, &opts);
        let framed = self.overlay_frame(at, overlay, body);
        let framed = self.visible(&overlay.common, framed);
        let rendered = self.scope_layer("state", &values, &setters, &extra, framed);
        self.hoist(name, &lines, &rendered);
    }

    fn overlay_host(
        &mut self,
        owner: &str,
        at: &NodePath,
        overlays: &BTreeMap<String, Overlay>,
        inner: String,
    ) -> String {
        if overlays.is_empty() {
            return inner;
        }
        let host = self.import("runtime/overlays", "OverlayHost");
        let mut table = Vec::new();
        for (name, overlay) in overlays {
            let component = self.component_name(&format!("{owner}{}Overlay", ts::pascal(name)));
            self.overlay_component(&component, &at.child("overlays").child(name), overlay);
            table.push((name.clone(), Some(format!("() => <{component} />"))));
        }
        El::new(&host)
            .expr("overlays", multiline_object(&table))
            .child(inner)
            .render()
    }

    // ── sections ─────────────────────────────────────────────────────────────────────────

    fn states_literal(states: &SectionStates) -> String {
        ts::object([
            (
                "loading",
                states
                    .loading
                    .as_ref()
                    .and_then(variant)
                    .map(|v| ts::string(&v)),
            ),
            (
                "refreshing",
                states
                    .refreshing
                    .as_ref()
                    .and_then(variant)
                    .map(|v| ts::string(&v)),
            ),
            (
                "empty",
                states
                    .empty
                    .as_ref()
                    .map(|empty| ts::object([("message", Some(ts::string(&empty.message)))])),
            ),
            (
                "failed",
                states.failed.as_ref().map(|failed| {
                    ts::object([
                        ("message", quoted(failed.message.as_ref())),
                        ("retry", Some(failed.retry.to_string())),
                    ])
                }),
            ),
            (
                "forbidden",
                states
                    .forbidden
                    .as_ref()
                    .map(|forbidden| ts::object([("message", quoted(forbidden.message.as_ref()))])),
            ),
            (
                "stale",
                states.stale.as_ref().and_then(|stale| {
                    variant(&stale.mark).map(|mark| ts::object([("mark", Some(ts::string(&mark)))]))
                }),
            ),
        ])
    }

    fn live_literal(live: &Live) -> String {
        ts::object([
            ("channel", Some(ts::string(&live.channel))),
            ("on", (!live.on.is_empty()).then(|| strings(&live.on))),
            (
                "effect",
                Some(ts::string(
                    &variant(&live.effect).unwrap_or_else(|| "refetch".to_owned()),
                )),
            ),
            ("match", quoted(live.match_field.as_ref())),
            ("onlyIf", expr(live.only_if.as_ref())),
            ("coalesce", duration(live.coalesce.as_ref())),
            (
                "whenPagedAway",
                live.when_paged_away
                    .as_ref()
                    .and_then(variant)
                    .map(|v| ts::string(&v)),
            ),
            ("maxRows", live.max_rows.map(|v| v.to_string())),
        ])
    }

    /// The read a section runs for its composite; forms load their own.
    fn section_reads(body: &Body) -> Option<&Reads> {
        match body {
            Body::Composite(composite) => match composite {
                Composite::Collection(c) => c.reads.as_ref(),
                Composite::Record(c) => c.reads.as_ref(),
                Composite::Metric(c) => c.reads.as_ref(),
                Composite::Chart(c) => Some(&c.reads),
                Composite::Board(c) => Some(&c.reads),
                Composite::GraphEditor(c) => Some(&c.reads),
                Composite::References(c) => Some(&c.reads),
                Composite::Form(_)
                | Composite::Choice(_)
                | Composite::FilterBar(_)
                | Composite::Confirm(_)
                | Composite::RichText(_) => None,
            },
            _ => None,
        }
    }

    fn selection_state(page: &Page, section: &Section) -> Option<String> {
        section
            .common
            .state
            .iter()
            .chain(page.state.iter())
            .find(|(_, state)| state.class == StateClass::Selection)
            .map(|(name, _)| format!("state.{name}"))
    }

    /// The expression whose value keys a `session: {per}` channel's instance on this page: the
    /// param of that name in the read of a section the channel feeds, else the page param.
    /// A page with neither (nor a shell, which has no params) has no value to key on: refused at `at`.
    fn session_expr(
        &mut self,
        at: &NodePath,
        page: Option<&Page>,
        channel: &str,
    ) -> Option<String> {
        let per = self
            .doc
            .channels
            .get(channel)?
            .session
            .as_ref()?
            .per
            .clone();
        let per = &per;
        let from_read = page.and_then(|page| {
            page.sections
                .iter()
                .filter(|section| {
                    section
                        .live
                        .as_ref()
                        .is_some_and(|live| live.channel == channel)
                })
                .find_map(|section| {
                    Self::section_reads(&section.body)
                        .and_then(|reads| reads.params.get(per))
                        .map(|value| value.0.clone())
                })
        });
        let from_param = page
            .filter(|page| page.params.contains_key(per))
            .map(|_| format!("params.{per}"));
        if from_read.is_none() && from_param.is_none() {
            self.refuse(
                at,
                &format!(
                    "channel `{channel}` has one instance per `{per}` (session.per), and nothing here gives a `{per}`: no read param of a section it feeds and no page param of that name"
                ),
            );
        }
        Some(
            from_read
                .or(from_param)
                .unwrap_or_else(|| format!("params.{per}")),
        )
    }

    /// `, { <channel>: <session value> }` for the `session: {per}` channels among `channels`,
    /// evaluated against `values`; empty when none has a session.
    fn sessions(
        &mut self,
        at: &NodePath,
        page: Option<&Page>,
        channels: &[String],
        values: &str,
    ) -> String {
        let mut entries = Vec::new();
        for channel in channels {
            if let Some(session) = self.session_expr(at, page, channel) {
                let evaluate = self.import("runtime/expr", "evaluate");
                entries.push((
                    channel.as_str(),
                    Some(format!("{evaluate}({}, {values})", ts::string(&session))),
                ));
            }
        }
        if entries.is_empty() {
            String::new()
        } else {
            format!(", {}", ts::object(entries))
        }
    }

    #[allow(clippy::too_many_lines)] // one component, one hook per concern
    /// How often a bound `live:` section or nested node polls, in milliseconds: no served surface
    /// streams, so it takes its `no_live` fallback, polling unless it says `refuse`. Its read's
    /// `refresh:` is the interval (5 s without one) and must be a duration from 1 s to 24 h; one
    /// the generator cannot read as a duration is refused, never replaced by the default. `what`
    /// names the holder in a refusal: `section`, or `node` (beyond10x/ess#354).
    fn poll_interval(
        &mut self,
        at: &NodePath,
        degrades: &BTreeMap<String, String>,
        reads: Option<&Reads>,
        what: &str,
    ) -> u64 {
        let fallback = degrades.get("no_live").map_or("poll", String::as_str);
        if fallback == "refuse" {
            self.refuse(
                &at.child("live"),
                &format!(
                    "the served surface streams no events, and this {what}'s \
                     `degrades: {{no_live: refuse}}` refuses to poll instead"
                ),
            );
        }
        let Some(refresh) = reads.and_then(|reads| reads.refresh.as_ref()) else {
            return POLL_MS;
        };
        let at = at.child("reads").child("refresh");
        let Some(every) = millis(&refresh.0) else {
            self.refuse(
                &at,
                &format!(
                    "a live {what} bound to the served surface polls at its `refresh:`, and `{}` \
                     is not a whole number of ms, s, m or h this generator can poll at",
                    refresh.0
                ),
            );
            return POLL_MS;
        };
        if !(MIN_POLL_MS..=MAX_POLL_MS).contains(&every) {
            self.refuse(
                &at,
                &format!(
                    "a live {what} bound to the served surface polls at its `refresh:`, and \
                     {every} ms is outside 1 s to 24 h"
                ),
            );
        }
        every
    }

    /// What a reading section shows: `__read`, or `__data` when it is live — polled in a bound
    /// project, played from its channel otherwise.
    fn live_data(
        &mut self,
        lines: &mut Vec<String>,
        page: &Page,
        at: &NodePath,
        section: &Section,
    ) -> &'static str {
        let Some(live) = &section.live else {
            return "__read";
        };
        let reads = Self::section_reads(&section.body);
        let session = if self.bound {
            None
        } else {
            self.session_expr(at, Some(page), &live.channel)
        };
        self.live_rows(
            lines,
            at,
            live,
            reads,
            &section.common.degrades,
            "section",
            session,
        )
    }

    /// `const __data = …`: `__read` with `live` applied — polled at the read's `refresh:` in a
    /// bound project, played from the channel otherwise, keyed by `session` for a `session: {per}`
    /// channel. A live insert is kept only when it passes the read's `filter`.
    #[allow(clippy::too_many_arguments)]
    fn live_rows(
        &mut self,
        lines: &mut Vec<String>,
        at: &NodePath,
        live: &Live,
        reads: Option<&Reads>,
        degrades: &BTreeMap<String, String>,
        what: &str,
        session: Option<String>,
    ) -> &'static str {
        if self.bound {
            let every = self.poll_interval(at, degrades, reads, what);
            let use_poll = self.import("runtime/data", "usePoll");
            lines.push(format!("const __data = {use_poll}(__read, {every});"));
            return "__data";
        }
        let use_live = self.import("runtime/live", "useLive");
        let mut session = match session {
            Some(session) => {
                let evaluate = self.import("runtime/expr", "evaluate");
                format!(", {evaluate}({}, __scope.values)", ts::string(&session))
            }
            None => String::new(),
        };
        if let Some(filter) = reads.and_then(|reads| reads.filter.as_ref()) {
            if session.is_empty() {
                session.push_str(", undefined");
            }
            let keeps = self.import("runtime/expr", "keeps");
            let _ = write!(
                session,
                ", row => {keeps}({}, {{ ...__scope.values, row }})",
                ts::string(&filter.0)
            );
        }
        lines.push(format!(
            "const __data = {use_live}(__read, {}{session});",
            Self::live_literal(live)
        ));
        "__data"
    }

    fn filtered_data(
        &mut self,
        lines: &mut Vec<String>,
        data: &'static str,
        reads: Option<&Reads>,
    ) -> &'static str {
        let Some(filter) = reads.and_then(|reads| reads.filter.as_ref()) else {
            return data;
        };
        let filter_read = self.import("runtime/data", "filterRead");
        lines.push(format!(
            "const __filtered = {filter_read}({data}, {}, __scope.values);",
            ts::string(&filter.0)
        ));
        "__filtered"
    }

    /// `frame` with the section's `states:`, and the action its empty state offers.
    fn with_states(&mut self, mut frame: El, at: &NodePath, section: &Section) -> El {
        if let Some(states) = &section.states {
            frame = frame.expr("states", Self::states_literal(states));
            if let Some(action) = states
                .empty
                .as_ref()
                .and_then(|empty| empty.action.as_ref())
            {
                let control =
                    self.action_control(&at.child("states").child("empty").child("action"), action);
                frame = frame.expr("emptyAction", control);
            }
        }
        frame
    }

    /// Whether the page header's title reads its record from `section` (`header.title_from`,
    /// beyond10x/ess#354): that section reports its first row.
    fn titles_header(page: &Page, section: &Section) -> bool {
        page.header
            .as_ref()
            .and_then(|header| header.title_from.as_ref())
            .is_some_and(|from| from.section == section.name)
    }

    fn section_component(&mut self, name: &str, page: &Page, at: &NodePath, section: &Section) {
        let mut lines = Vec::new();
        self.section_profile = section.profile.as_ref().and_then(variant);
        self.begin_component();
        let reads = Self::section_reads(&section.body).and_then(Self::reads);
        let mut frame = El::new(&self.import("runtime/core", "SectionFrame"))
            .path(&at.to_string())
            .expr("name", ts::string(&section.name))
            .opt("title", quoted(section.title.as_ref()));
        if Self::titles_header(page, section) && reads.is_some() {
            frame = frame.expr("reportsRow", "true");
        }
        if let Some(reads) = &reads {
            let use_scope = self.import("runtime/core", "useScope");
            let trigger = self.import("runtime/core", "useLoadTrigger");
            let use_read = self.import("runtime/data", "useRead");
            let load = section
                .load
                .as_ref()
                .and_then(variant)
                .unwrap_or_else(|| "eager".to_owned());
            lines.push(format!("const __scope = {use_scope}();"));
            lines.push(format!(
                "const [__active, __activate] = {trigger}({});",
                ts::string(&load)
            ));
            let enabled = match &section.depends_on {
                Some(dependency) => {
                    let evaluate = self.import("runtime/expr", "evaluate");
                    let truthy = self.import("runtime/expr", "truthy");
                    format!(
                        "__active && {truthy}({evaluate}({}, __scope.values))",
                        ts::string(&format!("section.{dependency}.selection"))
                    )
                }
                None => "__active".to_owned(),
            };
            lines.push(format!(
                "const __read = {use_read}({reads}, __scope.values, {enabled});"
            ));
            let data = self.live_data(&mut lines, page, at, section);
            let data = self.filtered_data(&mut lines, data, Self::section_reads(&section.body));
            frame = frame
                .expr("data", data)
                .expr("active", "__active")
                .expr("onActivate", "__activate");
            if load != "eager" {
                frame = frame.expr("load", ts::string(&load));
            }
        }
        frame = self
            .with_states(frame, at, section)
            .opt(
                "degrades",
                (!section.common.degrades.is_empty())
                    .then(|| ts::string_map(&section.common.degrades)),
            )
            .opt(
                "profile",
                section
                    .profile
                    .as_ref()
                    .and_then(variant)
                    .map(|p| ts::string(&p)),
            );
        let mut extra = Vec::new();
        let mut draft_setter = None;
        let mut opts = Opts {
            rows: if reads.is_some() {
                Rows::FromSection
            } else {
                Rows::Own
            },
            binds: None,
            local_draft: true,
            selection_state: Self::selection_state(page, section),
            degrades: section.common.degrades.clone(),
            framed: true,
            row_key: section.row_key().map(str::to_owned),
            choice_value: None,
            data: None,
        };
        if let Body::Composite(Composite::Form(form)) = &section.body {
            let (local, setter) = self.draft_hook(&mut lines, at, form.draft.as_ref());
            extra.push(("draft".to_owned(), local));
            draft_setter = Some(setter);
            opts.local_draft = false;
        }
        let (values, mut setters) =
            self.state_hooks(&mut lines, at, &section.common.state, "state");
        if let Some(setter) = draft_setter {
            setters.push(("draft".to_owned(), setter));
        }
        self.end_component();
        let body = self.body(at, &section.body, &opts);
        let children = self.node_list(at, "children", &section.children);
        self.section_profile = None;
        let framed = frame.child(body).children(children).render();
        let framed = self.visible(&section.common, framed);
        let rendered = self.scope_layer("state", &values, &setters, &extra, framed);
        self.hoist(name, &lines, &rendered);
    }

    // ── pages ────────────────────────────────────────────────────────────────────────────

    fn header(&mut self, at: &NodePath, name: &str, page: &Page, header: &Header) -> String {
        let frame = self.import("runtime/core", "HeaderFrame");
        let mut switch: Vec<&String> = header.switch.iter().collect();
        for target in &page.switch_to {
            if !switch.contains(&target) {
                switch.push(target);
            }
        }
        let switch_to = (!switch.is_empty()).then(|| {
            let href = self.import("routes", "hrefFor");
            ts::array(
                switch
                    .iter()
                    .filter(|target| target.as_str() != name)
                    .map(|target| {
                        ts::object([
                            (
                                "label",
                                Some(ts::string(&nav_label(target, self.doc.pages.get(*target)))),
                            ),
                            (
                                "href",
                                Some(format!("{href}({}, __params)", ts::string(target))),
                            ),
                        ])
                    }),
            )
        });
        let actions: Vec<String> = header
            .actions
            .iter()
            .map(|action| self.action_control(&at.child("actions").child(&action.name), action))
            .collect();
        let metrics = self.node_fragment(at, "metrics", &header.metrics);
        let help = header.help.as_ref().map(|help| {
            ts::object([
                ("text", quoted(help.text.as_ref())),
                ("link", quoted(help.link.as_ref())),
            ])
        });
        let title_from = header.title_from.as_ref().map(|from| {
            ts::object([
                ("section", Some(ts::string(&from.section))),
                ("field", Some(ts::string(&from.field))),
            ])
        });
        El::new(&frame)
            .path(&at.to_string())
            .opt("title", quoted(header.title.as_ref()))
            .opt("titleFrom", title_from)
            .opt("total", quoted(header.total.as_ref()))
            .opt("filters", quoted(header.filters.as_ref()))
            .opt("switchTo", switch_to)
            // A bound project holds no channel open, so its header shows no live state.
            .opt(
                "live",
                (!header.live.is_empty() && !self.bound).then(|| strings(&header.live)),
            )
            .opt("help", help)
            .opt("actions", (!actions.is_empty()).then(|| fragment(actions)))
            .opt("metrics", metrics)
            .render()
    }

    fn layout(
        &mut self,
        at: &NodePath,
        page: &Page,
        sections: &BTreeMap<String, String>,
    ) -> String {
        let element = |name: &String| format!("<{} />", sections[name]);
        let placed: BTreeSet<&String> = match &page.layout {
            PageLayout::Stack(_) => BTreeSet::new(),
            PageLayout::Columns(columns) => columns
                .columns
                .iter()
                .flat_map(|c| c.sections.iter())
                .collect(),
            PageLayout::Areas(areas) => areas.areas.place.values().flatten().collect(),
        };
        let rest: Vec<String> = page
            .sections
            .iter()
            .map(|section| &section.name)
            .filter(|name| !placed.contains(name) && sections.contains_key(*name))
            .map(element)
            .collect();
        let stack = self.import("runtime/core", "StackLayout");
        let body = match &page.layout {
            PageLayout::Stack(_) => return El::new(&stack).children(rest).render(),
            PageLayout::Columns(columns) => {
                let layout = self.import("runtime/core", "ColumnsLayout");
                let column = self.import("runtime/core", "LayoutColumn");
                let mut children = Vec::new();
                for entry in &columns.columns {
                    let inner: Vec<String> = entry
                        .sections
                        .iter()
                        .filter(|name| sections.contains_key(*name))
                        .map(element)
                        .collect();
                    children.push(
                        El::new(&column)
                            .path(
                                &at.child("layout")
                                    .child("columns")
                                    .child(&entry.name)
                                    .to_string(),
                            )
                            .opt(
                                "width",
                                entry
                                    .width
                                    .as_ref()
                                    .and_then(variant)
                                    .map(|w| ts::string(&w)),
                            )
                            .children(inner)
                            .render(),
                    );
                }
                El::new(&layout).children(children).render()
            }
            PageLayout::Areas(areas) => {
                let layout = self.import("runtime/core", "AreasLayout");
                let grid = ts::array(areas.areas.grid.iter().map(strings));
                let entries: Vec<(String, Option<String>)> = areas
                    .areas
                    .place
                    .iter()
                    .map(|(area, names)| {
                        let inner: Vec<String> = names
                            .iter()
                            .filter(|name| sections.contains_key(*name))
                            .map(element)
                            .collect();
                        (area.clone(), Some(fragment(inner)))
                    })
                    .collect();
                El::new(&layout)
                    .expr("grid", grid)
                    .expr("areas", multiline_object(&entries))
                    .render()
            }
        };
        if rest.is_empty() {
            body
        } else {
            El::new(&stack).child(body).children(rest).render()
        }
    }

    fn channels_used(&self, text: &str, extra: &[String]) -> Vec<String> {
        let mut names: BTreeSet<String> = extra.iter().cloned().collect();
        for name in self.doc.channels.keys() {
            if text.contains(&format!("channel.{name}."))
                || text.contains(&format!("channel.{name}\""))
            {
                names.insert(name.clone());
            }
        }
        names
            .into_iter()
            .filter(|name| self.doc.channels.contains_key(name))
            .collect()
    }

    /// `src/pages/<Page>.tsx`.
    #[allow(clippy::too_many_lines)] // sections, state, header, layout, overlays and channels of one page
    pub(crate) fn page(&mut self, name: &str, page: &Page) -> String {
        let owner = ts::pascal(name);
        self.prefix.clone_from(&owner);
        self.components = BTreeSet::from([format!("{owner}Page")]);
        self.page_profile = page.profile.as_ref().and_then(variant);
        self.page = Some(name.to_owned());
        let at = NodePath::root().child("pages").child(name);
        let mut section_components = BTreeMap::new();
        for section in &page.sections {
            let component =
                self.component_name(&format!("{owner}{}Section", ts::pascal(&section.name)));
            self.section_component(
                &component,
                page,
                &at.child("sections").child(&section.name),
                section,
            );
            section_components.insert(section.name.clone(), component);
        }
        let mut lines = Vec::new();
        let params = self.import("runtime/core", "usePageParams");
        lines.push(format!("const __params = {params}();"));
        self.begin_component();
        let (values, setters) = self.state_hooks(&mut lines, &at, &page.state, "state");
        self.end_component();
        let header = page
            .header
            .as_ref()
            .map(|header| self.header(&at.child("header"), name, page, header));
        let layout = self.layout(&at, page, &section_components);
        let frame = self.import("runtime/core", "PageFrame");
        let mut page_frame = El::new(&frame).path(&at.to_string()).opt(
            "profile",
            page.profile
                .as_ref()
                .and_then(variant)
                .map(|p| ts::string(&p)),
        );
        if header.is_none() {
            page_frame = page_frame.opt("title", quoted(page.title.as_ref()));
        }
        let framed = page_frame.children(header).child(layout).render();
        let hosted = self.overlay_host(&owner, &at, &page.overlays, framed);
        let hosted = match &page.visible {
            Some(condition) => {
                let visible = self.import("runtime/core", "Visible");
                El::new(&visible)
                    .expr("when", ts::string(&condition.0))
                    .child(hosted)
                    .render()
            }
            None => hosted,
        };
        let mut live: Vec<String> = page
            .header
            .as_ref()
            .map(|h| h.live.clone())
            .unwrap_or_default();
        // A bound project's live sections poll, and hold no channel open.
        live.extend(
            page.sections
                .iter()
                .filter(|_| !self.bound)
                .filter_map(|s| s.live.as_ref().map(|l| l.channel.clone())),
        );
        let scanned = format!("{}\n{hosted}", self.hoisted.join("\n"));
        let channels = self.channels_used(&scanned, &live);
        let mut extra = vec![("params".to_owned(), "__params".to_owned())];
        // A bound project runs no channel; `channel.*` reads nothing there.
        if !channels.is_empty() && !self.bound {
            let use_channels = self.import("runtime/live", "useChannels");
            // The same values the page's scope layer gives its sections: params and page state.
            let page_values = ts::object([
                ("params", Some("__params".to_owned())),
                (
                    "state",
                    (!values.is_empty()).then(|| {
                        ts::object(
                            values
                                .iter()
                                .map(|(name, local)| (name.as_str(), Some(local.clone()))),
                        )
                    }),
                ),
            ]);
            let sessions = self.sessions(&at, Some(page), &channels, &page_values);
            lines.push(format!(
                "const __channel = {use_channels}({}{sessions});",
                strings(&channels)
            ));
            extra.push(("channel".to_owned(), "__channel".to_owned()));
        }
        self.page_profile = None;
        self.page = None;
        let rendered = self.scope_layer("state", &values, &setters, &extra, hosted);
        self.import("react", "type ReactNode");
        let mut w = Writer::default();
        w.line(format!(
            "// Generated by ess-ui-react from pages/{name}. Do not edit."
        ));
        let body_imports = self.take_imports(1);
        for line in body_imports.lines() {
            w.line(line);
        }
        w.line("");
        w.open(format!("export function {owner}Page(): ReactNode {{"));
        for line in &lines {
            w.line(line);
        }
        w.line("return (");
        w.line(format!("  {}", indent(&rendered, 4)));
        w.line(");");
        w.close("}");
        for component in std::mem::take(&mut self.hoisted) {
            w.line("");
            for line in component.lines() {
                w.line(line);
            }
        }
        w.finish()
    }

    // ── shells ───────────────────────────────────────────────────────────────────────────

    fn region(&mut self, at: &NodePath, region: &Region, outlet: &str) -> String {
        let component = self.import("runtime/shell", "Region");
        let kind = variant(&region.kind).unwrap_or_else(|| "unmapped".to_owned());
        let prop = |name: &str| region.props.get(name);
        let content = match &region.kind {
            RegionKind::Navigation => Some(self.navigation(prop("collapsible"), prop("search"))),
            RegionKind::PageOutlet => Some(outlet.to_owned()),
            RegionKind::Notifications => {
                let notifications = self.import("runtime/shell", "Notifications");
                Some(format!("<{notifications} />"))
            }
            RegionKind::Assistant => {
                let assistant = self.import("runtime/shell", "Assistant");
                let does: Vec<String> = match prop("does") {
                    Some(Value::Sequence(items)) => items
                        .iter()
                        .filter_map(Value::as_str)
                        .map(ts::string)
                        .collect(),
                    Some(Value::String(one)) => vec![ts::string(one)],
                    _ => Vec::new(),
                };
                Some(El::new(&assistant).expr("does", ts::array(does)).render())
            }
            RegionKind::AccountMenu => {
                let menu = self.import("runtime/shell", "AccountMenu");
                let entries: Vec<String> = match prop("actions") {
                    Some(Value::Sequence(items)) => items
                        .iter()
                        .filter_map(Value::as_mapping)
                        .map(|entry| {
                            let text =
                                |key: &str| entry.get(key).and_then(Value::as_str).map(ts::string);
                            ts::object([
                                ("name", text("name")),
                                ("label", text("label")),
                                ("opens", text("opens")),
                                ("does", text("does")),
                            ])
                        })
                        .collect(),
                    _ => Vec::new(),
                };
                Some(El::new(&menu).expr("actions", ts::array(entries)).render())
            }
            RegionKind::OverlayOutlet | RegionKind::Unmapped(_) => None,
        };
        El::new(&component)
            .path(&at.to_string())
            .expr("kind", ts::string(&kind))
            .opt("visible", expr(region.visible.as_ref()))
            .children(content)
            .render()
    }

    fn navigation(&mut self, collapsible: Option<&Value>, search: Option<&Value>) -> String {
        let navigation = self.import("runtime/navigation", "Navigation");
        let section = self.import("runtime/navigation", "NavSection");
        let doc = self.doc;
        let restrictions = ts::array(doc.navigation.restrictions.iter().map(|restriction| {
            ts::object([
                ("when", Some(ts::string(&restriction.when.0))),
                ("sections", Some(strings(&restriction.sections))),
            ])
        }));
        let mut children = Vec::new();
        for entry in &doc.navigation.sections {
            let at = NodePath::root()
                .child("navigation")
                .child("sections")
                .child(&entry.name);
            let mut el = El::new(&section)
                .path(&at.to_string())
                .expr("name", ts::string(&entry.name))
                .opt("label", quoted(entry.label.as_ref()))
                .opt("icon", quoted(entry.icon.as_ref()));
            match &entry.pages {
                NavPages::Fixed(pages) => {
                    let links = ts::array(pages.iter().map(|page| {
                        let target = doc.pages.get(page);
                        let synonyms: Vec<String> = target
                            .and_then(|target| target.nav.as_ref())
                            .map(|nav| nav.synonyms.clone())
                            .unwrap_or_default();
                        ts::object([
                            ("page", Some(ts::string(page))),
                            ("label", Some(ts::string(&nav_label(page, target)))),
                            ("synonyms", Some(strings(&synonyms))),
                        ])
                    }));
                    el = el.expr("links", links);
                }
                NavPages::Dynamic(dynamic) => {
                    el = el.expr(
                        "dynamic",
                        ts::object([
                            ("fromView", Some(ts::string(&dynamic.from_view))),
                            ("page", Some(ts::string(&dynamic.page))),
                            ("param", Some(ts::string(&dynamic.param))),
                            ("label", expr(dynamic.label.as_ref())),
                            ("synonyms", Some(strings(&dynamic.synonyms))),
                            ("filter", expr(dynamic.filter.as_ref())),
                        ]),
                    );
                }
            }
            children.push(el.render());
        }
        let search = search.is_some() || doc.navigation.search.is_some();
        El::new(&navigation)
            .opt("search", search.then(|| "true".to_owned()))
            .opt(
                "collapsible",
                collapsible
                    .and_then(Value::as_bool)
                    .filter(|flag| *flag)
                    .map(|_| "true".to_owned()),
            )
            .expr("restrictions", restrictions)
            .children(children)
            .render()
    }

    /// `src/shells/<Shell>Shell.tsx`.
    #[allow(clippy::too_many_lines)] // state, guards, preload, regions and overlays of one shell
    pub(crate) fn shell(&mut self, name: &str, shell: &Shell) -> String {
        let owner = format!("{}Shell", ts::pascal(name));
        self.prefix.clone_from(&owner);
        self.components = BTreeSet::from([owner.clone()]);
        self.page_profile = None;
        self.page = None;
        let at = NodePath::root().child("shells").child(name);
        let mut lines = Vec::new();
        self.begin_component();
        let (values, setters) = self.state_hooks(&mut lines, &at, &shell.state, "shell");
        self.end_component();
        let mut frame_lines = Vec::new();
        let mut blocked = false;
        if shell.guards.is_empty() {
            frame_lines.push("const __refusal: string | undefined = undefined;".to_owned());
        } else {
            let use_guards = self.import("runtime/shell", "useGuards");
            let guards = ts::array(shell.guards.iter().map(|guard| {
                let (redirect, refuse, set) = match &guard.then {
                    ess_ui::GuardThen::Redirect(redirect) => (
                        Some(ts::object([
                            ("page", Some(ts::string(&redirect.redirect))),
                            (
                                "params",
                                (!redirect.params.is_empty()).then(|| exprs(&redirect.params)),
                            ),
                        ])),
                        None,
                        None,
                    ),
                    ess_ui::GuardThen::Refuse(refuse) => {
                        (None, Some(ts::string(&refuse.refuse)), None)
                    }
                    ess_ui::GuardThen::Set(set) => (None, None, Some(exprs(&set.set))),
                };
                ts::object([
                    ("name", Some(ts::string(&guard.name))),
                    ("when", Some(ts::string(&guard.when.0))),
                    ("redirect", redirect),
                    ("refuse", refuse),
                    ("set", set),
                ])
            }));
            frame_lines.push(format!("const __refusal = {use_guards}({guards});"));
            blocked = true;
        }
        if let Some(preload) = &shell.preload {
            let use_preload = self.import("runtime/shell", "usePreload");
            let views = ts::array(preload.views.iter().map(|view| {
                ts::object([
                    ("view", Some(ts::string(&view.view))),
                    (
                        "params",
                        (!view.params.is_empty()).then(|| exprs(&view.params)),
                    ),
                ])
            }));
            let policy = preload
                .policy
                .as_ref()
                .and_then(variant)
                .unwrap_or_else(|| "in_background".to_owned());
            frame_lines.push(format!(
                "const __preloaded = {use_preload}({views}, {}, {});",
                ts::string(&policy),
                strings(&preload.except_on)
            ));
            blocked = true;
        } else {
            frame_lines.push("const __preloaded = true;".to_owned());
        }
        let outlet = self.import("runtime/router", "Outlet");
        let outlet = if blocked {
            format!(
                "{{__refusal !== undefined || !__preloaded ? <p className=\"ui-state\">{{__refusal ?? \"Loading…\"}}</p> : <{outlet} />}}"
            )
        } else {
            format!("<{outlet} />")
        };
        let mut regions = Vec::new();
        for (region_name, region) in &shell.regions {
            regions.push(self.region(&at.child("regions").child(region_name), region, &outlet));
        }
        let shell_frame = self.import("runtime/shell", "ShellFrame");
        let framed = El::new(&shell_frame)
            .path(&at.to_string())
            .children(regions)
            .render();
        let frame_component = self.component_name(&format!("{owner}Frame"));
        self.hoist(&frame_component, &frame_lines, &framed);
        let hosted = self.overlay_host(
            &owner,
            &at,
            &shell.overlays,
            format!("<{frame_component} />"),
        );
        let scanned = format!("{}\n{hosted}", self.hoisted.join("\n"));
        let channels = self.channels_used(&scanned, &[]);
        let mut extra = Vec::new();
        if !channels.is_empty() && !self.bound {
            let use_channels = self.import("runtime/live", "useChannels");
            let sessions = self.sessions(&at, None, &channels, "{}");
            lines.push(format!(
                "const __channel = {use_channels}({}{sessions});",
                strings(&channels)
            ));
            extra.push(("channel".to_owned(), "__channel".to_owned()));
        }
        let rendered = self.scope_layer("shell", &values, &setters, &extra, hosted);
        self.import("react", "type ReactNode");
        let mut w = Writer::default();
        w.line(format!(
            "// Generated by ess-ui-react from shells/{name}. Do not edit."
        ));
        let imports = self.take_imports(1);
        for line in imports.lines() {
            w.line(line);
        }
        w.line("");
        w.open(format!("export function {owner}(): ReactNode {{"));
        for line in &lines {
            w.line(line);
        }
        w.line("return (");
        w.line(format!("  {}", indent(&rendered, 4)));
        w.line(");");
        w.close("}");
        for component in std::mem::take(&mut self.hoisted) {
            w.line("");
            for line in component.lines() {
                w.line(line);
            }
        }
        w.finish()
    }

    // ── app, routes, channels, model ─────────────────────────────────────────────────────

    /// `src/App.tsx`: the router around the app root, and `Pages`, which renders the page the
    /// path matches at its shell's outlet. Every page of one shell renders under the same shell
    /// element, so the shell stays mounted from one of its pages to the next; a page whose shell
    /// the document does not define renders directly. Any other path redirects home.
    pub(crate) fn app(&mut self) -> String {
        let doc = self.doc;
        let router = self.import("runtime/router", "Router");
        let matched = self.import("runtime/router", "Matched");
        let redirect = self.import("runtime/router", "Redirect");
        let use_location = self.import("runtime/router", "useLocation");
        let href = self.import("routes", "hrefFor");
        let match_page = self.import("routes", "matchPage");
        let root = self.import("runtime/core", "AppRoot");
        self.import("react", "type ReactNode");
        let actor = match doc.actor {
            Some(ess_ui::ActorSource::FromSession) => "from_session",
            _ => "anonymous",
        };
        let tree = El::new(&router)
            .child(
                El::new(&root)
                    .expr("actor", ts::string(actor))
                    .child("<Pages />")
                    .render(),
            )
            .render();
        let mut body = Writer::default();
        body.open("export function App(): ReactNode {");
        body.line("return (");
        body.line(format!("  {}", indent(&tree, 4)));
        body.line(");");
        body.close("}");
        body.line("");
        body.line(
            "/** The page the current path matches, at its shell's outlet; any other path goes home. */",
        );
        body.open("export function Pages(): ReactNode {");
        body.line(format!("const location = {use_location}();"));
        body.line(format!("const matched = {match_page}(location.pathname);"));
        body.open("if (matched === undefined) {");
        body.line(format!(
            "return <{redirect} to={{{href}({}, {{}})}} />;",
            ts::string(&doc.navigation.home)
        ));
        body.close("}");
        body.line("let page: ReactNode = null;");
        body.open("switch (matched.page) {");
        for name in doc.pages.keys() {
            let component = format!("{}Page", ts::pascal(name));
            self.import(&format!("pages/{}", ts::pascal(name)), &component);
            body.line(format!("case {}:", ts::string(name)));
            body.line(format!("  page = <{component} />;"));
            body.line("  break;");
        }
        body.close("}");
        let shells: BTreeSet<&str> = doc
            .pages
            .values()
            .map(|page| page.shell.as_str())
            .filter(|shell| doc.shells.contains_key(*shell))
            .collect();
        if !shells.is_empty() {
            let table = self.import("routes", "pages");
            body.open(format!("switch ({table}[matched.page]?.shell) {{"));
            for shell in shells {
                let component = format!("{}Shell", ts::pascal(shell));
                self.import(&format!("shells/{component}"), &component);
                body.line(format!("case {}:", ts::string(shell)));
                body.line(format!(
                    "  return <{matched} params={{matched.params}} outlet={{page}}><{component} /></{matched}>;"
                ));
            }
            body.close("}");
        }
        body.line(format!(
            "return <{matched} params={{matched.params}}>{{page}}</{matched}>;"
        ));
        body.close("}");
        let mut w = Writer::default();
        w.line("// Generated by ess-ui-react. Do not edit.");
        for line in self.take_imports(0).lines() {
            w.line(line);
        }
        w.line("");
        let mut text = w.finish();
        text.push_str(&body.finish());
        text
    }

    /// Every path pattern a page answers on — its own, then its aliases' — ordered so that at
    /// the first segment two patterns differ in, a static segment comes before a param. Two
    /// pages whose patterns have the same shape (the same static segments, case-insensitively,
    /// and params at the same places) would answer the same paths: that is a refusal.
    fn route_table(&mut self) -> Vec<(String, &'d str)> {
        let doc = self.doc;
        let mut shapes: BTreeMap<Vec<(bool, String)>, (String, &'d str)> = BTreeMap::new();
        for (name, page) in &doc.pages {
            let patterns = std::iter::once(name)
                .chain(&page.aliases)
                .map(|path| route_pattern(path, page));
            for pattern in patterns {
                let shape: Vec<(bool, String)> = pattern
                    .split('/')
                    .skip(1)
                    .map(|segment| {
                        if segment.starts_with(':') {
                            (true, String::new())
                        } else {
                            (false, segment.to_lowercase())
                        }
                    })
                    .collect();
                match shapes.get(&shape) {
                    Some((_, owner)) if *owner == name.as_str() => {}
                    Some((other, owner)) => self.errors.push(GenerateError::new(format!(
                        "pages/{name}: its route {pattern} answers the same paths as {other} of pages/{owner}"
                    ))),
                    None => {
                        shapes.insert(shape, (pattern, name.as_str()));
                    }
                }
            }
        }
        shapes.into_values().collect()
    }

    /// `src/routes.ts`.
    pub(crate) fn routes(&mut self) -> String {
        let doc = self.doc;
        let table = self.route_table();
        let mut w = Writer::default();
        w.line(
            "// Generated by ess-ui-react from the document's pages and navigation. Do not edit.",
        );
        w.line("");
        w.line("/** Where a page lives. */");
        w.line("export interface PageRoute {");
        w.line("  path: string;");
        w.line("  params: string[];");
        w.line("  shell: string;");
        w.line("  title: string;");
        w.line("  label: string;");
        w.line("}");
        w.line("");
        w.open("export const pages: Record<string, PageRoute> = {");
        for (name, page) in &doc.pages {
            w.line(format!(
                "{}: {},",
                ts::string(name),
                ts::object([
                    ("path", Some(ts::string(&route_pattern(name, page)))),
                    ("params", Some(strings(page.params.keys()))),
                    ("shell", Some(ts::string(&page.shell))),
                    (
                        "title",
                        Some(ts::string(page.title.as_deref().unwrap_or(name)))
                    ),
                    ("label", Some(ts::string(&nav_label(name, Some(page))))),
                ])
            ));
        }
        w.close("};");
        w.line("");
        w.line(format!(
            "export const routing = {};",
            ts::object([
                ("home", Some(ts::string(&doc.navigation.home))),
                ("hidden", Some(strings(&doc.navigation.hidden))),
            ])
        ));
        w.line("");
        w.line("/** The href of a page with its params filled in. */");
        w.open("export function hrefFor(page: string, params: Record<string, unknown>): string {");
        w.line("const route = pages[page];");
        w.open("if (!route) {");
        w.line("return `/${page.replace(/\\./g, \"/\")}`;");
        w.close("}");
        w.line("return route.path.replace(/:([A-Za-z0-9_]+)/g, (_, name: string) => encodeURIComponent(String(params[name] ?? \"\")));");
        w.close("}");
        w.line("");
        w.line("/** The page a path matched, and its path params, decoded. */");
        w.line("export interface PageMatch {");
        w.line("  page: string;");
        w.line("  params: Record<string, string>;");
        w.line("}");
        w.line("");
        w.line("/** Every pattern a page answers on, its aliases' too; a static segment before a param. */");
        w.open("const table: [pattern: string, page: string][] = [");
        for (pattern, page) in &table {
            w.line(format!("[{}, {}],", ts::string(pattern), ts::string(page)));
        }
        w.close("];");
        w.line("");
        for line in MATCH_PAGE.lines() {
            w.line(line);
        }
        w.finish()
    }

    /// `src/channels.ts`: the document's channels, emitted with the live runtime.
    pub(crate) fn channels(&self) -> String {
        let doc = self.doc;
        let mut w = Writer::default();
        w.line("// Generated by ess-ui-react from the document's channels. Do not edit.");
        w.line("// One-way channels use server-sent events, both-way channels a WebSocket; until");
        w.line("// `useNetworkChannels` is called every channel plays its fixture script.");
        w.line("import type { ChannelSpec } from \"./runtime/live\";");
        w.line("");
        w.open("export const channels: Record<string, ChannelSpec> = {");
        for (name, channel) in &doc.channels {
            let both = matches!(channel.direction, Direction::Both);
            let (events, view) = match &channel.carries {
                Carries::Events(events) => (events.events.clone(), None),
                Carries::View(view) => (Vec::new(), Some(ts::string(&view.view))),
            };
            let reconnect = channel.reconnect.as_ref().and_then(|value| {
                let backoff = value.get("backoff").unwrap_or(value);
                let from = backoff
                    .get("from")
                    .and_then(Value::as_str)
                    .and_then(millis)?;
                let to = backoff.get("to").and_then(Value::as_str).and_then(millis)?;
                Some(ts::object([
                    ("from", Some(from.to_string())),
                    ("to", Some(to.to_string())),
                ]))
            });
            let spec = ts::object([
                ("name", Some(ts::string(name))),
                (
                    "direction",
                    Some(ts::string(if both { "both" } else { "server_to_client" })),
                ),
                (
                    "transport",
                    Some(ts::string(if both { "websocket" } else { "sse" })),
                ),
                (
                    "delivery",
                    Some(ts::string(
                        &variant(&channel.delivery).unwrap_or_else(|| "every_event".to_owned()),
                    )),
                ),
                (
                    "resume",
                    Some(ts::string(
                        &variant(&channel.resume).unwrap_or_else(|| "refetch".to_owned()),
                    )),
                ),
                ("events", Some(strings(&events))),
                ("view", view),
                ("sends", Some(strings(&channel.sends))),
                ("scope", quoted(channel.scope.as_ref())),
                (
                    "sessionPer",
                    channel
                        .session
                        .as_ref()
                        .map(|session| ts::string(&session.per)),
                ),
                ("lifecycle", Some(strings(&channel.lifecycle))),
                ("reconnect", reconnect),
                ("staleAfter", duration(channel.stale_after.as_ref())),
                ("fields", Some(ts::string_map(&channel.fields))),
            ]);
            w.line(format!("{}: {spec},", ts::key(name)));
        }
        w.close("};");
        w.finish()
    }

    /// `src/model.ts`: the document's types, and every model type it names as `Json`.
    pub(crate) fn model(&mut self) -> String {
        let doc = self.doc;
        let mut w = Writer::default();
        w.line("// Generated by ess-ui-react from the document's types. Do not edit.");
        w.line("import type { Json } from \"./runtime/json\";");
        w.line("");
        w.line("export type { Json };");
        let mut defined = Vec::new();
        for (name, ty) in &doc.types {
            if !is_identifier(name) {
                continue;
            }
            let lowered = self.types.lower_local(ty);
            defined.push(format!("export type {name} = {lowered};"));
        }
        if !defined.is_empty() {
            w.line("");
            w.line("// Types the document defines.");
            for line in defined {
                w.line(line);
            }
        }
        let model: Vec<String> = self
            .types
            .model
            .iter()
            .filter(|name| is_identifier(name))
            .cloned()
            .collect();
        if !model.is_empty() {
            w.line("");
            w.line(
                "// Types of the ESS model the document names; their shape comes from the model.",
            );
            for name in model {
                w.line(format!("export type {name} = Json;"));
            }
        }
        w.finish()
    }
}

/// The accessible label of a choice: its own name, or the field or action it picks for.
fn choice_label(at: &NodePath) -> Option<String> {
    let segments = at.segments();
    match segments.split_last() {
        Some((last, rest)) if last == "choice" => rest.last().cloned(),
        Some((last, _)) => Some(last.clone()),
        None => None,
    }
}

fn is_identifier(name: &str) -> bool {
    name.chars()
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        && name != "Json"
}

/// An object literal with one entry per line, for values that are JSX.
fn multiline_object(entries: &[(String, Option<String>)]) -> String {
    let parts: Vec<String> = entries
        .iter()
        .filter_map(|(name, value)| {
            value
                .as_ref()
                .map(|value| format!("{}: {},", ts::key(name), indent(value, 2)))
        })
        .collect();
    if parts.is_empty() {
        return "{}".to_owned();
    }
    format!("{{\n  {}\n}}", parts.join("\n  "))
}
