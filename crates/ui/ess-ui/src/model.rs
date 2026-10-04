//! The typed `ess-ui/1` document, after expansion.
//!
//! Every struct refuses a key it does not declare. Ordered collections are `Vec`s of named
//! entries; a `BTreeMap` appears only where the schema says order carries no meaning. Composites
//! are one union, [`Composite`], read with `#[serde(tag = "component")]`; a section, an overlay
//! and a nested [`Node`] each hold one member beside their own frame fields, split apart by hand
//! so that a key belonging to neither is still refused.

use std::collections::BTreeMap;

use serde::de::Error as _;
use serde::{Deserialize, Deserializer};
use serde_yaml::{Mapping, Value};

/// The members of the composite union, in the order the schema lists them.
pub const COMPOSITE_KINDS: &[&str] = &[
    "collection",
    "record",
    "form",
    "choice",
    "filter_bar",
    "confirm",
    "metric",
    "chart",
    "board",
    "graph_editor",
    "rich_text",
    "references",
];

/// A binding expression. Written as a string; a bare number or boolean is read as its text.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Expr(pub String);

impl<'de> Deserialize<'de> for Expr {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        match Value::deserialize(deserializer)? {
            Value::String(text) => Ok(Self(text)),
            Value::Number(number) => Ok(Self(number.to_string())),
            Value::Bool(flag) => Ok(Self(flag.to_string())),
            other => Err(D::Error::custom(format!(
                "an expression is a scalar, not {other:?}"
            ))),
        }
    }
}

/// The schema's `unmapped_marker`: `UNMAPPED: <reason>`, accepted by any field whatever its
/// declared type. Every enum of this model carries it as its `Unmapped` variant; a validator
/// reports it as a warning and a renderer treats the field as absent.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct UnmappedMarker(pub String);

impl UnmappedMarker {
    /// The reason after `UNMAPPED: `.
    pub fn reason(&self) -> &str {
        self.0.strip_prefix(UNMAPPED_PREFIX).unwrap_or_default()
    }
}

const UNMAPPED_PREFIX: &str = "UNMAPPED: ";

impl<'de> Deserialize<'de> for UnmappedMarker {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        match text.strip_prefix(UNMAPPED_PREFIX) {
            Some(reason) if !reason.is_empty() => Ok(Self(text)),
            _ => Err(D::Error::custom(format!(
                "`{text}` is neither a value of this field nor an `UNMAPPED: <reason>` marker"
            ))),
        }
    }
}

/// A structural type expression (`type_rule` in the schema). Never a string to be parsed.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum TypeExpr {
    /// A primitive name (`string`) or a capitalized named type (`PartnerId`).
    Named(String),
    /// `{list: T, unique?}`.
    List(ListType),
    /// `{map: {key: K, value: V}}`.
    Map(MapType),
    /// `{optional: T}`.
    Optional(OptionalType),
    /// `{enum: [a, b]}`.
    Enum(EnumType),
    /// `{one_of: [T1, T2]}`.
    OneOf(OneOfType),
    /// `{record: {field: T}}`.
    Record(RecordType),
    /// `{ref: kind}`.
    Ref(RefType),
    /// `{const: value}`.
    Const(ConstType),
}

/// `{list: T}`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ListType {
    /// The element type.
    pub list: Box<TypeExpr>,
    /// No duplicate elements: a set.
    #[serde(default)]
    pub unique: bool,
}

/// `{map: {key: K, value: V}}`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MapType {
    /// Key and value types.
    pub map: MapEntryType,
}

/// The key and value of a map type.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MapEntryType {
    /// Key type.
    pub key: Box<TypeExpr>,
    /// Value type.
    pub value: Box<TypeExpr>,
}

/// `{optional: T}`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OptionalType {
    /// The type that may be absent.
    pub optional: Box<TypeExpr>,
}

/// `{enum: [a, b]}`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnumType {
    /// The allowed values, in order.
    #[serde(rename = "enum")]
    pub values: Vec<String>,
}

/// `{one_of: [T1, T2]}`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OneOfType {
    /// The alternatives.
    pub one_of: Vec<TypeExpr>,
}

/// `{record: {field: T}}`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordType {
    /// Field types by field name.
    pub record: BTreeMap<String, TypeExpr>,
}

/// `{ref: kind}`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RefType {
    /// What the name resolves to: page, overlay, view, command …
    #[serde(rename = "ref")]
    pub kind: String,
}

/// `{const: value}`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConstType {
    /// The only allowed value.
    #[serde(rename = "const")]
    pub value: Value,
}

// ── document ─────────────────────────────────────────────────────────────────────────────────

/// The root of an `ess-ui/1` document, after expansion.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Document {
    /// Always `ess-ui/1`.
    pub format: String,
    /// Root of every fully qualified name.
    pub app: String,
    /// Human title of the application.
    pub title: Option<String>,
    /// The ESS system views, commands and events resolve against.
    pub model: String,
    /// Whose grants decide what is visible.
    pub actor: Option<ActorSource>,
    /// Default state placement for the whole document.
    pub placement_profile: PlacementProfile,
    /// Overrides of the profile's default store per state class.
    #[serde(default)]
    pub placement_defaults: BTreeMap<StateClass, Store>,
    /// Application frames.
    pub shells: BTreeMap<String, Shell>,
    /// Menu structure and home page.
    pub navigation: Navigation,
    /// App-defined page templates, resolved against the kinds they extend.
    #[serde(default)]
    pub page_kinds: BTreeMap<String, PageKind>,
    /// App-defined composites, as declared (instances carry the expanded body).
    #[serde(default)]
    pub widgets: BTreeMap<String, Widget>,
    /// Named value types.
    #[serde(default)]
    pub types: BTreeMap<String, TypeExpr>,
    /// Every route of the app, with its page kind merged in.
    pub pages: BTreeMap<String, Page>,
    /// Live data sources.
    #[serde(default)]
    pub channels: BTreeMap<String, Channel>,
    /// Sample data so renderers run without a backend.
    pub fixtures: Option<FixtureIndex>,
    /// Document-level gaps found by a retrofit.
    #[serde(default)]
    pub unmapped: Vec<String>,
    /// Design tokens, merged over the built-in table ([`Document::base_tokens`]).
    #[serde(default)]
    pub tokens: Tokens,
    /// Named looks, each the tokens it overrides ([`Document::theme_tokens`]).
    #[serde(default)]
    pub themes: BTreeMap<String, Tokens>,
    /// Which theme is shown, and the shell state that chooses it.
    pub theme: Option<ThemeChoice>,
    /// Value-to-tone maps, named by `tone_by.tones`.
    #[serde(default)]
    pub tone_maps: BTreeMap<String, ToneMap>,
    /// Every name a choice's `options` write that resolved to an enum of the model the document
    /// was loaded with ([`crate::load_str_with`]), to its qualified name (beyond10x/ess#330).
    /// Never authored: empty for a document loaded without a model.
    #[serde(skip)]
    pub model_enums: BTreeMap<String, String>,
}

/// Whose grants decide what is visible.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActorSource {
    /// The signed-in actor.
    FromSession,
    /// Nobody is signed in.
    Anonymous,
    /// A value a retrofit could not determine (`unmapped_marker`).
    #[serde(untagged)]
    Unmapped(UnmappedMarker),
}

/// A placement profile.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlacementProfile {
    /// The server holds all state but the URL.
    Thin,
    /// The client holds UI state.
    Fat,
    /// Each page or section picks.
    Hybrid,
    /// A value a retrofit could not determine (`unmapped_marker`).
    #[serde(untagged)]
    Unmapped(UnmappedMarker),
}

/// A page or section profile in a hybrid document.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Profile {
    /// Server-held.
    Thin,
    /// Client-held.
    Fat,
    /// A value a retrofit could not determine (`unmapped_marker`).
    #[serde(untagged)]
    Unmapped(UnmappedMarker),
}

// ── navigation ───────────────────────────────────────────────────────────────────────────────

/// An application frame.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Shell {
    /// Named areas of the frame.
    pub regions: BTreeMap<String, Region>,
    /// Views read when the shell starts.
    pub preload: Option<Preload>,
    /// Checks run, in order, before each page of the shell opens.
    #[serde(default)]
    pub guards: Vec<Guard>,
    /// Shell-wide state.
    #[serde(default)]
    pub state: BTreeMap<String, State>,
    /// Overlays reachable from every page.
    #[serde(default)]
    pub overlays: BTreeMap<String, Overlay>,
    /// Traceability to retrofitted files.
    #[serde(default)]
    pub source: Vec<String>,
}

