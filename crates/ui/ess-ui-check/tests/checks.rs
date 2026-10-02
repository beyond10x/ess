//! Every check `ess ui check` runs, each tripped by the smallest document that breaks it.
//!
//! One test per check, named by the check's id: `every_check_has_a_test_named_by_its_id` reads
//! this file and fails when an id of [`ess_ui_check::CHECKS`] has no `fn <id>()` here, and
//! `every_schema_check_is_run` fails when the schema's `checks.list` names a check the crate does
//! not run, or gives it another severity.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use ess_ui_check::{
    check_source, load_model, model_from_sources, Finding, Model, Options, OutputFormat, Report,
    Severity, CHECKS,
};

const BASE: &[(&str, &str)] = &[
    ("format", "ess-ui/1"),
    ("app", "t"),
    ("model", "t.system"),
    ("placement_profile", "fat"),
    ("shells", "{app: {regions: {main: {kind: page_outlet}}}}"),
    (
        "navigation",
        "{home: p, sections: [{name: all, pages: [p]}]}",
    ),
    (
        "pages",
        "{p: {kind: detail_page, title: P, sections: [{name: summary, reads: t.ById}]}}",
    ),
];

/// The base document with some top-level keys replaced and others added, in order.
fn doc(over: &[(&str, &str)]) -> String {
    let mut text = String::new();
    for (key, value) in BASE {
        let value = over
            .iter()
            .find(|(replaced, _)| replaced == key)
            .map_or(*value, |(_, value)| *value);
        let _ = writeln!(text, "{key}: {value}");
    }
    for (key, value) in over {
        if !BASE.iter().any(|(base, _)| base == key) {
            let _ = writeln!(text, "{key}: {value}");
        }
    }
    text
}

/// The base document with page `p` replaced.
fn page(body: &str) -> String {
    doc(&[("pages", &format!("{{p: {body}}}"))])
}

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn report_with(text: &str, model: Option<&Model>, options: &Options) -> Report {
    check_source(text, "test.yaml", &crate_dir(), model, options)
}

fn report(text: &str) -> Report {
    report_with(text, None, &Options::default())
}

fn tripped<'a>(report: &'a Report, id: &str) -> Vec<&'a Finding> {
    report
        .findings
        .iter()
        .filter(|finding| finding.check == id)
        .collect()
}

/// Asserts `text` trips `id` at `path` with the check's severity, and returns that finding.
fn trips(text: &str, id: &str, path: &str) -> Finding {
    trips_in(&report(text), id, path)
}

fn trips_in(report: &Report, id: &str, path: &str) -> Finding {
    let found = tripped(report, id);
    let finding = found
        .iter()
        .find(|finding| finding.path == path)
        .unwrap_or_else(|| {
            panic!(
                "no `{id}` finding at `{path}`; findings: {:#?}",
                report.findings
            )
        });
    let declared = CHECKS
        .iter()
        .find(|check| check.id == id)
        .unwrap_or_else(|| panic!("`{id}` is not a check"));
    assert_eq!(finding.severity, declared.severity, "{finding:?}");
    (*finding).clone()
}

fn errors(report: &Report) -> Vec<&Finding> {
    report
        .findings
        .iter()
        .filter(|finding| finding.severity == Severity::Error)
        .collect()
}

// ── the base ─────────────────────────────────────────────────────────────────────────────────

#[test]
fn the_base_document_has_no_error() {
    let report = report(&doc(&[]));
    assert!(errors(&report).is_empty(), "{:#?}", report.findings);
    assert!(!report.has_errors());
}

// ── schema checks ────────────────────────────────────────────────────────────────────────────

#[test]
fn names_unique() {
    let text = page(
        "{kind: detail_page, title: P, sections: [{name: summary, reads: t.ById}, \
         {name: extra, component: record, reads: t.A}, {name: extra, component: record, reads: t.B}]}",
    );
    trips(&text, "names_unique", "pages/p/sections/extra");
    let pasted = "format: ess-ui/1\napp: t\nmodel: t.system\nplacement_profile: fat\n\
                  shells: {app: {regions: {main: {kind: page_outlet}}}}\n\
                  navigation: {home: p, sections: [{name: all, pages: [p]}]}\n\
                  pages:\n  p: {kind: detail_page, title: P, sections: [{name: a, reads: t.ById}]}\n  \
                  p: {kind: detail_page, title: Q, sections: [{name: b, reads: t.ById}]}\n";
    trips(pasted, "names_unique", "pages/p");
}

#[test]
fn nav_resolves() {
    let text = doc(&[(
        "navigation",
        "{home: ghost, sections: [{name: all, pages: [p, phantom]}], hidden: [spectre]}",
    )]);
    let report = report(&text);
    assert!(trips_in(&report, "nav_resolves", "navigation/home")
        .message
        .contains("ghost"));
    let listed = trips_in(&report, "nav_resolves", "navigation/sections/all/pages");
    assert!(listed.message.contains("phantom"), "{listed:?}");
    assert!(trips_in(&report, "nav_resolves", "navigation/hidden")
        .message
        .contains("spectre"));
}

#[test]
fn nav_unique() {
    let pages = (
        "pages",
        "{p: {kind: detail_page, title: P, sections: [{name: summary, reads: t.ById}]}, \
          q: {kind: detail_page, title: Q, sections: [{name: summary, reads: t.ById}]}}",
    );
    let twice_in_one = doc(&[
        pages,
        (
            "navigation",
            "{home: p, sections: [{name: all, pages: [p, q, p]}]}",
        ),
    ]);
    let finding = trips(&twice_in_one, "nav_unique", "navigation/sections/all/pages");
    assert!(finding.message.contains("`p`"), "{finding:?}");
    let across_two = doc(&[
        pages,
        (
            "navigation",
            "{home: p, sections: [{name: all, pages: [p]}, {name: more, pages: [q, p]}]}",
        ),
    ]);
    trips(&across_two, "nav_unique", "navigation/sections/more/pages");
    let once = doc(&[
        pages,
        (
            "navigation",
            "{home: p, sections: [{name: all, pages: [p]}, {name: more, pages: [q]}]}",
        ),
    ]);
    assert!(tripped(&report(&once), "nav_unique").is_empty());
}

#[test]
fn shell_refs() {
    let text = page(
        "{kind: detail_page, title: P, shell: nowhere, sections: [{name: summary, reads: t.ById}]}",
    );
    let finding = trips(&text, "shell_refs", "pages/p/shell");
    assert!(finding.message.contains("nowhere"), "{finding:?}");
    assert!(tripped(&report(&doc(&[])), "shell_refs").is_empty());
}

#[test]
fn page_outlet() {
    let text = doc(&[("shells", "{app: {regions: {menu: {kind: navigation}}}}")]);
    let finding = trips(&text, "page_outlet", "shells/app");
    assert!(finding.message.contains("`p`"), "{finding:?}");
    let unused = doc(&[(
        "shells",
        "{app: {regions: {main: {kind: page_outlet}}}, bare: {regions: {menu: {kind: navigation}}}}",
    )]);
    assert!(tripped(&report(&unused), "page_outlet").is_empty());
}

#[test]
fn page_reachable() {
    let text = doc(&[(
        "pages",
        "{p: {kind: detail_page, title: P, sections: [{name: summary, reads: t.ById}]}, \
          q: {kind: detail_page, title: Q, sections: [{name: summary, reads: t.ById}]}}",
    )]);
    trips(&text, "page_reachable", "pages/q");
    assert!(tripped(&report(&text), "page_reachable")
        .iter()
        .all(|finding| finding.path != "pages/p"));
}

