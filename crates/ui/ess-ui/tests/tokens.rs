//! Style tokens, themes, the theme choice and tone maps (`docs/design/ui-style-tokens.md`): what
//! the loader reads, how a theme's table is merged, and how `tone_by.tones` is resolved.

use std::collections::BTreeMap;
use std::path::Path;

use ess_ui::{
    Body, Document, Expr, LoadError, NodeRef, Primitive, ThemeChoice, TokenValue, Tokens, Tone,
    ToneBy, ToneToken, TypeToken,
};
use serde_yaml::Value;

const BASE: &str = r"format: ess-ui/1
app: t
model: t.system
placement_profile: fat
shells:
  app:
    regions: {main: {kind: page_outlet}}
    state:
      theme: {type: {enum: [light, dark]}, class: preference, store: local_storage, pinned: true, default: light}
navigation: {home: p, sections: [{name: all, pages: [p]}]}
";

const PAGE: &str = r"pages:
  p:
    kind: detail_page
    title: P
    sections:
      - name: summary
        reads: t.ById
        children:
          - {name: state, primitive: badge, field: state, tone_by: {value: row.state, tones: job_state}}
          - name: alarm
            primitive: icon
            icon: alert
            label: Job state
            tone_by: {value: row.state, tones: job_state}
";

/// The construct block of the design page, as written there.
const STYLE: &str = r"tokens:
  color:
    surface: '#ffffff'
    text: '#1c1d21'
    danger: '#c0392b'
    danger_fill: '#f6d5d1'
  space:  {xs: 0.25rem, sm: 0.5rem, md: 1rem, lg: 1.5rem}
  radius: {sm: 4px, md: 6px, lg: 8px, pill: 999px}
  type:
    heading: {size: 1.05rem, weight: 600}
    mono:    {family: 'ui-monospace, monospace'}
  tone:
    danger: {text: danger, fill: danger_fill}
themes:
  light: {}
  dark:
    color: {surface: '#1f2024', text: '#ecedf0', line: '#34363c', danger_fill: '#4a2420'}
  dense:
    space: {xs: 0.125rem, sm: 0.25rem, md: 0.5rem, lg: 0.75rem}
theme: {default: light, chosen_by: shell.theme}
tone_maps:
  job_state:  {Done: success, Failed: danger, HumanEscalation: warning, Running: info}
  deal_stage: {lead: neutral, qualified: info, proposal: warning, won: success, lost: danger}
";

fn load(parts: &[&str]) -> Result<Document, LoadError> {
    let mut text = BASE.to_owned();
    for part in parts {
        text.push_str(part);
    }
    ess_ui::load_str(&text)
}

fn loaded(parts: &[&str]) -> Document {
    load(parts).unwrap_or_else(|error| panic!("{error}"))
}

fn value(text: &str) -> TokenValue {
    TokenValue(text.to_owned())
}

fn yaml(text: &str) -> Value {
    serde_yaml::from_str(text).expect("YAML")
}

/// The `tone_by` of the badge or icon at `path`.
fn tone_by<'a>(document: &'a Document, path: &str) -> &'a ToneBy {
    let located = document
        .nodes()
        .into_iter()
        .find(|located| located.path.to_string() == path)
        .unwrap_or_else(|| panic!("no node at `{path}`"));
    let NodeRef::Node(node) = located.node else {
        panic!("not a node: {path}")
    };
    match &node.body {
        Body::Primitive(Primitive::Badge(badge)) => badge.tone_by.as_ref(),
        Body::Primitive(Primitive::Icon(icon)) => icon.tone_by.as_ref(),
        other => panic!("neither a badge nor an icon: {other:?}"),
    }
    .unwrap_or_else(|| panic!("{path} has no tone_by"))
}

// ── reading ──────────────────────────────────────────────────────────────────────────────────

#[test]
fn a_document_without_style_keys_loads_as_it_did() {
    let page =
        "pages: {p: {kind: detail_page, title: P, sections: [{name: summary, reads: t.ById}]}}\n";
    let document = loaded(&[page]);
    assert_eq!(document.tokens, Tokens::default());
    assert!(document.themes.is_empty());
    assert_eq!(document.theme, None);
    assert!(document.tone_maps.is_empty());
    assert_eq!(document.base_tokens(), Tokens::builtin().filled());
}