/// One named area of a shell.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Region {
    /// Role of the region.
    pub kind: RegionKind,
    /// Kind-specific options.
    #[serde(default)]
    pub props: BTreeMap<String, Value>,
    /// Shows the region only when true.
    pub visible: Option<Expr>,
}

/// The role of a shell region.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RegionKind {
    /// The menu.
    Navigation,
    /// Where the page renders.
    PageOutlet,
    /// Where drawers and dialogs appear.
    OverlayOutlet,
    /// Toasts and banners.
    Notifications,
    /// A chat helper.
    Assistant,
    /// The account menu.
    AccountMenu,
    /// A value a retrofit could not determine (`unmapped_marker`).
    #[serde(untagged)]
    Unmapped(UnmappedMarker),
}

/// Views a shell reads before or alongside the first page.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Preload {
    /// Views to read.
    pub views: Vec<PreloadView>,
    /// Whether the first page waits.
    pub policy: Option<PreloadPolicy>,
    /// Pages that skip the preload.
    #[serde(default)]
    pub except_on: Vec<String>,
}

/// One preloaded view.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreloadView {
    /// The view.
    pub view: String,
    /// Its params.
    #[serde(default)]
    pub params: BTreeMap<String, Expr>,
    /// Traceability.
    pub endpoint: Option<String>,
}

/// Whether the first page waits for a preload.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PreloadPolicy {
    /// Block the first page.
    BeforeFirstPage,
    /// Read alongside.
    InBackground,
    /// Read when first needed.
    OnDemand,
    /// A value a retrofit could not determine (`unmapped_marker`).
    #[serde(untagged)]
    Unmapped(UnmappedMarker),
}

/// A condition checked before a page opens.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Guard {
    /// Node name among the shell's guards.
    pub name: String,
    /// Condition over actor, url and page.
    pub when: Expr,
    /// What happens when it holds.
    pub then: GuardThen,
}

/// The outcome of a guard.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum GuardThen {
    /// Redirect to a page.
    Redirect(GuardRedirect),
    /// Refuse with a message.
    Refuse(GuardRefuse),
    /// Set shell state.
    Set(GuardSet),
}

/// Redirect to a page.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GuardRedirect {
    /// The page.
    pub redirect: String,
    /// Its params.
    #[serde(default)]
    pub params: BTreeMap<String, Expr>,
}

/// Refuse with a message.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GuardRefuse {
    /// The message.
    pub refuse: String,
}

/// Set shell state.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GuardSet {
    /// Target expression to value expression.
    pub set: BTreeMap<String, Expr>,
}

/// The menu.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Navigation {
    /// Page opened at the app root.
    pub home: String,
    /// Menu groups in display order.
    pub sections: Vec<NavSection>,
    /// Routed pages not shown in the menu.
    #[serde(default)]
    pub hidden: Vec<String>,
    /// How the menu decides what to show.
    pub visibility: Option<NavVisibility>,
    /// Menu search.
    pub search: Option<NavSearch>,
    /// When a condition holds only these sections show.
    #[serde(default)]
    pub restrictions: Vec<NavRestriction>,
}

/// How the menu decides what to show.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NavVisibility {
    /// From actor grants.
    ByActorGrants,
    /// From role lists.
    ByRoles,
    /// A value a retrofit could not determine (`unmapped_marker`).
    #[serde(untagged)]
    Unmapped(UnmappedMarker),
}

/// Menu search fields.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NavSearch {
    /// What the search matches: `label`, `synonyms`.
    pub over: Vec<String>,
}

/// A navigation restriction.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NavRestriction {
    /// The condition.
    pub when: Expr,
    /// The only sections shown while it holds.
    pub sections: Vec<String>,
    /// Gaps found by a retrofit.
    #[serde(default)]
    pub unmapped: Vec<String>,
}

/// One group of the menu.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NavSection {
    /// Node name.
    pub name: String,
    /// Heading text.
    pub label: Option<String>,
    /// Semantic icon name.
    pub icon: Option<String>,
    /// Fixed pages or entries from a view.
    pub pages: NavPages,
}

/// The pages of a menu group.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum NavPages {
    /// Fixed pages, in order.
    Fixed(Vec<String>),
    /// One entry per row of a view.
    Dynamic(DynamicNavEntries),
}

/// Menu entries generated from the rows of a view.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DynamicNavEntries {
    /// One entry per row.
    pub from_view: String,
    /// Page each entry opens.
    pub page: String,
    /// Page param filled from the row id.
    pub param: String,
    /// Entry text.
    pub label: Option<Expr>,
    /// Extra search words.
    #[serde(default)]
    pub synonyms: Vec<String>,
    /// Rows kept when the predicate is true.
    pub filter: Option<Expr>,
}

/// How a page appears in the menu.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NavEntry {
    /// Menu text (the page title when absent).
    pub label: Option<String>,
    /// Extra words the menu search matches.
    #[serde(default)]
    pub synonyms: Vec<String>,
    /// Traceability only.
    #[serde(default)]
    pub roles_today: Vec<String>,
}

// ── pages ────────────────────────────────────────────────────────────────────────────────────

/// One route, with its page kind merged in.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Page {
    /// Template the page starts from.
    pub kind: String,
    /// Frame the page renders in.
    pub shell: String,
    /// Header title.
    pub title: Option<String>,
    /// Menu label and synonyms.
    pub nav: Option<NavEntry>,
    /// Path parameters.
    #[serde(default)]
    pub params: BTreeMap<String, TypeExpr>,
    /// Legacy route paths.
    #[serde(default)]
    pub aliases: Vec<String>,
    /// Sibling pages offered in the header.
    #[serde(default)]
    pub switch_to: Vec<String>,
    /// Extra condition beyond grants.
    pub visible: Option<Expr>,
    /// How sections are arranged.
    pub layout: PageLayout,
    /// Page state.
    #[serde(default)]
    pub state: BTreeMap<String, State>,
    /// Title, total, actions, live status.
    pub header: Option<Header>,
    /// Regions of the page, in order.
    pub sections: Vec<Section>,
    /// Drawers and dialogs of the page.
    #[serde(default)]
    pub overlays: BTreeMap<String, Overlay>,
    /// Placement for this page in a hybrid document.
    pub profile: Option<Profile>,
    /// Files the page was retrofitted from.
    #[serde(default)]
    pub source: Vec<String>,
    /// Gaps the retrofit could not resolve.
    #[serde(default)]
    pub unmapped: Vec<String>,
}

/// A page template, merged over the kinds it extends.
///
/// A kind is a template, not a page: its overlays and sections are partial (a create drawer
/// with no command yet), completed only by the page that uses it. So its inherited fields stay
/// the YAML they were merged into, and are read as types in every [`Page`] of the kind.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PageKind {
    /// Kind it starts from.
    pub extends: Option<String>,
    /// When to use it.
    pub purpose: Option<String>,
    /// State every page of the kind has.
    pub state: Option<Value>,
    /// Sections every page of the kind has, in order.
    pub sections: Option<Value>,
    /// Overlays every page of the kind has.
    pub overlays: Option<Value>,
    /// Default header.
    pub header: Option<Value>,
    /// Default layout.
    pub layout: Option<Value>,
}

/// A renderer-neutral arrangement of a page's sections.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum PageLayout {
    /// `stack`: sections top to bottom in declaration order.
    Stack(StackLayout),
    /// Named columns left to right.
    Columns(ColumnsLayout),
    /// A grid of area names.
    Areas(AreasLayout),
}

/// The literal `stack`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StackLayout {
    /// Sections one below the other.
    Stack,
}

/// `{columns: [...]}`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ColumnsLayout {
    /// Named columns left to right.
    pub columns: Vec<LayoutColumn>,
}

/// One layout column.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LayoutColumn {
    /// Node name.
    pub name: String,
    /// Sections in the column, top to bottom.
    pub sections: Vec<String>,
    /// Relative width.
    pub width: Option<ColumnWidth>,
}

/// Relative width of a layout column.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ColumnWidth {
    /// Narrow.
    Narrow,
    /// Normal.
    Normal,
    /// Wide.
    Wide,
    /// A value a retrofit could not determine (`unmapped_marker`).
    #[serde(untagged)]
    Unmapped(UnmappedMarker),
}

/// `{areas: {grid, place}}`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AreasLayout {
    /// The grid and its placements.
    pub areas: Areas,
}

/// A grid of area names and the sections placed in each.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Areas {
    /// Rows of area names.
    pub grid: Vec<Vec<String>>,
    /// Sections placed in each area.
    pub place: BTreeMap<String, Vec<String>>,
}