#[test]
fn opens_resolves() {
    let text = page(
        "{kind: detail_page, title: P, overlays: {here: {kind: dialog, component: confirm, does: t.X}}, \
         sections: [{name: summary, reads: t.ById, \
         actions: [{name: go, opens: nowhere}, {name: fine, opens: here}]}]}",
    );
    let finding = trips(
        &text,
        "opens_resolves",
        "pages/p/sections/summary/actions/go",
    );
    assert!(finding.message.contains("nowhere"), "{finding:?}");
    assert!(tripped(&report(&text), "opens_resolves")
        .iter()
        .all(|finding| !finding.path.ends_with("/fine")));
}

#[test]
fn same_as_resolves() {
    let text = page(
        "{kind: detail_page, title: P, sections: [{name: summary, reads: t.ById}], \
         overlays: {edit: {kind: drawer, component: form, same_as: p.missing}}}",
    );
    trips(&text, "same_as_resolves", "pages/p/overlays/edit/same_as");
}

#[test]
fn page_refs() {
    let text = page(
        "{kind: detail_page, title: P, switch_to: [ghost], sections: [{name: summary, reads: t.ById, \
         actions: [{name: away, navigate: {to: nowhere}}]}]}",
    );
    let report = report(&text);
    trips_in(&report, "page_refs", "pages/p/switch_to");
    trips_in(
        &report,
        "page_refs",
        "pages/p/sections/summary/actions/away",
    );
}

#[test]
fn channel_refs() {
    let text = page(
        "{kind: detail_page, title: P, sections: [{name: summary, reads: t.ById, \
         live: {channel: ghost, effect: refetch}, visible: channel.phantom.up}]}",
    );
    let report = report(&text);
    trips_in(&report, "channel_refs", "pages/p/sections/summary/live");
    // An expression finding names the node the expression sits in, and the field in its message.
    let expression = trips_in(&report, "channel_refs", "pages/p/sections/summary");
    assert!(expression.message.contains("phantom"), "{expression:?}");
    assert!(
        expression.message.contains("in `visible`"),
        "{expression:?}"
    );
}

#[test]
fn section_refs() {
    let text = page(
        "{kind: detail_page, title: P, header: {total: ghost}, \
         sections: [{name: summary, reads: t.ById, depends_on: phantom}]}",
    );
    let report = report(&text);
    trips_in(
        &report,
        "section_refs",
        "pages/p/sections/summary/depends_on",
    );
    trips_in(&report, "section_refs", "pages/p/header/total");
}

#[test]
fn layout_complete() {
    let text = page(
        "{kind: detail_page, title: P, layout: {columns: [{name: main, sections: []}]}, \
         sections: [{name: summary, reads: t.ById}]}",
    );
    let finding = trips(&text, "layout_complete", "pages/p/layout");
    assert!(finding.message.contains("summary"), "{finding:?}");
}

#[test]
fn widget_expands() {
    let widgets = (
        "widgets",
        "{w: {summary: W, params: {x: {type: string, required: true, note: n}}, \
          body: [{name: t, primitive: text, text: args.x}]}}",
    );
    let pages = (
        "pages",
        "{p: {kind: detail_page, title: P, sections: [{name: summary, reads: t.ById, \
          children: [{name: use, component: w}]}]}}",
    );
    trips(
        &doc(&[widgets, pages]),
        "widget_expands",
        "pages/p/sections/summary/children/use",
    );
    let indirect = doc(&[(
        "widgets",
        "{a: {summary: A, body: [{name: inner, component: b}]}, \
          b: {summary: B, body: [{name: back, component: a}]}}",
    )]);
    let finding = trips(&indirect, "widget_expands", "widgets/a");
    assert!(finding.message.contains("a → b → a"), "{finding:?}");
    trips(&indirect, "widget_expands", "widgets/b");
    let unknown = doc(&[(
        "widgets",
        "{a: {summary: A, body: [{name: inner, component: nowhere}]}}",
    )]);
    trips(&unknown, "widget_expands", "widgets/a/body/inner");
    // args_match_param_types: a literal is checked against the param type; an expression is
    // not, since its type is only known at render time.
    let typed = (
        "widgets",
        "{w: {summary: W, params: {level: {type: {enum: [low, high]}, required: true, note: n}, \
          counts: {type: {list: integer}, note: n}, \
          span: {type: {record: {from: date, to: date}}, note: n}}, \
          body: [{name: t, primitive: text, text: args.level}]}}",
    );
    let using = |args: &str| {
        doc(&[
            typed,
            (
                "pages",
                &format!(
                    "{{p: {{kind: detail_page, title: P, sections: [{{name: summary, reads: t.ById, \
                     children: [{{name: use, component: w, args: {args}}}]}}]}}}}"
                ),
            ),
        ])
    };
    let at = "pages/p/sections/summary/children/use";
    for bad in [
        "{level: sideways}",
        "{level: low, counts: [1, two]}",
        "{level: low, span: {from: '2026-01-01', until: '2026-02-01'}}",
    ] {
        trips(&using(bad), "widget_expands", at);
    }
    for good in [
        "{level: high}",
        "{level: row.level}",
        "{level: low, counts: [1, 2], span: {from: '2026-01-01', to: state.until}}",
    ] {
        let report = report(&using(good));
        assert!(
            tripped(&report, "widget_expands").is_empty(),
            "{good}: {:#?}",
            report.findings
        );
    }
}

#[test]
fn a_chain_of_2000_widgets_is_checked_in_under_5_seconds() {
    let mut widgets = String::from("{");
    for index in 0..2000 {
        let body = if index == 1999 {
            "[{name: leaf, primitive: text, text: T}]".to_owned()
        } else {
            format!("[{{name: next, component: w{}}}]", index + 1)
        };
        let _ = write!(widgets, "w{index}: {{summary: W, body: {body}}}, ");
    }
    widgets.push_str("loop: {summary: L, body: [{name: again, component: loop}]}}");
    let text = doc(&[("widgets", &widgets)]);
    let started = std::time::Instant::now();
    let report = report(&text);
    let took = started.elapsed();
    let cycles: Vec<&str> = tripped(&report, "widget_expands")
        .iter()
        .map(|finding| finding.path.as_str())
        .collect();
    assert_eq!(cycles, ["widgets/loop"]);
    assert!(took < std::time::Duration::from_secs(5), "took {took:?}");
}

/// `w0` uses `w1` twice, `w1` uses `w2` twice, …: `2^depth` widget bodies once expanded.
fn doubling_widgets(depth: usize) -> String {
    let mut widgets = String::from("{");
    for index in 0..depth {
        let next = index + 1;
        let _ = write!(
            widgets,
            "w{index}: {{summary: W, body: [{{name: a, component: w{next}}}, \
             {{name: b, component: w{next}}}]}}, "
        );
    }
    let _ = write!(
        widgets,
        "w{depth}: {{summary: W, body: [{{name: leaf, primitive: text, text: T}}]}}}}"
    );
    doc(&[
        ("widgets", &widgets),
        (
            "pages",
            "{p: {kind: detail_page, title: P, sections: [{name: summary, reads: t.ById, \
              children: [{name: use, component: w0}]}]}}",
        ),
    ])
}

#[test]
fn a_widget_doubling_at_each_of_64_levels_is_refused_in_under_5_seconds() {
    let started = std::time::Instant::now();
    let report = report(&doubling_widgets(64));
    let took = started.elapsed();
    let finding = trips_in(
        &report,
        "widget_expands",
        "pages/p/sections/summary/children/use",
    );
    assert!(finding.message.contains("exceeds"), "{finding:?}");
    assert!(took < std::time::Duration::from_secs(5), "took {took:?}");
}

#[test]
fn a_widget_doubling_at_each_of_8_levels_is_checked_clean() {
    let report = report(&doubling_widgets(8));
    assert!(errors(&report).is_empty(), "{:#?}", report.findings);
}

