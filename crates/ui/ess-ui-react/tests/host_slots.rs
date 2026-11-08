//! A document with a host slot is refused by the React generator, naming the slot, until host
//! slots are generated (0.91.0): the project would otherwise leave the slot's region blank.

use std::path::Path;

const DOCUMENT: &str = "format: ess-ui/1
app: t
model: t.system
placement_profile: fat
shells: {app: {regions: {main: {kind: page_outlet}}}}
navigation: {home: p, sections: [{name: all, pages: [p]}]}
pages:
  p:
    kind: detail_page
    title: P
    sections:
      - {name: summary, reads: t.ById, fields: [name]}
      - {name: map, host: RouteMap, reason: a map widget, stands_in_for: chart}
";

#[test]
fn the_react_generator_refuses_a_host_slot_by_name() {
    let document = ess_ui::load_str(DOCUMENT).unwrap_or_else(|error| panic!("{error}"));
    let error = ess_ui_react::render(&document, Path::new("."))
        .expect_err("a host slot is not generated yet");
    let message = error.to_string();
    assert!(
        message.contains("pages/p/sections/map")
            && message.contains("RouteMap")
            && message.contains("host slots are validated by `ess ui check` and not generated yet"),
        "{message}"
    );
}