/// Fields every node frame shares: a composite, a widget instance or a primitive.
#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NodeCommon {
    /// Node name; required inside lists, the key inside maps.
    pub name: Option<String>,
    /// State local to this node.
    #[serde(default)]
    pub state: BTreeMap<String, State>,
    /// Shows the node only when true.
    pub visible: Option<Expr>,
    /// Fallbacks for renderers lacking a capability.
    #[serde(default)]
    pub degrades: BTreeMap<String, String>,
    /// Gaps found by a retrofit.
    #[serde(default)]
    pub unmapped: Vec<String>,
}

const NODE_COMMON_KEYS: &[&str] = &["name", "state", "visible", "degrades", "unmapped"];

/// A region of a page with one read and its own loading lifecycle.
#[derive(Debug, Clone, PartialEq)]
pub struct Section {
    /// Node name among the page's sections.
    pub name: String,
    /// The heading shown above the section, where it has one (beyond10x/ess#281).
    pub title: Option<String>,
    /// State, visibility, degrades and unmapped notes of the section.
    pub common: NodeCommon,
    /// How channel events change the rows.
    pub live: Option<Live>,
    /// When the read starts.
    pub load: Option<Load>,
    /// Sibling whose selection the params use.
    pub depends_on: Option<String>,
    /// How each lifecycle state renders.
    pub states: Option<SectionStates>,
    /// Extra named nodes rendered after the composite.
    pub children: Vec<Node>,
    /// Placement for this section in a hybrid document.
    pub profile: Option<Profile>,
    /// What the section renders: a composite or a widget instance.
    pub body: Body,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SectionFrame {
    name: String,
    title: Option<String>,
    #[serde(default)]
    state: BTreeMap<String, State>,
    visible: Option<Expr>,
    #[serde(default)]
    degrades: BTreeMap<String, String>,
    #[serde(default)]
    unmapped: Vec<String>,
    live: Option<Live>,
    load: Option<Load>,
    depends_on: Option<String>,
    states: Option<SectionStates>,
    #[serde(default)]
    children: Vec<Node>,
    profile: Option<Profile>,
}

const SECTION_FRAME_KEYS: &[&str] = &[
    "name",
    "title",
    "state",
    "visible",
    "degrades",
    "unmapped",
    "live",
    "load",
    "depends_on",
    "states",
    "children",
    "profile",
];

impl<'de> Deserialize<'de> for Section {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let mapping = Mapping::deserialize(deserializer)?;
        let (frame, rest) = split(mapping, SECTION_FRAME_KEYS);
        let frame: SectionFrame = from_mapping(frame).map_err(D::Error::custom)?;
        let body = Body::from_mapping(rest, false).map_err(D::Error::custom)?;
        // A `live` block without `match` matches events by the read's `key` (#320).
        let mut live = frame.live;
        if let Some(live) = live.as_mut() {
            if live.match_field.is_none() {
                live.match_field = body.reads().and_then(|reads| reads.key.clone());
            }
        }
        Ok(Self {
            name: frame.name,
            title: frame.title,
            common: NodeCommon {
                name: None,
                state: frame.state,
                visible: frame.visible,
                degrades: frame.degrades,
                unmapped: frame.unmapped,
            },
            live,
            load: frame.load,
            depends_on: frame.depends_on,
            states: frame.states,
            children: frame.children,
            profile: frame.profile,
            body,
        })
    }
}

impl Section {
    /// The field this section's rows are keyed by, where the document names one: its read's
    /// `key`, else its `live.match`. A renderer keys by `id` when this is `None`.
    pub fn row_key(&self) -> Option<&str> {
        self.body
            .reads()
            .and_then(|reads| reads.key.as_deref())
            .or_else(|| self.live.as_ref()?.match_field.as_deref())
    }
}

/// When a section's read starts.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Load {
    /// At once.
    Eager,
    /// When scrolled into view.
    OnVisible,
    /// When asked for.
    OnDemand,
    /// A value a retrofit could not determine (`unmapped_marker`).
    #[serde(untagged)]
    Unmapped(UnmappedMarker),
}

/// How a section renders each lifecycle state.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SectionStates {
    /// First load.
    pub loading: Option<LoadingStyle>,
    /// Later loads.
    pub refreshing: Option<RefreshingStyle>,
    /// Ready with no rows.
    pub empty: Option<EmptyState>,
    /// Read refused or transport error.
    pub failed: Option<FailedState>,
    /// Actor lacks the grant.
    pub forbidden: Option<ForbiddenState>,
    /// Live channel down.
    pub stale: Option<StaleState>,
}

/// How the first load renders.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LoadingStyle {
    /// A skeleton.
    Skeleton,
    /// A spinner.
    Spinner,
    /// Nothing.
    None,
    /// A value a retrofit could not determine (`unmapped_marker`).
    #[serde(untagged)]
    Unmapped(UnmappedMarker),
}

/// How later loads render.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RefreshingStyle {
    /// Keep the current rows.
    KeepRows,
    /// Show a skeleton.
    Skeleton,
    /// A value a retrofit could not determine (`unmapped_marker`).
    #[serde(untagged)]
    Unmapped(UnmappedMarker),
}

/// What an empty section shows.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EmptyState {
    /// The message.
    pub message: String,
    /// An action offered.
    pub action: Option<Action>,
}

/// What a failed section shows.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FailedState {
    /// The message.
    pub message: Option<String>,
    /// Whether a retry is offered.
    pub retry: bool,
}

/// What a forbidden section shows.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ForbiddenState {
    /// The message.
    pub message: Option<String>,
}

/// How a stale section is marked.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StaleState {
    /// The mark.
    pub mark: StaleMark,
}

/// A stale mark.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StaleMark {
    /// A badge.
    Badge,
    /// Dimmed.
    Dim,
    /// Both.
    Both,
    /// A value a retrofit could not determine (`unmapped_marker`).
    #[serde(untagged)]
    Unmapped(UnmappedMarker),
}

// ── nodes ────────────────────────────────────────────────────────────────────────────────────

/// Anything that can appear inside a section, composite or widget.
#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    /// Name, state, visibility, degrades and unmapped notes.
    pub common: NodeCommon,
    /// A composite, a widget instance or a primitive.
    pub body: Body,
}

impl<'de> Deserialize<'de> for Node {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let mapping = Mapping::deserialize(deserializer)?;
        let (frame, rest) = split(mapping, NODE_COMMON_KEYS);
        let common: NodeCommon = from_mapping(frame).map_err(D::Error::custom)?;
        let body = Body::from_mapping(rest, true).map_err(D::Error::custom)?;
        Ok(Self { common, body })
    }
}

/// What a node, section or overlay holds.
#[derive(Debug, Clone, PartialEq)]
pub enum Body {
    /// A member of the composite union.
    Composite(Composite),
    /// A use of an app-defined widget.
    Widget(WidgetUse),
    /// One of the nine primitives.
    Primitive(Primitive),
}

impl Body {
    /// The read of a composite body, where it has one; a widget's reads are in its expanded body.
    pub fn reads(&self) -> Option<&Reads> {
        match self {
            Self::Composite(composite) => composite.reads(),
            Self::Widget(_) | Self::Primitive(_) => None,
        }
    }

    fn from_mapping(rest: Mapping, allow_primitive: bool) -> Result<Self, String> {
        let component = rest.get("component").cloned();
        let has_primitive = rest.contains_key("primitive");
        match (component, has_primitive) {
            (Some(_), true) => Err("a node has exactly one of `component` and `primitive`".into()),
            (None, true) if allow_primitive => from_mapping::<Primitive>(rest).map(Self::Primitive),
            (None, true) => Err("a primitive cannot stand here; name a `component`".into()),
            (None, false) => Err("a node needs `component` or `primitive`".into()),
            (Some(Value::String(kind)), false) => {
                if COMPOSITE_KINDS.contains(&kind.as_str()) {
                    from_mapping::<Composite>(rest)
                        .map(Self::Composite)
                        .map_err(|message| format!("`{kind}`: {message}"))
                } else {
                    from_mapping::<WidgetUse>(rest)
                        .map(Self::Widget)
                        .map_err(|message| format!("widget `{kind}`: {message}"))
                }
            }
            (Some(other), false) => Err(format!("`component` is a name, not {other:?}")),
        }
    }
}

/// The composite union, discriminated by `component`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(tag = "component", rename_all = "snake_case")]
#[allow(clippy::large_enum_variant)] // a document holds dozens of these, not millions
pub enum Composite {
    /// Rows of a view.
    Collection(Collection),
    /// One row as read-only fields.
    Record(Record),
    /// Inputs bound to one command's input.
    Form(Form),
    /// Pick one or many values.
    Choice(Choice),
    /// Search, choices and a window bound to page state.
    FilterBar(FilterBar),
    /// A yes/no step before a command.
    Confirm(Confirm),
    /// One number.
    Metric(Metric),
    /// A series drawn as a chart.
    Chart(Chart),
    /// A user-arranged grid of widgets.
    Board(Board),
    /// Nodes and edges on a canvas.
    GraphEditor(GraphEditor),
    /// Text with expression completion.
    RichText(RichText),
    /// The "used by" list of a record.
    References(References),
}

