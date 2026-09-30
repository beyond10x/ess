//! The terminal's renderer profile, and the check that holds a document to it.
//!
//! The schema's `Degrades` rule: a renderer profile lists the capabilities it lacks; a construct
//! that uses one declares a fallback in its `degrades`. The terminal resolves each use against
//! the construct's own `degrades` (or, for `PageLayout`, the `degrades` its schema entry
//! declares), else the first fallback of the schema's capability table (`Degrades.rule`). It
//! refuses the document, naming the node, when the fallback resolved is `refuse`, when none
//! resolves, or when it names a fallback this renderer does not implement.

use std::collections::BTreeMap;
use std::fmt;
use std::sync::OnceLock;

use ess_ui::{
    Body, ChartKind, Columns, Composite, Document, Field, Located, NodePath, NodeRef, PageLayout,
};
use serde_yaml::Value;

/// Whether the terminal has a capability, and what it does instead when it lacks one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Support {
    /// The terminal has it; the note says how it is drawn.
    Has(&'static str),
    /// The terminal lacks it and implements these fallbacks.
    Lacks(&'static [&'static str]),
}

/// One row of the profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CapabilityEntry {
    /// The schema's capability name (`no_charts`, …).
    pub capability: &'static str,
    /// What the terminal does about it.
    pub support: Support,
}

/// What a renderer can draw.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RendererProfile {
    /// The renderer.
    pub name: &'static str,
    /// Every capability of the schema's `Degrades` table.
    pub capabilities: &'static [CapabilityEntry],
    /// Chart kinds drawn natively; any other kind uses `no_charts`.
    pub chart_kinds: &'static [&'static str],
}

impl RendererProfile {
    /// The row for `capability`.
    pub fn support(&self, capability: &str) -> Option<Support> {
        self.capabilities
            .iter()
            .find(|entry| entry.capability == capability)
            .map(|entry| entry.support)
    }
}

/// The terminal renderer's profile.
pub const TUI: RendererProfile = RendererProfile {
    name: "tui",
    capabilities: &[
        CapabilityEntry {
            capability: "no_free_layout",
            support: Support::Lacks(&["stack"]),
        },
        CapabilityEntry {
            capability: "no_charts",
            support: Support::Lacks(&["table", "metric"]),
        },
        CapabilityEntry {
            capability: "no_graph_editor",
            support: Support::Lacks(&["collection"]),
        },
        CapabilityEntry {
            capability: "no_rich_text",
            support: Support::Has("an editable text line with `{{` completion from the view"),
        },
        CapabilityEntry {
            capability: "no_drawer",
            support: Support::Has("every overlay is a full-screen pane closed with esc"),
        },
        CapabilityEntry {
            capability: "no_drag",
            support: Support::Lacks(&["move_buttons"]),
        },
        CapabilityEntry {
            capability: "no_audio",
            support: Support::Lacks(&["link"]),
        },
        CapabilityEntry {
            capability: "no_file_upload",
            support: Support::Has("a file path typed at a prompt"),
        },
        CapabilityEntry {
            capability: "no_live",
            support: Support::Has("channels play in-process"),
        },
        CapabilityEntry {
            capability: "no_iframe",
            support: Support::Lacks(&["link"]),
        },
        CapabilityEntry {
            capability: "no_columns",
            support: Support::Lacks(&["stack"]),
        },
        CapabilityEntry {
            capability: "no_areas",
            support: Support::Lacks(&["stack"]),
        },
    ],
    chart_kinds: &["line", "bar", "single_number", "list", "table"],
};

/// A document the terminal cannot render, and the node at fault.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refusal {
    /// The node.
    pub path: NodePath,
    /// Why.
    pub message: String,
}

impl fmt::Display for Refusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.path, self.message)
    }
}

/// The fallback chosen for each node that uses a capability the terminal lacks.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Plan {
    /// Node path → capability → fallback.
    pub degraded: BTreeMap<String, BTreeMap<String, String>>,
}

impl Plan {
    /// The fallback `path` uses for `capability`, if it was degraded.
    pub fn fallback(&self, path: &NodePath, capability: &str) -> Option<&str> {
        self.degraded
            .get(&path.to_string())
            .and_then(|capabilities| capabilities.get(capability))
            .map(String::as_str)
    }
}

