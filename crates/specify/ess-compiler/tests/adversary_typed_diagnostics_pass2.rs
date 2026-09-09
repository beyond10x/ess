//! Adversary pass 2 against `story:review-typed-diagnostics`.
//!
//! Pass 1 attacked the needle derivation. This pass attacks the unit's own contract document,
//! `docs/design/review-typed-diagnostics.md`, read as the specification it claims to be:
//!
//! 1. `:126` — "**Migrated (this wave):** the `command` family". Three `command.…` refusals are
//!    produced by `ess-domain/src/entity.rs` (`:935`, `:1072`, `:1126`) and are still on the string
//!    heuristic; the page's inventory records them under the `entity.rs` row.
//! 2. `:166` — the deferred families are "Same shape as `command.rs`". `entity.rs` and
//!    `component.rs` write `entity <name>` and `component <name>`, with a space, which
//!    `ConstructRef::render` cannot produce — the class the page's own `validate_sets` row calls
//!    "a location change, not a wording change".
//! 3. `:34` — the table row says `ConstructKind` is `#[non_exhaustive]`; `:89` and the type's own
//!    doc comment say it is deliberately not.
//! 4. `:219` — `repeated_names.yaml` is the fixture for the repeated-name hazard and every refusal
//!    it produces is unlocated, so nothing in the suite checks the story's Validation clause that
//!    repeated names "retain correct codes/locations".
//!
//! The rewording transformation used below is the unit's own: `resolve.rs`'s
//! `a_sited_refusal_takes_its_family_from_the_construct_not_the_location_head` rewrites a
//! refusal's `location` head to `reworded.` and requires the emitted family not to move.

use ess_compiler::resolve::diagnose_locating;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::error::{ConstructKind, ConstructRef, ValidationError, ValidationErrors};

const REPEATED: &str = include_str!("fixtures/typed_diagnostics/repeated_names.yaml");

/// A command with a `wrong_state:` branch and no `moves:`.
///
/// `entity.rs`'s `validate_wrong_state_is_reachable` (`:1038`) refuses it at
/// `command.shop.wrong.Touch.outcomes.wrong-state` — a `command`-family location, written by
/// `format!` in `entity.rs:1072`. The same document also draws `entity.rs:1004`'s
/// `entity shop.wrong.Order.transitions[0]`, the space-separated shape.
const WRONG_STATE: &str = "\
format: ess/1
system: shop
version: v1
domains: [shop.wrong]
domain: shop.wrong
types:
  - name: shop.wrong.OrderId
    kind: newtype
    of: Uuid
entities:
  - name: shop.wrong.Order
    identity:
      name: order_id
      type: shop.wrong.OrderId
    fields:
      - name: total
        type: Decimal
    lifecycle:
      initial: Open
      states: [Open, Closed]
      terminal: [Closed]
      transitions:
        - name: close
          from: [Open]
          to: Closed
errors:
  - name: shop.wrong.Conflict
    summary: The order was in the wrong state.
events:
  - name: shop.wrong.Touched
    fields:
      - name: order_id
        type: shop.wrong.OrderId
commands:
  - name: shop.wrong.Touch
    input:
      - name: order_id
        type: shop.wrong.OrderId
    outcomes:
      - name: touched
        emits:
          - shop.wrong.Touched
        payload:
          shop.wrong.Touched:
            order_id: input.order_id
      - name: wrong-state
        wrong_state: true
        error: shop.wrong.Conflict
";

/// Every refusal a document set produces, unbridged.
fn refusals(files: &[(&str, &str)]) -> ValidationErrors {
    Specification::assemble(files.iter().map(|(label, text)| {
        (
            Source::new(*label),
            RawSpecFile::parse(text).expect("the fixture is well formed YAML"),
        )
    }))
    .expect_err("the fixture is refused on purpose")
}

/// The sources, as the compiler reads them.
fn sources_of(files: &[(&str, &str)]) -> (SourceMap, Vec<String>) {
    let mut sources = SourceMap::new();
    let mut labels = Vec::new();
    for (label, text) in files {
        sources.insert(*label, *text);
        labels.push((*label).to_owned());
    }
    (sources, labels)
}

/// The one refusal whose location is exactly `path`.
fn refusal_at<'a>(errors: &'a ValidationErrors, path: &str) -> &'a ValidationError {
    errors
        .as_slice()
        .iter()
        .find(|error| error.location == path)
        .unwrap_or_else(|| {
            panic!(
                "no refusal is located at {path:?}; the document produced {:?}",
                errors
                    .as_slice()
                    .iter()
                    .map(|error| error.location.clone())
                    .collect::<Vec<_>>()
            )
        })
}

/// The repository root, from this crate's manifest directory.
fn root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

/// The design page: "**Migrated (this wave):** the `command` family" (`:126`).
///
/// It is not. `entity.rs:1072` writes a `command.…` location and builds it with
/// `ValidationError::new`, so this refusal carries no site and its family and cited line are still
/// recovered by parsing the string F14 says must not be parsed.
#[test]
fn every_command_family_refusal_carries_a_site_as_the_migration_table_says() {
    let files = [("wrong_state.yaml", WRONG_STATE)];
    let errors = refusals(&files);
    let unsited: Vec<&str> = errors
        .as_slice()
        .iter()
        .filter(|error| error.location.starts_with("command."))
        .filter(|error| error.site().is_none())
        .map(|error| error.location.as_str())
        .collect();
    assert!(
        unsited.is_empty(),
        "the design page says the `command` family is migrated; these `command.…` refusals carry \
         no typed site and are still bridged from their string: {unsited:?}"
    );
}