impl Composite {
    /// The view this composite reads, where it reads one.
    pub fn reads(&self) -> Option<&Reads> {
        match self {
            Self::Collection(c) => c.reads.as_ref(),
            Self::Record(c) => c.reads.as_ref(),
            Self::Metric(c) => c.reads.as_ref(),
            Self::Chart(c) => Some(&c.reads),
            Self::Board(c) => Some(&c.reads),
            Self::GraphEditor(c) => Some(&c.reads),
            Self::References(c) => Some(&c.reads),
            Self::Form(_)
            | Self::Choice(_)
            | Self::FilterBar(_)
            | Self::Confirm(_)
            | Self::RichText(_) => None,
        }
    }
}

/// Rows of a view with columns, sorting, paging, selection and row actions.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Collection {
    /// The rows.
    pub reads: Option<Reads>,
    /// Fixed or user-selectable columns.
    pub columns: Option<Columns>,
    /// Default and allowed sort.
    pub sort: Option<Sort>,
    /// Presentation hint.
    pub style: Option<CollectionStyle>,
    /// Row selection.
    pub selection: Option<Selection>,
    /// Actions per row.
    #[serde(default)]
    pub row_actions: Vec<Action>,
    /// Actions on the selection.
    #[serde(default)]
    pub bulk_actions: Vec<Action>,
    /// Actions on the whole collection.
    #[serde(default)]
    pub actions: Vec<Action>,
    /// Detail shown when a row expands.
    pub expand: Option<Box<Node>>,
    /// Nested named nodes per row, in order.
    #[serde(default)]
    pub item: Vec<Node>,
    /// Drag to reorder.
    pub reorder: Option<Reorder>,
    /// Field rows are grouped under.
    pub group_by: Option<String>,
    /// The order groups are shown in; values not listed follow, in the order they first appear.
    #[serde(default)]
    pub group_order: Vec<String>,
    /// Shows a heading for every `group_order` value, even one no row falls under.
    #[serde(default)]
    pub show_empty_groups: bool,
}

/// The columns of a collection.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum Columns {
    /// Fixed columns, in order.
    Fixed(Vec<Field>),
    /// User-selectable columns.
    Selectable(SelectableColumns),
    /// An UNMAPPED marker.
    Unmapped(String),
}

/// User-selectable columns.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelectableColumns {
    /// State holding the chosen columns.
    pub binds: Expr,
    /// Every column offered, in order.
    pub all: Vec<Field>,
}

/// Sort of a collection.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sort {
    /// Default sort field.
    pub by: String,
    /// Default direction.
    pub dir: Option<SortDir>,
    /// Fields the user may sort by.
    #[serde(default)]
    pub allowed: Vec<String>,
    /// Who sorts.
    pub mode: Option<SortMode>,
}

/// A sort direction.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SortDir {
    /// Ascending.
    Asc,
    /// Descending.
    Desc,
    /// A value a retrofit could not determine (`unmapped_marker`).
    #[serde(untagged)]
    Unmapped(UnmappedMarker),
}

/// Who sorts.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SortMode {
    /// The view.
    Server,
    /// The renderer.
    Client,
    /// A value a retrofit could not determine (`unmapped_marker`).
    #[serde(untagged)]
    Unmapped(UnmappedMarker),
}

/// Collection presentation hint.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CollectionStyle {
    /// A table.
    Table,
    /// Cards.
    Cards,
    /// A list.
    List,
    /// A tree.
    Tree,
    /// A value a retrofit could not determine (`unmapped_marker`).
    #[serde(untagged)]
    Unmapped(UnmappedMarker),
}

/// Row selection.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum Selection {
    /// Always this mode.
    Mode(SelectionMode),
    /// Only while a condition holds.
    Conditional(ConditionalSelection),
}

/// A selection mode.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SelectionMode {
    /// No selection.
    None,
    /// One row.
    Single,
    /// Many rows.
    Multiple,
    /// A value a retrofit could not determine (`unmapped_marker`).
    #[serde(untagged)]
    Unmapped(UnmappedMarker),
}

/// Selection enabled by a condition.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConditionalSelection {
    /// The mode.
    pub mode: SelectionMode,
    /// When it is enabled.
    pub enabled: Expr,
}

/// Drag to reorder.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Reorder {
    /// Command that saves the order.
    pub does: String,
    /// Traceability.
    pub endpoint: Option<String>,
}

/// One row shown as labelled, read-only fields.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    /// The row.
    pub reads: Option<Reads>,
    /// Fields to show, in order.
    #[serde(default)]
    pub fields: Vec<Field>,
    /// Fields split into tabs.
    #[serde(default)]
    pub tabs: Vec<Tab>,
    /// Nested named nodes, in order.
    #[serde(default)]
    pub item: Vec<Node>,
    /// Actions on the record.
    #[serde(default)]
    pub actions: Vec<Action>,
}

/// Inputs bound to one ESS command's input.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Form {
    /// Command the form submits.
    pub does: String,
    /// Initial values for edit forms.
    pub loads: Option<Reads>,
    /// Inputs shown, in order.
    #[serde(default)]
    pub fields: Vec<Field>,
    /// Independently saved groups.
    #[serde(default)]
    pub groups: Vec<FormGroup>,
    /// Fields split into tabs.
    #[serde(default)]
    pub tabs: Vec<Tab>,
    /// Nested named nodes, in order.
    #[serde(default)]
    pub parts: Vec<Node>,
    /// Extra actions next to submit.
    #[serde(default)]
    pub actions: Vec<Action>,
    /// Shows the command's outcome.
    pub result: Option<Box<Node>>,
    /// Read-only fields above the inputs.
    pub record: Option<Box<Node>>,
    /// The submit button.
    pub submit: Option<Submit>,
    /// The unsaved input.
    pub draft: Option<State>,
    /// When the command runs.
    pub save: Option<FormSave>,
    /// One generated form per value of a field.
    pub variant_by: Option<VariantBy>,
    /// Traceability.
    pub endpoint: Option<String>,
}

/// The submit button of a form.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Submit {
    /// Button text.
    pub label: Option<String>,
    /// Whether submitting closes the overlay.
    pub closes: Option<bool>,
    /// Submit on open.
    pub auto: Option<bool>,
}

/// When a form's command runs.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FormSave {
    /// On submit.
    OnSubmit,
    /// On every change.
    OnChange,
    /// When an input loses focus.
    OnBlur,
    /// A value a retrofit could not determine (`unmapped_marker`).
    #[serde(untagged)]
    Unmapped(UnmappedMarker),
}

/// One generated form per value of a field.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VariantBy {
    /// The field.
    pub field: String,
    /// Form per value.
    pub forms: BTreeMap<String, String>,
}

/// Pick one or many values from a view or a fixed list.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Choice {
    /// Options from a view.
    pub reads: Option<Reads>,
    /// Fixed options, in order.
    #[serde(default)]
    pub options: Vec<ChoiceOption>,
    /// The row field each option of `reads` sends (beyond10x/ess#328).
    pub value: Option<String>,
    /// The row field each option of `reads` shows (beyond10x/ess#328).
    pub label: Option<String>,
    /// State the value is written to.
    pub binds: Option<Expr>,
    /// Many values.
    #[serde(default)]
    pub multiple: bool,
    /// Presentation hint.
    pub style: Option<ChoiceStyle>,
    /// Typed values create new options.
    pub creatable: Option<Creatable>,
    /// Author remark.
    pub note: Option<String>,
}

impl Choice {
    /// The row field the author names as each option's value: `value`, else the read's `key`.
    pub fn value_field(&self) -> Option<&str> {
        self.value
            .as_deref()
            .or_else(|| self.reads.as_ref()?.key.as_deref())
    }

    /// The option a row of `reads` offers: the value it sends and the value its label shows.
    ///
    /// The value is the row's [`Self::value_field`] when the author names one, and a row without
    /// that field offers no option, so no other value can be sent in its place. Otherwise it is,
    /// in order: the row's field named `field` (the form field the choice picks for), its field
    /// named `identity` (the identity of the entity the view projects, from the binding), its
    /// `id`, and last the row itself; a field present as `null` is present, and `null` is sent.
    /// The label is the row's `label` field when the author names one and the row holds a value
    /// there; otherwise the row's `label`, else its `name`, else the value, a `null` counting as
    /// absent.
    pub fn row_option(
        &self,
        row: &Value,
        field: Option<&str>,
        identity: Option<&str>,
    ) -> Option<(Value, Value)> {
        let value = match self.value_field() {
            Some(named) => row.get(named)?.clone(),
            None => field
                .and_then(|field| row.get(field))
                .or_else(|| identity.and_then(|identity| row.get(identity)))
                .or_else(|| row.get("id"))
                .unwrap_or(row)
                .clone(),
        };
        // A label field present as `null` is absent.
        let present = |key: &str| row.get(key).filter(|cell| !cell.is_null());
        let label = self
            .label
            .as_deref()
            .and_then(present)
            .or_else(|| present("label"))
            .or_else(|| present("name"))
            .cloned()
            .unwrap_or_else(|| value.clone());
        Some((value, label))
    }
}