/// Holds `document` to `profile`.
pub fn check(document: &Document, profile: &RendererProfile) -> Result<Plan, Refusal> {
    let mut plan = Plan::default();
    for Located { path, node } in document.nodes() {
        let (degrades, uses, construct_degrades) = match node {
            NodeRef::Section(section) => {
                (&section.common.degrades, uses(&section.body, profile), None)
            }
            NodeRef::Node(node) => (&node.common.degrades, uses(&node.body, profile), None),
            NodeRef::Overlay(overlay) => {
                (&overlay.common.degrades, uses(&overlay.body, profile), None)
            }
            NodeRef::Page(page) => {
                let used = match &page.layout {
                    PageLayout::Stack(_) => Vec::new(),
                    PageLayout::Columns(_) => vec!["no_columns"],
                    PageLayout::Areas(_) => vec!["no_areas"],
                };
                (&EMPTY, used, Some(construct_degrades("PageLayout")))
            }
            _ => continue,
        };
        let at = match node {
            NodeRef::Page(_) => path.child("layout"),
            _ => path.clone(),
        };
        for capability in uses {
            let Some(Support::Lacks(implemented)) = profile.support(capability) else {
                continue;
            };
            let declared = degrades
                .get(capability)
                .cloned()
                .or_else(|| construct_degrades.and_then(|table| table.get(capability).cloned()));
            let known = schema_fallbacks(capability);
            // Degrades.rule: `use: [construct.degrades.$c, capabilities.$c.fallbacks[0]]`,
            // `if_none: refuse`.
            let fallback = match declared.or_else(|| known.first().cloned()) {
                None => {
                    return Err(Refusal {
                        path: at,
                        message: format!(
                            "the {} renderer lacks `{capability}`, this node declares no \
                             fallback and the schema's capability table lists none",
                            profile.name
                        ),
                    })
                }
                Some(fallback) if fallback == "refuse" => {
                    return Err(Refusal {
                        path: at,
                        message: format!(
                            "the {} renderer lacks `{capability}` and it degrades to `refuse` \
                             (declared here or first in the schema's capability table)",
                            profile.name
                        ),
                    })
                }
                Some(fallback) => fallback,
            };
            if !implemented.contains(&fallback.as_str()) || !known.contains(&fallback) {
                return Err(Refusal {
                    path: at,
                    message: format!(
                        "`{capability}: {fallback}` names a fallback the {} renderer does not \
                         implement (it implements {}; the schema allows {})",
                        profile.name,
                        implemented.join(", "),
                        known.join(", ")
                    ),
                });
            }
            plan.degraded
                .entry(at.to_string())
                .or_default()
                .insert(capability.to_owned(), fallback);
        }
    }
    Ok(plan)
}

static EMPTY: BTreeMap<String, String> = BTreeMap::new();

/// The capabilities a node's body uses that a profile might lack.
fn uses(body: &Body, profile: &RendererProfile) -> Vec<&'static str> {
    let Body::Composite(composite) = body else {
        return Vec::new();
    };
    let mut used = Vec::new();
    match composite {
        Composite::Board(_) => used.push("no_free_layout"),
        Composite::GraphEditor(_) => used.push("no_graph_editor"),
        Composite::Chart(chart) => {
            let kinds: Vec<&str> = match &chart.chart {
                ChartKind::Fixed(kind) => vec![kind.as_str()],
                ChartKind::Chosen(chosen) => chosen.options.iter().map(String::as_str).collect(),
            };
            if kinds.iter().any(|kind| !profile.chart_kinds.contains(kind)) {
                used.push("no_charts");
            }
        }
        Composite::Collection(collection) => {
            if collection.reorder.is_some() {
                used.push("no_drag");
            }
            let fields: &[Field] = match &collection.columns {
                Some(Columns::Fixed(fields)) => fields,
                Some(Columns::Selectable(columns)) => &columns.all,
                _ => &[],
            };
            used.extend(media(fields));
        }
        Composite::Record(record) => used.extend(media(&record.fields)),
        _ => {}
    }
    used
}

fn media(fields: &[Field]) -> Vec<&'static str> {
    let mut used = Vec::new();
    for field in fields {
        match field.field_as.as_deref() {
            Some("audio") => used.push("no_audio"),
            Some("iframe") => used.push("no_iframe"),
            _ => {}
        }
    }
    used
}

fn schema() -> &'static Value {
    static SCHEMA: OnceLock<Value> = OnceLock::new();
    SCHEMA.get_or_init(|| serde_yaml::from_str(ess_ui::SCHEMA).expect("the schema is YAML"))
}

/// The `degrades` a construct's schema entry declares (`constructs.<name>.degrades`).
fn construct_degrades(construct: &str) -> &'static BTreeMap<String, String> {
    static TABLES: OnceLock<BTreeMap<String, BTreeMap<String, String>>> = OnceLock::new();
    let tables = TABLES.get_or_init(|| {
        let mut tables = BTreeMap::new();
        if let Some(constructs) = schema()["constructs"].as_mapping() {
            for (name, entry) in constructs {
                if let (Some(name), Ok(table)) = (
                    name.as_str(),
                    serde_yaml::from_value::<BTreeMap<String, String>>(entry["degrades"].clone()),
                ) {
                    tables.insert(name.to_owned(), table);
                }
            }
        }
        tables
    });
    tables.get(construct).unwrap_or(&EMPTY)
}

/// The fallbacks the schema's capability table lists for `capability`.
pub(crate) fn schema_fallbacks(capability: &str) -> Vec<String> {
    schema()["constructs"]["Degrades"]["capabilities"][capability]["fallbacks"]
        .as_sequence()
        .map(|fallbacks| {
            fallbacks
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_profile_covers_every_capability_of_the_schema_and_only_its_fallbacks() {
        let table = schema()["constructs"]["Degrades"]["capabilities"]
            .as_mapping()
            .expect("the schema has a capability table");
        let declared: Vec<&str> = table.keys().filter_map(Value::as_str).collect();
        let covered: Vec<&str> = TUI
            .capabilities
            .iter()
            .map(|entry| entry.capability)
            .collect();
        let mut sorted_declared = declared.clone();
        sorted_declared.sort_unstable();
        let mut sorted_covered = covered.clone();
        sorted_covered.sort_unstable();
        assert_eq!(sorted_declared, sorted_covered);
        for entry in TUI.capabilities {
            if let Support::Lacks(fallbacks) = entry.support {
                let known = schema_fallbacks(entry.capability);
                for fallback in fallbacks {
                    assert!(known.iter().any(|known| known == fallback), "{fallback}");
                }
            }
        }
    }
}
