//! One generated view query: the rows of a fully declared view, read through its source entity's
//! storage port.
//!
//! The rows the port lists are the rows the query reads — in the order the port lists them, which
//! is the order an unordered view answers in. A `filter:` keeps a row where it holds and drops it
//! where it is false or unknown; an `order_by:` sorts what is kept, stably, so rows equal on every
//! key stay in the port's order; a projection moves each kept row's fields into the view's row.
//! An aggregation partitions the kept rows by their group keys, in the order each partition's first
//! row arrives, and computes every function over each partition as the conformance suite's
//! `aggregate::evaluate` does; an ungrouped aggregation is one partition, present even when empty.
//! `crate::view_query` has already decided that every construct here is one this renders.

use std::fmt::Write as _;

use ess_compiler::ir::{EssIr, ResolvedAggregate, ResolvedEntity, ResolvedField, ResolvedView};
use ess_domain::name::QualifiedName;
use ess_domain::view::{AggregateFunction, Direction};

use super::{declared_path, snapshot_path, Bounds, Guards, Layout, Uses};
use crate::determined::{Env, Kind};
use crate::view_query::{self, Values};

/// The query's whole impl block.
pub(super) fn implementation(
    ir: &EssIr,
    layout: &Layout,
    view: &ResolvedView,
    storages: &std::collections::BTreeMap<QualifiedName, String>,
    uses: &mut Uses,
) -> String {
    let entity = ir.entity(&view.source);
    uses.storages.insert(entity.name.clone());
    uses.listed.insert(entity.name.clone());
    let mut bounds = Bounds::default();
    bounds.storages.insert(entity.name.clone());
    let mut query = Query {
        ir,
        layout,
        view,
        entity,
        uses,
        bounds: &mut bounds,
    };
    let body = query.body(&storages[&entity.name]);
    let type_name = layout.type_name(&view.name);
    let module = layout.module(layout.owner(&view.name)).to_owned();
    let method = super::name::value_ident(&type_name);
    format!(
        "\n/// `{}`, generated: every row is one the specification fully determines from the stored \
         `{}`s.\nimpl<P> crate::{module}::obligations::{type_name}Query for Generated<P>\nwhere\n    \
         P: {},\n{{\n    fn {method}(&self) -> Result<Vec<crate::{module}::{type_name}>, \
         UnmetObligation> {{\n{body}    }}\n}}\n",
        view.name, entity.name, storages[&entity.name]
    )
}

/// Everything rendering one query needs.
struct Query<'a> {
    ir: &'a EssIr,
    layout: &'a Layout,
    view: &'a ResolvedView,
    entity: &'a ResolvedEntity,
    uses: &'a mut Uses,
    bounds: &'a mut Bounds,
}