#[test]
fn primitive_props() {
    let both = page(
        "{kind: detail_page, title: P, sections: [{name: summary, reads: t.ById, \
         children: [{name: out, primitive: link, text: Out, to: {to: p}, href: row.url}]}]}",
    );
    trips(
        &both,
        "primitive_props",
        "pages/p/sections/summary/children/out",
    );
    let neither = page(
        "{kind: detail_page, title: P, sections: [{name: summary, reads: t.ById, \
         children: [{name: pill, primitive: badge, tone: info}]}]}",
    );
    trips(
        &neither,
        "primitive_props",
        "pages/p/sections/summary/children/pill",
    );
    let foreign = page(
        "{kind: detail_page, title: P, sections: [{name: summary, reads: t.ById, \
         children: [{name: caption, primitive: text, text: T, icon: star}]}]}",
    );
    trips(
        &foreign,
        "primitive_props",
        "pages/p/sections/summary/children/caption",
    );
}

#[test]
fn unbound_placeholder() {
    let text = page(
        "{kind: detail_page, title: P, sections: [{name: summary, \
         reads: {placeholder: t.Later, fixture: fixtures/later.yaml}}]}",
    );
    trips(
        &text,
        "unbound_placeholder",
        "pages/p/sections/summary/reads",
    );
}

#[test]
fn fixture_per_view() {
    trips(
        &doc(&[]),
        "fixture_per_view",
        "pages/p/sections/summary/reads",
    );
    let answered = doc(&[("fixtures", "{views: {t.ById: rows.yaml}}")]);
    assert!(tripped(&report(&answered), "fixture_per_view").is_empty());
    let derived = doc(&[("fixtures", "{derived: {t.ById: {same_as: t.All}}}")]);
    assert!(tripped(&report(&derived), "fixture_per_view").is_empty());
}

#[test]
fn script_per_channel() {
    let channel = (
        "channels",
        "{feed: {carries: {events: [t.Happened]}, direction: server_to_client, \
          delivery: every_event, resume: refetch}}",
    );
    trips(&doc(&[channel]), "script_per_channel", "channels/feed");
    let scripted = doc(&[channel, ("fixtures", "{scripts: {feed: feed.yaml}}")]);
    assert!(tripped(&report(&scripted), "script_per_channel").is_empty());
}

#[test]
fn state_resolves() {
    let unresolved = doc(&[
        ("placement_profile", "hybrid"),
        (
            "pages",
            "{p: {kind: detail_page, title: P, state: {picked: {type: string, class: selection}}, \
              sections: [{name: summary, reads: t.ById}]}}",
        ),
    ]);
    trips(&unresolved, "state_resolves", "pages/p/state/picked");
    let fallback = page(
        "{kind: detail_page, title: P, sections: [{name: summary, reads: t.ById}], \
         state: {token: {type: string, class: credential, store: memory, \
         fallback: {when: actor.signed_in, store: local_storage}}}}",
    );
    trips(&fallback, "state_resolves", "pages/p/state/token/fallback");
    let by_profile = doc(&[
        ("placement_profile", "hybrid"),
        ("placement_defaults", "{preference: local_storage}"),
        (
            "pages",
            "{p: {kind: detail_page, title: P, \
              state: {secret: {type: string, class: preference, sensitive: true}}, \
              sections: [{name: summary, reads: t.ById}]}}",
        ),
    ]);
    trips(&by_profile, "state_resolves", "pages/p/state/secret");
}

#[test]
fn types_structural() {
    let text = page(
        "{kind: detail_page, title: P, params: {id: 'list<ItemId>'}, \
         sections: [{name: summary, reads: t.ById}]}",
    );
    trips(&text, "types_structural", "pages/p/params/id");
    let declared = doc(&[("types", "{Stage: 'enum of a, b'}")]);
    trips(&declared, "types_structural", "types/Stage");
    let fine = doc(&[(
        "types",
        "{Stage: {enum: [a, b]}, Label: string, Ref: ItemId}",
    )]);
    assert!(tripped(&report(&fine), "types_structural").is_empty());
}

#[test]
fn unmapped_reported() {
    let text = page(
        "{kind: detail_page, title: P, unmapped: [the old page polled], \
         sections: [{name: summary, reads: t.ById, load: 'UNMAPPED: nobody knows'}]}",
    );
    let report = report(&text);
    let marker = trips_in(&report, "unmapped_reported", "pages/p/sections/summary");
    assert!(marker.message.contains("in `load`"), "{marker:?}");
    let gap = trips_in(&report, "unmapped_reported", "pages/p");
    assert!(gap.message.contains("in `unmapped`"), "{gap:?}");
    assert!(!report.has_errors(), "{:#?}", report.findings);
}

// ── style: tokens, themes, the theme choice, tone maps ──────────────────────────────────────

/// No finding of `id` at `path`.
fn clean_at(report: &Report, id: &str, path: &str) {
    let found: Vec<&Finding> = tripped(report, id)
        .into_iter()
        .filter(|finding| finding.path == path)
        .collect();
    assert!(
        found.is_empty(),
        "unexpected `{id}` at `{path}`: {found:#?}"
    );
}

/// The base page with a badge whose tone is picked by `tone_by`.
fn badge_page(tone_by: &str) -> String {
    format!(
        "{{p: {{kind: detail_page, title: P, sections: [{{name: summary, reads: t.ById, \
         children: [{{name: state, primitive: badge, field: state, tone_by: {tone_by}}}]}}]}}}}"
    )
}

/// A shell whose `theme` state is `state`.
fn theme_shell(state: &str) -> String {
    format!("{{app: {{regions: {{main: {{kind: page_outlet}}}}, state: {{theme: {state}}}}}}}")
}

const THEME_STATE: &str =
    "{type: {enum: [light, dark]}, class: preference, store: local_storage, pinned: true, default: light}";

#[test]
fn token_values() {
    let text = doc(&[(
        "tokens",
        "{color: {surface: 'hsl(0 0% 100%)', ink: red, short: '#12345', four: '#abcd', \
         bright: 'rgb(256 0 0 / 1)', opaque: 'rgb(0 0 0)', ok: '#a1b2c3', veil: 'rgb(0 0 0 / 0.3)', \
         hex8: '#a1b2c3d4', tiny: '#fff'}, \
         space: {md: 1rem, big: 2, wide: 3vw, none: 0, half: 0.5em}, \
         radius: {sm: 4px, odd: 4, neg: -4px}, \
         type: {heading: {size: large, weight: 650}, body: {weight: 1000}, caption: {weight: 300, size: 0.8rem}}}",
    )]);
    let written = report(&text);
    for path in [
        "tokens/color/surface",
        "tokens/color/ink",
        "tokens/color/short",
        "tokens/color/four",
        "tokens/color/bright",
        "tokens/color/opaque",
        "tokens/space/big",
        "tokens/space/wide",
        "tokens/radius/odd",
        "tokens/radius/neg",
        "tokens/type/heading/size",
        "tokens/type/heading/weight",
        "tokens/type/body/weight",
    ] {
        trips_in(&written, "token_values", path);
    }
    for path in [
        "tokens/color/ok",
        "tokens/color/veil",
        "tokens/color/hex8",
        "tokens/color/tiny",
        "tokens/space/md",
        "tokens/space/none",
        "tokens/space/half",
        "tokens/radius/sm",
        "tokens/type/caption/weight",
        "tokens/type/caption/size",
    ] {
        clean_at(&written, "token_values", path);
    }

    // The built-in table, restated as a document's `tokens:`, is in its own grammar.
    let schema: serde_yaml::Value = serde_yaml::from_str(ess_ui::SCHEMA).expect("schema");
    let builtins = serde_yaml::to_string(&schema["constructs"]["Tokens"]["builtins"])
        .expect("the built-in table writes");
    let mut restated = doc(&[]);
    restated.push_str("tokens:\n");
    for line in builtins.lines() {
        let _ = writeln!(restated, "  {line}");
    }
    let restated = report(&restated);
    assert!(errors(&restated).is_empty(), "{:#?}", restated.findings);
}