/// One fixed option.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChoiceOption {
    /// The value written.
    pub value: Value,
    /// The text shown.
    pub label: String,
}

/// Choice presentation hint.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChoiceStyle {
    /// A dropdown.
    Dropdown,
    /// Tags.
    Tags,
    /// A tree.
    Tree,
    /// Grouped.
    Grouped,
    /// Radio buttons.
    Radio,
    /// A segmented control.
    Segmented,
    /// A checklist.
    Checklist,
    /// A value a retrofit could not determine (`unmapped_marker`).
    #[serde(untagged)]
    Unmapped(UnmappedMarker),
}

/// Typed values create new options.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Creatable {
    /// The creating command.
    pub does: String,
}

/// Search, choices, inputs and a time window bound to page state.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FilterBar {
    /// State the bar writes.
    pub binds: Vec<Expr>,
    /// Free-text search.
    pub search: Option<FilterSearch>,
    /// Date or date-time range.
    pub window: Option<FilterWindow>,
    /// Choice composites, named, in order.
    #[serde(default)]
    pub choices: Vec<Node>,
    /// Free inputs, in order.
    #[serde(default)]
    pub inputs: Vec<Field>,
    /// Actions next to the filters.
    #[serde(default)]
    pub actions: Vec<Action>,
    /// Offers a reset-all button.
    #[serde(default)]
    pub reset: bool,
}

/// Free-text search of a filter bar.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FilterSearch {
    /// State written.
    pub binds: Expr,
    /// Hint text.
    pub placeholder: Option<String>,
}

/// Time window of a filter bar.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FilterWindow {
    /// State written.
    pub binds: Expr,
    /// Date-time rather than date.
    pub time: Option<bool>,
}

/// Page title, count, primary actions, view switch, filters and live status.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Header {
    /// Header title (`from_page` expanded on pages).
    pub title: Option<String>,
    /// Section whose total is shown.
    pub total: Option<String>,
    /// Primary actions, in order.
    #[serde(default)]
    pub actions: Vec<Action>,
    /// Pages or modes offered as a switch.
    #[serde(default)]
    pub switch: Vec<String>,
    /// Filter bar rendered in the header.
    pub filters: Option<String>,
    /// Channels whose lifecycle is shown.
    #[serde(default)]
    pub live: Vec<String>,
    /// Headline metrics, named, in order.
    #[serde(default)]
    pub metrics: Vec<Node>,
    /// Help text or link.
    pub help: Option<Help>,
}

/// Help text or link.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Help {
    /// Text.
    pub text: Option<String>,
    /// Link.
    pub link: Option<String>,
}

/// A drawer, dialog, fullscreen pane or popover: a frame around one union member or widget.
#[derive(Debug, Clone, PartialEq)]
pub struct Overlay {
    /// Presentation hint.
    pub kind: OverlayKind,
    /// Overlay title; a confirm shows it as its question.
    pub title: Option<String>,
    /// Values passed by the opener.
    pub params: BTreeMap<String, Expr>,
    /// Overlay-local state, degrades and unmapped notes.
    pub common: NodeCommon,
    /// The overlay this one was copied from, kept for traceability.
    pub same_as: Option<String>,
    /// What the overlay renders.
    pub body: Body,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OverlayFrame {
    kind: OverlayKind,
    title: Option<String>,
    #[serde(default)]
    params: BTreeMap<String, Expr>,
    #[serde(default)]
    state: BTreeMap<String, State>,
    visible: Option<Expr>,
    #[serde(default)]
    degrades: BTreeMap<String, String>,
    #[serde(default)]
    unmapped: Vec<String>,
    same_as: Option<String>,
}

const OVERLAY_FRAME_KEYS: &[&str] = &[
    "kind", "title", "params", "state", "visible", "degrades", "unmapped", "same_as",
];

impl<'de> Deserialize<'de> for Overlay {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let mapping = Mapping::deserialize(deserializer)?;
        let (frame, rest) = split(mapping, OVERLAY_FRAME_KEYS);
        let frame: OverlayFrame = from_mapping(frame).map_err(D::Error::custom)?;
        let body = Body::from_mapping(rest, false).map_err(D::Error::custom)?;
        Ok(Self {
            kind: frame.kind,
            title: frame.title,
            params: frame.params,
            common: NodeCommon {
                name: None,
                state: frame.state,
                visible: frame.visible,
                degrades: frame.degrades,
                unmapped: frame.unmapped,
            },
            same_as: frame.same_as,
            body,
        })
    }
}

/// Presentation of an overlay.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OverlayKind {
    /// A side drawer.
    Drawer,
    /// A modal dialog.
    Dialog,
    /// A full-screen pane.
    Fullscreen,
    /// A popover.
    Popover,
    /// A value a retrofit could not determine (`unmapped_marker`).
    #[serde(untagged)]
    Unmapped(UnmappedMarker),
}

/// A yes/no step before a command.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Confirm {
    /// Explanation.
    pub body: Option<String>,
    /// Command run on confirm.
    pub does: Option<String>,
    /// The used-by view shown first.
    pub references: Option<String>,
    /// What the command will do.
    #[serde(default)]
    pub consequences: Vec<String>,
    /// Confirm button text.
    pub confirm_label: Option<String>,
    /// Styles the confirm as destructive.
    pub danger: Option<bool>,
    /// Type-to-confirm.
    pub input: Option<ConfirmInput>,
    /// Other actions offered.
    #[serde(default)]
    pub alternatives: Vec<Action>,
    /// Traceability.
    pub endpoint: Option<String>,
}

/// Type-to-confirm.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfirmInput {
    /// Prompt.
    pub label: String,
    /// The value that must be typed.
    pub must_equal: Option<Expr>,
}

/// One number from a view or channel.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Metric {
    /// Value from a view.
    pub reads: Option<Reads>,
    /// Value from a channel field.
    pub from: Option<Expr>,
    /// Row field that picks a keyed value.
    #[serde(rename = "match")]
    pub match_field: Option<String>,
    /// Time window the value covers.
    pub window: Option<String>,
    /// Display format.
    pub format: Option<MetricFormat>,
    /// Caption.
    pub label: Option<String>,
    /// Computes the value over every row of the read instead of reading one.
    pub aggregate: Option<MetricAggregate>,
    /// The row field `aggregate` reads; `count` needs none.
    pub field: Option<String>,
}

/// What a metric computes over the rows of its read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetricAggregate {
    /// The number of rows.
    Count,
    /// The sum of a field.
    Sum,
    /// The least value of a field.
    Min,
    /// The greatest value of a field.
    Max,
    /// The mean of a field.
    Avg,
}

impl MetricAggregate {
    /// The aggregate as the document spells it.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Count => "count",
            Self::Sum => "sum",
            Self::Min => "min",
            Self::Max => "max",
            Self::Avg => "avg",
        }
    }
}

/// Metric display format.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetricFormat {
    /// A number.
    Number,
    /// A duration.
    Duration,
    /// A percentage.
    Percent,
    /// A byte count.
    Bytes,
    /// A value a retrofit could not determine (`unmapped_marker`).
    #[serde(untagged)]
    Unmapped(UnmappedMarker),
}

/// A series drawn as a chart.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Chart {
    /// The series.
    pub reads: Reads,
    /// Chart kind, fixed or chosen by data.
    pub chart: ChartKind,
    /// X-axis field.
    pub x: Option<String>,
    /// Y fields.
    #[serde(default)]
    pub series: Vec<String>,
}

/// A chart kind.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum ChartKind {
    /// A fixed kind: line, bar, doughnut, pie, polar, `single_number`, list, table.
    Fixed(String),
    /// Chosen by state.
    Chosen(ChosenChart),
}

/// A chart kind chosen by state.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChosenChart {
    /// State holding the kind.
    pub binds: Expr,
    /// Kinds offered.
    pub options: Vec<String>,
}

