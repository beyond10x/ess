//! A guard over the rows a `List` of references names, `when_related: {via: input.<list>, exists:
//! false}`, and the first element naming no row, `{missing: input.<list>}` (`ess/28`,
//! beyond10x/ess#558): what validation admits and refuses.

const LEDGER: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/list-of-references.yaml");

/// The fixture's payload source naming the first missing element, as written.
const MISSING: &str = "{service_id: {missing: input.services}}";

fn spec(body: &str) -> Result<ess_domain::Specification, String> {
    let raw = ess_domain::spec::RawSpecFile::parse(body).map_err(|e| e.to_string())?;
    ess_domain::Specification::assemble([(ess_domain::system::Source::new("ledger.yaml"), raw)])
        .map_err(|e| e.to_string())
}

fn refused(body: &str) -> String {
    match spec(body) {
        Ok(_) => panic!("the document validates:\n{body}"),
        Err(errors) => errors,
    }
}

fn with(from: &str, to: &str) -> String {
    assert!(LEDGER.contains(from), "the fixture holds `{from}`");
    LEDGER.replacen(from, to, 1)
}

#[test]
fn the_fixture_validates_at_ess_28() {
    let spec = spec(LEDGER).unwrap_or_else(|errors| panic!("{errors}"));
    let command = &spec.commands()[&"ledger.desk.OpenIncident".parse().unwrap()];
    let refusal = command
        .outcomes
        .iter()
        .find(|outcome| outcome.name.as_str() == "unknown-service")
        .expect("the refusal");
    assert!(matches!(
        &refusal.condition,
        ess_domain::command::OutcomeCondition::Related { via, .. } if via.field() == "services"
    ));
    assert_eq!(
        refusal.error_payload.get("service_id"),
        Some(&ess_domain::command::PayloadSource::MissingElement {
            field: "services".to_owned()
        })
    );
}

#[test]
fn below_ess_28_the_list_guard_is_refused_naming_it() {
    let errors = refused(&with("format: ess/28", "format: ess/27"));
    assert!(
        errors.contains("`input.services` is `List<ledger.desk.ServiceId>`")
            && errors.contains("ess/28"),
        "{errors}"
    );
}

#[test]
fn a_predicate_over_the_rows_a_list_names_is_refused_by_name() {
    let errors = refused(&with(
        "      - name: opened\n",
        "      - name: unlabelled\n        when_related: {via: input.services, predicate: label == x}\n        error: ledger.desk.UnknownService\n      - name: opened\n",
    ));
    assert!(
        errors.contains("unsupported_construct")
            && errors.contains("tested for a missing row only"),
        "{errors}"
    );
}

#[test]
fn the_missing_element_is_read_only_on_the_branch_guarding_the_list() {
    // On the accepting branch, which no list guard selects.
    let moved = with(
        "        payload: {ledger.desk.IncidentOpened: {incident_id: {generated: true}}}",
        "        payload: {ledger.desk.IncidentOpened: {incident_id: {missing: input.services}}}",
    );
    let errors = refused(&moved);
    assert!(
        errors.contains("{missing: input.services}") && errors.contains("not guarded by"),
        "{errors}"
    );
}

#[test]
fn the_missing_element_is_typed_as_the_lists_element() {
    let errors = refused(&with(
        "      - {name: service_id, type: ledger.desk.ServiceId}\nevents:",
        "      - {name: service_id, type: String}\nevents:",
    ));
    assert!(
        errors.contains("type_mismatch") && errors.contains("is `ledger.desk.ServiceId`"),
        "{errors}"
    );
}

#[test]
fn below_ess_28_the_missing_source_is_the_nested_mapping_it_was() {
    // Below ess/28 `{missing: …}` reads as a nested mapping, refused over a non-struct field; the
    // list guard is refused too, so both name what they are under the older format.
    let errors = refused(&with("format: ess/28", "format: ess/27"));
    assert!(
        !errors.contains("{missing: input.services}` reads"),
        "{errors}"
    );
    assert!(LEDGER.contains(MISSING));
}

