//! The checks run on a loaded document, walking [`ess_ui::Document::nodes`] so every finding
//! carries the node's canonical path.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use ess_ui::{
    Action, ActionAs, ActionConfirm, Body, Composite, Document, Export, ExportParams, FixtureIndex,
    GuardThen, Header, Located, NavPages, NodeCommon, NodePath, NodeRef, OverlayKind, PageLayout,
    Pager, Paging, PlacementProfile, Primitive, Profile, Reads, RegionKind, State, StateClass,
    Store, TabFields, TypeExpr, WidgetUse,
};
use ess_yaml::Value;

use crate::expr;
use crate::schema::{self, Capability};
use crate::walk::{body_of, composite_kind, composite_reads, in_declaration, page_of, section_of};
use crate::{Options, Sink};

/// How one layout column or area frames its sections, as written.
struct Framing<'a> {
    frame: Option<&'a str>,
    title: Option<&'a str>,
    collapsed: Option<bool>,
    sections: &'a [String],
}

/// `frame_rules`: `title` and `collapsed` belong to a shared frame, `collapsed` needs the `title`
/// a reader clicks, and a shared frame holds sections that are consecutive in the page's section
/// order, since a renderer that stacks sections draws one block at the first one's place. A
/// `frame` outside `each` and `shared` is `enum_values`' to report.
fn frame_rules(sink: &mut Sink, at: &NodePath, order: &[&str], framing: &Framing<'_>) {
    match framing.frame {
        Some("shared") => {}
        None | Some("each") => {
            for (key, written) in [
                ("title", framing.title.is_some()),
                ("collapsed", framing.collapsed.is_some()),
            ] {
                if written {
                    sink.push(
                        "frame_rules",
                        at,
                        format!(
                            "`{key}` is the heading or the state of a shared frame, and these \
                             sections are `frame: each`: write `frame: shared`, or drop `{key}`"
                        ),
                    );
                }
            }
            return;
        }
        Some(_) => return,
    }
    if framing.collapsed == Some(true) && framing.title.is_none_or(|title| title.trim().is_empty())
    {
        sink.push(
            "frame_rules",
            at,
            "`collapsed` needs a `title`: a collapsed frame is its title, which a reader clicks \
             to open it",
        );
    }
    let mut previous: Option<(&str, usize)> = None;
    for section in framing.sections {
        let Some(index) = order.iter().position(|name| *name == section) else {
            continue;
        };
        if let Some((before, last)) = previous {
            if index != last + 1 {
                sink.push(
                    "frame_rules",
                    at,
                    format!(
                        "section `{section}` does not follow `{before}` in the page's section \
                         order, and a shared frame holds consecutive sections"
                    ),
                );
                return;
            }
        }
        previous = Some((section, index));
    }
}

/// `frame_rules`: a collapsed group is its `label`, which a reader clicks to open it, so a
/// collapsed group without one starts closed with nothing to click.
fn collapsed_group_has_label(
    sink: &mut Sink,
    at: &NodePath,
    collapsed: Option<bool>,
    label: Option<&str>,
) {
    if collapsed == Some(true) && label.is_none_or(|label| label.trim().is_empty()) {
        sink.push(
            "frame_rules",
            at,
            "`collapsed` needs a `label`: a collapsed group is its label, which a reader clicks \
             to open it",
        );
    }
}

pub(crate) fn run(document: &Document, base: &Path, options: &Options, sink: &mut Sink) {
    let located = document.nodes();
    let fixtures = Fixtures::read(document, base, sink);
    let checker = Checker {
        document,
        fixtures: &fixtures,
    };
    checker.navigation(sink);
    checker.pages(sink);
    checker.shells(sink);
    account_menu_keys(document, sink);
    checker.declared_types(sink);
    checker.widget_cycles(&located, sink);
    for node in &located {
        checker.node(node, sink);
        if !in_declaration(&node.path) {
            if let Some(Body::Composite(composite)) = body_of(node.node) {
                keys_together(composite, &node.path, sink);
            }
            pagers(node, sink);
        }
    }
    degrades_cover(&located, options, sink);
    crate::enums::run(document, &located, sink);
    crate::names::overlay_params(&located, sink);
}

/// `export_selection`: an export's params read `selection.<field>` only on one of a collection's
/// `bulk_actions`, the one place the runtime puts the section's selection in the action's scope
/// (`bulk: true` on any other action puts nothing there). Anywhere else the path reads nothing
/// and the view is read without it.
fn export_selection(sink: &mut Sink, path: &NodePath, export: &Export) {
    if path.segments().iter().rev().nth(1).map(String::as_str) == Some("bulk_actions") {
        return;
    }
    let params = path.child("export").child("params");
    let written: Vec<(NodePath, &str)> = match &export.params {
        None => Vec::new(),
        Some(ExportParams::Expression(expr)) => vec![(params, expr.0.as_str())],
        Some(ExportParams::Map(map)) => map
            .iter()
            .map(|(name, expr)| (params.child(name), expr.0.as_str()))
            .collect(),
    };
    for (at, text) in written {
        for field in selection_paths(text) {
            sink.push(
                "export_selection",
                &at,
                format!(
                    "`selection.{field}` reads the section's selection, which only an action \
                     of `bulk_actions` has in scope"
                ),
            );
        }
    }
}

/// The field of each `selection.<field>` path in an expression, where `selection` stands alone
/// (not the tail of `section.<name>.selection` or of a longer name).
fn selection_paths(text: &str) -> Vec<&str> {
    let name_char = |character: char| character.is_ascii_alphanumeric() || character == '_';
    let mut found = Vec::new();
    let mut from = 0;
    while let Some(offset) = text[from..].find("selection.") {
        let start = from + offset;
        let field_start = start + "selection.".len();
        let field_end = text[field_start..]
            .find(|character: char| !name_char(character))
            .map_or(text.len(), |end| field_start + end);
        let alone = text[..start]
            .chars()
            .next_back()
            .is_none_or(|before| !name_char(before) && before != '.');
        if alone && field_end > field_start {
            found.push(&text[field_start..field_end]);
        }
        from = field_start;
    }
    found
}

/// The keys of an account menu entry every renderer draws: the terminal reads an entry as a whole
/// action, the React menu these keys.
const ACCOUNT_MENU_KEYS: &[&str] = &["name", "label", "opens", "does", "sets", "navigate"];

/// `account_menu_keys`: each key an `account_menu` region's `actions` entry declares is one every
/// renderer draws, so no key is accepted and dropped without a word. Region `props` are opaque to
/// the loader, so nothing else reads them.
///
/// Refused besides: an entry that does not read as an action, the terminal's own reading of it,
/// since a renderer then drops the whole entry (no `name`, `navigate` written as a page name,
/// `sets` as a scalar); and `sets` beside `does`, `navigate` or `opens`, which both renderers run
/// instead of it.
fn account_menu_keys(document: &Document, sink: &mut Sink) {
    for (shell_name, shell) in &document.shells {
        for (region_name, region) in &shell.regions {
            if region.kind != RegionKind::AccountMenu {
                continue;
            }
            let Some(Value::Sequence(entries)) = region.props.get("actions") else {
                continue;
            };
            let actions = NodePath::root()
                .child("shells")
                .child(shell_name)
                .child("regions")
                .child(region_name)
                .child("props")
                .child("actions");
            for entry in entries.iter().filter_map(Value::as_mapping) {
                let at = match entry.get("name").and_then(Value::as_str) {
                    Some(name) => actions.child(name),
                    None => actions.clone(),
                };
                for key in entry.keys().filter_map(Value::as_str) {
                    if !ACCOUNT_MENU_KEYS.contains(&key) {
                        sink.push(
                            "account_menu_keys",
                            &at.child(key),
                            format!(
                                "`{key}` on an account menu entry is dropped by a renderer: an \
                                 entry is drawn from `name`, `label`, `opens`, `does`, `sets` \
                                 and `navigate` only"
                            ),
                        );
                    }
                }
                // The keys refused above are left out, so each reason below is about a drawn key.
                let drawn = entry
                    .iter()
                    .filter(|(key, _)| {
                        key.as_str()
                            .is_some_and(|key| ACCOUNT_MENU_KEYS.contains(&key))
                    })
                    .map(|(key, value)| (key.clone(), value.clone()))
                    .collect();
                let action = match ess_yaml::from_value::<ess_ui::Action>(Value::Mapping(drawn)) {
                    Ok(action) => action,
                    Err(error) => {
                        sink.push(
                            "account_menu_keys",
                            &at,
                            format!(
                                "this account menu entry does not read as an action, so a \
                                 renderer drops it whole: {error}"
                            ),
                        );
                        continue;
                    }
                };
                let instead = [
                    ("does", action.does.is_some()),
                    ("navigate", action.navigate.is_some()),
                    ("opens", action.opens.is_some()),
                ];
                if !action.sets.is_empty() {
                    if let Some((other, _)) = instead.iter().find(|(_, declared)| *declared) {
                        sink.push(
                            "account_menu_keys",
                            &at.child("sets"),
                            format!(
                                "`sets` beside `{other}` on an account menu entry is dropped: a \
                                 renderer runs `{other}` instead; give the state change an \
                                 entry of its own"
                            ),
                        );
                    }
                }
            }
        }
    }
}