/// The acceptance statement, on a `command`-family refusal the page says was migrated.
///
/// The transformation and the assertion are the unit's own
/// (`resolve.rs::a_sited_refusal_takes_its_family_from_the_construct_not_the_location_head`):
/// rewrite the human-facing path's head and require the emitted machine code not to move. It moves.
#[test]
fn rewording_the_path_of_a_command_family_refusal_does_not_move_its_code() {
    let files = [("wrong_state.yaml", WRONG_STATE)];
    let errors = refusals(&files);
    let original = refusal_at(&errors, "command.shop.wrong.Touch.outcomes.wrong-state");

    let mut reworded = original.clone();
    reworded.location = "reworded.shop.wrong.Touch.outcomes.wrong-state".to_owned();
    let mut one = ValidationErrors::new();
    one.push(reworded);

    let (sources, labels) = sources_of(&files);
    let before = diagnose_locating(&ValidationErrors::from(original.clone()), &sources, &labels);
    let after = diagnose_locating(&one, &sources, &labels);

    assert_eq!(
        after.as_slice()[0].code.to_string(),
        before.as_slice()[0].code.to_string(),
        "rewording the human-facing path changed the machine code of a `command`-family refusal"
    );
}

/// The design page's inventory (`:166`): the deferred families are "Same shape as `command.rs`".
///
/// `entity.rs` writes `entity <name>` with a space (`:830`, `:1004`, `:1490`), and `component.rs`
/// writes `component <name>` (`:461`, `:513`, `:553`, …). `ConstructRef::render` always writes
/// `<kind>.<name>`, so migrating those sites moves the adopter-facing string — which the page's own
/// `validate_sets` row (`:162`) calls "a location change, not a wording change" and puts outside
/// this story's acceptance. They are recorded as ordinary deferred work instead.
#[test]
fn a_deferred_family_location_is_renderable_by_a_construct_reference() {
    let files = [("wrong_state.yaml", WRONG_STATE)];
    let errors = refusals(&files);
    let entity = errors
        .as_slice()
        .iter()
        .find(|error| error.location.contains("shop.wrong.Order.transitions"))
        .expect("the lifecycle refusal");

    let rendered = ConstructRef::new(ConstructKind::Entity, "shop.wrong.Order")
        .key("transitions")
        .index(0)
        .render();
    assert_eq!(
        rendered, entity.location,
        "`entity.rs` is inventoried as the same shape as `command.rs`, and no `ConstructRef` \
         renders the string it writes"
    );
}

/// The design page's table row for `ConstructKind` (`:34`) against the type it describes.
///
/// The row says `#[non_exhaustive]`. The prose at `:89` and the type's own doc comment say the
/// opposite, and give the reason: a `#[non_exhaustive]` enum forces a downstream `_` arm, which is
/// how a new kind reaches `family::SPEC` with nobody deciding it should. The row is the summary a
/// reader adding a kind reads first.
#[test]
fn the_page_and_the_source_agree_on_whether_construct_kind_is_non_exhaustive() {
    let page = std::fs::read_to_string(root().join("docs/design/review-typed-diagnostics.md"))
        .expect("the design page exists");
    let row = page
        .lines()
        .find(|line| line.starts_with("| `ConstructKind` |"))
        .expect("the page's type table has a `ConstructKind` row");
    let claimed = row.contains("#[non_exhaustive]");

    let source = std::fs::read_to_string(root().join("crates/specify/ess-primitives/src/error.rs"))
        .expect("the error module exists");
    let before = source
        .split_once("pub enum ConstructKind")
        .expect("`ConstructKind` is declared there")
        .0;
    let attributes = before
        .rsplit_once("#[derive(")
        .expect("the enum carries a derive")
        .1;
    let declared = attributes.contains("#[non_exhaustive]");

    assert_eq!(
        claimed, declared,
        "the design page's `ConstructKind` row says `#[non_exhaustive]` is {claimed} and \
         `error.rs` declares it {declared}"
    );
}

/// The story's Validation clause: repeated names "retain correct codes/locations".
///
/// `repeated_names.yaml` is the fixture the design page (`:219`) assigns to that hazard, and every
/// refusal it produces is unlocated, so no assertion anywhere in the suite pins a *location* for a
/// repeated-name defect. The correction commit reached that state by renaming the fixture's command
/// from `shop.repeat.FileOne` to `shop.repeat.File`, which makes the fallback needle a substring of
/// two sibling declarations.
#[test]
fn the_repeated_name_fixture_locates_at_least_one_of_its_refusals() {
    let files = [("repeated_names.yaml", REPEATED)];
    let errors = refusals(&files);
    let (sources, labels) = sources_of(&files);
    let diagnostics = diagnose_locating(&errors, &sources, &labels);
    let located: Vec<&str> = diagnostics
        .as_slice()
        .iter()
        .filter_map(|diagnostic| diagnostic.span.as_ref())
        .filter(|span| span.located.is_some())
        .map(|span| span.path.as_str())
        .collect();
    assert!(
        !located.is_empty(),
        "no refusal from the repeated-name fixture carries a source location, so the suite checks \
         no location for the hazard the story names: {:?}",
        diagnostics
            .as_slice()
            .iter()
            .filter_map(|diagnostic| diagnostic.span.as_ref())
            .map(|span| (span.path.clone(), span.located))
            .collect::<Vec<_>>()
    );
}