/// From `ess/38` a `when_related:` guard names the row it reads through a member of a struct input
/// — `via: input.settings.primary`, an `all:` element, a `List<…>` member with its
/// `{missing: input.settings.depots}` — and below it each is refused naming `ess/38`, while a struct
/// value written `{missing: input.a.b}` keeps its reading (beyond10x/ess#640).
mod struct_input_member {
    use ess_domain::command::{
        OutcomeCondition, PayloadField, PayloadSource, RelatedTest, RelatedVia,
    };

    const MODEL: &str = include_str!(
        "../../../verify/ess-conformance/tests/fixtures/struct-input-member-related-rows.yaml.in"
    );

    /// A note publishes a mark whose one member is named `missing`, set from an input path.
    const TAG: &str = r"format: ess/37
system: shop
version: v1
domain: shop.tag
summary: A note publishes a mark whose one member is named missing.
types:
  - {name: shop.tag.SpotId, kind: newtype, of: Uuid}
  - name: shop.tag.Spot
    kind: struct
    fields:
      - {name: id, type: shop.tag.SpotId}
  - name: shop.tag.Mark
    kind: struct
    fields:
      - {name: missing, type: shop.tag.SpotId}
events:
  - name: shop.tag.Noted
    fields: [{name: mark, type: shop.tag.Mark}]
actors:
  - name: shop.tag.Clerk
    may: [shop.tag.Note]
commands:
  - name: shop.tag.Note
    input:
      - {name: spot, type: shop.tag.Spot}
    outcomes:
      - name: noted
        emits: [shop.tag.Noted]
        payload: {shop.tag.Noted: {mark: {missing: input.spot.id}}}
";

    fn spec(body: &str) -> Result<ess_domain::Specification, String> {
        let raw = ess_domain::spec::RawSpecFile::parse(body).map_err(|e| e.to_string())?;
        ess_domain::Specification::assemble([(ess_domain::system::Source::new("shop.yaml"), raw)])
            .map_err(|e| e.to_string())
    }

    fn accepted(body: &str) -> ess_domain::Specification {
        spec(body).unwrap_or_else(|errors| panic!("{errors}"))
    }

    fn refused(body: &str) -> String {
        match spec(body) {
            Ok(_) => panic!("validated"),
            Err(errors) => errors,
        }
    }

    fn with(from: &str, to: &str) -> String {
        assert!(MODEL.contains(from), "the fixture holds `{from}`");
        MODEL.replacen(from, to, 1)
    }

    /// `body` with both of `shop.route.Check`'s related branches reading `input.settings.<member>`
    /// in place of `input.settings.primary`, so the command still reads one row, answered when
    /// missing.
    fn check_reads(body: &str, member: &str) -> String {
        let absent = "via: input.settings.primary, exists: false}";
        let holds = "via: input.settings.primary, predicate: open == false}";
        assert!(body.contains(absent) && body.contains(holds));
        body.replacen(
            absent,
            &format!("via: input.settings.{member}, exists: false}}"),
            1,
        )
        .replacen(
            holds,
            &format!("via: input.settings.{member}, predicate: open == false}}"),
            1,
        )
    }