/// Every read a node holds, with its path: a composite's own, a graph editor's edges', an
/// action's `loads`.
fn reads_of<'a>(located: &'a Located<'_>) -> Vec<(NodePath, &'a Reads)> {
    let mut reads = Vec::new();
    if let Some(Body::Composite(composite)) = body_of(located.node) {
        if let Some((key, read)) = composite_reads(composite) {
            reads.push((located.path.child(key), read));
        }
        if let Composite::GraphEditor(editor) = composite {
            if let Some(read) = editor.edges.as_ref().and_then(|edges| edges.reads.as_ref()) {
                reads.push((located.path.child("edges").child("reads"), read));
            }
        }
    }
    if let NodeRef::Action(action) = located.node {
        if let Some(read) = &action.loads {
            reads.push((located.path.child("loads"), read));
        }
    }
    reads
}

/// `pager_paging`, `pager_params`, `pager_sizes`: a read's `pager` pages a server read or a
/// client read. On a server read it names its `page` param and may name a `size` param, each a
/// param of that read bound to a `state.<path>` a renderer can set, neither path the other or
/// inside it (both are written when a size is chosen). On a client read it names no param: the
/// renderer pages the rows it holds, so `page` and `size` are refused by name. Either offers at
/// least one page size, each at least 1 and none listed twice. Nothing is recognised by its name: a param is a pager
/// param only when the pager names it.
///
/// `pager_outside_collection`: only a collection renders a pager footer, so a `pager` on any
/// other read (another composite's, a graph editor's edges', an action's `loads`) is a key no
/// renderer honours.
fn pagers(located: &Located<'_>, sink: &mut Sink) {
    let collection = matches!(
        body_of(located.node),
        Some(Body::Composite(Composite::Collection(_)))
    );
    for (at, read) in reads_of(located) {
        let Some(pager) = &read.pager else {
            continue;
        };
        let pager_at = at.child("pager");
        if !collection {
            sink.push(
                "pager_outside_collection",
                &pager_at,
                "only a collection renders a pager; a `pager` here is shown by no renderer, so \
                 move it to a collection's `reads` or remove it",
            );
            continue;
        }
        let (page, size) = pager_paging(read, pager, &at, sink);
        for (key, name) in [("page", page), ("size", size)] {
            let Some(name) = name else {
                continue;
            };
            let message = match read.params.get(name) {
                None => format!(
                    "`pager.{key}` names `{name}`, which is no param of this read; name one of \
                     its `params`"
                ),
                Some(bound) if read.state_path(name).is_none() => format!(
                    "`pager.{key}` names `{name}`, bound to `{}`; a pager param is bound to a \
                     `state.<path>` the pager sets",
                    bound.0
                ),
                Some(_) if key == "size" && Some(name) == page => format!(
                    "`pager.size` names `{name}`, the page param; the size needs a param of its \
                     own"
                ),
                Some(_) => continue,
            };
            sink.push("pager_params", &pager_at.child(key), message);
        }
        if let (Some(page), Some(size)) = (page, size.filter(|size| Some(*size) != page)) {
            if let (Some(page_path), Some(size_path)) =
                (read.state_path(page), read.state_path(size))
            {
                if overlaps(page_path, size_path) {
                    sink.push(
                        "pager_params",
                        &pager_at,
                        format!(
                            "`pager.page` is bound to `{page_path}` and `pager.size` to \
                             `{size_path}`; one path holds the other, so writing one replaces the \
                             other: bind each to a state path of its own"
                        ),
                    );
                }
            }
        }
        if pager.sizes.is_empty() {
            sink.push(
                "pager_sizes",
                &pager_at.child("sizes"),
                "`sizes` offers no page size; list at least one, or leave it out for the default",
            );
        }
        for size in pager.sizes.iter().filter(|size| **size < 1) {
            sink.push(
                "pager_sizes",
                &pager_at.child("sizes"),
                format!("a page size is at least 1, not {size}"),
            );
        }
        // A repeated size is offered twice: React keys its options by the size, and the
        // terminal's `+` never passes the repeat to the sizes after it.
        let mut seen = std::collections::BTreeSet::new();
        let mut repeated = std::collections::BTreeSet::new();
        for size in &pager.sizes {
            if !seen.insert(*size) {
                repeated.insert(*size);
            }
        }
        for size in repeated {
            sink.push(
                "pager_sizes",
                &pager_at.child("sizes"),
                format!("the page size {size} is listed more than once; list each size once"),
            );
        }
    }
}

/// `pager_paging`, reported at the read: a server pager names its `page`, a client pager names
/// neither `page` nor `size`, and no other paging takes a pager. Returns the page and size params
/// `pager_params` goes on to check: none on a client read, whose params were refused here.
fn pager_paging<'r>(
    read: &Reads,
    pager: &'r Pager,
    at: &NodePath,
    sink: &mut Sink,
) -> (Option<&'r String>, Option<&'r String>) {
    match read.paging {
        Some(Paging::Server) => {
            if pager.page.is_none() {
                sink.push(
                    "pager_paging",
                    at,
                    "with `paging: server` a `pager` names the param that carries the page: add \
                     `page`",
                );
            }
        }
        Some(Paging::Client) => {
            for (key, name) in [("page", &pager.page), ("size", &pager.size)] {
                if let Some(name) = name {
                    sink.push(
                        "pager_paging",
                        at,
                        format!(
                            "with `paging: client` the renderer pages the rows it holds and a \
                             `pager` offers only `sizes`; `{key}` names the param `{name}`, which \
                             only `paging: server` sends: remove `{key}`"
                        ),
                    );
                }
            }
            return (None, None);
        }
        _ => sink.push(
            "pager_paging",
            at,
            "a `pager` pages a read the server pages or the renderer pages: it needs `paging: \
             server` or `paging: client`",
        ),
    }
    (pager.page.as_ref(), pager.size.as_ref())
}

/// Whether two `state.<path>` texts are the same path or one lies inside the other, compared
/// segment by segment (`state.q` holds `state.q.size`, not `state.qs`).
fn overlaps(left: &str, right: &str) -> bool {
    left.split('.')
        .zip(right.split('.'))
        .all(|(left, right)| left == right)
}

/// Keys that mean something only beside another: a metric's `aggregate`, `field` and `reads`
/// (beyond10x/ess#358), a collection's `group_order` and `group_by` (#351), and a choice's
/// `value` and `label` and its `reads` (#328).
fn keys_together(composite: &Composite, path: &NodePath, sink: &mut Sink) {
    match composite {
        Composite::Metric(metric) => match (metric.aggregate, &metric.field) {
            (Some(kind), _) if metric.reads.is_none() => sink.push(
                "metric_aggregate",
                &path.child("aggregate"),
                format!("`aggregate: {}` is computed over the rows of `reads`, which this metric does not have", kind.as_str()),
            ),
            (Some(kind), None) if kind != ess_ui::MetricAggregate::Count => sink.push(
                "metric_aggregate",
                &path.child("aggregate"),
                format!("`aggregate: {}` needs the row `field` it reads", kind.as_str()),
            ),
            (None, Some(_)) => sink.push(
                "metric_aggregate",
                &path.child("field"),
                "`field` is the row field an `aggregate` reads; this metric has no `aggregate`",
            ),
            _ => {}
        },
        Composite::Collection(collection) if collection.group_by.is_none() => {
            if !collection.group_order.is_empty() || collection.show_empty_groups {
                sink.push(
                    "group_order",
                    path,
                    "`group_order` and `show_empty_groups` order the groups of `group_by`, which \
                     this collection does not have",
                );
            }
        }
        Composite::Collection(collection)
            if collection.show_empty_groups && collection.group_order.is_empty() =>
        {
            sink.push(
                "group_order",
                &path.child("show_empty_groups"),
                "`show_empty_groups` shows the `group_order` values no row has; there is no \
                 `group_order`",
            );
        }
        Composite::Choice(choice) if choice.reads.is_none() => {
            for (key, named) in [("value", &choice.value), ("label", &choice.label)] {
                if named.is_some() {
                    sink.push(
                        "choice_projection",
                        &path.child(key),
                        format!(
                            "`{key}` names a field of the rows `reads` reads, and this choice \
                             reads no view"
                        ),
                    );
                }
            }
        }
        _ => {}
    }
}

