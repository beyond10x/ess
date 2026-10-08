//! Adversary case (pass 2) for `synthesis_suite_bytes_table.rs`: each model written out under
//! `tests/fixtures/claim-search/` is, byte for byte, the model its in-test builder synthesizes.
//!
//! The table pins the fixtures; the #464 tests synthesize the builders. Nothing else ties the two,
//! so a builder edited later (or a fixture copied with a trailing newline) leaves the table pinning a
//! model no #464 test reads. Builders: `external_beside_held_guard.rs` (`DOOR`, `RELATED`,
//! `ROW_SET`, `external_guarded_by_when_keeps_own_guard`, `unrefutable_sibling_refused_by_name`)
//! and `adversary_464_pass1.rs` (`adversary_464_dump_scenarios_for_base_comparison`).
use std::path::Path;

const HOLDER: &str = include_str!("external_beside_held_guard.rs");
const ADVERSARY: &str = include_str!("adversary_464_pass1.rs");
const MODEL: &str = include_str!("fixtures/external-beside-held-guard.yaml");

/// A `const NAME: &str = r"…";` of `file`, as `adversary_464_pass1.rs` `unit_model` reads one.
fn constant(file: &str, name: &str) -> String {
    let open = format!("const {name}: &str = r\"");
    let start = file.find(&open).unwrap() + open.len();
    let end = start + file[start..].find("\";\n").unwrap();
    file[start..end].to_owned()
}

fn fixture(name: &str) -> String {
    std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/claim-search")
            .join(name),
    )
    .unwrap()
}

fn builders() -> Vec<(&'static str, String)> {
    let door = constant(HOLDER, "DOOR");
    let offline = constant(ADVERSARY, "DOOR_OFFLINE");
    let row_set = constant(HOLDER, "ROW_SET");
    let guarded = MODEL
        .replace(
            "      - {name: revision, type: Integer}\n    outcomes:\n      - name: stale",
            "      - {name: revision, type: Integer}\n      - {name: channel, type: String}\n    outcomes:\n      - name: stale",
        )
        .replace(
            "        external: the current list does not name the pick\n",
            "        external: the current list does not name the pick\n        when: channel == \"fax\"\n",
        );
    let frozen = MODEL.replace(
        "        when_subject:\n          predicate: revision != input.revision\n",
        "        when_subject:\n          predicate: state == Picked\n",
    );
    vec![
        ("door-gentle.yaml", door.replace("PUSH", "Gentle")),
        ("door-rough.yaml", door.replace("PUSH", "Rough")),
        ("door-offline-gentle.yaml", offline.replace("PUSH", "Gentle")),
        ("door-offline-rough.yaml", offline.replace("PUSH", "Rough")),
        ("door-relabel.yaml", constant(ADVERSARY, "DOOR_RELABEL")),
        (
            "driver-when-subject.yaml",
            constant(ADVERSARY, "DRIVER_WHEN_SUBJECT"),
        ),
        ("pick-external-when.yaml", guarded),
        ("pick-frozen.yaml", frozen),
        (
            "pick-revision-gt.yaml",
            MODEL.replace("revision != input.revision", "revision > input.revision"),
        ),
        ("related.yaml", constant(HOLDER, "RELATED")),
        ("row-set-gt-0.yaml", row_set.replace("LIMIT", "0")),
        ("row-set-gt-1.yaml", row_set.replace("LIMIT", "1")),
    ]
}

#[test]
fn every_claim_search_fixture_is_its_builders_model() {
    let mut differ = Vec::new();
    let built = builders();
    for (name, text) in &built {
        if fixture(name) != *text {
            differ.push(*name);
        }
    }
    let mut on_disk: Vec<String> = std::fs::read_dir(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/claim-search"),
    )
    .unwrap()
    .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
    .collect();
    on_disk.sort();
    let mut named: Vec<String> = built.iter().map(|(name, _)| (*name).to_owned()).collect();
    named.sort();
    assert_eq!(on_disk, named, "every fixture has a builder");
    assert_eq!(differ, Vec::<&str>::new(), "fixtures that are not their builder's model");
}