#[test]
fn token_names() {
    let text = doc(&[(
        "tokens",
        "{type: {subtitle: {size: 1rem}, heading: {size: 1.2rem}}, \
         tone: {critical: {text: danger, fill: danger_fill}, danger: {text: danger, fill: danger_fill}}}",
    )]);
    let report = report(&text);
    trips_in(&report, "token_names", "tokens/type/subtitle");
    trips_in(&report, "token_names", "tokens/tone/critical");
    clean_at(&report, "token_names", "tokens/type/heading");
    clean_at(&report, "token_names", "tokens/tone/danger");
}

#[test]
fn token_refs() {
    let text = doc(&[(
        "tokens",
        "{color: {alarm: '#ff0000'}, \
         tone: {danger: {text: alarm, fill: blood}, info: {text: info, fill: line}}}",
    )]);
    let report = report(&text);
    let finding = trips_in(&report, "token_refs", "tokens/tone/danger/fill");
    assert!(finding.message.contains("blood"), "{finding:?}");
    clean_at(&report, "token_refs", "tokens/tone/danger/text");
    clean_at(&report, "token_refs", "tokens/tone/info/text");
    clean_at(&report, "token_refs", "tokens/tone/info/fill");
}

#[test]
fn theme_tokens() {
    let text = doc(&[
        ("tokens", "{color: {brand: '#123456'}}"),
        (
            "themes",
            "{light: {}, dark: {color: {surface: '#1f2024', brand: '#654321', glow: '#ffffff', \
             text: crimson}, space: {huge: 3rem, md: 1}, radius: {sm: 2px}, \
             type: {subtitle: {size: 1rem}, heading: {weight: 50}}, \
             tone: {danger: {text: danger, fill: abyss}, info: {text: brand, fill: line}}}}",
        ),
    ]);
    let report = report(&text);
    for path in [
        "themes/dark/color/glow",
        "themes/dark/color/text",
        "themes/dark/space/huge",
        "themes/dark/space/md",
        "themes/dark/type/subtitle",
        "themes/dark/type/heading/weight",
        "themes/dark/tone/danger/fill",
    ] {
        trips_in(&report, "theme_tokens", path);
    }
    for path in [
        "themes/dark/color/surface",
        "themes/dark/color/brand",
        "themes/dark/radius/sm",
        "themes/dark/tone/info/text",
    ] {
        clean_at(&report, "theme_tokens", path);
    }
    assert!(
        tripped(&report, "token_values").is_empty(),
        "{:#?}",
        report.findings
    );
}

#[test]
fn theme_choice() {
    let themes = ("themes", "{light: {}, dark: {color: {surface: '#1f2024'}}}");
    let fine = doc(&[
        ("shells", &theme_shell(THEME_STATE)),
        themes,
        ("theme", "{default: light, chosen_by: shell.theme}"),
    ]);
    let report_fine = report(&fine);
    assert!(
        tripped(&report_fine, "theme_choice").is_empty(),
        "{:#?}",
        report_fine.findings
    );
    let named = doc(&[
        ("types", "{Look: {enum: [light, dark]}}"),
        (
            "shells",
            &theme_shell("{type: Look, class: preference, default: dark}"),
        ),
        themes,
        ("theme", "{default: light, chosen_by: shell.theme}"),
    ]);
    assert!(tripped(&report(&named), "theme_choice").is_empty());

    trips(
        &doc(&[("theme", "{default: light}")]),
        "theme_choice",
        "theme",
    );
    trips(
        &doc(&[themes, ("theme", "{default: sepia}")]),
        "theme_choice",
        "theme/default",
    );
    for chosen_by in ["state.theme", "shell.theme.mode", "shell.palette"] {
        let text = doc(&[
            ("shells", &theme_shell(THEME_STATE)),
            themes,
            (
                "theme",
                &format!("{{default: light, chosen_by: {chosen_by}}}"),
            ),
        ]);
        trips(&text, "theme_choice", "theme/chosen_by");
    }
    for (state, path) in [
        (
            "{type: {enum: [light, dark]}, class: component_state}",
            "shells/app/state/theme",
        ),
        (
            "{type: boolean, class: preference}",
            "shells/app/state/theme/type",
        ),
        (
            "{type: {enum: [light, sepia]}, class: preference}",
            "shells/app/state/theme/type",
        ),
        (
            "{type: {enum: [light, dark]}, class: preference, default: sepia}",
            "shells/app/state/theme/default",
        ),
    ] {
        let text = doc(&[
            ("shells", &theme_shell(state)),
            themes,
            ("theme", "{default: light, chosen_by: shell.theme}"),
        ]);
        trips(&text, "theme_choice", path);
    }
}

#[test]
fn tone_map_refs() {
    let text = doc(&[
        ("tone_maps", "{job_state: {Done: success}}"),
        ("pages", &badge_page("{value: row.state, tones: ghost}")),
    ]);
    let finding = trips(
        &text,
        "tone_map_refs",
        "pages/p/sections/summary/children/state/tone_by/tones",
    );
    assert!(finding.message.contains("ghost"), "{finding:?}");
    let fine = doc(&[
        ("tone_maps", "{job_state: {Done: success}}"),
        ("pages", &badge_page("{value: row.state, tones: job_state}")),
    ]);
    let report = report(&fine);
    assert!(errors(&report).is_empty(), "{:#?}", report.findings);
}

#[test]
fn tone_map_unused() {
    let text = doc(&[
        (
            "tone_maps",
            "{job_state: {Done: success}, deal_stage: {won: success}}",
        ),
        ("pages", &badge_page("{value: row.state, tones: job_state}")),
    ]);
    let report = report(&text);
    trips_in(&report, "tone_map_unused", "tone_maps/deal_stage");
    clean_at(&report, "tone_map_unused", "tone_maps/job_state");
    assert!(!report.has_errors(), "{:#?}", report.findings);
}

// ── rules the schema states outside `checks.list` ───────────────────────────────────────────

#[test]
fn document_loads() {
    let text = page("{kind: no_such_kind, title: P, sections: [{name: summary, reads: t.ById}]}");
    trips(&text, "document_loads", "pages/p/kind");
    let broken = report("format: ess-ui/1\npages: [\n");
    assert!(
        tripped(&broken, "document_loads").len() == 1,
        "{:#?}",
        broken.findings
    );
}

#[test]
fn layer_rules() {
    let text = page(
        "{kind: detail_page, title: P, sections: [{name: summary, reads: t.ById}, \
         {name: loose, primitive: text, text: T}]}",
    );
    trips(&text, "layer_rules", "pages/p/sections/loose");
}

#[test]
fn enum_values() {
    let text = page(
        "{kind: detail_page, title: P, sections: [{name: summary, reads: t.ById}, \
         {name: rows, component: collection, reads: t.Rows, \
          columns: [{field: state, as: tag}, {field: stage, as: badge}, {field: owner, as: 'UNMAPPED: unknown'}]}, \
         {name: totals, component: chart, chart: donut, reads: t.Totals}, \
         {name: bars, component: chart, chart: bar, reads: t.Totals}, \
         {name: logo, component: record, reads: t.ById, \
          children: [{name: pic, primitive: image, src: row.logo, alt: Logo, fit: stretch}, \
                     {name: rule, primitive: divider, orientation: diagonal}]}]}",
    );
    let report = report(&text);
    let finding = trips_in(
        &report,
        "enum_values",
        "pages/p/sections/rows/columns/state",
    );
    assert!(
        finding.message.contains("`tag`") && finding.message.contains("badge"),
        "{finding:?}"
    );
    trips_in(&report, "enum_values", "pages/p/sections/totals");
    trips_in(&report, "enum_values", "pages/p/sections/logo/children/pic");
    trips_in(
        &report,
        "enum_values",
        "pages/p/sections/logo/children/rule",
    );
    let reported: Vec<&str> = tripped(&report, "enum_values")
        .iter()
        .map(|finding| finding.path.as_str())
        .collect();
    assert_eq!(reported.len(), 4, "{reported:?}");
}