    fn outcome<'a>(
        spec: &'a ess_domain::Specification,
        command: &str,
        name: &str,
    ) -> &'a ess_domain::command::Outcome {
        spec.commands()[&command.parse().unwrap()]
            .outcomes
            .iter()
            .find(|outcome| outcome.name.as_str() == name)
            .unwrap_or_else(|| panic!("{command} has {name}"))
    }

    fn reads(condition: &OutcomeCondition) -> (Vec<String>, bool) {
        let OutcomeCondition::Related {
            via, test, also, ..
        } = condition
        else {
            panic!("a related guard: {condition:?}")
        };
        let mut fields = vec![field_of(via)];
        fields.extend(also.iter().map(|row| field_of(&row.via)));
        (fields, matches!(test, RelatedTest::Absent))
    }

    fn field_of(via: &RelatedVia) -> String {
        let RelatedVia::Input(field) = via else {
            panic!("an input via: {via:?}")
        };
        field.clone()
    }

    #[test]
    fn a_list_member_names_a_row_per_element_at_ess_38() {
        let spec = accepted(MODEL);
        let refusal = outcome(&spec, "shop.route.Send", "unknown-depot");
        assert_eq!(
            reads(&refusal.condition),
            (vec!["settings.depots".to_owned()], true)
        );
        assert_eq!(
            refusal.error_payload.get("depot_id"),
            Some(&PayloadSource::MissingElement {
                field: "settings.depots".to_owned()
            })
        );
    }

    #[test]
    fn a_scalar_member_names_one_row_with_exists_false_and_with_a_predicate() {
        let spec = accepted(MODEL);
        assert_eq!(
            reads(&outcome(&spec, "shop.route.Check", "no-depot").condition),
            (vec!["settings.primary".to_owned()], true)
        );
        let closed = &outcome(&spec, "shop.route.Check", "closed").condition;
        assert!(
            matches!(
                closed,
                OutcomeCondition::Related {
                    test: RelatedTest::Holds(_),
                    ..
                }
            ),
            "{closed:?}"
        );
        assert_eq!(reads(closed), (vec!["settings.primary".to_owned()], false));
    }

    #[test]
    fn an_all_element_names_a_member() {
        let spec = accepted(MODEL);
        assert_eq!(
            reads(&outcome(&spec, "shop.route.Pair", "both-closed").condition),
            (
                vec!["settings.primary".to_owned(), "settings.backup".to_owned()],
                false
            )
        );
    }

    #[test]
    fn at_ess_37_each_member_address_is_refused_naming_ess_38() {
        let errors = refused(&with("format: ess/38", "format: ess/37"));
        assert!(
            errors.contains(
                "`via: input.settings.depots` names a member of a struct input, which requires \
                 specification format ess/38"
            ),
            "{errors}"
        );
        for (command, branches) in [
            ("shop.route.Send", 1),
            ("shop.route.Check", 2),
            ("shop.route.Pair", 3),
            ("shop.route.Probe", 2),
        ] {
            let named = errors
                .lines()
                .filter(|line| {
                    line.contains(&format!("command.{command}."))
                        && line.contains("requires specification format ess/38")
                })
                .count();
            assert_eq!(
                named, branches,
                "{command}: one refusal per branch\n{errors}"
            );
        }
        assert!(
            errors.contains("write `format: ess/38` on the source that declares the system"),
            "{errors}"
        );
        assert!(!errors.contains("is not an input of"), "{errors}");
    }

    #[test]
    fn a_member_the_struct_does_not_declare_is_refused_naming_its_members() {
        let errors = refused(&check_reads(MODEL, "primry"));
        assert!(
            errors.contains("`primry` is not a field of `shop.route.Settings`")
                && errors.contains("depots, primary, backup, extra, note"),
            "{errors}"
        );
    }

    #[test]
    fn a_member_that_is_no_identity_is_a_type_mismatch_naming_its_type() {
        let errors = refused(&check_reads(MODEL, "note"));
        assert!(
            errors.contains("`input.settings.note` is `String`")
                && errors.contains("no entity's identity"),
            "{errors}"
        );
    }

    #[test]
    fn a_path_crossing_a_list_is_a_type_mismatch_naming_the_list() {
        let errors = refused(&with(
            "via: input.settings.depots,",
            "via: input.settings.depots.first,",
        ));
        assert!(
            errors.contains(
                "reads past `input.settings.depots`, which is `List<shop.route.DepotId>`"
            ),
            "{errors}"
        );
    }

    #[test]
    fn a_route_through_an_optional_member_validates_as_an_optional_reference() {
        let spec = accepted(MODEL);
        let probe = &spec.commands()[&"shop.route.Probe".parse().unwrap()];
        assert!(
            !probe
                .outcomes
                .iter()
                .any(|outcome| matches!(outcome.condition, OutcomeCondition::InputAbsent)),
            "no branch answers the absent reference"
        );
        assert_eq!(
            reads(&outcome(&spec, "shop.route.Probe", "no-spare").condition),
            (vec!["settings.extra.spare".to_owned()], true)
        );
        accepted(&with(
            "      - {name: extra, type: \"Optional<shop.route.Extra>\"}",
            "      - {name: extra, type: shop.route.Extra}",
        ));
    }

    #[test]
    fn at_ess_37_a_path_missing_source_is_the_nested_mapping_it_was() {
        // Below ess/38 `{missing: input.<path>}` reads as a nested mapping, so it is never read as
        // the first element naming no row; the via is refused naming the format that admits it.
        let errors = refused(&with("format: ess/38", "format: ess/37"));
        assert!(
            !errors.contains("{missing: input.settings.depots}` reads"),
            "{errors}"
        );
        assert!(MODEL.contains("{depot_id: {missing: input.settings.depots}}"));
    }
    #[test]
    fn reads_member_matches_a_read_at_or_below_the_member() {
        use ess_domain::command::input_path::reads_member;
        assert!(reads_member(&["input", "settings", "primary"], "settings"));
        assert!(reads_member(
            &["input", "settings", "primary"],
            "settings.primary"
        ));
        assert!(!reads_member(
            &["input", "settings", "primary"],
            "settings.prim"
        ));
        assert!(reads_member(&["settings", "primary"], "settings.primary"));
        assert!(!reads_member(&["input", "settings"], "settings.primary"));
    }

    #[test]
    fn a_path_crossing_a_union_is_a_type_mismatch_naming_the_union() {
        let body = with(
            "      - {name: note, type: String}\n",
            "      - {name: note, type: String}\n      - {name: choice, type: shop.route.Choice}\n  - {name: shop.route.Choice, kind: union, tag: kind, variants: {depot: shop.route.DepotId, shipment: shop.route.ShipmentId}}\n",
        );
        let errors = refused(&check_reads(&body, "choice.depot"));
        assert!(
            errors.contains(
                "`input.settings.choice.depot` reads past `input.settings.choice`, which is \
                 `shop.route.Choice` and has no members"
            ),
            "{errors}"
        );
    }

    #[test]
    fn a_fixture_over_a_struct_input_a_via_reads_is_refused() {
        let errors = refused(&with(
            "  - name: shop.route.Check\n",
            "  - name: shop.route.Check\n    fixture_inputs: {settings: provided-value}\n",
        ));
        assert!(
            errors.lines().any(|line| {
                line.contains("command.shop.route.Check.fixture_inputs.settings")
                    && line.contains(
                        "a fixture input cannot replace a field read by an outcome predicate",
                    )
            }),
            "{errors}"
        );
    }

    #[test]
    fn a_member_named_missing_keeps_its_struct_reading_at_ess_37_and_is_the_missing_source_at_ess_38(
    ) {
        let spec = accepted(TAG);
        let noted = outcome(&spec, "shop.tag.Note", "noted");
        assert_eq!(
            noted.payload[&"shop.tag.Noted".parse().unwrap()].get("mark"),
            Some(&PayloadSource::Struct {
                fields: vec![PayloadField {
                    target: "missing".to_owned(),
                    source: PayloadSource::InputField {
                        field: "spot.id".to_owned()
                    },
                }]
            })
        );
        let errors = refused(&TAG.replacen("format: ess/37", "format: ess/38", 1));
        assert!(
            errors.contains(
                "`{missing: input.spot.id}` reads the first element of `input.spot.id` that names \
                 no row, and outcome `noted` is not guarded by `when_related: {via: input.spot.id, \
                 exists: false}` over a list of references"
            ),
            "{errors}"
        );
    }
}