#[test]
fn the_style_keys_of_the_design_page_load_into_the_document() {
    let document = loaded(&[STYLE, PAGE]);
    let tokens = &document.tokens;
    assert_eq!(tokens.color["danger_fill"], value("#f6d5d1"));
    assert_eq!(tokens.space["md"], value("1rem"));
    assert_eq!(tokens.radius["pill"], value("999px"));
    assert_eq!(
        tokens.typography["heading"],
        TypeToken {
            family: None,
            size: Some(value("1.05rem")),
            weight: Some(value("600")),
        }
    );
    assert_eq!(
        tokens.typography["mono"].family.as_deref(),
        Some("ui-monospace, monospace")
    );
    assert_eq!(
        tokens.tone["danger"],
        ToneToken {
            text: "danger".to_owned(),
            fill: "danger_fill".to_owned(),
        }
    );
    assert_eq!(document.themes["light"], Tokens::default());
    assert_eq!(document.themes["dark"].color["surface"], value("#1f2024"));
    assert_eq!(document.themes["dense"].space["xs"], value("0.125rem"));
    assert_eq!(
        document.theme,
        Some(ThemeChoice {
            default: "light".to_owned(),
            chosen_by: Some(Expr("shell.theme".to_owned())),
        })
    );
    assert_eq!(document.tone_maps["job_state"]["Failed"], Tone::Danger);
    assert_eq!(document.tone_maps["deal_stage"]["won"], Tone::Success);
}

#[test]
fn a_bare_number_is_read_as_its_text_and_checked_later() {
    let document = loaded(&[
        "tokens: {space: {none: 0}, type: {heading: {weight: 600}}, color: {odd: 12}}\n",
        PAGE.replace("tones: job_state", "map: {}").as_str(),
    ]);
    assert_eq!(document.tokens.space["none"], value("0"));
    assert_eq!(
        document.tokens.typography["heading"].weight,
        Some(value("600"))
    );
    assert_eq!(document.tokens.color["odd"], value("12"));
}

#[test]
fn an_unknown_key_in_tokens_a_theme_theme_or_an_entry_is_refused() {
    let page = PAGE.replace("tones: job_state", "map: {}");
    for style in [
        "tokens: {elevation: {low: 1px}}\n",
        "themes: {dark: {shadow: {low: 1px}}}\n",
        "themes: {light: {}}\ntheme: {default: light, follow_os: true}\n",
        "tokens: {type: {heading: {style: italic}}}\n",
        "tokens: {tone: {danger: {text: danger, fill: danger_fill, border: line}}}\n",
    ] {
        let error = load(&[style, &page]).expect_err(style);
        assert!(
            error.message().contains("unknown field"),
            "{style}: {error}"
        );
    }
}

// ── the built-in table and the merge ─────────────────────────────────────────────────────────

/// The React stylesheet every generated project ships today.
fn stylesheet() -> String {
    let file = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../ess-ui-react/templates/runtime/styles.css.tmpl");
    std::fs::read_to_string(file).expect("the stylesheet reads")
}

/// The `--ui-<name>: <value>;` declarations of the stylesheet's `:root` block.
fn root_colors(css: &str) -> BTreeMap<String, String> {
    let root = &css[..css.find('}').expect(":root closes")];
    root.lines()
        .filter_map(|line| line.trim().strip_prefix("--ui-"))
        .filter_map(|line| line.trim_end_matches(';').split_once(':'))
        .map(|(name, value)| (name.trim().to_owned(), value.trim().to_owned()))
        .collect()
}

/// `true` when the stylesheet has a rule whose selector contains `selector` and whose body
/// contains `declaration`.
fn rule_has(css: &str, selector: &str, declaration: &str) -> bool {
    css.lines().any(|line| {
        line.split_once('{').is_some_and(|(selectors, body)| {
            selectors.contains(selector) && body.contains(declaration)
        })
    })
}