#[test]
fn icon_tone_map_values_are_checked_like_badge_values() {
    let text = page("{kind: detail_page, title: P, sections: [{name: summary, component: record, reads: t.ById, children: [{name: mark, primitive: icon, icon: star, label: Star, tone_by: {value: row.state, map: {open: misspelled}}}]}]}");
    let checked = report(&text);
    trips_in(
        &checked,
        "enum_values",
        "pages/p/sections/summary/children/mark",
    );
}

#[test]
fn degrades_known() {
    let text = page(
        "{kind: detail_page, title: P, sections: [{name: summary, reads: t.ById, \
         degrades: {no_chartz: table}}]}",
    );
    let finding = trips(&text, "degrades_known", "pages/p/sections/summary/degrades");
    assert!(finding.message.contains("no_chartz"), "{finding:?}");
    let wrong_fallback = page(
        "{kind: detail_page, title: P, sections: [{name: summary, reads: t.ById, \
         degrades: {no_charts: hologram}}]}",
    );
    trips(
        &wrong_fallback,
        "degrades_known",
        "pages/p/sections/summary/degrades",
    );
}

#[test]
fn degrades_cover() {
    let text = page(
        "{kind: detail_page, title: P, sections: [{name: summary, reads: t.ById, \
         actions: [{name: up, upload: {accept: [text/csv], does: t.Import}}]}]}",
    );
    let lacking = Options {
        lacks: vec!["no_file_upload".to_owned()],
    };
    let lacks = report_with(&text, None, &lacking);
    trips_in(
        &lacks,
        "degrades_cover",
        "pages/p/sections/summary/actions/up",
    );
    assert!(tripped(&report(&text), "degrades_cover").is_empty());
    let covered = page(
        "{kind: detail_page, title: P, sections: [{name: summary, component: graph_editor, \
         reads: t.Graph}]}",
    );
    let graph = Options {
        lacks: vec!["no_graph_editor".to_owned()],
    };
    assert!(tripped(&report_with(&covered, None, &graph), "degrades_cover").is_empty());
    let refused = page(
        "{kind: detail_page, title: P, sections: [{name: summary, component: graph_editor, \
         reads: t.Graph, degrades: {no_graph_editor: refuse}}]}",
    );
    trips_in(
        &report_with(&refused, None, &graph),
        "degrades_cover",
        "pages/p/sections/summary",
    );
    let unknown = Options {
        lacks: vec!["no_teleport".to_owned()],
    };
    trips_in(
        &report_with(&doc(&[]), None, &unknown),
        "degrades_cover",
        "/",
    );
}

// ── model checks ─────────────────────────────────────────────────────────────────────────────

fn model() -> Model {
    load_model(&crate_dir().join("tests/fixtures/model")).unwrap_or_else(|error| panic!("{error}"))
}

fn model_report(pages: &str, channels: &str) -> Report {
    let text = doc(&[("model", "shop"), ("pages", pages), ("channels", channels)]);
    report_with(&text, Some(&model()), &Options::default())
}

const QUIET: &str = "{}";

/// A graph editor's `edges.reads` is a read like the graph's own: its view must be in the model
/// and answered by a fixture.
#[test]
fn a_graph_edge_read_is_checked_like_the_graph_read() {
    let pages =
        "{p: {kind: detail_page, title: P, sections: [{name: summary, reads: stock.Items}, \
                 {name: graph, component: graph_editor, reads: stock.Items, \
                 nodes: {key: item_id, label: label}, \
                 edges: {reads: {view: stock.Missing}, from: from_id, to: to_id}}]}}";
    let report = model_report(pages, QUIET);
    trips_in(
        &report,
        "view_in_model",
        "pages/p/sections/graph/edges/reads",
    );
    trips_in(
        &report,
        "fixture_per_view",
        "pages/p/sections/graph/edges/reads",
    );
}

#[test]
fn view_in_model() {
    let report = model_report(
        "{p: {kind: detail_page, title: P, sections: [{name: summary, reads: stock.Items}, \
          {name: gone, component: collection, reads: stock.Missing}, \
          {name: full, component: collection, reads: shop.stock.Items}]}}",
        QUIET,
    );
    trips_in(&report, "view_in_model", "pages/p/sections/gone/reads");
    let flagged = tripped(&report, "view_in_model");
    assert_eq!(flagged.len(), 1, "{flagged:#?}");
}

#[test]
fn command_in_model() {
    let report = model_report(
        "{p: {kind: detail_page, title: P, sections: [{name: summary, reads: stock.Items, \
          actions: [{name: add, does: stock.AddItem}, {name: drop, does: stock.RemoveItem}]}]}}",
        QUIET,
    );
    trips_in(
        &report,
        "command_in_model",
        "pages/p/sections/summary/actions/drop",
    );
    assert_eq!(tripped(&report, "command_in_model").len(), 1);
}

#[test]
fn event_in_model() {
    let report = model_report(
        "{p: {kind: detail_page, title: P, sections: [{name: summary, reads: stock.Items, \
          live: {channel: stock, on: [stock.ItemAdded, stock.ItemGone], effect: refetch}}]}}",
        "{stock: {carries: {events: [stock.ItemAdded, stock.ItemSold]}, direction: server_to_client, \
          delivery: every_event, resume: refetch}, \
          lens: {carries: {view: stock.Lens}, direction: server_to_client, \
          delivery: latest_value, resume: refetch}}",
    );
    let live = trips_in(&report, "event_in_model", "pages/p/sections/summary/live");
    assert!(live.message.contains("stock.ItemGone"), "{live:?}");
    let carried = trips_in(&report, "event_in_model", "channels/stock/carries");
    assert!(carried.message.contains("stock.ItemSold"), "{carried:?}");
    trips_in(&report, "view_in_model", "channels/lens/carries");
}

#[test]
fn type_in_model() {
    let report = model_report(
        "{p: {kind: detail_page, title: P, \
          params: {item: ItemId, full: shop.stock.ItemId, text: String, other: NoSuchTypeId, \
                   many: {list: NoSuchRow}}, \
          sections: [{name: summary, reads: stock.Items}]}}",
        QUIET,
    );
    let other = trips_in(&report, "type_in_model", "pages/p/params/other");
    assert!(other.message.contains("NoSuchTypeId"), "{other:?}");
    trips_in(&report, "type_in_model", "pages/p/params/many");
    assert_eq!(tripped(&report, "type_in_model").len(), 2, "{report:#?}");
    let declared = doc(&[
        ("model", "shop"),
        ("types", "{Window: {enum: [day, week]}}"),
        (
            "pages",
            "{p: {kind: detail_page, title: P, params: {window: Window}, \
              sections: [{name: summary, reads: stock.Items}]}}",
        ),
    ]);
    let report = report_with(&declared, Some(&model()), &Options::default());
    assert!(tripped(&report, "type_in_model").is_empty(), "{report:#?}");
}