/// A user-arranged grid of widgets.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Board {
    /// The dashboard record.
    pub reads: Reads,
    /// Row field choosing the widget kind.
    pub widget_by: Option<String>,
    /// Node per widget kind (unordered: rows decide placement).
    #[serde(default)]
    pub widgets: BTreeMap<String, Node>,
    /// Actions on each placed widget.
    #[serde(default)]
    pub item_actions: Vec<Action>,
    /// How the arrangement is saved.
    pub layout: Option<BoardLayout>,
}

/// How a board's arrangement is saved.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BoardLayout {
    /// Command saving it.
    pub persisted_by: String,
    /// Command allowed to edit it.
    pub editable_by: Option<String>,
    /// State holding it.
    pub state: Option<Expr>,
}

/// Nodes and edges of a model, editable on a canvas.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphEditor {
    /// The graph: its nodes, and its edges too unless `edges.reads` names their own view.
    pub reads: Reads,
    /// Node kind field and edit overlay.
    pub nodes: Option<GraphNodes>,
    /// Edge endpoints and kind.
    pub edges: Option<GraphEdges>,
    /// Context menu of a node.
    #[serde(default)]
    pub node_actions: Vec<Action>,
    /// Context menu of an edge.
    #[serde(default)]
    pub edge_actions: Vec<Action>,
    /// Named nodes above the canvas, left to right.
    #[serde(default)]
    pub toolbar: Vec<Node>,
}

/// Graph node settings.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphNodes {
    /// Field identifying a node, which edge endpoints name (`id` when absent).
    pub key: Option<String>,
    /// Field a node is labelled by (`label`, `name`, then the key when absent).
    pub label: Option<String>,
    /// Field choosing the node kind.
    pub kind_by: Option<String>,
    /// Edit overlay.
    pub opens: Option<String>,
}

/// Graph edge settings.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphEdges {
    /// The edges, from a view of their own; absent, they are rows of the graph's `reads`.
    pub reads: Option<Reads>,
    /// Source field.
    pub from: String,
    /// Target field.
    pub to: String,
    /// Field choosing the edge kind.
    pub kind_by: Option<String>,
}

/// Text with expression completion or markup.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RichText {
    /// View of completable expressions.
    pub completes: Option<String>,
    /// Language of the text.
    pub syntax: Option<RichTextSyntax>,
    /// Draft field the text is written to.
    pub binds: Option<Expr>,
}

/// Language of rich text.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RichTextSyntax {
    /// Plain text.
    Plain,
    /// Text with `{{expression}}`.
    Expression,
    /// Speech markup.
    Ssml,
    /// JSON.
    Json,
    /// A curl command.
    Curl,
    /// A value a retrofit could not determine (`unmapped_marker`).
    #[serde(untagged)]
    Unmapped(UnmappedMarker),
}

/// The "used by" list of a record.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct References {
    /// The referencing records.
    pub reads: Reads,
    /// Columns shown, in order.
    #[serde(default)]
    pub columns: Vec<Field>,
    /// Rows link to the referencing record.
    pub navigates: Option<bool>,
}

/// An app-defined composite with typed parameters.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Widget {
    /// One line shown in pickers and docs.
    pub summary: String,
    /// Longer description.
    pub doc: Option<String>,
    /// Typed parameters.
    #[serde(default)]
    pub params: BTreeMap<String, WidgetParam>,
    /// How body nodes are arranged.
    pub arrange: Option<Arrange>,
    /// Named nodes, in order; each may use `args`.
    pub body: Vec<Node>,
}

/// One widget parameter.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WidgetParam {
    /// Its type.
    #[serde(rename = "type")]
    pub ty: TypeExpr,
    /// Whether a use must supply it.
    #[serde(default)]
    pub required: bool,
    /// Value used when absent.
    pub default: Option<Value>,
    /// What it is.
    pub note: String,
}

/// How a widget's body is arranged.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Arrange {
    /// Left to right.
    Row,
    /// Top to bottom.
    Column,
    /// A grid.
    Grid,
    /// A value a retrofit could not determine (`unmapped_marker`).
    #[serde(untagged)]
    Unmapped(UnmappedMarker),
}

/// A use of a widget, with arguments and the expanded body.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WidgetUse {
    /// The widget used.
    pub component: String,
    /// One entry per param.
    #[serde(default)]
    pub args: BTreeMap<String, Value>,
    /// The widget body with args substituted; empty inside a widget's own declaration.
    #[serde(default)]
    pub body: Vec<Node>,
}

// ── primitives ───────────────────────────────────────────────────────────────────────────────

/// The closed set of nine renderer-neutral leaves, discriminated by `primitive`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(tag = "primitive", rename_all = "snake_case")]
pub enum Primitive {
    /// A run of text.
    Text(Text),
    /// A toned pill.
    Badge(Badge),
    /// A semantic icon.
    Icon(Icon),
    /// A button running one action.
    Button(Button),
    /// A link.
    Link(Link),
    /// A single free input.
    Input(Input),
    /// An on/off switch.
    Toggle(Toggle),
    /// An image.
    Image(Image),
    /// A separator.
    Divider(Divider),
}

/// A run of text.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Text {
    /// Literal or expression.
    pub text: Option<Expr>,
    /// Row field shown.
    pub field: Option<String>,
    /// Typographic role.
    pub style: Option<TextStyle>,
    /// Value formatting.
    pub format: Option<TextFormat>,
    /// Currency code for format currency.
    pub currency: Option<Expr>,
}

/// Typographic role.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextStyle {
    /// Body text.
    Body,
    /// A caption.
    Caption,
    /// A heading.
    Heading,
    /// Monospace.
    Mono,
    /// A value a retrofit could not determine (`unmapped_marker`).
    #[serde(untagged)]
    Unmapped(UnmappedMarker),
}

/// Value formatting of text.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextFormat {
    /// As is.
    Plain,
    /// A number.
    Number,
    /// Money.
    Currency,
    /// A percentage.
    Percent,
    /// A date.
    Date,
    /// A time.
    Time,
    /// A duration.
    Duration,
    /// A value a retrofit could not determine (`unmapped_marker`).
    #[serde(untagged)]
    Unmapped(UnmappedMarker),
}

/// A colour role.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Tone {
    /// Neutral.
    Neutral,
    /// Informational.
    Info,
    /// Success.
    Success,
    /// Warning.
    Warning,
    /// Danger.
    Danger,
    /// A value a retrofit could not determine (`unmapped_marker`).
    #[serde(untagged)]
    Unmapped(UnmappedMarker),
}

/// A toned pill.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Badge {
    /// Literal or expression.
    pub text: Option<Expr>,
    /// Row field shown.
    pub field: Option<String>,
    /// Fixed tone.
    pub tone: Option<Tone>,
    /// Tone per value.
    pub tone_by: Option<ToneBy>,
}

/// Tone per value.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToneBy {
    /// The value.
    pub value: Expr,
    /// Value to tone: written as `map:`, or the `tone_maps` entry the loader resolved `tones:`
    /// to; an expression inside a widget declaration, a map once expanded.
    pub map: Value,
    /// The `tone_maps` entry `map` was resolved from, when it was written as `tones:` (the
    /// unbound `args.<param>` inside a widget declaration). No renderer needs to read it.
    pub tones: Option<String>,
}

/// A semantic icon.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Icon {
    /// Semantic icon name.
    pub icon: String,
    /// Colour role.
    pub tone: Option<Tone>,
    /// Tone per value, as on a badge.
    pub tone_by: Option<ToneBy>,
    /// Accessible text.
    pub label: String,
}

/// A button that runs one action.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Button {
    /// Button text.
    pub label: String,
    /// What it does.
    pub action: Action,
    /// Emphasis.
    pub tone: Option<ButtonTone>,
    /// Semantic icon.
    pub icon: Option<String>,
}

/// Emphasis of a button.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ButtonTone {
    /// Primary.
    Primary,
    /// Secondary.
    Secondary,
    /// Destructive.
    Danger,
    /// Borderless.
    Ghost,
    /// A value a retrofit could not determine (`unmapped_marker`).
    #[serde(untagged)]
    Unmapped(UnmappedMarker),
}

/// Text that navigates.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Link {
    /// Link text.
    pub text: Expr,
    /// Page to open.
    pub to: Option<Navigate>,
    /// External address.
    pub href: Option<Expr>,
}

/// A single free input.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Input {
    /// Input type: text, number, search, date, time, secret.
    #[serde(rename = "as")]
    pub input_as: Option<String>,
    /// State or draft field written.
    pub binds: Expr,
    /// Hint text.
    pub placeholder: Option<String>,
}

/// An on/off switch.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Toggle {
    /// Switch text.
    pub label: String,
    /// Boolean written.
    pub binds: Option<Expr>,
    /// Command run on change.
    pub action: Option<Action>,
}