/// Every view and channel the document's fixtures answer.
#[derive(Default)]
struct Fixtures {
    views: BTreeSet<String>,
    scripts: BTreeSet<String>,
    /// The `rows` each view's fixture file holds; a view whose file cannot be read has none.
    rows: BTreeMap<String, Vec<Value>>,
}

impl Fixtures {
    /// The inline index merged with the `index` file, read relative to the document.
    fn read(document: &Document, base: &Path, sink: &mut Sink) -> Self {
        let mut fixtures = Self::default();
        let Some(index) = &document.fixtures else {
            return fixtures;
        };
        let dir = index
            .dir
            .as_ref()
            .map_or_else(|| base.to_path_buf(), |dir| base.join(dir));
        fixtures.absorb(index, &dir);
        if let Some(file) = &index.index {
            let path = base.join(file);
            let read = std::fs::read_to_string(&path)
                .map_err(|error| error.to_string())
                .and_then(|text| {
                    ess_yaml::from_str::<FixtureIndex>(&text).map_err(|error| error.to_string())
                });
            match read {
                Ok(file) => fixtures.absorb(&file, &dir),
                Err(error) => sink.push(
                    "fixture_per_view",
                    &NodePath::root().child("fixtures").child("index"),
                    format!(
                        "the fixture index {} cannot be read: {error}",
                        path.display()
                    ),
                ),
            }
        }
        fixtures
    }

    fn absorb(&mut self, index: &FixtureIndex, dir: &Path) {
        self.views.extend(index.views.keys().cloned());
        self.views.extend(index.derived.keys().cloned());
        self.scripts.extend(index.scripts.keys().cloned());
        for (view, file) in &index.views {
            if let Some(rows) = view_rows(&dir.join(file), view) {
                self.rows.entry(view.clone()).or_insert(rows);
            }
        }
    }
}

/// The `rows` of `view` in a fixture file: one view (`view:` and `rows:`) or several (`views:`).
fn view_rows(path: &Path, view: &str) -> Option<Vec<Value>> {
    let file: Value = ess_yaml::from_str(&std::fs::read_to_string(path).ok()?).ok()?;
    let body = match file.as_mapping()?.get("views") {
        Some(Value::Mapping(views)) => views.get(view)?.clone(),
        _ if file.as_mapping()?.get("view").and_then(Value::as_str) == Some(view) => file,
        _ => return None,
    };
    Some(body.as_mapping()?.get("rows")?.as_sequence()?.clone())
}

struct Checker<'a> {
    document: &'a Document,
    fixtures: &'a Fixtures,
}