#[test]
fn field_in_model() {
    let report = model_report(
        "{p: {kind: detail_page, title: P, sections: [\
          {name: summary, component: collection, reads: stock.Items, \
           columns: [label, item_id, sandbox_bogus], \
           row_actions: [{name: add, does: stock.AddItem, bind: {label: row.label, lable: row.label}}, \
                         {name: peek, opens: nowhere_needed, visible: row.labelX == x}]}, \
          {name: entry, component: form, does: stock.AddItem, fields: [label, resolutoin]}, \
          {name: one, component: record, reads: stock.Items, fields: [label, labl]}]}}",
        QUIET,
    );
    let at = |suffix: &str| format!("pages/p/sections/{suffix}");
    let column = trips_in(
        &report,
        "field_in_model",
        &at("summary/columns/sandbox_bogus"),
    );
    assert!(column.message.contains("shop.stock.Items"), "{column:?}");
    let bind = trips_in(&report, "field_in_model", &at("summary/row_actions/add"));
    assert!(bind.message.contains("`lable`"), "{bind:?}");
    let visible = trips_in(&report, "field_in_model", &at("summary/row_actions/peek"));
    assert!(visible.message.contains("labelX"), "{visible:?}");
    let input = trips_in(&report, "field_in_model", &at("entry/fields/resolutoin"));
    assert!(input.message.contains("shop.stock.AddItem"), "{input:?}");
    trips_in(&report, "field_in_model", &at("one/fields/labl"));
    assert_eq!(
        tripped(&report, "field_in_model").len(),
        5,
        "{:#?}",
        tripped(&report, "field_in_model")
    );
}

#[test]
fn overlay_params() {
    let text = page(
        "{kind: detail_page, title: P, sections: [{name: summary, reads: t.ById}], \
         overlays: {edit: {kind: drawer, component: record, \
           reads: {view: t.ById, params: {id: params.item_id}}, \
           params: {item_id: row.id, iten_id: row.id}}}}",
    );
    let report = report(&text);
    let finding = trips_in(&report, "overlay_params", "pages/p/overlays/edit");
    assert!(finding.message.contains("`params.iten_id`"), "{finding:?}");
    assert_eq!(tripped(&report, "overlay_params").len(), 1, "{report:#?}");
}

#[test]
fn section_readable() {
    let report = model_report(
        "{p: {kind: detail_page, title: P, sections: [{name: summary, reads: stock.Items}, \
          {name: audit, component: collection, reads: audit.Entries}]}}",
        QUIET,
    );
    let finding = trips_in(&report, "section_readable", "pages/p/sections/audit");
    assert!(
        finding.message.contains("shop.audit.Entries"),
        "{finding:?}"
    );
    assert!(finding.message.contains("approximation"), "{finding:?}");
    let anonymous = doc(&[
        ("model", "shop"),
        ("actor", "anonymous"),
        (
            "pages",
            "{p: {kind: detail_page, title: P, sections: [{name: summary, reads: stock.Items}, \
              {name: audit, component: collection, reads: audit.Entries}]}}",
        ),
    ]);
    let public = report_with(&anonymous, Some(&model()), &Options::default());
    assert!(
        tripped(&public, "section_readable").is_empty(),
        "an anonymous document reads without grants: {:#?}",
        public.findings
    );
    assert_eq!(tripped(&report, "section_readable").len(), 1);
}

/// The fixture model with one more view, `shop.stock.Labelled`, that declares a required and an
/// optional parameter.
fn param_model() -> Model {
    let root = crate_dir().join("tests/fixtures/model");
    let read = |file: &str| {
        std::fs::read_to_string(root.join(file)).unwrap_or_else(|error| panic!("{file}: {error}"))
    };
    let mut stock = read("domains/stock.yaml");
    stock.push_str(
        "\n  - name: shop.stock.Labelled\n    source: shop.stock.Item\n    consistency: \
         read_your_writes\n    params:\n      - name: wanted\n        type: String\n      - \
         name: hint\n        type: Optional<String>\n    filter: {any: [label == param.wanted, \
         label == param.hint]}\n    fields:\n      - name: item_id\n        type: \
         shop.stock.ItemId\n",
    );
    let sources = [
        ("system.yaml", read("system.yaml")),
        ("components.yaml", read("components.yaml")),
        ("domains/stock.yaml", stock),
        ("domains/audit.yaml", read("domains/audit.yaml")),
    ]
    .map(|(label, text)| (label.to_owned(), text));
    model_from_sources(&sources, Path::new("shop")).unwrap_or_else(|error| panic!("{error}"))
}

fn param_report(reads: &str) -> Report {
    let text = doc(&[
        ("model", "shop"),
        (
            "pages",
            &format!(
                "{{p: {{kind: detail_page, title: P, params: {{q: string}}, sections: \
                 [{{name: summary, reads: stock.Items}}, \
                 {{name: labelled, component: collection, reads: {reads}}}]}}}}"
            ),
        ),
    ]);
    report_with(&text, Some(&param_model()), &Options::default())
}

#[test]
fn read_params() {
    // Every declared parameter bound, the optional one too, and nothing else: nothing to report.
    let report = param_report("{view: stock.Labelled, params: {wanted: params.q, hint: params.q}}");
    assert!(
        tripped(&report, "read_params").is_empty(),
        "{:#?}",
        report.findings
    );
    // The optional parameter may be left unbound.
    let report = param_report("{view: stock.Labelled, params: {wanted: params.q}}");
    assert!(
        tripped(&report, "read_params").is_empty(),
        "{:#?}",
        report.findings
    );
}

#[test]
fn a_read_binding_an_undeclared_view_param_is_an_error() {
    let report = param_report("{view: stock.Labelled, params: {wanted: params.q, nope: params.q}}");
    let finding = trips_in(&report, "read_params", "pages/p/sections/labelled/reads");
    assert_eq!(finding.severity, Severity::Error);
    assert!(finding.message.contains("`nope`"), "{finding:?}");
    assert!(
        finding.message.contains("shop.stock.Labelled"),
        "{finding:?}"
    );
    assert_eq!(tripped(&report, "read_params").len(), 1);
    // A view that declares no parameter at all is bound with none.
    let report = param_report("{view: stock.Items, params: {wanted: params.q}}");
    trips_in(&report, "read_params", "pages/p/sections/labelled/reads");
}

#[test]
fn an_unbound_required_view_param_is_an_error() {
    let report = param_report("{view: stock.Labelled, params: {hint: params.q}}");
    let finding = trips_in(&report, "read_params", "pages/p/sections/labelled/reads");
    assert_eq!(finding.severity, Severity::Error);
    assert!(finding.message.contains("`wanted`"), "{finding:?}");
    assert!(!finding.message.contains("`hint`"), "{finding:?}");
    assert_eq!(tripped(&report, "read_params").len(), 1);
}

#[test]
fn a_confirm_referencing_a_view_with_a_required_param_is_not_an_error() {
    // A confirm's `references` has no `params:` slot; the confirm's context may supply them, and
    // the specification does not say how, so nothing is reported unbound there.
    let text = doc(&[
        ("model", "shop"),
        (
            "pages",
            "{p: {kind: detail_page, title: P, sections: [{name: summary, reads: stock.Items}], \
             overlays: {sure: {kind: dialog, component: confirm, does: stock.AddItem, \
             references: stock.Labelled}}}}",
        ),
    ]);
    let report = report_with(&text, Some(&param_model()), &Options::default());
    assert!(
        tripped(&report, "view_in_model").is_empty(),
        "the reference resolves: {:#?}",
        report.findings
    );
    assert!(
        tripped(&report, "read_params").is_empty(),
        "{:#?}",
        report.findings
    );
}

#[test]
fn a_model_that_does_not_compile_is_refused() {
    let error = load_model(&crate_dir().join("tests/fixtures/no-such-model"))
        .expect_err("a missing model is refused");
    assert!(!error.to_string().is_empty());
}

// ── paths, output and the check list itself ──────────────────────────────────────────────────