/// An image.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Image {
    /// Image address.
    pub src: Expr,
    /// Alternative text.
    pub alt: String,
    /// Scaling: cover or contain.
    pub fit: Option<String>,
}

/// A visual separator.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Divider {
    /// Direction: horizontal or vertical.
    pub orientation: Option<String>,
}

// ── fields, tabs, groups ─────────────────────────────────────────────────────────────────────

/// One input of a form or one column of a collection.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Field {
    /// Field of the command input or view row.
    pub field: String,
    /// Node name (the field when not written).
    pub name: String,
    /// Caption.
    pub label: Option<String>,
    /// Semantic widget.
    #[serde(rename = "as")]
    pub field_as: Option<String>,
    /// The choice composite for `as: choice`.
    pub choice: Option<Box<Node>>,
    /// Column can sort.
    #[serde(default)]
    pub sortable: bool,
    /// Shows the field only when true.
    pub visible: Option<Expr>,
    /// Bind to UI state instead of the command input.
    pub binds: Option<Expr>,
    /// Shows a field of a related view, matched by this field's value, instead of the value.
    pub label_from: Option<LabelFrom>,
    /// Author remark.
    pub note: Option<String>,
}

/// A field of a related view shown in place of a key (`Field.label_from`).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LabelFrom {
    /// The related view, read once without params.
    pub view: String,
    /// The field of its row that is shown.
    pub field: String,
    /// The field of its row the key matches; `id` when absent.
    pub key: Option<String>,
}

impl LabelFrom {
    /// The field of the related row the key matches.
    pub fn key(&self) -> &str {
        self.key.as_deref().unwrap_or("id")
    }
}

/// One tab of a form or record.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tab {
    /// Node name.
    pub name: String,
    /// Tab text.
    pub label: Option<String>,
    /// Shows the tab only when true.
    pub visible: Option<Expr>,
    /// Fields in the tab, or an UNMAPPED string.
    pub fields: Option<TabFields>,
    /// Nested node or an action.
    pub form: Option<TabForm>,
}

/// Fields of a tab.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum TabFields {
    /// Fields, in order.
    Fields(Vec<Field>),
    /// An UNMAPPED marker.
    Unmapped(String),
}

/// The nested content of a tab.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum TabForm {
    /// A nested node.
    Node(Box<Node>),
    /// An action.
    Action(Box<Action>),
}

/// A group of settings fields saved on its own.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FormGroup {
    /// Node name.
    pub name: String,
    /// Group heading.
    pub label: Option<String>,
    /// Fields in the group, in order.
    pub fields: Vec<Field>,
    /// When the group saves: `with_form` or `on_change`.
    pub save: Option<String>,
    /// Command for this group if not the form's.
    pub does: Option<String>,
    /// Extra actions.
    #[serde(default)]
    pub actions: Vec<Action>,
    /// Traceability.
    pub endpoint: Option<String>,
}

// ── data ─────────────────────────────────────────────────────────────────────────────────────

/// The ESS view a section or composite reads, or a placeholder backed by a fixture.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Reads {
    /// Client row predicate, applied after live effects and before local paging. Never authorization.
    pub filter: Option<Expr>,
    /// ESS view name.
    pub view: Option<String>,
    /// A view name not yet bound to the model.
    pub placeholder: Option<String>,
    /// Fixture file answering the placeholder.
    pub fixture: Option<String>,
    /// The field that identifies a row: rows, row paths and row actions are keyed by it, and a
    /// `live` block without `match` matches events by it. Absent: the section's `live.match`,
    /// else `id` (beyond10x/ess#320).
    pub key: Option<String>,
    /// View params bound to state.
    #[serde(default)]
    pub params: BTreeMap<String, Expr>,
    /// Who pages.
    pub paging: Option<Paging>,
    /// Coalesce param changes before reading.
    pub debounce: Option<String>,
    /// Poll interval.
    pub refresh: Option<Expr>,
    /// Where the result is cached.
    pub cache: Option<ReadCache>,
    /// Traceability to the HTTP call replaced.
    pub endpoint: Option<String>,
    /// Traceability when data is computed client-side today.
    pub derived: Option<String>,
}

/// Who pages a read.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Paging {
    /// The view.
    Server,
    /// The renderer, over a bounded view.
    Client,
    /// A cursor.
    Cursor,
    /// Append on scroll.
    Append,
    /// No paging.
    None,
    /// A value a retrofit could not determine (`unmapped_marker`).
    #[serde(untagged)]
    Unmapped(UnmappedMarker),
}

/// Where a read result is cached.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReadCache {
    /// The store.
    pub store: Store,
    /// Expiry.
    pub ttl: Option<String>,
}

/// One user-triggered effect.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
#[allow(clippy::struct_excessive_bools)]
pub struct Action {
    /// Node name among sibling actions (derived when not written).
    pub name: String,
    /// ESS command to run.
    pub does: Option<String>,
    /// Command input from row, selection or state.
    #[serde(default)]
    pub bind: BTreeMap<String, Expr>,
    /// Overlay to open.
    pub opens: Option<String>,
    /// Page to go to.
    pub navigate: Option<Navigate>,
    /// Download a view.
    pub export: Option<Export>,
    /// File picker feeding a command.
    pub upload: Option<Upload>,
    /// Value copied to the clipboard.
    pub copy: Option<Expr>,
    /// UI state change only (`toggle` expanded to `not <key>`).
    #[serde(default)]
    pub sets: BTreeMap<String, Expr>,
    /// Button or menu text.
    pub label: Option<String>,
    /// Presentation hint.
    #[serde(rename = "as")]
    pub action_as: Option<ActionAs>,
    /// Options for `as: choice`.
    pub choice: Option<Box<Node>>,
    /// Current value for a header toggle or choice.
    pub loads: Option<Reads>,
    /// Confirm first.
    pub confirm: Option<ActionConfirm>,
    /// Apply the expected outcome at once.
    #[serde(default)]
    pub optimistic: bool,
    /// Applies to the collection's selection.
    #[serde(default)]
    pub bulk: bool,
    /// UI condition beyond grants.
    pub visible: Option<Expr>,
    /// Traceability.
    pub endpoint: Option<String>,
}

/// Presentation hint of an action.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionAs {
    /// A button.
    Button,
    /// An icon button.
    Icon,
    /// A toggle.
    Toggle,
    /// A choice.
    Choice,
    /// A menu item.
    MenuItem,
    /// A link.
    Link,
    /// A value a retrofit could not determine (`unmapped_marker`).
    #[serde(untagged)]
    Unmapped(UnmappedMarker),
}

/// Page navigation.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Navigate {
    /// The page.
    pub to: String,
    /// Its params.
    #[serde(default)]
    pub params: BTreeMap<String, Expr>,
}

/// Download a view.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Export {
    /// The view.
    pub reads: String,
    /// File format: csv, json, zip, png.
    #[serde(rename = "as")]
    pub export_as: String,
    /// Params.
    pub params: Option<Expr>,
}

/// File picker feeding a command.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Upload {
    /// Accepted media types.
    pub accept: Vec<String>,
    /// The command.
    pub does: String,
}

/// Confirm before an action runs.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum ActionConfirm {
    /// An overlay of the page.
    Opens(String),
    /// An inline dialog, expanded from `{title: …}`.
    Inline(InlineConfirm),
}

/// An inline confirm dialog.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InlineConfirm {
    /// The dialog.
    pub overlay: Box<Overlay>,
    /// Only confirm when this holds.
    pub show: Option<Expr>,
}

/// Sample data per view and event scripts per channel.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FixtureIndex {
    /// Fixture directory relative to the document.
    pub dir: Option<String>,
    /// File holding views, derived and scripts.
    pub index: Option<String>,
    /// View to fixture file.
    #[serde(default)]
    pub views: BTreeMap<String, String>,
    /// Views answered from another fixture.
    #[serde(default)]
    pub derived: BTreeMap<String, Value>,
    /// Channel to event script file.
    #[serde(default)]
    pub scripts: BTreeMap<String, String>,
}

// ── live ─────────────────────────────────────────────────────────────────────────────────────

/// A live source of ESS events or a live view.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Channel {
    /// Events (deltas) or a live view.
    pub carries: Carries,
    /// Direction.
    pub direction: Direction,
    /// Commands sent over the channel.
    #[serde(default)]
    pub sends: Vec<String>,
    /// Whether values may be dropped.
    pub delivery: Delivery,
    /// Behaviour after reconnect.
    pub resume: Resume,
    /// Which events reach the actor.
    pub scope: Option<String>,
    /// One channel instance per value.
    pub session: Option<ChannelSession>,
    /// Connection states shown to the user.
    #[serde(default)]
    pub lifecycle: Vec<String>,
    /// Reconnect backoff.
    pub reconnect: Option<Value>,
    /// Down time after which fed sections are stale.
    pub stale_after: Option<String>,
    /// Unapplied events and last-seen cursor.
    pub buffer: Option<State>,
    /// Named fields readable as `channel.<name>.<field>`.
    #[serde(default)]
    pub fields: BTreeMap<String, String>,
    /// Traceability only.
    pub transport_today: Option<String>,
    /// Traceability.
    #[serde(default)]
    pub source: Vec<String>,
    /// Gaps found by a retrofit.
    #[serde(default)]
    pub unmapped: Vec<String>,
}

