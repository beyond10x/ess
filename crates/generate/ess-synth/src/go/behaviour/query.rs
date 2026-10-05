//! One generated view query for the Go target: the rows of a fully declared view, read through its
//! source entity's storage port — the Go rendering of the Rust target's `behaviour::query`.
//!
//! The rows the port lists are the rows the query reads, in the order the port lists them. A
//! `filter:` keeps a row where it holds and drops it where it is false or unknown; an `order_by:`
//! sorts what is kept, stably; a projection moves each kept row's fields into the view's row. An
//! aggregation partitions the kept rows by their group keys, in the order each partition's first
//! row arrives, and computes every function over each partition as the conformance suite's
//! `aggregate::evaluate` does; an ungrouped aggregation is one partition, present even when empty.

use std::collections::{BTreeMap, BTreeSet};

use ess_compiler::ir::{ResolvedAggregate, ResolvedEntity, ResolvedField, ResolvedView};
use ess_domain::name::QualifiedName;
use ess_domain::view::{AggregateFunction, Direction};

use super::super::invariant::{fresh, Lines};
use super::super::selection::go_string;
use super::super::{items, Emit};
use super::{Guards, Uses, Writer};
use crate::determined::{Env, Kind};
use crate::view_query::{self, Values};

/// The query's whole method.
pub(super) fn method(
    emit: &Emit<'_>,
    view: &ResolvedView,
    storages: &BTreeMap<QualifiedName, String>,
    uses: &mut Uses,
    reserved: &BTreeSet<String>,
    receiver: &str,
) -> String {
    let entity = emit.ir.entity(&view.source);
    uses.storages.insert(entity.name.clone());
    uses.listed.insert(entity.name.clone());
    // The parameters the query string carries, decoded by the route as the port takes them, named
    // apart from the receiver, every package-level name the body calls or qualifies by — each
    // helper, each imported package — and every local the body binds; none for a view without
    // parameters, which keeps its bytes (beyond10x/ess#200).
    let package = super::package_level(reserved);
    let taken: Vec<&str> = package
        .iter()
        .map(String::as_str)
        .chain([receiver])
        .collect();
    let arguments = super::super::port::view_params(emit, view, &taken);
    let mut reserved = reserved.clone();
    reserved.extend(arguments.iter().map(|(ident, _)| ident.clone()));
    let params = view
        .params
        .iter()
        .cloned()
        .zip(arguments.iter().map(|(ident, _)| ident.clone()))
        .collect();
    let mut query = Query {
        emit,
        view,
        entity,
        uses,
        lines: Lines::new(1),
        reserved: &reserved,
        receiver,
        next: 0,
        params,
    };
    query.body(&storages[&entity.name]);
    let method = emit.layout.declared(&view.name);
    let row = emit.reference(&view.name);
    let unmet = emit.unmet();
    let signature = super::super::port::signature(&arguments);
    format!(
        "\n// {method} is `{}`, generated: every row is one the specification fully determines \
         from the\n// stored `{}`s.\nfunc ({receiver} *Generated) {method}({signature}) ([]{row}, {unmet}) \
         {{\n{}}}\n",
        view.name,
        entity.name,
        query.lines.text()
    )
}

/// Everything rendering one query needs.
struct Query<'a, 'u> {
    emit: &'a Emit<'a>,
    view: &'a ResolvedView,
    entity: &'a ResolvedEntity,
    uses: &'u mut Uses,
    lines: Lines,
    reserved: &'a BTreeSet<String>,
    receiver: &'a str,
    next: usize,
    /// The view's parameters, each with the identifier its argument is bound to.
    params: Vec<(ResolvedField, String)>,
}