impl Checker<'_> {
    fn page_exists(&self, name: &str) -> bool {
        self.document.pages.contains_key(name)
    }

    fn page_ref(&self, sink: &mut Sink, path: &NodePath, name: &str) {
        if !self.page_exists(name) {
            sink.push("page_refs", path, format!("`{name}` names no page"));
        }
    }

    // ── nav_resolves, page_reachable ─────────────────────────────────────────────────────────

    fn navigation(&self, sink: &mut Sink) {
        let navigation = &self.document.navigation;
        let at = NodePath::root().child("navigation");
        let mut listed = BTreeSet::new();
        let mut resolve = |sink: &mut Sink, path: &NodePath, name: &str| {
            listed.insert(name.to_owned());
            if !self.page_exists(name) {
                sink.push("nav_resolves", path, format!("`{name}` names no page"));
            }
        };
        if !self.page_exists(&navigation.home) {
            sink.push(
                "nav_resolves",
                &at.child("home"),
                format!("the home page `{}` names no page", navigation.home),
            );
        }
        let mut entries: BTreeMap<&str, String> = BTreeMap::new();
        for section in &navigation.sections {
            let here = at.child("sections").child(&section.name).child("pages");
            match &section.pages {
                NavPages::Fixed(pages) => {
                    for page in pages {
                        resolve(sink, &here, page);
                        if let Some(first) = entries.get(page.as_str()) {
                            sink.push(
                                "nav_unique",
                                &here,
                                format!(
                                    "`{page}` is already listed in navigation section `{first}`; \
                                     a page is listed once"
                                ),
                            );
                        } else {
                            entries.insert(page, section.name.clone());
                        }
                    }
                }
                NavPages::Dynamic(entries) => resolve(sink, &here, &entries.page),
            }
        }
        for page in &navigation.hidden {
            resolve(sink, &at.child("hidden"), page);
        }
        for name in self.document.pages.keys() {
            if !listed.contains(name) {
                sink.push(
                    "page_reachable",
                    &NodePath::root().child("pages").child(name),
                    format!(
                        "no navigation section lists `{name}`, and `navigation.hidden` does not \
                         either"
                    ),
                );
            }
        }
    }

    // ── page_refs, section_refs, layout_complete, types_structural on params ─────────────────

    // ── shell_refs, page_outlet ──────────────────────────────────────────────────────────────

    fn shells(&self, sink: &mut Sink) {
        let mut rendered: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
        for (name, page) in &self.document.pages {
            if self.document.shells.contains_key(&page.shell) {
                rendered.entry(page.shell.as_str()).or_default().push(name);
            } else {
                sink.push(
                    "shell_refs",
                    &NodePath::root().child("pages").child(name).child("shell"),
                    format!("`{}` names no shell", page.shell),
                );
            }
        }
        for (shell, pages) in rendered {
            let regions = &self.document.shells[shell].regions;
            let outlet = regions.values().any(|region| {
                matches!(
                    region.kind,
                    RegionKind::PageOutlet | RegionKind::Unmapped(_)
                )
            });
            if !outlet {
                sink.push(
                    "page_outlet",
                    &NodePath::root().child("shells").child(shell),
                    format!(
                        "no region of shell `{shell}` is a `page_outlet`, so {} cannot show",
                        pages
                            .iter()
                            .map(|page| format!("`{page}`"))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                );
            }
        }
    }

    fn pages(&self, sink: &mut Sink) {
        for (name, page) in &self.document.pages {
            let at = NodePath::root().child("pages").child(name);
            for target in &page.switch_to {
                self.page_ref(sink, &at.child("switch_to"), target);
            }
            for (param, ty) in &page.params {
                type_expr(sink, &at.child("params").child(param), ty);
            }
            let sections: Vec<&str> = page.sections.iter().map(|s| s.name.as_str()).collect();
            let layout = at.child("layout");
            let mut placed: BTreeMap<String, usize> = BTreeMap::new();
            let mut place = |sink: &mut Sink, path: &NodePath, section: &str| {
                *placed.entry(section.to_owned()).or_default() += 1;
                if !sections.contains(&section) {
                    sink.push(
                        "section_refs",
                        path,
                        format!("`{section}` names no section of page `{name}`"),
                    );
                }
            };
            match &page.layout {
                PageLayout::Stack(_) => continue,
                PageLayout::Columns(columns) => {
                    for column in &columns.columns {
                        let here = layout.child("columns").child(&column.name);
                        for section in &column.sections {
                            place(sink, &here, section);
                        }
                        frame_rules(
                            sink,
                            &here,
                            &sections,
                            &Framing {
                                frame: column.frame.as_deref(),
                                title: column.title.as_deref(),
                                collapsed: column.collapsed,
                                sections: &column.sections,
                            },
                        );
                    }
                }
                PageLayout::Areas(areas) => {
                    for (area, entries) in &areas.areas.place {
                        let here = layout.child("areas").child("place").child(area);
                        for section in entries.sections() {
                            place(sink, &here, section);
                        }
                        if let Some(framed) = entries.framed() {
                            frame_rules(
                                sink,
                                &here,
                                &sections,
                                &Framing {
                                    frame: framed.frame.as_deref(),
                                    title: framed.title.as_deref(),
                                    collapsed: framed.collapsed,
                                    sections: &framed.sections,
                                },
                            );
                        }
                    }
                }
            }
            for section in &sections {
                match placed.get(*section).copied().unwrap_or(0) {
                    1 => {}
                    0 => sink.push(
                        "layout_complete",
                        &layout,
                        format!("section `{section}` is placed nowhere in the layout"),
                    ),
                    times => sink.push(
                        "layout_complete",
                        &layout,
                        format!("section `{section}` is placed {times} times; once is the rule"),
                    ),
                }
            }
        }
    }

    // ── types_structural on declared types and widget params ─────────────────────────────────

    fn declared_types(&self, sink: &mut Sink) {
        let root = NodePath::root();
        for (name, ty) in &self.document.types {
            type_expr(sink, &root.child("types").child(name), ty);
        }
        for (widget, declaration) in &self.document.widgets {
            for (param, spec) in &declaration.params {
                let at = root
                    .child("widgets")
                    .child(widget)
                    .child("params")
                    .child(param)
                    .child("type");
                type_expr(sink, &at, &spec.ty);
                // `WidgetParam.default` is the argument of every use that omits it.
                let problem = spec
                    .default
                    .as_ref()
                    .and_then(|default| self.mismatch(&spec.ty, default, 0));
                if let Some(problem) = problem {
                    sink.push(
                        "widget_expands",
                        &root.child("widgets").child(widget),
                        format!(
                            "the default of param `{param}` does not match its type: {problem}"
                        ),
                    );
                }
            }
        }
    }

    // ── widget_expands: resolve_widget and no_recursion, on every declaration ───────────────

    fn widget_cycles(&self, located: &[Located<'_>], sink: &mut Sink) {
        let widgets = &self.document.widgets;
        let mut uses: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
        for node in located {
            let [top, widget, ..] = node.path.segments() else {
                continue;
            };
            if top != "widgets" {
                continue;
            }
            let Some(Body::Widget(instance)) = body_of(node.node) else {
                continue;
            };
            uses.entry(widget.as_str())
                .or_default()
                .insert(instance.component.as_str());
            if !widgets.contains_key(&instance.component) {
                sink.push(
                    "widget_expands",
                    &node.path,
                    format!(
                        "`{}` names neither a member of the composite union nor a widget",
                        instance.component
                    ),
                );
            }
        }
        let names: Vec<&str> = widgets.keys().map(String::as_str).collect();
        for (widget, cycle) in cycles(&names, &uses) {
            sink.push(
                "widget_expands",
                &NodePath::root().child("widgets").child(widget),
                format!(
                    "widget `{widget}` contains itself ({}); a widget may not contain itself, \
                     directly or indirectly",
                    cycle.join(" → ")
                ),
            );
        }
    }

    // ── widget_expands: args_match_param_types, at each use ──────────────────────────────────

    fn arguments(&self, sink: &mut Sink, path: &NodePath, instance: &WidgetUse) {
        let Some(widget) = self.document.widgets.get(&instance.component) else {
            return; // the loader refuses a use of an undeclared widget
        };
        for (argument, value) in &instance.args {
            let Some(param) = widget.params.get(argument) else {
                continue; // the loader refuses an argument naming no param
            };
            if let Some(problem) = self.mismatch(&param.ty, value, 0) {
                sink.push(
                    "widget_expands",
                    path,
                    format!(
                        "argument `{argument}` of widget `{}` does not match its param type: \
                         {problem}",
                        instance.component
                    ),
                );
            }
        }
    }

    /// Why a literal argument is not a value of `ty`, or `None` when it is one. An expression
    /// (`row.tier`, `state.window`, as `expressions.forms` parses it) is accepted whatever the
    /// type: its value exists only at render time. An UNMAPPED marker is accepted by the types
    /// `unmapped_marker.accepted_by` lists, and reported by `unmapped_reported`. A capitalized
    /// name that is not a type of this document is a type of the model, which this check does
    /// not read.
    fn mismatch(&self, ty: &TypeExpr, value: &Value, depth: usize) -> Option<String> {
        if depth > 32 {
            return None;
        }
        if value.as_str().is_some_and(schema::is_unmapped_marker) {
            return self.marker_refused(ty, depth);
        }
        if value.as_str().is_some_and(expr::is_expression) {
            return None;
        }
        let shown = || shown(value);
        match ty {
            TypeExpr::Named(name) => {
                if let Some(declared) = self.document.types.get(name) {
                    return self.mismatch(declared, value, depth + 1);
                }
                let fits = match name.as_str() {
                    "integer" => value.as_i64().is_some() || value.as_u64().is_some(),
                    "number" => value.is_number(),
                    "boolean" => value.is_bool(),
                    "duration" => value.as_str().is_some_and(is_duration),
                    "string" | "expr" | "name" | "timestamp" | "date" | "time" => {
                        value.is_string() || value.is_number() || value.is_bool()
                    }
                    _ => true, // json, or a model type
                };
                (!fits).then(|| format!("{} is not a `{name}`", shown()))
            }
            TypeExpr::Enum(allowed) => {
                let text = scalar_text(value);
                (!text.is_some_and(|text| allowed.values.contains(&text)))
                    .then(|| format!("{} is not one of {}", shown(), allowed.values.join(", ")))
            }
            TypeExpr::Const(constant) => {
                (value != &constant.value).then(|| format!("{} is not the constant", shown()))
            }
            TypeExpr::Optional(inner) => {
                if value.is_null() {
                    None
                } else {
                    self.mismatch(&inner.optional, value, depth + 1)
                }
            }
            TypeExpr::List(list) => match value.as_sequence() {
                Some(entries) => entries
                    .iter()
                    .find_map(|entry| self.mismatch(&list.list, entry, depth + 1)),
                None => Some(format!("{} is not a list", shown())),
            },
            TypeExpr::Map(map) => match value.as_mapping() {
                Some(entries) => entries
                    .values()
                    .find_map(|entry| self.mismatch(&map.map.value, entry, depth + 1)),
                None => Some(format!("{} is not a map", shown())),
            },
            TypeExpr::Record(record) => {
                let Some(entries) = value.as_mapping() else {
                    return Some(format!("{} is not a record", shown()));
                };
                for key in entries.keys() {
                    let key = scalar_text(key).unwrap_or_default();
                    if !record.record.contains_key(&key) {
                        return Some(format!("the record has no field `{key}`"));
                    }
                }
                record
                    .record
                    .iter()
                    .find_map(|(field, ty)| match entries.get(field.as_str()) {
                        Some(entry) => self.mismatch(ty, entry, depth + 1),
                        None if matches!(ty, TypeExpr::Optional(_)) => None,
                        None => Some(format!("the record lacks field `{field}`")),
                    })
            }
            TypeExpr::OneOf(alternatives) => {
                let every: Vec<String> = alternatives
                    .one_of
                    .iter()
                    .filter_map(|alternative| self.mismatch(alternative, value, depth + 1))
                    .collect();
                (every.len() == alternatives.one_of.len())
                    .then(|| format!("{} matches no alternative", shown()))
            }
            TypeExpr::Ref(reference) => (!value.is_string())
                .then(|| format!("{} is not a name of a {}", shown(), reference.kind)),
        }
    }

    /// Why `ty` refuses the UNMAPPED marker, or `None` when `unmapped_marker.accepted_by` lists
    /// it. A declared type is followed to what it names; an optional accepts what its inner type
    /// does, a `one_of` what any alternative does. A capitalized name this document does not
    /// declare is a model type, which this check does not read.
    fn marker_refused(&self, ty: &TypeExpr, depth: usize) -> Option<String> {
        if depth > 32 {
            return None;
        }
        let accepted = schema::marker_accepted_by();
        let constructor = |name: &str| {
            (!accepted.constructors.contains(name))
                .then(|| format!("a `{{{name}: …}}` refuses the UNMAPPED marker"))
        };
        match ty {
            TypeExpr::Named(name) => {
                if let Some(declared) = self.document.types.get(name) {
                    return self.marker_refused(declared, depth + 1);
                }
                let refused =
                    schema::primitive_types().contains(name) && !accepted.names.contains(name);
                refused.then(|| format!("a `{name}` refuses the UNMAPPED marker"))
            }
            TypeExpr::Optional(inner) => self.marker_refused(&inner.optional, depth + 1),
            TypeExpr::OneOf(alternatives) => {
                let every: Vec<String> = alternatives
                    .one_of
                    .iter()
                    .filter_map(|alternative| self.marker_refused(alternative, depth + 1))
                    .collect();
                (every.len() == alternatives.one_of.len())
                    .then(|| "no alternative accepts the UNMAPPED marker".to_owned())
            }
            TypeExpr::Enum(_) => constructor("enum"),
            TypeExpr::Ref(_) => constructor("ref"),
            TypeExpr::Const(_) => constructor("const"),
            TypeExpr::List(_) => constructor("list"),
            TypeExpr::Map(_) => constructor("map"),
            TypeExpr::Record(_) => constructor("record"),
        }
    }

    // ── per node ─────────────────────────────────────────────────────────────────────────────

    /// Every per-node check. None runs inside a widget declaration, where the arguments are
    /// unbound and no use site's page or section profile applies: each runs at every use, on
    /// the body the loader expanded there. What a declaration is held to on its own —
    /// `widget_cycles` and `declared_types` — does not depend on either.
    fn node(&self, located: &Located<'_>, sink: &mut Sink) {
        let path = &located.path;
        if in_declaration(path) {
            return;
        }
        match located.node {
            NodeRef::Shell(shell) => {
                for page in shell.preload.iter().flat_map(|preload| &preload.except_on) {
                    self.page_ref(sink, &path.child("preload"), page);
                }
            }
            NodeRef::Guard(guard) => {
                if let GuardThen::Redirect(redirect) = &guard.then {
                    self.page_ref(sink, &path.child("then"), &redirect.redirect);
                }
            }
            NodeRef::State(state) => {
                type_expr(sink, &path.child("type"), &state.ty);
                self.state(sink, path, state);
            }
            NodeRef::Header(header) => self.header(sink, path, header),
            NodeRef::Section(section) => {
                if let Some(live) = &section.live {
                    self.channel(sink, &path.child("live"), &live.channel);
                }
                if let Some(target) = &section.depends_on {
                    let known = page_of(self.document, path).is_some_and(|(_, page)| {
                        page.sections.iter().any(|sibling| &sibling.name == target)
                    });
                    if !known {
                        sink.push(
                            "section_refs",
                            &path.child("depends_on"),
                            format!("`{target}` names no section of this page"),
                        );
                    }
                }
                if section.collapsed == Some(true)
                    && section
                        .title
                        .as_deref()
                        .is_none_or(|title| title.trim().is_empty())
                {
                    sink.push(
                        "frame_rules",
                        path,
                        "`collapsed` needs a `title`: a collapsed section is its title, which a \
                         reader clicks to open it",
                    );
                }
                degrades_known(sink, path, &section.common);
                self.body(sink, path, &section.body);
            }
            NodeRef::FormGroup(group) => {
                collapsed_group_has_label(sink, path, group.collapsed, group.label.as_deref());
            }
            NodeRef::RecordGroup(group) => {
                collapsed_group_has_label(sink, path, group.collapsed, group.label.as_deref());
            }
            NodeRef::Overlay(overlay) => {
                degrades_known(sink, path, &overlay.common);
                self.body(sink, path, &overlay.body);
            }
            NodeRef::Node(node) => {
                if let Some(live) = &node.live {
                    self.channel(sink, &path.child("live"), &live.channel);
                }
                degrades_known(sink, path, &node.common);
                self.body(sink, path, &node.body);
            }
            NodeRef::Action(action) => self.action(sink, path, action),
            NodeRef::Field(field) => {
                if let Some(suggests) = &field.suggests {
                    suggests_needs_tags(sink, path, field.field_as.as_deref());
                    self.reads(
                        sink,
                        &path.child("suggests").child("reads"),
                        &suggests.reads,
                    );
                }
            }
            NodeRef::Channel(_) => {
                let name = path.segments().last().map_or("", String::as_str);
                if !self.fixtures.scripts.contains(name) {
                    sink.push(
                        "script_per_channel",
                        path,
                        format!("no fixture script (`fixtures.scripts`) plays channel `{name}`"),
                    );
                }
            }
            _ => {}
        }
    }

    fn channel(&self, sink: &mut Sink, path: &NodePath, name: &str) {
        if !self.document.channels.contains_key(name) {
            sink.push("channel_refs", path, format!("`{name}` names no channel"));
        }
    }

    fn header(&self, sink: &mut Sink, path: &NodePath, header: &Header) {
        let sections: BTreeSet<&str> = page_of(self.document, path)
            .map(|(_, page)| page.sections.iter().map(|s| s.name.as_str()).collect())
            .unwrap_or_default();
        for (key, target) in [("total", &header.total), ("filters", &header.filters)] {
            if let Some(target) = target {
                if !sections.contains(target.as_str()) {
                    sink.push(
                        "section_refs",
                        &path.child(key),
                        format!("`{target}` names no section of this page"),
                    );
                }
            }
        }
        for channel in &header.live {
            self.channel(sink, &path.child("live"), channel);
        }
        // beyond10x/ess#354: the title's record is the first row of a section that reads.
        if let Some(from) = &header.title_from {
            let at = path.child("title_from");
            let section = page_of(self.document, path).and_then(|(_, page)| {
                page.sections
                    .iter()
                    .find(|section| section.name == from.section)
            });
            match section {
                None => sink.push(
                    "section_refs",
                    &at,
                    format!("`{}` names no section of this page", from.section),
                ),
                Some(section) if section.body.reads().is_none() => sink.push(
                    "header_record",
                    &at,
                    format!(
                        "section `{}` reads nothing, so it holds no record for the title",
                        from.section
                    ),
                ),
                Some(_) => {}
            }
        }
    }

    fn body(&self, sink: &mut Sink, path: &NodePath, body: &Body) {
        match body {
            Body::Composite(composite) => {
                if let Some((key, reads)) = composite_reads(composite) {
                    self.reads(sink, &path.child(key), reads);
                }
                if let Composite::Board(board) = composite {
                    self.board_widget_kinds(sink, path, board);
                }
                if let Composite::GraphEditor(editor) = composite {
                    if let Some(opens) = editor.nodes.as_ref().and_then(|n| n.opens.as_ref()) {
                        self.opens(sink, &path.child("nodes"), opens);
                    }
                    if let Some(reads) = editor.edges.as_ref().and_then(|e| e.reads.as_ref()) {
                        self.reads(sink, &path.child("edges").child("reads"), reads);
                    }
                }
            }
            Body::Primitive(primitive) => self.primitive(sink, path, primitive),
            Body::Widget(instance) => self.arguments(sink, path, instance),
            Body::Host(_) => {}
        }
    }

    /// `board_widget_kinds`: a fixture row of the board's read whose `widget_by` field is absent,
    /// or names no entry of `widgets`, is drawn as a placeholder, never as a widget.
    fn board_widget_kinds(&self, sink: &mut Sink, path: &NodePath, board: &ess_ui::Board) {
        let Some(rows) = board
            .reads
            .view
            .as_ref()
            .and_then(|view| self.fixtures.rows.get(view))
        else {
            return;
        };
        let by = board.widget_by.as_deref().unwrap_or("type");
        let mut unnamed = BTreeSet::new();
        let mut absent = false;
        for row in rows {
            let kind = row
                .as_mapping()
                .and_then(|row| row.get(by))
                .and_then(scalar_text)
                .filter(|kind| !kind.is_empty());
            match kind {
                None => absent = true,
                Some(kind) if !board.widgets.contains_key(&kind) => {
                    unnamed.insert(kind);
                }
                Some(_) => {}
            }
        }
        if absent {
            sink.push(
                "board_widget_kinds",
                path,
                format!("a fixture row of the board's read has no `{by}`, so no `widgets` entry draws it"),
            );
        }
        for kind in unnamed {
            sink.push(
                "board_widget_kinds",
                path,
                format!("a fixture row of the board's read has `{by}: {kind}`, which no `widgets` entry names"),
            );
        }
    }

    fn reads(&self, sink: &mut Sink, path: &NodePath, reads: &Reads) {
        if let Some(placeholder) = &reads.placeholder {
            sink.push(
                "unbound_placeholder",
                path,
                format!(
                    "`{placeholder}` is a placeholder read answered by `{}`; bind it to a view",
                    reads.fixture.as_deref().unwrap_or("no fixture")
                ),
            );
        }
        if let Some(view) = &reads.view {
            if !self.fixtures.views.contains(view) {
                sink.push(
                    "fixture_per_view",
                    path,
                    format!(
                        "no fixture answers view `{view}` (`fixtures.views` or `fixtures.derived`)"
                    ),
                );
            }
        }
    }

    fn action(&self, sink: &mut Sink, path: &NodePath, action: &Action) {
        if let Some(overlay) = &action.opens {
            self.opens(sink, path, overlay);
        }
        if let Some(ActionConfirm::Opens(overlay)) = &action.confirm {
            self.opens(sink, &path.child("confirm"), overlay);
        }
        if let Some(navigate) = &action.navigate {
            self.page_ref(sink, path, &navigate.to);
        }
        if let Some(reads) = &action.loads {
            self.reads(sink, &path.child("loads"), reads);
        }
        toggle_state(sink, path, action);
        if let Some(export) = &action.export {
            export_selection(sink, path, export);
        }
    }

    /// `opens_resolves`: in a page, the page's overlays (its kind's included, since the page was
    /// merged over it) and its shell's; in a shell, the shell's. A widget declaration has no
    /// scope of its own: its instances are checked where they are expanded.
    fn opens(&self, sink: &mut Sink, path: &NodePath, overlay: &str) {
        let document = self.document;
        let (found, scope) = match path.segments() {
            [top, ..] if top == "pages" => {
                let Some((name, page)) = page_of(document, path) else {
                    return;
                };
                let in_shell = document
                    .shells
                    .get(&page.shell)
                    .is_some_and(|shell| shell.overlays.contains_key(overlay));
                (
                    page.overlays.contains_key(overlay) || in_shell,
                    format!("page `{name}` or its shell `{}`", page.shell),
                )
            }
            [top, shell, ..] if top == "shells" => (
                document
                    .shells
                    .get(shell)
                    .is_some_and(|shell| shell.overlays.contains_key(overlay)),
                format!("shell `{shell}`"),
            ),
            _ => return,
        };
        if !found {
            sink.push(
                "opens_resolves",
                path,
                format!("`{overlay}` names no overlay of {scope}"),
            );
        }
    }

    fn primitive(&self, sink: &mut Sink, path: &NodePath, primitive: &Primitive) {
        match primitive {
            Primitive::Text(text) => exactly_one(
                sink,
                path,
                "text",
                &[
                    ("text", text.text.is_some()),
                    ("field", text.field.is_some()),
                ],
            ),
            Primitive::Badge(badge) => exactly_one(
                sink,
                path,
                "badge",
                &[
                    ("text", badge.text.is_some()),
                    ("field", badge.field.is_some()),
                ],
            ),
            Primitive::Link(link) => {
                exactly_one(
                    sink,
                    path,
                    "link",
                    &[("to", link.to.is_some()), ("href", link.href.is_some())],
                );
                if let Some(to) = &link.to {
                    self.page_ref(sink, path, &to.to);
                }
            }
            Primitive::Toggle(toggle) => exactly_one(
                sink,
                path,
                "toggle",
                &[
                    ("binds", toggle.binds.is_some()),
                    ("action", toggle.action.is_some()),
                ],
            ),
            Primitive::Icon(_)
            | Primitive::Button(_)
            | Primitive::Input(_)
            | Primitive::Image(_)
            | Primitive::Divider(_) => {}
        }
    }

    // ── state_resolves ───────────────────────────────────────────────────────────────────────

    fn state(&self, sink: &mut Sink, path: &NodePath, state: &State) {
        let Some(class) = class_name(&state.class) else {
            return; // an UNMAPPED class is reported by `unmapped_reported`
        };
        let store = match &state.store {
            Some(store) => Ok(store_name(store).map(str::to_owned)),
            None => self.resolve(path, &state.class, class),
        };
        match store {
            Ok(Some(store)) => refusals(sink, path, state, class, store),
            Ok(None) => {}
            Err(reason) => sink.push("state_resolves", path, reason),
        }
        if let Some(fallback) = &state.fallback {
            if let Some(store) = store_name(&fallback.store) {
                refusals(sink, &path.child("fallback"), state, class, store);
            }
        }
    }

    /// `PlacementProfile.resolution.order` after `explicit_store`: the section's profile, the
    /// page's, the document's `placement_defaults`, then the document's profile. `Ok(None)` when
    /// an UNMAPPED value stops the resolution.
    fn resolve(
        &self,
        path: &NodePath,
        class: &StateClass,
        name: &str,
    ) -> Result<Option<String>, String> {
        let document = self.document;
        let profiles = [
            section_of(document, path).and_then(|section| section.profile.as_ref()),
            page_of(document, path).and_then(|(_, page)| page.profile.as_ref()),
        ];
        for profile in profiles.into_iter().flatten() {
            let profile = match profile {
                Profile::Thin => "thin",
                Profile::Fat => "fat",
                Profile::Unmapped(_) => return Ok(None),
            };
            if let Some(store) = schema::profile_defaults(profile).remove(name) {
                return Ok(Some(store));
            }
        }
        if let Some(store) = document.placement_defaults.get(class) {
            return Ok(store_name(store).map(str::to_owned));
        }
        let profile = match &document.placement_profile {
            PlacementProfile::Thin => "thin",
            PlacementProfile::Fat => "fat",
            PlacementProfile::Hybrid => "hybrid",
            PlacementProfile::Unmapped(_) => return Ok(None),
        };
        schema::profile_defaults(profile)
            .remove(name)
            .map(Some)
            .ok_or_else(|| {
                format!(
                    "no store resolves for this `{name}` state: it names no `store`, no page or \
                     section profile applies, `placement_defaults` has no `{name}` entry, and \
                     profile `{profile}` has no default for it"
                )
            })
    }
}

/// Every store the state entry at `path` is placed in: its own `store` or the one
/// `PlacementProfile.resolution` gives it, and its `fallback` store, each by the schema's name.
/// A store an UNMAPPED value or an unresolved profile leaves undecided is not among them; those
/// are `state_resolves` and `unmapped_reported` findings.
pub(crate) fn placements(document: &Document, path: &NodePath, state: &State) -> Vec<String> {
    let mut stores = Vec::new();
    // An explicit store is a placement whatever the class says; only the profile resolution
    // needs a class it can read.
    let placed = match &state.store {
        Some(store) => store_name(store).map(str::to_owned),
        None => class_name(&state.class).and_then(|class| {
            let fixtures = Fixtures::default();
            let checker = Checker {
                document,
                fixtures: &fixtures,
            };
            checker.resolve(path, &state.class, class).ok().flatten()
        }),
    };
    stores.extend(placed);
    if let Some(fallback) = &state.fallback {
        stores.extend(store_name(&fallback.store).map(str::to_owned));
    }
    stores
}

/// `PlacementProfile.resolution.refusals` that a declared or resolved store can break.
fn refusals(sink: &mut Sink, path: &NodePath, state: &State, class: &str, store: impl AsRef<str>) {
    let store = store.as_ref();
    if state.sensitive && ["url", "session_storage", "local_storage"].contains(&store) {
        sink.push(
            "state_resolves",
            path,
            format!(
                "sensitive state may not live in `{store}`: the URL and browser storage are refused"
            ),
        );
    }
    if class == "credential" && !["memory", "server_session"].contains(&store) {
        sink.push(
            "state_resolves",
            path,
            format!("a credential lives in `memory` or `server_session`, never in `{store}`"),
        );
    }
}

fn class_name(class: &StateClass) -> Option<&'static str> {
    Some(match class {
        StateClass::PageState => "page_state",
        StateClass::Selection => "selection",
        StateClass::ComponentState => "component_state",
        StateClass::Draft => "draft",
        StateClass::ViewCache => "view_cache",
        StateClass::ChannelBuffer => "channel_buffer",
        StateClass::Preference => "preference",
        StateClass::Credential => "credential",
        StateClass::Unmapped(_) => return None,
    })
}