/// What a channel carries.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum Carries {
    /// ESS events.
    Events(CarriesEvents),
    /// A live view.
    View(CarriesView),
}

/// ESS events.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CarriesEvents {
    /// The events.
    pub events: Vec<String>,
}

/// A live view.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CarriesView {
    /// The view.
    pub view: String,
}

/// One channel instance per value.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChannelSession {
    /// The param.
    pub per: String,
}

/// Channel direction.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    /// Server to client only.
    ServerToClient,
    /// Both ways.
    Both,
    /// A value a retrofit could not determine (`unmapped_marker`).
    #[serde(untagged)]
    Unmapped(UnmappedMarker),
}

/// Channel delivery.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Delivery {
    /// Intermediate values may be dropped.
    LatestValue,
    /// No event may be dropped.
    EveryEvent,
    /// A value a retrofit could not determine (`unmapped_marker`).
    #[serde(untagged)]
    Unmapped(UnmappedMarker),
}

/// Behaviour after a reconnect.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Resume {
    /// Replay from the last seen event.
    FromLastSeen,
    /// Read again.
    Refetch,
    /// A value a retrofit could not determine (`unmapped_marker`).
    #[serde(untagged)]
    Unmapped(UnmappedMarker),
}

/// How a section applies a channel's events.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Live {
    /// Channel to consume.
    pub channel: String,
    /// Subset of the channel's events.
    #[serde(default)]
    pub on: Vec<String>,
    /// What an event does to the rows.
    pub effect: Effect,
    /// Row identity field; the loader fills it from the read's `key` when absent.
    #[serde(rename = "match")]
    pub match_field: Option<String>,
    /// Drop events that fail the condition.
    pub only_if: Option<Expr>,
    /// Batch bursts into one render.
    pub coalesce: Option<String>,
    /// Behaviour when the reader is not on page one.
    pub when_paged_away: Option<PagedAway>,
    /// Cap for `insert_top` feeds.
    pub max_rows: Option<u32>,
}

/// What a live event does to rows.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Effect {
    /// Patch a row.
    PatchRow,
    /// Insert or patch.
    InsertOrPatch,
    /// Insert at the top of a feed.
    InsertTop,
    /// Remove a row.
    RemoveRow,
    /// Replace the value.
    Replace,
    /// Read again.
    Refetch,
    /// A value a retrofit could not determine (`unmapped_marker`).
    #[serde(untagged)]
    Unmapped(UnmappedMarker),
}

/// Behaviour when paged away.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PagedAway {
    /// Show a count of new rows.
    CountNew,
    /// Ignore.
    Ignore,
    /// Insert anyway.
    Insert,
    /// A value a retrofit could not determine (`unmapped_marker`).
    #[serde(untagged)]
    Unmapped(UnmappedMarker),
}

// ── state ────────────────────────────────────────────────────────────────────────────────────

/// One piece of UI state, its class and where it is stored.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct State {
    /// Structural type of the value.
    #[serde(rename = "type")]
    pub ty: TypeExpr,
    /// What kind of state this is.
    pub class: StateClass,
    /// Explicit placement.
    pub store: Option<Store>,
    /// No profile may move it.
    #[serde(default)]
    pub pinned: bool,
    /// Initial value.
    pub default: Option<Value>,
    /// Sharing.
    pub scope: Option<String>,
    /// Forbids url, `session_storage` and `local_storage`.
    #[serde(default)]
    pub sensitive: bool,
    /// Expiry.
    pub ttl: Option<String>,
    /// Events that clear it.
    #[serde(default)]
    pub clear_on: Vec<String>,
    /// Second placement used when the condition holds.
    pub fallback: Option<StateFallback>,
    /// Traceability for store server.
    pub endpoint: Option<String>,
    /// Author remark.
    pub note: Option<String>,
}

/// A second placement.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StateFallback {
    /// The condition.
    pub when: Expr,
    /// The store.
    pub store: Store,
}

/// The kind of a state.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StateClass {
    /// What a link should reproduce.
    PageState,
    /// Rows or a record the user picked.
    Selection,
    /// Open/closed, expanded, active tab.
    ComponentState,
    /// Unsaved form input.
    Draft,
    /// The last result of a read.
    ViewCache,
    /// Events not yet applied.
    ChannelBuffer,
    /// Per-user choices.
    Preference,
    /// Tokens.
    Credential,
    /// A value a retrofit could not determine (`unmapped_marker`).
    #[serde(untagged)]
    Unmapped(UnmappedMarker),
}

/// Where a state lives.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Store {
    /// The component instance.
    Memory,
    /// Path or query.
    Url,
    /// Browser sessionStorage.
    SessionStorage,
    /// Browser localStorage.
    LocalStorage,
    /// Server, per login session.
    ServerSession,
    /// An ESS entity or view.
    Server,
    /// A value a retrofit could not determine (`unmapped_marker`).
    #[serde(untagged)]
    Unmapped(UnmappedMarker),
}

// ── style ────────────────────────────────────────────────────────────────────────────────────

/// One token value as written: a color, a length or a weight. It is read as its text, so that a
/// value outside its group's grammar is a finding of `ess ui check` (`token_values`,
/// `theme_tokens`) rather than a load error. A bare number (`0`, `600`) is read as its text, and
/// an empty value as the empty string: an unquoted `#` starts a YAML comment, so `surface: #fff`
/// is empty.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TokenValue(pub String);

impl<'de> Deserialize<'de> for TokenValue {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        match Value::deserialize(deserializer)? {
            Value::String(text) => Ok(Self(text)),
            Value::Number(number) => Ok(Self(number.to_string())),
            Value::Bool(flag) => Ok(Self(flag.to_string())),
            Value::Null => Ok(Self(String::new())),
            other => Err(D::Error::custom(format!(
                "a token value is a scalar, not {other:?}"
            ))),
        }
    }
}

/// Design tokens in five groups, each a map of name to value: a document's `tokens:`, a theme's
/// overrides, and the built-in table ([`Tokens::builtin`]) both are merged over.
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tokens {
    /// Color literals by name.
    #[serde(default)]
    pub color: BTreeMap<String, TokenValue>,
    /// Lengths by name.
    #[serde(default)]
    pub space: BTreeMap<String, TokenValue>,
    /// Lengths by name.
    #[serde(default)]
    pub radius: BTreeMap<String, TokenValue>,
    /// `type`: family, size and weight per text style.
    #[serde(default, rename = "type")]
    pub typography: BTreeMap<String, TypeToken>,
    /// The text and fill color names per tone.
    #[serde(default)]
    pub tone: BTreeMap<String, ToneToken>,
}

/// The type of one text style.
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TypeToken {
    /// Font family list.
    pub family: Option<String>,
    /// A length.
    pub size: Option<TokenValue>,
    /// 100 to 900 in steps of 100.
    pub weight: Option<TokenValue>,
}

/// The colors one tone is drawn with, each the name of a color token.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToneToken {
    /// Text color.
    pub text: String,
    /// Fill color.
    pub fill: String,
}

/// Which theme is shown.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ThemeChoice {
    /// The theme shown where no state chooses one.
    pub default: String,
    /// The shell state (`shell.<name>`) whose value names the theme shown.
    pub chosen_by: Option<Expr>,
}

/// A value-to-tone map, declared once in `tone_maps` and named by `tone_by.tones`.
pub type ToneMap = BTreeMap<String, Tone>;

// ── helpers ──────────────────────────────────────────────────────────────────────────────────

/// Splits a mapping into the entries whose key is in `keys` and the rest, keeping order.
fn split(mapping: Mapping, keys: &[&str]) -> (Mapping, Mapping) {
    let mut frame = Mapping::new();
    let mut rest = Mapping::new();
    for (key, value) in mapping {
        if key.as_str().is_some_and(|key| keys.contains(&key)) {
            frame.insert(key, value);
        } else {
            rest.insert(key, value);
        }
    }
    (frame, rest)
}

fn from_mapping<T: serde::de::DeserializeOwned>(mapping: Mapping) -> Result<T, String> {
    serde_yaml::from_value(Value::Mapping(mapping)).map_err(|error| error.to_string())
}