#[test]
fn a_finding_keeps_its_path_when_a_sibling_is_inserted() {
    let before = page(
        "{kind: detail_page, title: P, sections: [{name: summary, reads: t.ById, \
         actions: [{name: go, opens: nowhere}]}]}",
    );
    let after = page(
        "{kind: detail_page, title: P, sections: [{name: first, component: record, reads: t.A, \
         actions: [{name: other, opens: elsewhere}]}, {name: summary, reads: t.ById, \
         actions: [{name: before, sets: {state.x: '1'}}, {name: go, opens: nowhere}]}]}",
    );
    let path = "pages/p/sections/summary/actions/go";
    let old = trips(&before, "opens_resolves", path);
    let new = trips(&after, "opens_resolves", path);
    assert_eq!(old, new);
}

#[test]
fn json_output_is_ess_ui_check_1() {
    let report = report(&doc(&[(
        "navigation",
        "{home: ghost, sections: [{name: all, pages: [p]}]}",
    )]));
    let json: serde_json::Value =
        serde_json::from_str(&report.render(OutputFormat::Json)).expect("the output is JSON");
    assert_eq!(json["format"], "ess-ui-check/1");
    assert_eq!(json["document"], "test.yaml");
    let findings = json["findings"].as_array().expect("findings is a list");
    assert!(!findings.is_empty());
    for finding in findings {
        let mut keys: Vec<&str> = finding
            .as_object()
            .expect("a finding is an object")
            .keys()
            .map(String::as_str)
            .collect();
        keys.sort_unstable();
        assert_eq!(keys, ["check", "message", "path", "severity"], "{finding}");
        assert!(matches!(
            finding["severity"].as_str(),
            Some("error" | "warning")
        ));
    }
    assert!(findings
        .iter()
        .any(|finding| finding["check"] == "nav_resolves" && finding["path"] == "navigation/home"));
}

#[test]
fn text_output_names_path_check_and_severity() {
    let report = report(&doc(&[(
        "navigation",
        "{home: ghost, sections: [{name: all, pages: [p]}]}",
    )]));
    let text = report.render(OutputFormat::Text);
    assert!(
        text.lines()
            .any(|line| line.starts_with("error nav_resolves navigation/home: ")),
        "{text}"
    );
    assert!(text.trim_end().ends_with("warning(s)"), "{text}");
}

#[test]
fn findings_are_ordered_by_path_then_check() {
    let report = report(&doc(&[(
        "navigation",
        "{home: ghost, sections: [{name: all, pages: [p, phantom]}]}",
    )]));
    let keys: Vec<(&str, &str)> = report
        .findings
        .iter()
        .map(|finding| (finding.path.as_str(), finding.check.as_str()))
        .collect();
    let mut sorted = keys.clone();
    sorted.sort_unstable();
    assert_eq!(keys, sorted);
}

#[test]
fn run_exits_1_on_an_error_and_0_on_warnings_only() {
    let dir = crate_dir().join("tests/fixtures");
    let failing = ess_ui_check::CheckArgs {
        path: dir.join("broken-nav.yaml"),
        model: None,
        format: OutputFormat::Json,
        lacks: Vec::new(),
    };
    assert_eq!(
        ess_ui_check::exit_code(&failing).expect("the check runs"),
        1
    );
    let passing = ess_ui_check::CheckArgs {
        path: crate_dir().join("../../../examples/partner-portal/ui.yaml"),
        ..failing
    };
    assert_eq!(
        ess_ui_check::exit_code(&passing).expect("the check runs"),
        0
    );
}

#[test]
fn every_schema_check_is_run() {
    let schema: serde_yaml::Value = serde_yaml::from_str(ess_ui::SCHEMA).expect("schema");
    let list = schema["checks"]["list"].as_sequence().expect("checks.list");
    assert!(list.len() >= 17);
    for entry in list {
        let id = entry["id"].as_str().expect("a check has an id");
        let severity = entry["severity"].as_str().expect("a check has a severity");
        let check = CHECKS
            .iter()
            .find(|check| check.id == id)
            .unwrap_or_else(|| panic!("the schema's check `{id}` is not run"));
        assert!(check.from_schema, "`{id}` is declared by the schema");
        assert_eq!(check.severity.as_str(), severity, "`{id}`");
    }
    for check in CHECKS.iter().filter(|check| check.from_schema) {
        assert!(
            list.iter()
                .any(|entry| entry["id"].as_str() == Some(check.id)),
            "`{}` claims the schema declares it",
            check.id
        );
    }
}

#[test]
fn every_check_has_a_test_named_by_its_id() {
    let source = include_str!("checks.rs");
    for check in CHECKS {
        assert!(
            source.contains(&format!("fn {}() {{", check.id)),
            "no test named `{}`",
            check.id
        );
    }
}

#[test]
fn a_document_file_is_checked_relative_to_its_directory() {
    let file = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/broken-nav.yaml");
    let report = ess_ui_check::check(&file, None).expect("the check runs");
    assert_eq!(report.document, file.display().to_string());
    assert!(report.has_errors());
    assert!(
        tripped(&report, "fixture_per_view").is_empty(),
        "{:#?}",
        report.findings
    );
}

/// The fixture model with an enum: every item sits on a `shop.stock.Shelf` (Top, Middle,
/// Bottom), which `AddItem` takes and the `Items` view shows.
fn shelf_model() -> Model {
    let read = |file: &str| {
        std::fs::read_to_string(crate_dir().join("tests/fixtures/model").join(file))
            .unwrap_or_else(|error| panic!("{file}: {error}"))
    };
    let stock = read("domains/stock.yaml")
        .replace(
            "    of: Uuid\n",
            "    of: Uuid\n  - name: shop.stock.Shelf\n    kind: enum\n    variants: [Top, Middle, Bottom]\n",
        )
        .replace(
            "      - name: label\n        type: String\n",
            "      - name: label\n        type: String\n      - name: shelf\n        type: shop.stock.Shelf\n",
        );
    let sources = [
        ("system.yaml", read("system.yaml")),
        ("components.yaml", read("components.yaml")),
        ("domains/stock.yaml", stock),
        ("domains/audit.yaml", read("domains/audit.yaml")),
    ]
    .map(|(label, text)| (label.to_owned(), text));
    model_from_sources(&sources, Path::new("shop")).unwrap_or_else(|error| panic!("{error}"))
}

fn shelf_report(sections: &str) -> Report {
    let text = doc(&[
        ("model", "shop"),
        (
            "pages",
            &format!("{{p: {{kind: detail_page, title: P, sections: [{{name: summary, reads: stock.Items}}, {sections}]}}}}"),
        ),
    ]);
    report_with(&text, Some(&shelf_model()), &Options::default())
}

/// beyond10x/ess#351, #358, #364: a `group_by`, an aggregate's `field` and a `label_from` name
/// row fields of their views.
#[test]
fn row_fields() {
    let report = shelf_report(
        "{name: list, component: collection, reads: stock.Items, group_by: aisle, \
          columns: [{field: item_id, label_from: {view: stock.Items, field: title}}, \
                    {field: label, label_from: {view: stock.Items, field: label, key: label}}]}, \
         {name: tally, component: metric, reads: stock.Items, aggregate: sum, field: weight}",
    );
    trips_in(&report, "row_fields", "pages/p/sections/list/group_by");
    trips_in(
        &report,
        "row_fields",
        "pages/p/sections/list/columns/item_id/label_from/field",
    );
    trips_in(&report, "row_fields", "pages/p/sections/tally/field");
    // The default key, `id`, is no row field of `Items` either: its identity is `item_id`.
    trips_in(
        &report,
        "row_fields",
        "pages/p/sections/list/columns/item_id/label_from/key",
    );
    assert_eq!(tripped(&report, "row_fields").len(), 4, "{report:#?}");
}