fn store_name(store: &Store) -> Option<&'static str> {
    Some(match store {
        Store::Memory => "memory",
        Store::Url => "url",
        Store::SessionStorage => "session_storage",
        Store::LocalStorage => "local_storage",
        Store::ServerSession => "server_session",
        Store::Server => "server",
        Store::Unmapped(_) => return None,
    })
}

/// `checked_needs_toggle`: only a toggle shows a state, so `checked:` on any other action would
/// be read by no renderer. `toggle_state_unknown`: a toggle that runs a command shows the state
/// [`ess_ui::toggle_state`] names; with neither `checked:` nor one `bind` entry written
/// `not <expr>` (`<expr>` reading paths from `row`, `params`, `state`, `shell` or `args` only;
/// `not item.enabled` is literal text) it names none, and every renderer draws the toggle off. No
/// renderer reads a toggle's state from its `loads`, so `loads` exempts nothing.
/// `checked_unreadable`: the terminal evaluates a toggle's `checked:` with the filter grammar
/// ([`ess_ui::filter::parse`], paths from [`ess_ui::filter::Predicate::allowed_roots`] only); one
/// outside it (`row.id in row.owners`, `actor.admin`) the terminal cannot evaluate, so it may draw
/// the toggle off where React draws it on.
fn toggle_state(sink: &mut Sink, path: &NodePath, action: &Action) {
    let toggle = action.action_as == Some(ActionAs::Toggle);
    if action.checked.is_some() && !toggle {
        sink.push(
            "checked_needs_toggle",
            path,
            "`checked` is the state of a toggle; write `as: toggle` or drop `checked`",
        );
    }
    if let Some(checked) = action.checked.as_ref().filter(|_| toggle) {
        let readable =
            ess_ui::filter::parse(&checked.0).is_some_and(|predicate| predicate.allowed_roots());
        if !readable {
            sink.push(
                "checked_unreadable",
                &path.child("checked"),
                "the terminal cannot evaluate this `checked`: it reads a toggle's state in the \
                 filter grammar (paths from row, params, state, shell or args; `in` over a \
                 literal list), so it may draw the toggle off where React draws it on; rewrite \
                 it in that grammar",
            );
        }
    }
    if toggle && action.does.is_some() && ess_ui::toggle_state(action).is_none() {
        sink.push(
            "toggle_state_unknown",
            path,
            "this toggle runs a command but names no state, so every renderer draws it off; \
             write `checked: <expr>`, or bind exactly one input as `not <expr>` over a path \
             from row, params, state, shell or args",
        );
    }
}