/// From `ess/38` (beyond10x/ess#640) `instance:` on `updates:`, `moves:` and `deletes:`, the
/// markers answering it and `each: {in:}` name a member of a struct input; below it each is
/// refused naming `ess/38`.
mod struct_input_member_addressed {
    use ess_domain::command::OutcomeCondition;

    const ADDRESSED: &str = include_str!(
        "../../../verify/ess-conformance/tests/fixtures/struct-input-member-addressed-rows.yaml.in"
    );
    const MODEL: &str = include_str!(
        "../../../verify/ess-conformance/tests/fixtures/struct-input-member-related-rows.yaml.in"
    );

    fn spec(body: &str) -> Result<ess_domain::Specification, String> {
        let raw = ess_domain::spec::RawSpecFile::parse(body).map_err(|e| e.to_string())?;
        ess_domain::Specification::assemble([(ess_domain::system::Source::new("shop.yaml"), raw)])
            .map_err(|e| e.to_string())
    }

    fn accepted(body: &str) -> ess_domain::Specification {
        spec(body).unwrap_or_else(|errors| panic!("{errors}"))
    }

    fn refused(body: &str) -> String {
        match spec(body) {
            Ok(_) => panic!("validated"),
            Err(errors) => errors,
        }
    }

    fn with_in(model: &str, from: &str, to: &str) -> String {
        assert!(model.contains(from), "the model holds `{from}`");
        model.replacen(from, to, 1)
    }