impl<'a> Query<'a, '_> {
    /// A fresh local named from `base`.
    fn temp(&mut self, base: &str) -> String {
        let candidate = format!("{base}{}", self.next);
        self.next += 1;
        fresh(self.reserved, &candidate)
    }

    /// A fixed local name, moved out of the way of every package name.
    fn local(&self, base: &str) -> String {
        fresh(self.reserved, base)
    }

    /// A guard renderer reading a stored row from `row`.
    fn guards<'g>(&'g mut self, row: &str) -> Guards<'a, 'g> {
        Guards {
            ir: self.emit.ir,
            emit: self.emit,
            uses: &mut *self.uses,
            lines: &mut self.lines,
            reserved: self.reserved,
            next: &mut self.next,
            receiver: self.receiver,
            row: row.to_owned(),
            input: String::new(),
            command: None,
            row_entity: Some(self.entity),
            params: self.params.clone(),
        }
    }

    /// The source entity's snapshot type.
    fn snapshot(&self) -> String {
        self.emit.qualify(
            self.emit.layout.package_of(&self.entity.name),
            self.emit.layout.snapshot(&self.entity.name),
        )
    }

    /// The method body: list, filter, then order and project, or aggregate.
    fn body(&mut self, storage: &str) {
        let view = self.view;
        let listed = self.local("listed");
        let receiver = self.receiver;
        self.lines
            .push(&format!("{listed} := {receiver}.ports.{storage}.List()"));
        let kept = if let Some(filter) = &view.filter {
            let kept = self.local("kept");
            let held = self.local("held");
            let snapshot = self.snapshot();
            self.lines
                .push("// `filter:` shows a row where it holds; false or unknown hides it.");
            self.lines.push(&format!("var {kept} []{snapshot}"));
            self.lines
                .open(&format!("for _, {held} := range {listed} {{"));
            let entity = self.entity;
            let truth = self.guards(&held).predicate(&Env::Row(entity), filter);
            self.lines.open(&format!("if {truth} == verity {{"));
            self.lines.push(&format!("{kept} = append({kept}, {held})"));
            self.lines.close("}");
            self.lines.close("}");
            kept
        } else {
            listed
        };
        if view.aggregation.is_some() {
            self.aggregation(&kept);
        } else {
            if !view.order_by.is_empty() {
                self.ordering(&kept);
            }
            self.projection(&kept);
        }
    }

    /// Sorts `kept` by `order_by:`, stably: rows equal on every key keep the port's order.
    fn ordering(&mut self, kept: &str) {
        self.emit.import("sort");
        let (left, right) = (self.local("left"), self.local("right"));
        self.lines
            .push("// `order_by:`, stably: rows equal on every key keep the port's order.");
        self.lines.open(&format!(
            "sort.SliceStable({kept}, func(i int, j int) bool {{"
        ));
        self.lines
            .push(&format!("{left}, {right} := {kept}[i], {kept}[j]"));
        for ranking in &self.view.order_by {
            let resolved = view_query::resolve(self.emit.ir, self.entity, &ranking.field)
                .expect("the plan admitted only keys that resolve");
            let left_read = self.guards(&left).read(&resolved);
            let right_read = self.guards(&right).read(&resolved);
            let compared = match view_query::values(self.emit.ir, self.entity, &ranking.field) {
                Some(Values::Numeric) => {
                    self.uses.helpers.insert("numberOrder");
                    self.uses.helpers.insert("readOrder");
                    format!("numberOrder(textOf({left_read}), textOf({right_read}))")
                }
                _ if resolved.kind == Kind::Bool => {
                    self.uses.helpers.insert("readOrder");
                    format!("flagOrder({left_read}, {right_read})")
                }
                _ => {
                    self.uses.helpers.insert("readOrder");
                    format!("readOrder({left_read}, {right_read})")
                }
            };
            let wins = match ranking.direction {
                Direction::Ascending => "c < 0",
                Direction::Descending => "c > 0",
            };
            self.lines.open(&format!("if c := {compared}; c != 0 {{"));
            self.lines.push(&format!("return {wins}"));
            self.lines.close("}");
        }
        self.lines.push("return false");
        self.lines.close("})");
    }

    /// Each kept row, moved into the view's row.
    fn projection(&mut self, kept: &str) {
        let row = self.emit.reference(&self.view.name);
        let (rows, held) = (self.local("rows"), self.local("held"));
        self.lines
            .push(&format!("{rows} := make([]{row}, 0, len({kept}))"));
        self.lines
            .open(&format!("for _, {held} := range {kept} {{"));
        let fields = self.row_fields(&held);
        self.lines
            .push(&format!("{rows} = append({rows}, {row}{{{fields}}})"));
        self.lines.close("}");
        self.lines.push(&format!("return {rows}, nil"));
    }

    /// Every field of the view's row read from the snapshot `held`, for a projection.
    fn row_fields(&mut self, held: &str) -> String {
        let mut fields = Vec::new();
        for field in &self.view.fields {
            let value = self.read_field(field, held);
            fields.push(format!(
                "{}: {value}",
                items::member_ident(&self.view.fields, &field.name)
            ));
        }
        fields.join(", ")
    }

    /// One field of the source row `held`, wrapped as present where the view declares it
    /// `Optional` and the entity holds it required.
    fn read_field(&mut self, field: &ResolvedField, held: &str) -> String {
        let value = if field.name == "state" {
            format!("{held}.State")
        } else {
            format!(
                "{held}.Data.{}",
                Writer::data_member(self.entity, &field.name)
            )
        };
        let source = if field.name == "state" {
            None
        } else {
            std::iter::once(&self.entity.identity)
                .chain(&self.entity.fields)
                .find(|stored| stored.name == field.name)
        };
        let required_source = source.is_none_or(|stored| !stored.type_ref.is_optional());
        if field.type_ref.is_optional() && required_source {
            self.uses.helpers.insert("some");
            format!("some({value})")
        } else {
            value
        }
    }

    /// Partitions and computes an aggregation over `kept`.
    // Grouping, then one field per aggregate, each in the order the suite reads it.
    #[allow(clippy::too_many_lines)]
    fn aggregation(&mut self, kept: &str) {
        let view = self.view;
        let aggregation = view
            .aggregation
            .as_ref()
            .expect("an aggregation is rendered only for an aggregate view");
        let row = self.emit.reference(&view.name);
        let snapshot = self.snapshot();
        let members = self.local("members");
        let first = self.local("first");
        if aggregation.group_by.is_empty() {
            self.lines
                .push("// Ungrouped: one row over every kept row, present when none is.");
            self.lines.push(&format!("{members} := {kept}"));
            let fields = self.aggregate_fields(&members, &first);
            self.lines
                .push(&format!("return []{row}{{{{{fields}}}}}, nil"));
            return;
        }
        self.uses.helpers.insert("groupKey");
        let (keys, groups, held, key, placed, index, seen, rows) = (
            self.local("groupKeys"),
            self.local("groups"),
            self.local("held"),
            self.local("key"),
            self.local("placed"),
            self.local("index"),
            self.local("seen"),
            self.local("rows"),
        );
        self.lines.push(
            "// One partition per group-key tuple, in the order its first row arrives; an absent \
             key is",
        );
        self.lines
            .push("// one value, and a number is keyed by its value, not its spelling.");
        self.lines.push(&format!("var {keys} [][]string"));
        self.lines.push(&format!("var {groups} [][]{snapshot}"));
        self.lines
            .open(&format!("for _, {held} := range {kept} {{"));
        let mut parts = Vec::new();
        for name in &aggregation.group_by {
            let resolved = view_query::resolve(self.emit.ir, self.entity, name)
                .expect("the plan admitted only keys that resolve");
            let read = self.guards(&held).text(&resolved);
            parts.push(match resolved.kind {
                Kind::Number(_) => {
                    self.uses.helpers.insert("numberKey");
                    format!("keyOf(numberKeyOf({read}))")
                }
                _ => format!("keyOf({read})"),
            });
        }
        self.lines
            .push(&format!("{key} := []string{{{}}}", parts.join(", ")));
        self.lines.push(&format!("{placed} := false"));
        self.lines
            .open(&format!("for {index}, {seen} := range {keys} {{"));
        self.lines.open(&format!("if sameKey({seen}, {key}) {{"));
        self.lines.push(&format!(
            "{groups}[{index}] = append({groups}[{index}], {held})"
        ));
        self.lines.push(&format!("{placed} = true"));
        self.lines.push("break");
        self.lines.close("}");
        self.lines.close("}");
        self.lines.open(&format!("if !{placed} {{"));
        self.lines.push(&format!("{keys} = append({keys}, {key})"));
        self.lines.push(&format!(
            "{groups} = append({groups}, []{snapshot}{{{held}}})"
        ));
        self.lines.close("}");
        self.lines.close("}");
        self.lines
            .push(&format!("{rows} := make([]{row}, 0, len({groups}))"));
        self.lines
            .open(&format!("for _, {members} := range {groups} {{"));
        self.lines.push(&format!("{first} := {members}[0]"));
        self.lines.push(&format!("_ = {first}"));
        let fields = self.aggregate_fields(&members, &first);
        self.lines
            .push(&format!("{rows} = append({rows}, {row}{{{fields}}})"));
        self.lines.close("}");
        self.lines.push(&format!("return {rows}, nil"));
    }

    /// Every field of one aggregate row: a group key read from `first`, or an aggregate over
    /// `members`.
    fn aggregate_fields(&mut self, members: &str, first: &str) -> String {
        let aggregation = self.view.aggregation.as_ref().expect("an aggregate view");
        let mut fields = Vec::new();
        for field in &self.view.fields {
            let value = if let Some(aggregate) = aggregation.functions.get(&field.name) {
                self.aggregate(aggregate, members)
            } else {
                self.read_field(field, first)
            };
            fields.push(format!(
                "{}: {value}",
                items::member_ident(&self.view.fields, &field.name)
            ));
        }
        fields.join(", ")
    }

    /// `return nil, unrepresentable("<view>")`, inside an `if` on `condition`.
    fn refuse_if(&mut self, condition: &str) {
        self.uses.helpers.insert("unrepresentable");
        self.lines.open(&format!("if {condition} {{"));
        self.lines.push(&format!(
            "return nil, unrepresentable({})",
            go_string(&self.view.name.to_string())
        ));
        self.lines.close("}");
    }

    /// The present values one aggregate reads over `members`, each in the form `keyed` gives it,
    /// collected into a fresh slice.
    fn values(&mut self, input: &ResolvedField, members: &str, keyed: &str) -> String {
        let resolved = view_query::resolve(self.emit.ir, self.entity, &input.name)
            .expect("the plan admitted only inputs that resolve");
        let (values, held) = (self.temp("a"), self.local("held"));
        self.lines.push(&format!("var {values} []string"));
        self.lines
            .open(&format!("for _, {held} := range {members} {{"));
        let read = self.guards(&held).text(&resolved);
        self.lines.open(&format!("if {read} != nil {{"));
        let value = if keyed.is_empty() {
            format!("*{read}")
        } else {
            format!("{keyed}(*{read})")
        };
        self.lines
            .push(&format!("{values} = append({values}, {value})"));
        self.lines.close("}");
        self.lines.close("}");
        values
    }

    /// One aggregate field over `members`, the rows of one partition — or, where the measure
    /// declares `where:` (beyond10x/ess#363), over the members its condition holds for. An
    /// unknown condition answers no row at all: the whole read is undetermined, never a partial
    /// one.
    fn aggregate(&mut self, aggregate: &ResolvedAggregate, members: &str) -> String {
        match &aggregate.r#where {
            None => self.unconditioned(aggregate, members),
            Some(condition) => {
                let selected = self.select(condition, members);
                self.unconditioned(aggregate, &selected)
            }
        }
    }

    /// The members of `members` `condition` holds for, collected into a fresh slice.
    fn select(
        &mut self,
        condition: &ess_primitives::predicate::Predicate,
        members: &str,
    ) -> String {
        let snapshot = self.snapshot();
        let (selected, held, reading) = (
            self.temp("selected"),
            self.local("held"),
            self.temp("reading"),
        );
        self.lines
            .push("// `where:`: this measure reads only the rows its condition holds for.");
        self.lines.push(&format!("var {selected} []{snapshot}"));
        self.lines
            .open(&format!("for _, {held} := range {members} {{"));
        let entity = self.entity;
        let truth = self.guards(&held).predicate(&Env::Row(entity), condition);
        self.lines.push(&format!("{reading} := {truth}"));
        self.refuse_if(&format!("{reading} == unknown"));
        self.lines.open(&format!("if {reading} == verity {{"));
        self.lines
            .push(&format!("{selected} = append({selected}, {held})"));
        self.lines.close("}");
        self.lines.close("}");
        selected
    }

    /// One aggregate field over every row of `members`.
    fn unconditioned(&mut self, aggregate: &ResolvedAggregate, members: &str) -> String {
        let Some(input) = &aggregate.input else {
            return format!("int64(len({members}))");
        };
        let values = view_query::values(self.emit.ir, self.entity, &input.name)
            .expect("the plan admitted only inputs it compares");
        match aggregate.function {
            AggregateFunction::Count => format!("int64(len({members}))"),
            AggregateFunction::CountDistinct => {
                self.uses.helpers.insert("distinct");
                let keyed = match values {
                    Values::Numeric => {
                        self.uses.helpers.insert("numberKey");
                        "numberKey"
                    }
                    Values::Instant => {
                        self.uses.helpers.insert("instantKey");
                        "instantKey"
                    }
                    Values::Text | Values::Other => "",
                };
                let collected = self.values(input, members, keyed);
                format!("distinct({collected})")
            }
            AggregateFunction::Sum => self.sum(aggregate, input, members),
            AggregateFunction::Avg => {
                self.uses.helpers.insert("sumValues");
                self.uses.helpers.insert("average");
                self.uses.helpers.insert("some");
                let collected = self.values(input, members, "");
                let (present, units, scale, ok) = (
                    self.temp("present"),
                    self.temp("units"),
                    self.temp("scale"),
                    self.temp("ok"),
                );
                self.lines.push(&format!(
                    "{present}, {units}, {scale}, {ok} := sumValues({collected})"
                ));
                self.refuse_if(&format!("!{ok}"));
                let (mean, averaged, result) =
                    (self.temp("mean"), self.temp("ok"), self.temp("avg"));
                self.lines.push(&format!(
                    "{mean}, {averaged} := average({present}, {units}, {scale})"
                ));
                self.refuse_if(&format!("!{averaged}"));
                let decimal = self
                    .emit
                    .primitive_type(ess_domain::types::Primitive::Decimal);
                let ctor = self
                    .emit
                    .primitive_ctor(ess_domain::types::Primitive::Decimal);
                self.lines.push(&format!("var {result} *{decimal}"));
                self.lines.open(&format!("if {mean} != nil {{"));
                self.lines
                    .push(&format!("{result} = some({ctor}(*{mean}))"));
                self.lines.close("}");
                result
            }
            AggregateFunction::Min | AggregateFunction::Max => {
                self.extreme(aggregate, input, members, values)
            }
        }
    }

    /// A `sum` over the present values: exact, an `Integer` or a `Decimal` as the input is; `0`
    /// over no value, or absent where the aggregate skips absent values.
    fn sum(
        &mut self,
        aggregate: &ResolvedAggregate,
        input: &ResolvedField,
        members: &str,
    ) -> String {
        self.uses.helpers.insert("sumValues");
        let decimal = matches!(
            view_query::leaf(self.emit.ir, &input.type_ref),
            Some(ess_domain::types::Primitive::Decimal)
        );
        let collected = self.values(input, members, "");
        let (present, units, scale, ok) = (
            self.temp("present"),
            self.temp("units"),
            self.temp("scale"),
            self.temp("ok"),
        );
        self.lines.push(&format!(
            "{}, {units}, {}, {ok} := sumValues({collected})",
            if aggregate.skip_absent {
                present.as_str()
            } else {
                "_"
            },
            if decimal { scale.as_str() } else { "_" }
        ));
        self.refuse_if(&format!("!{ok}"));
        let (value, go_type) = if decimal {
            self.uses.helpers.insert("spell");
            (
                format!(
                    "{}(spell({units}, {scale}))",
                    self.emit
                        .primitive_ctor(ess_domain::types::Primitive::Decimal)
                ),
                self.emit
                    .primitive_type(ess_domain::types::Primitive::Decimal),
            )
        } else {
            self.refuse_if(&format!("!{units}.IsInt64()"));
            (format!("{units}.Int64()"), "int64".to_owned())
        };
        if !aggregate.skip_absent {
            return value;
        }
        self.uses.helpers.insert("some");
        let result = self.temp("sum");
        self.lines.push(&format!("var {result} *{go_type}"));
        self.lines.open(&format!("if {present} > 0 {{"));
        self.lines.push(&format!("{result} = some({value})"));
        self.lines.close("}");
        result
    }

    /// A `min` or `max`: the value whose reading the order ranks first or last, the first of equal
    /// ones; absent over no value.
    fn extreme(
        &mut self,
        aggregate: &ResolvedAggregate,
        input: &ResolvedField,
        members: &str,
        values: Values,
    ) -> String {
        let order = match values {
            Values::Numeric => "numberOrder",
            Values::Instant => "instantOrder",
            Values::Text | Values::Other => "textOrder",
        };
        self.uses.helpers.insert(order);
        self.uses.helpers.insert("some");
        let wins = if aggregate.function == AggregateFunction::Min {
            "< 0"
        } else {
            "> 0"
        };
        let resolved = view_query::resolve(self.emit.ir, self.entity, &input.name)
            .expect("the plan admitted only inputs that resolve");
        let go_type = self.emit.go_type(input.type_ref.required());
        let (best, chosen, have, held) = (
            self.temp("best"),
            self.temp("chosen"),
            self.temp("have"),
            self.local("held"),
        );
        self.lines.push(&format!("var {best} string"));
        self.lines.push(&format!("var {chosen} {go_type}"));
        self.lines.push(&format!("{have} := false"));
        self.lines
            .open(&format!("for _, {held} := range {members} {{"));
        let read = self.guards(&held).text(&resolved);
        self.lines.open(&format!("if {read} == nil {{"));
        self.lines.push("continue");
        self.lines.close("}");
        let stored = format!(
            "{held}.Data.{}",
            Writer::data_member(self.entity, &input.name)
        );
        let value = if input.type_ref.is_optional() {
            self.lines.open(&format!("if {stored} == nil {{"));
            self.lines.push("continue");
            self.lines.close("}");
            format!("*{stored}")
        } else {
            stored
        };
        self.lines
            .open(&format!("if !{have} || {order}(*{read}, {best}) {wins} {{"));
        self.lines.push(&format!(
            "{best}, {chosen}, {have} = *{read}, {value}, true"
        ));
        self.lines.close("}");
        self.lines.close("}");
        let result = self.temp("extreme");
        self.lines.push(&format!("var {result} *{go_type}"));
        self.lines.open(&format!("if {have} {{"));
        self.lines.push(&format!("{result} = some({chosen})"));
        self.lines.close("}");
        result
    }
}