/// `suggests_needs_tags`: only an `as: tags` input offers suggestions and keeps typed values, so
/// `suggests` on any other field would be read by no renderer.
fn suggests_needs_tags(sink: &mut Sink, path: &NodePath, field_as: Option<&str>) {
    if field_as != Some("tags") {
        sink.push(
            "suggests_needs_tags",
            path,
            "`suggests` offers values to a tags input; write `as: tags` or drop `suggests`",
        );
    }
}

fn exactly_one(sink: &mut Sink, path: &NodePath, kind: &str, fields: &[(&str, bool)]) {
    let present = fields.iter().filter(|(_, present)| *present).count();
    if present != 1 {
        let names: Vec<String> = fields.iter().map(|(name, _)| format!("`{name}`")).collect();
        sink.push(
            "primitive_props",
            path,
            format!(
                "a {kind} has exactly one of {}; this one has {present}",
                names.join(" and ")
            ),
        );
    }
}

// ── types_structural ─────────────────────────────────────────────────────────────────────────

fn type_expr(sink: &mut Sink, path: &NodePath, ty: &TypeExpr) {
    let mut problems = Vec::new();
    type_problems(ty, &mut problems);
    for problem in problems {
        sink.push("types_structural", path, problem);
    }
}

fn type_problems(ty: &TypeExpr, problems: &mut Vec<String>) {
    match ty {
        TypeExpr::Named(name) => {
            if !schema::is_unmapped_marker(name)
                && !schema::primitive_types().contains(name)
                && !is_capitalized_name(name)
            {
                problems.push(format!(
                    "`{name}` is not a type: a type is a lowercase primitive, a constructor map \
                     or a capitalized name, never a string holding a type expression"
                ));
            }
        }
        TypeExpr::List(list) => type_problems(&list.list, problems),
        TypeExpr::Map(map) => {
            type_problems(&map.map.key, problems);
            type_problems(&map.map.value, problems);
        }
        TypeExpr::Optional(optional) => type_problems(&optional.optional, problems),
        TypeExpr::OneOf(one_of) => {
            for alternative in &one_of.one_of {
                type_problems(alternative, problems);
            }
        }
        TypeExpr::Record(record) => {
            for field in record.record.values() {
                type_problems(field, problems);
            }
        }
        TypeExpr::Ref(reference) => {
            if !schema::ref_kinds().contains(&reference.kind) {
                problems.push(format!(
                    "`{{ref: {}}}` names no kind of reference",
                    reference.kind
                ));
            }
        }
        TypeExpr::Enum(_) | TypeExpr::Const(_) => {}
    }
}