    fn with(from: &str, to: &str) -> String {
        with_in(ADDRESSED, from, to)
    }

    /// `model` without the commands `names`, by their blocks and in the actor's `may:`.
    fn without_commands(model: &str, names: &[&str]) -> String {
        let mut body = model.to_owned();
        for name in names {
            let start = body
                .find(&format!("  - name: {name}\n"))
                .unwrap_or_else(|| panic!("{name} is declared"));
            let rest = &body[start + 1..];
            let end = start
                + 1
                + [rest.find("\n  - name: "), rest.find("\nviews:")]
                    .into_iter()
                    .flatten()
                    .min()
                    .expect("a next block")
                + 1;
            body.replace_range(start..end, "");
            body = body.replacen(&format!(", {name}"), "", 1);
        }
        body
    }

    fn outcome<'a>(
        spec: &'a ess_domain::Specification,
        command: &str,
        name: &str,
    ) -> &'a ess_domain::command::Outcome {
        spec.commands()[&command.parse().unwrap()]
            .outcomes
            .iter()
            .find(|outcome| outcome.name.as_str() == name)
            .unwrap_or_else(|| panic!("{command} has {name}"))
    }

    #[test]
    fn an_instance_member_addresses_updates_moves_and_deletes() {
        let spec = accepted(ADDRESSED);
        for (command, name, path) in [
            ("shop.parcel.Relabel", "relabelled", "target.parcel_id"),
            ("shop.parcel.Hold", "held", "request.parcel_id"),
            ("shop.parcel.Drop", "dropped", "target.parcel_id"),
        ] {
            let subject = outcome(&spec, command, name)
                .subject
                .as_ref()
                .unwrap_or_else(|| panic!("{command}/{name} has a subject"));
            assert_eq!(subject.instance, path, "{command}/{name}");
            assert_eq!(
                outcome(&spec, command, "unknown").condition,
                OutcomeCondition::UnknownInstance,
                "{command}"
            );
        }
    }

    #[test]
    fn an_each_list_member_is_walked() {
        let spec = accepted(ADDRESSED);
        let labelled = outcome(&spec, "shop.parcel.Label", "labelled");
        let [entry] = labelled.set_effects.affects.as_slice() else {
            panic!("one entry: {:?}", labelled.set_effects.affects)
        };
        assert_eq!(
            entry.each.as_ref().map(|each| each.list.as_str()),
            Some("batch.lines")
        );
    }

    #[test]
    fn an_existing_instance_answers_a_creation_publishing_a_member() {
        let spec = accepted(ADDRESSED);
        assert_eq!(
            outcome(&spec, "shop.parcel.Register", "taken").condition,
            OutcomeCondition::ExistingInstance
        );
        // Without `Hold` nothing takes the `hold` transition, so the copy declares none.
        let only = without_commands(
            &with(
                "      states: [Open, Held]\n      terminal: [Held]\n      transitions:\n        - {name: hold, from: [Open], to: Held}\n",
                "      states: [Open]\n      terminal: [Open]\n",
            ),
            &[
                "shop.parcel.Relabel",
                "shop.parcel.Hold",
                "shop.parcel.Drop",
                "shop.parcel.Label",
            ],
        );
        for format in ["format: ess/38", "format: ess/37"] {
            let body = with_in(&only, "format: ess/38", format);
            let spec = accepted(&body);
            assert_eq!(
                outcome(&spec, "shop.parcel.Register", "taken").condition,
                OutcomeCondition::ExistingInstance,
                "{format}"
            );
        }
    }

    #[test]
    fn at_ess_37_every_member_address_is_refused_naming_ess_38() {
        let errors = refused(&with("format: ess/38", "format: ess/37"));
        for text in [
            "`instance: target.parcel_id` names a member of a struct input, which requires \
             specification format ess/38",
            "`each: {in: input.batch.lines}` names a member of a struct input, which requires \
             specification format ess/38",
            "write `format: ess/38` on the source that declares the system",
        ] {
            assert!(errors.contains(text), "{text}\n{errors}");
        }
        for command in [
            "shop.parcel.Relabel",
            "shop.parcel.Hold",
            "shop.parcel.Drop",
            "shop.parcel.Label",
        ] {
            assert!(
                errors.lines().any(|line| {
                    line.contains(&format!("command.{command}."))
                        && line.contains("requires specification format ess/38")
                }),
                "{command}\n{errors}"
            );
        }
        let related = refused(&with_in(MODEL, "format: ess/38", "format: ess/37"));
        for command in [
            "shop.route.Send",
            "shop.route.Check",
            "shop.route.Pair",
            "shop.route.Probe",
        ] {
            assert!(
                related.lines().any(|line| {
                    line.contains(&format!("command.{command}."))
                        && line.contains("requires specification format ess/38")
                }),
                "{command}\n{related}"
            );
        }
        for text in [&errors, &related] {
            for not in [
                "is not an input of",
                "no command input of it",
                "no input field of it",
                "walks a list of the command's input",
            ] {
                assert!(!text.contains(not), "{not}\n{text}");
            }
        }
    }

    #[test]
    fn a_member_the_struct_does_not_declare_is_refused_at_every_position() {
        for (body, owner, members) in [
            (
                with("instance: target.parcel_id", "instance: target.parcel"),
                "is not a field of `shop.parcel.Target`",
                "parcel_id, label",
            ),
            (
                with(
                    "each: {in: input.batch.lines",
                    "each: {in: input.batch.line",
                ),
                "is not a field of `shop.parcel.Batch`",
                "lines, note",
            ),
            (
                with_in(
                    &with_in(
                        MODEL,
                        "via: input.settings.primary, exists: false}",
                        "via: input.settings.primry, exists: false}",
                    ),
                    "via: input.settings.primary, predicate: open == false}",
                    "via: input.settings.primry, predicate: open == false}",
                ),
                "is not a field of `shop.route.Settings`",
                "depots, primary, backup, extra, note",
            ),
        ] {
            let errors = refused(&body);
            assert!(
                errors.contains(owner) && errors.contains(members),
                "{owner}\n{errors}"
            );
        }
    }

    #[test]
    fn a_member_of_the_wrong_type_or_a_crossed_list_is_a_type_mismatch() {
        for (body, text) in [
            (
                with("instance: target.parcel_id", "instance: target.label"),
                "names the instance by `target.label`, which is typed `String`",
            ),
            (
                with(
                    "each: {in: input.batch.lines",
                    "each: {in: input.batch.note",
                ),
                "`each:` reads `input.batch.note`, which is `String`",
            ),
            (
                with("instance: target.parcel_id", "instance: target.parcel_id.x"),
                "reads past `input.target.parcel_id`, which is `shop.parcel.ParcelId`",
            ),
            (
                with_in(
                    MODEL,
                    "via: input.settings.depots,",
                    "via: input.settings.depots.first,",
                ),
                "reads past `input.settings.depots`, which is `List<shop.route.DepotId>`",
            ),
        ] {
            let errors = refused(&body);
            assert!(errors.contains(text), "{text}\n{errors}");
        }
    }

    #[test]
    fn an_optional_route_reads_as_an_optional_reference() {
        accepted(MODEL);
        let errors = refused(&with(
            "      - {name: request, type: shop.parcel.Target}",
            "      - {name: request, type: \"Optional<shop.parcel.Target>\"}",
        ));
        assert!(
            errors.contains(
                "names the instance by `request.parcel_id`, which is typed \
                 `Optional<shop.parcel.ParcelId>`"
            ),
            "{errors}"
        );
    }

    #[test]
    fn a_guard_reading_the_member_that_names_the_instance_is_refused() {
        let errors = refused(&with(
            "      - name: relabelled\n",
            "      - name: relabelled\n        when: \"target.parcel_id != target.parcel_id\"\n",
        ));
        assert!(errors.contains("an identity is opaque"), "{errors}");
    }

    #[test]
    fn a_path_crossing_a_union_is_a_type_mismatch_at_instance_and_each() {
        let union = with_in(
            &with(
                "      - {name: note, type: String}\n",
                "      - {name: note, type: String}\n  - {name: shop.parcel.Kind, kind: union, \
                 tag: kind, variants: {parcel: shop.parcel.ParcelId, crate: shop.parcel.CrateId}}\n",
            ),
            "      - {name: label, type: String}\n",
            "      - {name: label, type: String}\n      - {name: kind, type: shop.parcel.Kind}\n",
        );
        let errors = refused(&with_in(
            &union,
            "instance: target.parcel_id",
            "instance: target.kind.parcel",
        ));
        assert!(
            errors.contains(
                "reads past `input.target.kind`, which is `shop.parcel.Kind` and has no members"
            ),
            "{errors}"
        );
        let errors = refused(&with_in(
            &with_in(
                &union,
                "      - {name: note, type: String}\n  - {name: shop.parcel.Kind",
                "      - {name: note, type: String}\n      - {name: choice, type: \
                 shop.parcel.Kind}\n  - {name: shop.parcel.Kind",
            ),
            "each: {in: input.batch.lines",
            "each: {in: input.batch.choice.lines",
        ));
        assert!(
            errors.contains("reads past `input.batch.choice`, which is `shop.parcel.Kind`"),
            "{errors}"
        );
    }

    #[test]
    fn a_distinct_over_a_member_list_holds_it_distinct() {
        let unordered = with(", repeated: last_wins}", "}");
        let errors = refused(&unordered);
        assert!(
            errors.contains("`input.batch.lines`")
                && errors.contains("no `distinct:` holds that member distinct across the list"),
            "{errors}"
        );
        for list in ["batch.lines", "input.batch.lines"] {
            accepted(&with_in(
                &unordered,
                "      - name: no-crate\n",
                &format!(
                    "      - name: duplicated\n        when: {{not: {{distinct: {{in: {list}, as: \
                     d, by: d.parcel_id}}}}}}\n        error: shop.parcel.Taken\n      - name: \
                     no-crate\n"
                ),
            ));
        }
    }
}
