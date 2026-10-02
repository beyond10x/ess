//! Widget expansion is bounded: a document whose expanded widget bodies would pass the limit is
//! refused at the use that passes it, in time linear in the limit, never after expanding an
//! exponential number of bodies (beyond10x/ess#300).

use std::fmt::Write as _;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

/// Widget `w0` uses `w1` twice, `w1` uses `w2` twice, … : `2^depth` bodies once expanded.
fn doubling(depth: usize) -> String {
    let mut widgets = String::new();
    for level in 0..depth {
        let next = level + 1;
        let _ = write!(
            widgets,
            "  w{level}:\n    summary: s\n    body:\n      - {{name: a, component: w{next}}}\n      - {{name: b, component: w{next}}}\n"
        );
    }
    let _ = write!(
        widgets,
        "  w{depth}:\n    summary: s\n    body: [{{name: t, primitive: text, text: leaf}}]\n"
    );
    format!(
        "format: ess-ui/1
app: t
model: t.system
placement_profile: fat
shells:
  app: {{regions: {{main: {{kind: page_outlet}}}}}}
navigation:
  home: p
  sections: [{{name: all, pages: [p]}}]
widgets:
{widgets}pages:
  p: {{kind: detail_page, title: P, sections: [{{name: s, component: w0}}]}}
"
    )
}

fn load_within(text: String, limit: Duration) -> Result<ess_ui::Document, ess_ui::LoadError> {
    let (send, receive) = mpsc::channel();
    thread::spawn(move || {
        let _ = send.send(ess_ui::load_str(&text));
    });
    receive
        .recv_timeout(limit)
        .unwrap_or_else(|_| panic!("load_str did not return within {limit:?}"))
}

#[test]
fn a_shallow_doubling_document_loads() {
    let document = load_within(doubling(4), Duration::from_secs(30))
        .unwrap_or_else(|error| panic!("refused: {error}"));
    assert_eq!(document.pages.len(), 1);
}

#[test]
fn a_deep_doubling_document_is_refused_promptly_at_its_outermost_use() {
    let Err(error) = load_within(doubling(64), Duration::from_secs(30)) else {
        panic!("a document expanding to 2^64 widget bodies loaded");
    };
    assert!(
        error.message().contains("widget expansion") && error.message().contains("exceeds"),
        "{error}"
    );
    assert_eq!(error.path().to_string(), "pages/p/sections/s", "{error}");
}