#[test]
fn the_builtin_table_is_todays_stylesheet() {
    let css = stylesheet();
    let builtin = Tokens::builtin();
    let text = |group: &BTreeMap<String, TokenValue>, name: &str| {
        group
            .get(name)
            .unwrap_or_else(|| panic!("the built-in table has no `{name}`"))
            .0
            .clone()
    };

    // The ten `:root` colors, under their names without `--ui-`.
    let root = root_colors(&css);
    assert_eq!(root.len(), 10, "{root:?}");
    for (name, css_value) in &root {
        assert_eq!(&text(&builtin.color, name), css_value, "color `{name}`");
    }
    // The seven literals, each where the design page found it.
    for (name, selector, property) in [
        ("danger_fill", ".ui-badge.ui-tone-danger", "background"),
        (
            "success_fill",
            r#".ui-live[data-lifecycle="live"]"#,
            "background",
        ),
        ("warning_fill", ".ui-stale-badge", "background"),
        ("info_fill", ".ui-new-count", "background"),
        ("focus", ".ui-focused", "background"),
        ("on_accent", ".ui-tone-primary", "color"),
        ("backdrop", ".ui-overlay-backdrop", "background"),
    ] {
        let declaration = format!("{property}: {};", text(&builtin.color, name));
        assert!(
            rule_has(&css, selector, &declaration),
            "`{selector}` has no `{declaration}`"
        );
    }
    assert_eq!(builtin.color.len(), 17, "{:?}", builtin.color.keys());

    // Type: the `:root` font for body, today's sizes and weights for the others.
    let family = |style: &str| builtin.typography[style].family.clone();
    assert_eq!(family("body").as_deref(), Some("system-ui, sans-serif"));
    assert!(css.contains("font-family: system-ui, sans-serif;"));
    let heading = &builtin.typography["heading"];
    let declaration = format!(
        "font-weight: {}; font-size: {};",
        heading.weight.as_ref().expect("a heading weight").0,
        heading.size.as_ref().expect("a heading size").0
    );
    assert!(
        rule_has(&css, ".ui-text-heading", &declaration),
        "{declaration}"
    );
    let caption = format!(
        "font-size: {};",
        builtin.typography["caption"]
            .size
            .as_ref()
            .expect("a caption size")
            .0
    );
    assert!(rule_has(&css, ".ui-text-caption", &caption), "{caption}");
    let mono = format!("font-family: {};", family("mono").expect("a mono family"));
    assert!(rule_has(&css, ".ui-text-mono", &mono), "{mono}");

    // Radii and spaces are lengths the stylesheet uses today.
    for (name, length) in builtin.radius.iter().chain(&builtin.space) {
        assert!(
            css.contains(&format!(" {};", length.0)) || css.contains(&format!(" {} ", length.0)),
            "`{name}`: {}",
            length.0
        );
    }
    assert_eq!(
        builtin.radius.keys().collect::<Vec<_>>(),
        ["lg", "md", "pill", "sm"]
    );

    // Tones reproduce today's badges: each tone's own color on `line`, danger on its fill.
    for (tone, text_color) in [
        ("neutral", "text"),
        ("info", "info"),
        ("success", "success"),
        ("warning", "warning"),
        ("danger", "danger"),
    ] {
        let entry = &builtin.tone[tone];
        assert_eq!(entry.text, text_color, "tone `{tone}`");
        let fill = if tone == "danger" {
            "danger_fill"
        } else {
            "line"
        };
        assert_eq!(entry.fill, fill, "tone `{tone}`");
    }
    assert!(rule_has(&css, ".ui-badge", "background: var(--ui-line);"));
}

#[test]
fn a_theme_is_the_builtin_table_then_tokens_then_its_overrides() {
    let document = loaded(&[
        "tokens: {color: {text: '#111111', brand: '#123456'}}\n\
         themes: {light: {}, dark: {color: {text: '#eeeeee', brand: '#654321'}, space: {md: 2rem}}}\n",
        PAGE.replace("tones: job_state", "map: {}").as_str(),
    ]);
    let builtin = Tokens::builtin();
    let base = document.base_tokens();
    assert_eq!(base.color["text"], value("#111111"));
    assert_eq!(base.color["brand"], value("#123456"));
    assert_eq!(base.color["accent"], builtin.color["accent"]);
    assert_eq!(document.theme_tokens("light"), Some(base.clone()));
    let dark = document.theme_tokens("dark").expect("the dark theme");
    assert_eq!(dark.color["text"], value("#eeeeee"));
    assert_eq!(dark.color["brand"], value("#654321"));
    assert_eq!(dark.color["surface"], builtin.color["surface"]);
    assert_eq!(dark.space["md"], value("2rem"));
    assert_eq!(dark.space["sm"], builtin.space["sm"]);
    assert_eq!(dark.tone, base.tone);
    assert_eq!(document.theme_tokens("sepia"), None);
}