fn is_capitalized_name(name: &str) -> bool {
    let mut characters = name.chars();
    characters
        .next()
        .is_some_and(|first| first.is_ascii_uppercase())
        && characters.all(|character| character.is_ascii_alphanumeric() || character == '_')
}

// ── widget recursion ─────────────────────────────────────────────────────────────────────────

const UNSEEN: usize = usize::MAX;

/// Hops of a reported cycle written out from each end before the middle is elided, so a report
/// stays linear in the widgets however long the cycle is.
const SHOWN: usize = 6;

/// Every declared widget that lies on a cycle of uses, with one closed walk of uses written
/// from it back to itself. A widget lies on a cycle exactly when its strongly connected
/// component (Tarjan's algorithm) has more than one member or it uses itself; the walk goes
/// from the widget to its component's root and back, along breadth-first trees grown inside
/// the component. Linear in widgets and uses, and iterative, so a deep chain neither slows it
/// down nor overflows the stack.
fn cycles<'a>(
    widgets: &[&'a str],
    uses: &BTreeMap<&'a str, BTreeSet<&'a str>>,
) -> BTreeMap<&'a str, Vec<&'a str>> {
    let position: BTreeMap<&str, usize> = widgets
        .iter()
        .enumerate()
        .map(|(index, widget)| (*widget, index))
        .collect();
    let mut forward: Vec<Vec<usize>> = vec![Vec::new(); widgets.len()];
    let mut backward: Vec<Vec<usize>> = vec![Vec::new(); widgets.len()];
    for (from, targets) in uses {
        let Some(&from) = position.get(from) else {
            continue;
        };
        for to in targets {
            if let Some(&to) = position.get(to) {
                forward[from].push(to);
                backward[to].push(from);
            }
        }
    }
    let components = strongly_connected(&forward);
    let mut component = vec![UNSEEN; widgets.len()];
    for (id, (_, members)) in components.iter().enumerate() {
        for &member in members {
            component[member] = id;
        }
    }
    // One array each for every component: components are disjoint, so no entry is written twice.
    let mut ahead = SearchTree::new(widgets.len());
    let mut behind = SearchTree::new(widgets.len());
    let mut found = BTreeMap::new();
    for (id, (root, members)) in components.iter().enumerate() {
        let root = *root;
        let inside = |widget: usize| component[widget] == id;
        let Some(last) = backward[root].iter().copied().find(|w| inside(*w)) else {
            continue; // no use inside the component reaches its root: no cycle
        };
        ahead.grow(root, &forward, inside);
        behind.grow(root, &backward, inside);
        for &member in members {
            let walk = closed_walk(member, root, last, &ahead, &behind);
            found.insert(
                widgets[member],
                walk.into_iter()
                    .map(|step| step.map_or("…", |index| widgets[index]))
                    .collect(),
            );
        }
    }
    found
}