impl Query<'_> {
    /// A guard renderer reading a stored row from `row`.
    fn guards<'g>(&'g mut self, row: &'g str) -> Guards<'g> {
        Guards {
            ir: self.ir,
            layout: self.layout,
            uses: &mut *self.uses,
            bounds: &mut *self.bounds,
            row,
        }
    }

    /// The method body: list, filter, then project or aggregate.
    fn body(&mut self, storage: &str) -> String {
        let view = self.view;
        let listed = if view.aggregation.is_some() && view_ungrouped(view) {
            "members"
        } else {
            "admitted"
        };
        let mutable = if view.filter.is_some() || !view.order_by.is_empty() {
            "mut "
        } else {
            ""
        };
        let mut out = format!("        let {mutable}{listed} = {storage}::list(&self.ports);\n");
        if let Some(filter) = &view.filter {
            let entity = self.entity;
            let truth = self.guards("held").predicate(&Env::Row(entity), filter);
            let _ = writeln!(
                out,
                "        // `filter:` shows a row where it holds; false or unknown hides it.\n        \
                 {listed}.retain(|held| {truth} == Some(true));"
            );
        }
        if view.aggregation.is_some() {
            out.push_str(&self.aggregation(listed));
        } else {
            if !view.order_by.is_empty() {
                let ordering = self.ordering();
                let _ = writeln!(
                    out,
                    "        // `order_by:`, stably: rows equal on every key keep the port's \
                     order.\n        admitted.sort_by(|left, right| {ordering});"
                );
            }
            out.push_str(&self.projection());
        }
        out
    }

    /// The comparison `order_by:` sorts by, most significant key first.
    fn ordering(&mut self) -> String {
        let mut keys = Vec::new();
        for ranking in &self.view.order_by {
            let resolved = view_query::resolve(self.ir, self.entity, &ranking.field)
                .expect("the plan admitted only keys that resolve");
            let left = self.guards("left").read(&resolved);
            let right = self.guards("right").read(&resolved);
            let compared = match view_query::values(self.ir, self.entity, &ranking.field) {
                Some(Values::Numeric) => {
                    self.uses.helpers.insert("number_order");
                    format!(
                        "number_order(&{left}.unwrap_or_default(), &{right}.unwrap_or_default())"
                    )
                }
                _ => format!("{left}.cmp(&{right})"),
            };
            keys.push(match ranking.direction {
                Direction::Ascending => compared,
                Direction::Descending => format!("{compared}.reverse()"),
            });
        }
        let mut ordering = keys.remove(0);
        for key in keys {
            let _ = write!(ordering, ".then_with(|| {key})");
        }
        ordering
    }

    /// Each kept row, moved into the view's row.
    fn projection(&self) -> String {
        let row = declared_path(self.layout, &self.view.name);
        let mut fields = Vec::new();
        for field in &self.view.fields {
            let value = if field.name == "state" {
                "held.state".to_owned()
            } else {
                format!("held.data.{}", super::name::value_ident(&field.name))
            };
            fields.push(format!(
                "                {}: {},",
                super::name::value_ident(&field.name),
                self.wrapped(field, value)
            ));
        }
        format!(
            "        Ok(admitted\n            .into_iter()\n            .map(|held| {row} {{\n{}\n            \
             }})\n            .collect())\n",
            fields.join("\n")
        )
    }

    /// `value`, wrapped as present where the view declares the field `Optional` and the entity
    /// holds it required.
    fn wrapped(&self, field: &ResolvedField, value: String) -> String {
        let source = if field.name == "state" {
            None
        } else {
            std::iter::once(&self.entity.identity)
                .chain(&self.entity.fields)
                .find(|stored| stored.name == field.name)
        };
        let required_source = source.is_none_or(|stored| !stored.type_ref.is_optional());
        if field.type_ref.is_optional() && required_source {
            format!("Some({value})")
        } else {
            value
        }
    }

    /// Partitions and computes an aggregation over `listed`, the kept rows.
    fn aggregation(&mut self, listed: &str) -> String {
        let view = self.view;
        let aggregation = view
            .aggregation
            .as_ref()
            .expect("an aggregation is rendered only for an aggregate view");
        let snapshot = snapshot_path(self.layout, &self.entity.name);
        let mut out = String::new();
        let first = if aggregation.group_by.is_empty() {
            None
        } else {
            let mut keys = Vec::new();
            for key in &aggregation.group_by {
                let resolved = view_query::resolve(self.ir, self.entity, key)
                    .expect("the plan admitted only keys that resolve");
                let read = self.guards("held").read(&resolved);
                keys.push(match resolved.kind {
                    Kind::Number(_) => {
                        self.uses.helpers.insert("number_key");
                        format!("{read}.map(|value| number_key(&value))")
                    }
                    _ => read,
                });
            }
            let _ = writeln!(
                out,
                "        // One partition per group-key tuple, in the order its first row arrives; \
                 an absent key is\n        // one value, and a number is keyed by its value, not \
                 its spelling.\n        let mut groups: Vec<(_, Vec<{snapshot}>)> = \
                 Vec::new();\n        for held in {listed} {{\n            let key = ({},);\n            \
                 match groups.iter_mut().find(|(seen, _)| *seen == key) {{\n                \
                 Some((_, members)) => members.push(held),\n                None => \
                 groups.push((key, vec![held])),\n            }}\n        }}\n        let mut rows = \
                 Vec::new();\n        for (_, members) in groups {{\n            let first = \
                 &members[0];",
                keys.join(", ")
            );
            Some("first")
        };
        let indent = if first.is_some() {
            "                "
        } else {
            "            "
        };
        let row = declared_path(self.layout, &view.name);
        let mut fields = Vec::new();
        for field in &view.fields {
            let value = if let Some(aggregate) = aggregation.functions.get(&field.name) {
                self.aggregate(aggregate)
            } else {
                let read = if field.name == "state" {
                    "first.state".to_owned()
                } else {
                    format!(
                        "first.data.{}.clone()",
                        super::name::value_ident(&field.name)
                    )
                };
                self.wrapped(field, read)
            };
            fields.push(format!(
                "{indent}{}: {value},",
                super::name::value_ident(&field.name)
            ));
        }
        if first.is_some() {
            let _ = writeln!(
                out,
                "            rows.push({row} {{\n{}\n            }});\n        }}\n        Ok(rows)",
                fields.join("\n")
            );
        } else {
            let _ = writeln!(
                out,
                "        // Ungrouped: one row over every kept row, present when none is.\n        \
                 Ok(vec![{row} {{\n{}\n        }}])",
                fields.join("\n")
            );
        }
        out
    }

    /// One aggregate field over `members`, the rows of one partition — or, where the measure
    /// declares `where:` (beyond10x/ess#363), over the members its condition holds for. An
    /// unknown condition answers no row at all: the whole read is undetermined, never a partial
    /// one.
    fn aggregate(&mut self, aggregate: &ResolvedAggregate) -> String {
        let computed = self.unconditioned(aggregate);
        let Some(condition) = &aggregate.r#where else {
            return computed;
        };
        self.uses.helpers.insert("unrepresentable");
        let source = self.view.name.to_string();
        let entity = self.entity;
        let truth = self.guards("held").predicate(&Env::Row(entity), condition);
        format!(
            "{{\n{indent}    // `where:`: this measure reads only the rows its condition holds for.\n{indent}    \
             let mut selected = Vec::new();\n{indent}    for held in members.iter() {{\n{indent}        \
             match {truth} {{\n{indent}            Some(true) => selected.push(held),\n{indent}            \
             Some(false) => {{}}\n{indent}            None => return Err(unrepresentable(\"{source}\")),\n{indent}        \
             }}\n{indent}    }}\n{indent}    let members = selected;\n{indent}    {computed}\n{indent}}}",
            indent = self.indent()
        )
    }

    /// One aggregate field over every row of `members`.
    fn unconditioned(&mut self, aggregate: &ResolvedAggregate) -> String {
        let source = self.view.name.to_string();
        let Some(input) = &aggregate.input else {
            self.uses.helpers.insert("count");
            return "count(members.len())".to_owned();
        };
        let resolved = view_query::resolve(self.ir, self.entity, &input.name)
            .expect("the plan admitted only inputs that resolve");
        let values = view_query::values(self.ir, self.entity, &input.name)
            .expect("the plan admitted only inputs it compares");
        let read = self.guards("held").text(&resolved);
        let over = format!("members.iter().filter_map(|held| {read})");
        match aggregate.function {
            AggregateFunction::Count => {
                self.uses.helpers.insert("count");
                "count(members.len())".to_owned()
            }
            AggregateFunction::CountDistinct => {
                self.uses.helpers.insert("distinct");
                self.uses.helpers.insert("count");
                let keyed = match values {
                    Values::Numeric => {
                        self.uses.helpers.insert("number_key");
                        format!("{read}.map(|value| number_key(&value))")
                    }
                    Values::Instant => {
                        self.uses.helpers.insert("instant_key");
                        self.uses.helpers.insert("instant");
                        format!("{read}.map(|value| instant_key(&value))")
                    }
                    Values::Text | Values::Other => read,
                };
                format!("distinct(members.iter().filter_map(|held| {keyed}))")
            }
            AggregateFunction::Sum => self.sum(aggregate, input, &over),
            AggregateFunction::Avg => {
                self.uses.helpers.insert("sum_values");
                self.uses.helpers.insert("average");
                self.uses.helpers.insert("spell");
                self.uses.helpers.insert("unrepresentable");
                format!(
                    "{{\n{indent}    let (present, units, scale) = sum_values({over})\n{indent}        \
                     .ok_or_else(|| unrepresentable(\"{source}\"))?;\n{indent}    average(present, \
                     units, scale)\n{indent}        .ok_or_else(|| \
                     unrepresentable(\"{source}\"))?\n{indent}        \
                     .map(crate::primitives::Decimal)\n{indent}}}",
                    indent = self.indent()
                )
            }
            AggregateFunction::Min | AggregateFunction::Max => {
                self.uses.helpers.insert("extreme");
                let order = match values {
                    Values::Numeric => "number_order",
                    Values::Instant => "instant_order",
                    Values::Text | Values::Other => "text_order",
                };
                self.uses.helpers.insert(order);
                if values == Values::Instant {
                    self.uses.helpers.insert("instant");
                }
                let wins = if aggregate.function == AggregateFunction::Min {
                    "Less"
                } else {
                    "Greater"
                };
                let held = format!(
                    "held.data.{}.clone()",
                    super::name::value_ident(&input.name)
                );
                let value = if input.type_ref.is_optional() {
                    format!("{held}?")
                } else {
                    held
                };
                format!(
                    "extreme(\n{indent}    members.iter().filter_map(|held| Some(({read}?, \
                     {value}))),\n{indent}    core::cmp::Ordering::{wins},\n{indent}    \
                     {order},\n{indent})",
                    indent = self.indent()
                )
            }
        }
    }

    /// A `sum` over `over`, the present values: exact, an `Integer` or a `Decimal` as the input
    /// is; `0` over no value, or absent where the aggregate skips absent values.
    fn sum(&mut self, aggregate: &ResolvedAggregate, input: &ResolvedField, over: &str) -> String {
        let source = self.view.name.to_string();
        self.uses.helpers.insert("sum_values");
        self.uses.helpers.insert("unrepresentable");
        let decimal = matches!(
            view_query::leaf(self.ir, &input.type_ref),
            Some(ess_domain::types::Primitive::Decimal)
        );
        let (present, scale) = (
            if aggregate.skip_absent {
                "present"
            } else {
                "_"
            },
            if decimal { "scale" } else { "_" },
        );
        let value = if decimal {
            self.uses.helpers.insert("spell");
            "crate::primitives::Decimal(spell(units, scale))".to_owned()
        } else {
            format!("i64::try_from(units).map_err(|_| unrepresentable(\"{source}\"))?")
        };
        let value = if aggregate.skip_absent {
            format!("if present == 0 {{ None }} else {{ Some({value}) }}")
        } else {
            value
        };
        format!(
            "{{\n{indent}    let ({present}, units, {scale}) = sum_values({over})\n{indent}        \
             .ok_or_else(|| unrepresentable(\"{source}\"))?;\n{indent}    {value}\n{indent}}}",
            indent = self.indent()
        )
    }

    /// The indentation of a row field's value.
    fn indent(&self) -> &'static str {
        if view_ungrouped(self.view) {
            "            "
        } else {
            "                "
        }
    }
}

/// `true` for an aggregate view with no group key: one partition.
fn view_ungrouped(view: &ResolvedView) -> bool {
    view.aggregation
        .as_ref()
        .is_some_and(|aggregation| aggregation.group_by.is_empty())
}