#[test]
fn a_type_entry_falls_back_field_by_field_to_body_then_to_the_builtin_body() {
    let document = loaded(&[
        "tokens: {type: {body: {size: 1.1rem}, heading: {weight: 700}}}\n",
        PAGE.replace("tones: job_state", "map: {}").as_str(),
    ]);
    let base = document.base_tokens();
    let builtin_family = Tokens::builtin().typography["body"].family.clone();
    assert_eq!(
        base.typography["body"],
        TypeToken {
            family: builtin_family.clone(),
            size: Some(value("1.1rem")),
            weight: None,
        }
    );
    assert_eq!(
        base.typography["heading"],
        TypeToken {
            family: builtin_family,
            size: Some(value("1.1rem")),
            weight: Some(value("700")),
        }
    );
    assert_eq!(
        base.typography["mono"].family.as_deref(),
        Some("ui-monospace, monospace")
    );
}

// ── tone maps ────────────────────────────────────────────────────────────────────────────────

#[test]
fn tones_resolves_to_the_named_map_on_a_badge_and_an_icon() {
    let document = loaded(&[STYLE, PAGE]);
    let job_state =
        yaml("{Done: success, Failed: danger, HumanEscalation: warning, Running: info}");
    for path in [
        "pages/p/sections/summary/children/state",
        "pages/p/sections/summary/children/alarm",
    ] {
        let tone_by = tone_by(&document, path);
        assert_eq!(tone_by.value, Expr("row.state".to_owned()), "{path}");
        assert_eq!(tone_by.map, job_state, "{path}");
        assert_eq!(tone_by.tones.as_deref(), Some("job_state"), "{path}");
    }
}

#[test]
fn a_written_map_is_kept_and_names_no_tone_map() {
    let document = loaded(&[
        STYLE,
        PAGE.replace("tones: job_state", "map: {Done: success}")
            .as_str(),
    ]);
    let tone_by = tone_by(&document, "pages/p/sections/summary/children/state");
    assert_eq!(tone_by.map, yaml("{Done: success}"));
    assert_eq!(tone_by.tones, None);
}

#[test]
fn tones_passed_through_a_widget_argument_resolves_at_each_use() {
    let widgets = r"widgets:
  status_badge:
    summary: A status as a toned badge.
    params:
      status: {type: string, required: true, note: the status value}
      tones:  {type: {ref: tone_map}, required: true, note: the tone map}
    body:
      - {name: badge, primitive: badge, text: args.status, tone_by: {value: args.status, tones: args.tones}}
";
    let page = r"pages:
  p:
    kind: detail_page
    title: P
    sections:
      - name: summary
        reads: t.ById
        children:
          - {name: job, component: status_badge, args: {status: row.state, tones: job_state}}
          - {name: deal, component: status_badge, args: {status: row.stage, tones: deal_stage}}
";
    let document = loaded(&[STYLE, widgets, page]);
    let job = tone_by(
        &document,
        "pages/p/sections/summary/children/job/body/badge",
    );
    assert_eq!(job.tones.as_deref(), Some("job_state"));
    assert_eq!(job.map["Failed"], Value::from("danger"));
    let deal = tone_by(
        &document,
        "pages/p/sections/summary/children/deal/body/badge",
    );
    assert_eq!(deal.tones.as_deref(), Some("deal_stage"));
    assert_eq!(deal.map["won"], Value::from("success"));
    let declared = tone_by(&document, "widgets/status_badge/body/badge");
    assert_eq!(declared.tones.as_deref(), Some("args.tones"));
    assert_eq!(declared.map, Value::from("args.tones"));
}

#[test]
fn a_tone_by_has_exactly_one_of_map_and_tones() {
    for (written, has) in [
        ("tones: job_state", "tones: job_state, map: {Done: success}"),
        ("tones: job_state", ""),
    ] {
        let page = PAGE.replacen(written, has, 1);
        let page = page.replace("value: row.state, }", "value: row.state}");
        let error = load(&[STYLE, &page]).expect_err(has);
        assert_eq!(
            error.path().to_string(),
            "pages/p/sections/summary/children/state/tone_by",
            "{error}"
        );
        assert!(
            error.message().contains("exactly one of `map` and `tones`"),
            "{error}"
        );
    }
}

#[test]
fn tones_naming_no_tone_map_is_refused_at_the_value() {
    let error = load(&[STYLE, &PAGE.replacen("tones: job_state", "tones: ghost", 1)])
        .expect_err("`ghost` names no tone map");
    assert_eq!(
        error.path().to_string(),
        "pages/p/sections/summary/children/state/tone_by/tones"
    );
    assert!(
        error.message().contains("names no entry of `tone_maps`"),
        "{error}"
    );
    let none = load(&[PAGE]).expect_err("no tone_maps at all");
    assert!(
        none.message().contains("names no entry of `tone_maps`"),
        "{none}"
    );
}