/// Tarjan's strongly connected components, iterative: each component with its root (the first
/// member the search reached).
fn strongly_connected(forward: &[Vec<usize>]) -> Vec<(usize, Vec<usize>)> {
    let count = forward.len();
    let mut index = vec![UNSEEN; count];
    let mut low = vec![0; count];
    let mut on_stack = vec![false; count];
    let mut stack = Vec::new();
    let mut next = 0;
    let mut components = Vec::new();
    for start in 0..count {
        if index[start] != UNSEEN {
            continue;
        }
        index[start] = next;
        low[start] = next;
        next += 1;
        stack.push(start);
        on_stack[start] = true;
        let mut calls: Vec<(usize, usize)> = vec![(start, 0)];
        while let Some(frame) = calls.last_mut() {
            let widget = frame.0;
            if let Some(&used) = forward[widget].get(frame.1) {
                frame.1 += 1;
                if index[used] == UNSEEN {
                    index[used] = next;
                    low[used] = next;
                    next += 1;
                    stack.push(used);
                    on_stack[used] = true;
                    calls.push((used, 0));
                } else if on_stack[used] {
                    low[widget] = low[widget].min(index[used]);
                }
                continue;
            }
            calls.pop();
            if let Some(&(caller, _)) = calls.last() {
                low[caller] = low[caller].min(low[widget]);
            }
            if low[widget] == index[widget] {
                let mut members = Vec::new();
                while let Some(member) = stack.pop() {
                    on_stack[member] = false;
                    members.push(member);
                    if member == widget {
                        break;
                    }
                }
                components.push((widget, members));
            }
        }
    }
    components
}

/// A breadth-first tree from a component's root: each reached widget's parent toward the root.
struct SearchTree {
    parent: Vec<usize>,
}

impl SearchTree {
    fn new(count: usize) -> Self {
        Self {
            parent: vec![UNSEEN; count],
        }
    }

    fn grow(&mut self, root: usize, edges: &[Vec<usize>], inside: impl Fn(usize) -> bool) {
        self.parent[root] = root;
        let mut queue = std::collections::VecDeque::from([root]);
        while let Some(widget) = queue.pop_front() {
            for &next in &edges[widget] {
                if inside(next) && self.parent[next] == UNSEEN {
                    self.parent[next] = widget;
                    queue.push_back(next);
                }
            }
        }
    }

    /// `from`, then up to `SHOWN` of its parents toward the root; and whether
    /// the root was reached.
    fn toward_root(&self, from: usize, root: usize) -> (Vec<usize>, bool) {
        let mut steps = vec![from];
        let mut at = from;
        while at != root {
            if steps.len() > SHOWN {
                return (steps, false);
            }
            at = self.parent[at];
            steps.push(at);
        }
        (steps, true)
    }
}

/// A closed walk of uses from `member` back to itself, `None` marking elided hops: along uses
/// from `member` to `root` (`behind` holds each widget's next use toward the root), then from
/// `root` to `member` (`ahead` holds each widget's previous use from the root). For the root
/// itself the walk goes from it to `last`, a widget of its component using it, and back.
fn closed_walk(
    member: usize,
    root: usize,
    last: usize,
    ahead: &SearchTree,
    behind: &SearchTree,
) -> Vec<Option<usize>> {
    let (to_root, reached) = behind.toward_root(member, root);
    let mut walk: Vec<Option<usize>> = to_root.into_iter().map(Some).collect();
    let target = if member == root { last } else { member };
    let (mut from_root, complete) = ahead.toward_root(target, root);
    if complete {
        from_root.pop(); // the root, already written
    }
    if !(reached && complete) {
        walk.push(None);
    }
    walk.extend(from_root.into_iter().rev().map(Some));
    if member == root {
        walk.push(Some(root));
    }
    walk
}

/// `type_rule.primitives.duration.pattern`: `^[0-9]+(ms|s|m|h)$`.
fn is_duration(text: &str) -> bool {
    let digits = text.chars().take_while(char::is_ascii_digit).count();
    digits > 0 && ["ms", "s", "m", "h"].contains(&&text[digits..])
}

fn scalar_text(value: &Value) -> Option<String> {
    match value {
        Value::String(text) => Some(text.clone()),
        Value::Number(number) => Some(number.to_string()),
        Value::Bool(flag) => Some(flag.to_string()),
        _ => None,
    }
}

fn shown(value: &Value) -> String {
    match scalar_text(value) {
        Some(text) => format!("`{text}`"),
        None => match value {
            Value::Sequence(_) => "a list".to_owned(),
            Value::Mapping(_) => "a map".to_owned(),
            _ => "an absent value".to_owned(),
        },
    }
}

// ── degrades ─────────────────────────────────────────────────────────────────────────────────

fn degrades_known(sink: &mut Sink, path: &NodePath, common: &NodeCommon) {
    let capabilities = schema::capabilities();
    for (capability, fallback) in &common.degrades {
        match capabilities.get(capability) {
            None => sink.push(
                "degrades_known",
                &path.child("degrades"),
                format!(
                    "`{capability}` is not a capability; the capabilities are {}",
                    capabilities.keys().cloned().collect::<Vec<_>>().join(", ")
                ),
            ),
            Some(known) if !known.fallbacks.contains(fallback) => sink.push(
                "degrades_known",
                &path.child("degrades"),
                format!(
                    "`{fallback}` is not a fallback for `{capability}`, whose fallbacks are {}",
                    known.fallbacks.join(", ")
                ),
            ),
            Some(_) => {}
        }
    }
}

/// `Degrades.rule`: for each capability the target lacks, every construct using it degrades to
/// its own `degrades` entry or else the capability's first fallback, and `refuse` stops.
fn degrades_cover(located: &[Located<'_>], options: &Options, sink: &mut Sink) {
    let capabilities = schema::capabilities();
    for lack in &options.lacks {
        let Some(capability) = capabilities.get(lack) else {
            sink.push(
                "degrades_cover",
                &NodePath::root(),
                format!(
                    "the target is said to lack `{lack}`, which is not a capability; the \
                     capabilities are {}",
                    capabilities.keys().cloned().collect::<Vec<_>>().join(", ")
                ),
            );
            continue;
        };
        for node in located.iter().filter(|node| !in_declaration(&node.path)) {
            let (uses, declared) = uses_capability(lack, capability, node.node);
            if !uses {
                continue;
            }
            let fallback = declared
                .and_then(|degrades| degrades.get(lack))
                .or_else(|| capability.fallbacks.first());
            if fallback.is_none_or(|fallback| fallback == "refuse") {
                sink.push(
                    "degrades_cover",
                    &node.path,
                    format!("the target renderer lacks `{lack}`, and this construct degrades to `refuse`"),
                );
            }
        }
    }
}

/// Whether the node uses the capability, and the `degrades` it may declare a fallback in.
fn uses_capability<'a>(
    lack: &str,
    capability: &Capability,
    node: NodeRef<'a>,
) -> (bool, Option<&'a BTreeMap<String, String>>) {
    let applies = |construct: &str| capability.applies_to.iter().any(|c| c == construct);
    match node {
        NodeRef::Section(section) => (
            uses_body(lack, capability, &section.body)
                || (section.live.is_some() && applies("Live")),
            Some(&section.common.degrades),
        ),
        NodeRef::Node(node) => (
            uses_body(lack, capability, &node.body) || (node.live.is_some() && applies("Live")),
            Some(&node.common.degrades),
        ),
        NodeRef::Overlay(overlay) => (
            uses_body(lack, capability, &overlay.body)
                || (applies("overlay") && overlay.kind == OverlayKind::Drawer),
            Some(&overlay.common.degrades),
        ),
        NodeRef::Action(action) => (applies("Action") && action.upload.is_some(), None),
        NodeRef::Channel(_) => (applies("Channel"), None),
        NodeRef::Page(page) => (
            applies("PageLayout")
                && matches!(
                    (lack, &page.layout),
                    ("no_columns", PageLayout::Columns(_)) | ("no_areas", PageLayout::Areas(_))
                ),
            None,
        ),
        _ => (false, None),
    }
}

fn uses_body(lack: &str, capability: &Capability, body: &Body) -> bool {
    let Body::Composite(composite) = body else {
        return false;
    };
    if !capability
        .applies_to
        .iter()
        .any(|construct| construct == composite_kind(composite))
    {
        return false;
    }
    match (lack, composite) {
        ("no_file_upload", Composite::Form(form)) => {
            let tabs = form.tabs.iter().filter_map(|tab| match &tab.fields {
                Some(TabFields::Fields(fields)) => Some(fields.as_slice()),
                _ => None,
            });
            std::iter::once(form.fields.as_slice())
                .chain(form.groups.iter().map(|group| group.fields.as_slice()))
                .chain(tabs)
                .flatten()
                .any(|field| field.field_as.as_deref() == Some("file"))
        }
        ("no_file_upload", _) => false,
        _ => true,
    }
}