/// beyond10x/ess#351, #330: a `group_order` over an enum field, and a form choice's fixed
/// options for an enum input, hold the model's variants.
#[test]
fn model_enum_values() {
    let report = shelf_report(
        "{name: list, component: collection, reads: stock.Items, group_by: shelf, \
          group_order: [Top, Floor]}, \
         {name: add, component: form, does: stock.AddItem, \
          fields: [label, {field: shelf, as: choice, choice: {component: choice, options: [Top, Middle]}}]}, \
         {name: fine, component: collection, reads: stock.Items, group_by: shelf, group_order: [Bottom, Top]}",
    );
    let order = trips_in(
        &report,
        "model_enum_values",
        "pages/p/sections/list/group_order",
    );
    assert!(order.message.contains("`Floor`"), "{order:#?}");
    let options = trips_in(
        &report,
        "model_enum_values",
        "pages/p/sections/add/fields/shelf/choice/options",
    );
    assert!(options.message.contains("`Bottom`"), "{options:#?}");
    assert_eq!(
        tripped(&report, "model_enum_values").len(),
        2,
        "{report:#?}"
    );
}

/// beyond10x/ess#358: `aggregate` needs `reads`, and a `field` unless it counts.
#[test]
fn metric_aggregate() {
    let report = report(&page(
        "{kind: detail_page, title: P, sections: [{name: summary, reads: t.ById}, \
          {name: a, component: metric, from: channel.c.n, aggregate: count}, \
          {name: b, component: metric, reads: t.All, aggregate: avg}, \
          {name: c, component: metric, reads: t.All, field: amount}, \
          {name: d, component: metric, reads: t.All, aggregate: count}, \
          {name: e, component: metric, reads: t.All, aggregate: max, field: amount}]}",
    ));
    trips_in(&report, "metric_aggregate", "pages/p/sections/a/aggregate");
    trips_in(&report, "metric_aggregate", "pages/p/sections/b/aggregate");
    trips_in(&report, "metric_aggregate", "pages/p/sections/c/field");
    assert_eq!(tripped(&report, "metric_aggregate").len(), 3, "{report:#?}");
}

/// beyond10x/ess#351: `group_order` and `show_empty_groups` order the groups of `group_by`.
#[test]
fn group_order() {
    let report = report(&page(
        "{kind: detail_page, title: P, sections: [{name: summary, reads: t.ById}, \
          {name: a, component: collection, reads: t.All, group_order: [x]}, \
          {name: b, component: collection, reads: t.All, group_by: s, show_empty_groups: true}, \
          {name: c, component: collection, reads: t.All, group_by: s, group_order: [x], show_empty_groups: true}]}",
    ));
    trips_in(&report, "group_order", "pages/p/sections/a");
    trips_in(
        &report,
        "group_order",
        "pages/p/sections/b/show_empty_groups",
    );
    assert_eq!(tripped(&report, "group_order").len(), 2, "{report:#?}");
}

fn filtered_page(filter: &str, extra: &str) -> String {
    page(&format!("{{kind: detail_page, title: P, params: {{id: string}}, sections: [{{name: list, component: collection, reads: {{view: t.All, filter: {filter}, {extra}}}}}]}}"))
}

#[test]
fn filter_expr() {
    for expression in ["row.enabled", "'row.id =='", "'row.id in params.id'"] {
        trips(
            &filtered_page(expression, ""),
            "filter_expr",
            "pages/p/sections/list/reads/filter",
        );
    }
    let valid = report(&filtered_page(
        "'not (row.id != params.id) and row.stage in [open, active]'",
        "",
    ));
    assert!(tripped(&valid, "filter_expr").is_empty(), "{valid:?}");
}

#[test]
fn filter_roots() {
    for expression in [
        "'actor.id == row.owner'",
        "'section.list.selection != null'",
        "'matches(params)'",
    ] {
        trips(
            &filtered_page(expression, ""),
            "filter_roots",
            "pages/p/sections/list/reads/filter",
        );
    }
}

#[test]
fn filter_paths() {
    for expression in [
        "'row.id == params.missing'",
        "'row.id == state.missing'",
        "'row.id == shell.missing'",
    ] {
        trips(
            &filtered_page(expression, ""),
            "filter_paths",
            "pages/p/sections/list/reads/filter",
        );
    }
}

#[test]
fn filter_place() {
    let text = filtered_page("'row.id == params.id'", "")
        .replace("component: collection", "component: record");
    trips(&text, "filter_place", "pages/p/sections/list/reads/filter");
}

#[test]
fn filter_paging() {
    for mode in ["server", "cursor", "append"] {
        trips(
            &filtered_page("'row.id == params.id'", &format!("paging: {mode}")),
            "filter_paging",
            "pages/p/sections/list/reads/filter",
        );
    }
    for mode in ["client", "none"] {
        assert!(tripped(
            &report(&filtered_page(
                "'row.id == params.id'",
                &format!("paging: {mode}")
            )),
            "filter_paging"
        )
        .is_empty());
    }
}

#[test]
fn filter_export() {
    let text = filtered_page("'row.id == params.id'", "").replace("title: P,", "title: P, header: {actions: [{name: download, export: {reads: t.All, as: csv, params: same_as(list)}}]},");
    trips(
        &text,
        "filter_export",
        "pages/p/header/actions/download/export/params",
    );
}

#[test]
fn filter_over_param() {
    let root = crate_dir().join("tests/fixtures/model");
    let read = |file: &str| std::fs::read_to_string(root.join(file)).unwrap();
    let mut stock = read("domains/stock.yaml");
    stock.push_str("\n  - name: shop.stock.Filterable\n    source: shop.stock.Item\n    consistency: read_your_writes\n    params: [{name: wanted, type: Optional<String>}]\n    filter: label == param.wanted\n    fields: [{name: label, type: String}]\n");
    let sources = [
        ("system.yaml", read("system.yaml")),
        ("components.yaml", read("components.yaml")),
        ("domains/stock.yaml", stock),
        ("domains/audit.yaml", read("domains/audit.yaml")),
    ]
    .map(|(label, text)| (label.to_owned(), text));
    let model = model_from_sources(&sources, Path::new("shop")).unwrap();
    let text = filtered_page("'row.label == params.id'", "").replace("t.All", "stock.Filterable");
    let report = report_with(&text, Some(&model), &Options::default());
    let finding = trips_in(
        &report,
        "filter_over_param",
        "pages/p/sections/list/reads/filter",
    );
    assert!(finding.message.contains("wanted"), "{finding:?}");
}

#[test]
fn filters_check_model_rows_menu_scope_and_all_unsupported_loads() {
    let source = filtered_page("'row.missing == yes'", "").replace("t.All", "stock.Items");
    trips_in(
        &report_with(&source, Some(&model()), &Options::default()),
        "filter_paths",
        "pages/p/sections/list/reads/filter",
    );
    for kind in ["record", "metric"] {
        trips(
            &filtered_page("'row.id != null'", "")
                .replace("component: collection", &format!("component: {kind}")),
            "filter_place",
            "pages/p/sections/list/reads/filter",
        );
    }
    let form = filtered_page("'row.id != null'", "")
        .replace("component: collection", "component: form, does: t.Save")
        .replace("reads:", "loads:");
    trips(&form, "filter_place", "pages/p/sections/list/loads/filter");
    let action = page("{kind: detail_page, title: P, header: {actions: [{name: edit, loads: {view: t.All, filter: row.id != null}, opens: edit}]}, sections: []}");
    trips(
        &action,
        "filter_place",
        "pages/p/header/actions/edit/loads/filter",
    );
    let menu = doc(&[("navigation", "{home: p, sections: [{name: all, pages: {from_view: t.All, page: p, param: id, filter: row.id == params.id}}]}")]);
    trips(
        &menu,
        "filter_paths",
        "navigation/sections/all/from_view/filter",
    );
    assert!(ess_ui::load_str(&doc(&[("shells", "{app: {regions: {main: {kind: page_outlet}}, preload: {policy: before_first_page, views: [{view: t.All, filter: row.id != null}]}}}")])).is_err());
}
