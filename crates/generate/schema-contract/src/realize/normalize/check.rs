//! Structural checking reuses the qualified type plan; schema refinements run at stage boundaries.

use std::collections::BTreeMap;

use serde_json::{json, Value};

use super::{Condition, Expr, Finding, Scope, Stage, Types};
use crate::realize::{finding, Node, Shape};

type Result<T> = std::result::Result<T, Finding>;

#[derive(Clone, PartialEq)]
struct Type {
    missing: bool,
    kind: Kind,
}

#[derive(Clone, PartialEq)]
enum Kind {
    Any,
    Never,
    Null,
    Boolean,
    String,
    Number,
    Integer,
    Literal(Value),
    Object(BTreeMap<String, Type>, Box<Type>),
    Array(Box<Type>),
    Union(Vec<Type>),
}

impl Type {
    fn new(kind: Kind) -> Self {
        Self {
            missing: false,
            kind,
        }
    }
    fn union(values: Vec<Self>) -> Self {
        let mut missing = false;
        let mut variants = Vec::new();
        for mut value in values {
            missing |= value.missing;
            value.missing = false;
            let members = match value.kind {
                Kind::Never => Vec::new(),
                Kind::Union(members) => members,
                _ => vec![value],
            };
            for mut member in members {
                missing |= member.missing;
                member.missing = false;
                if !variants.contains(&member) {
                    variants.push(member);
                }
            }
        }
        let kind = match variants.len() {
            0 => Kind::Never,
            1 => variants.pop().expect("one variant").kind,
            _ => Kind::Union(variants),
        };
        Self { missing, kind }
    }
    fn without_null(&self) -> Self {
        match &self.kind {
            Kind::Null => Self::new(Kind::Never),
            Kind::Literal(value) if value.is_null() => Self::new(Kind::Never),
            Kind::Union(values) => Self::union(values.iter().map(Self::without_null).collect()),
            _ => self.clone(),
        }
    }
}

pub(super) fn stage(
    stage: &Stage,
    input: &Types,
    output: &Types,
    at: &str,
    found: &mut Vec<Finding>,
) {
    let converted = from_node(input, &input.definitions[&stage.input.root], 0).and_then(|input| {
        from_node(output, &output.definitions[&stage.output.root], 0).map(|output| (input, output))
    });
    let (input, output) = match converted {
        Ok(pair) => pair,
        Err(error) => {
            found.push(finding(
                at,
                "normalization_shape",
                &format!("{}: {}", error.pointer, error.detail),
            ));
            return;
        }
    };
    let scope = Context {
        input: &input,
        item: None,
    };
    for (index, condition) in stage.requires.iter().enumerate() {
        if let Err(error) = scope.condition(condition, &format!("{at}/requires/{index}"), 0) {
            found.push(error);
        }
    }
    let at = format!("{at}/value");
    match scope.value(&stage.value, &at, 0) {
        Ok(value) if assignable(&value, &output) => {}
        Ok(_) => found.push(finding(
            &at,
            "output_type",
            "expression is not structurally assignable to the selected output root",
        )),
        Err(error) => found.push(error),
    }
}

fn from_node(plan: &Types, node: &Node, depth: usize) -> Result<Type> {
    limit(depth, &node.pointer)?;
    let nested = |node| from_node(plan, node, depth + 1);
    let kind = match &node.shape {
        Shape::Ref(name) => return nested(&plan.definitions[name]),
        Shape::Json => Kind::Any,
        Shape::Never => Kind::Never,
        Shape::Null => Kind::Null,
        Shape::Boolean => Kind::Boolean,
        Shape::String => Kind::String,
        Shape::Number => Kind::Number,
        Shape::Integer => Kind::Integer,
        Shape::Literal(value) => Kind::Literal(value.clone()),
        Shape::Object { fields, additional } => {
            let fields = fields.iter().map(|(name, field)| {
                let mut ty = nested(&field.value)?;
                ty.missing = !field.required;
                Ok((name.clone(), ty))
            }).collect::<Result<_>>()?;
            Kind::Object(fields, Box::new(nested(additional)?))
        }
        Shape::Array { prefix, items, .. } if prefix.is_empty() => Kind::Array(Box::new(nested(items)?)),
        Shape::Union { variants, .. } => return Ok(Type::union(variants.iter().map(nested).collect::<Result<_>>()?)),
        Shape::Array { .. } | Shape::Intersection(_) => return Err(finding(&node.pointer, "unsupported_shape", "normalization requires an explicit tuple/intersection mapping; it is not flattened implicitly")),
    };
    Ok(Type::new(kind))
}

fn limit(depth: usize, at: &str) -> Result<()> {
    if depth > 64 {
        Err(finding(
            at,
            "normalization_depth",
            "normalization expression or expanded shape exceeds 64 levels",
        ))
    } else {
        Ok(())
    }
}

struct Context<'a> {
    input: &'a Type,
    item: Option<&'a Type>,
}

impl Context<'_> {
    fn value(&self, value: &Expr, at: &str, depth: usize) -> Result<Type> {
        limit(depth, at)?;
        match value {
            Expr::Null => Ok(Type::new(Kind::Null)),
            Expr::Boolean { value } => Ok(Type::new(Kind::Literal(json!(value)))),
            Expr::String { value } => Ok(Type::new(Kind::Literal(json!(value)))),
            Expr::Integer { value } => Ok(Type::new(Kind::Literal(json!(value)))),
            Expr::Read { scope, path } => {
                let mut ty = match scope {
                    Scope::Input => self.input,
                    Scope::Item => self.item.ok_or_else(|| {
                        finding(at, "item_scope", "no collection item is bound here")
                    })?,
                }
                .clone();
                for key in path {
                    ty = member(&ty, key, at)?;
                }
                Ok(ty)
            }
            Expr::Field { object, name } => member(
                &self.value(object, &format!("{at}/object"), depth + 1)?,
                name,
                at,
            ),
            Expr::Record { fields } => self.record(fields, at, depth),
            Expr::List { items } => {
                let mut types = Vec::new();
                for (index, item) in items.iter().enumerate() {
                    let at = format!("{at}/items/{index}");
                    let ty = self.value(item, &at, depth + 1)?;
                    required(&ty, &at)?;
                    types.push(ty);
                }
                Ok(Type::new(Kind::Array(Box::new(Type::union(types)))))
            }
            Expr::Fallback {
                value,
                fallback,
                on_null,
            } => {
                let mut ty = self.value(value, &format!("{at}/value"), depth + 1)?;
                let fallback = self.value(fallback, &format!("{at}/fallback"), depth + 1)?;
                if *on_null {
                    ty = ty.without_null();
                }
                ty.missing = false;
                Ok(Type::union(vec![ty, fallback]))
            }
            Expr::Choose {
                condition,
                then_value,
                else_value,
            } => {
                self.condition(condition, &format!("{at}/condition"), depth + 1)?;
                Ok(Type::union(vec![
                    self.value(then_value, &format!("{at}/then_value"), depth + 1)?,
                    self.value(else_value, &format!("{at}/else_value"), depth + 1)?,
                ]))
            }
            Expr::Arithmetic { left, right, .. } => {
                self.integer(left, &format!("{at}/left"), depth + 1)?;
                self.integer(right, &format!("{at}/right"), depth + 1)?;
                Ok(Type::new(Kind::Integer))
            }
            Expr::Map { list, value } => {
                let item = self.item_type(list, &format!("{at}/list"), depth + 1)?;
                let ty = Context {
                    input: self.input,
                    item: Some(&item),
                }
                .value(value, &format!("{at}/value"), depth + 1)?;
                required(&ty, at)?;
                Ok(Type::new(Kind::Array(Box::new(ty))))
            }
            Expr::DistinctCount {
                list,
                condition,
                key,
            } => {
                let item = self.item_type(list, &format!("{at}/list"), depth + 1)?;
                let scope = Context {
                    input: self.input,
                    item: Some(&item),
                };
                scope.condition(condition, &format!("{at}/condition"), depth + 1)?;
                let key = scope.value(key, &format!("{at}/key"), depth + 1)?;
                if !assignable(&key, &Type::new(Kind::String)) {
                    return Err(finding(
                        at,
                        "count_key",
                        "distinct category keys must be present strings",
                    ));
                }
                Ok(Type::new(Kind::Integer))
            }
        }
    }

    fn record(&self, fields: &BTreeMap<String, Expr>, at: &str, depth: usize) -> Result<Type> {
        let fields = fields
            .iter()
            .map(|(key, value)| {
                self.value(
                    value,
                    &crate::realize::path(&format!("{at}/fields"), key),
                    depth + 1,
                )
                .map(|ty| (key.clone(), ty))
            })
            .collect::<Result<_>>()?;
        Ok(Type::new(Kind::Object(
            fields,
            Box::new(Type::new(Kind::Never)),
        )))
    }

    fn item_type(&self, value: &Expr, at: &str, depth: usize) -> Result<Type> {
        let ty = self.value(value, at, depth)?;
        required(&ty, at)?;
        array_item(&ty, at)
    }

    fn integer(&self, value: &Expr, at: &str, depth: usize) -> Result<()> {
        let ty = self.value(value, at, depth)?;
        if assignable(&ty, &Type::new(Kind::Integer)) {
            Ok(())
        } else {
            Err(finding(
                at,
                "integer_type",
                "operation requires a present integer",
            ))
        }
    }

    fn condition(&self, condition: &Condition, at: &str, depth: usize) -> Result<()> {
        limit(depth, at)?;
        match condition {
            Condition::Present { value } => {
                self.value(value, &format!("{at}/value"), depth + 1)?;
            }
            Condition::Boolean { value } => {
                if !assignable(
                    &self.value(value, &format!("{at}/value"), depth + 1)?,
                    &Type::new(Kind::Boolean),
                ) {
                    return Err(finding(
                        at,
                        "boolean_type",
                        "condition requires a present boolean",
                    ));
                }
            }
            Condition::Equal { left, right } => {
                for (name, value) in [("left", left), ("right", right)] {
                    let at = format!("{at}/{name}");
                    if !comparable(&self.value(value, &at, depth + 1)?) {
                        return Err(finding(
                            &at,
                            "equality_type",
                            "equality requires null, boolean, string or signed-integer scalars",
                        ));
                    }
                }
            }
            Condition::Greater { left, right } => {
                self.integer(left, &format!("{at}/left"), depth + 1)?;
                self.integer(right, &format!("{at}/right"), depth + 1)?;
            }
            Condition::StartsWith { value, prefix } => {
                for (name, value) in [("value", value), ("prefix", prefix)] {
                    let at = format!("{at}/{name}");
                    if !assignable(
                        &self.value(value, &at, depth + 1)?,
                        &Type::new(Kind::String),
                    ) {
                        return Err(finding(
                            &at,
                            "string_type",
                            "prefix comparison requires present non-null strings",
                        ));
                    }
                }
            }
            Condition::All { conditions } | Condition::Any { conditions } => {
                for (index, condition) in conditions.iter().enumerate() {
                    self.condition(condition, &format!("{at}/conditions/{index}"), depth + 1)?;
                }
            }
            Condition::Not { condition } => {
                self.condition(condition, &format!("{at}/condition"), depth + 1)?;
            }
        }
        Ok(())
    }
}

fn comparable(ty: &Type) -> bool {
    match &ty.kind {
        Kind::Never | Kind::Null | Kind::Boolean | Kind::String | Kind::Integer => true,
        Kind::Literal(value) => {
            value.is_null() || value.is_boolean() || value.is_string() || value.as_i64().is_some()
        }
        Kind::Union(values) => values.iter().all(comparable),
        _ => false,
    }
}

fn required(ty: &Type, at: &str) -> Result<()> {
    if ty.missing {
        Err(finding(
            at,
            "missing_value",
            "value can be absent; declare an explicit fallback",
        ))
    } else {
        Ok(())
    }
}

fn member(object: &Type, key: &str, at: &str) -> Result<Type> {
    if let Kind::Union(values) = &object.kind {
        let mut ty = Type::union(
            values
                .iter()
                .map(|value| member(value, key, at))
                .collect::<Result<_>>()?,
        );
        ty.missing |= object.missing;
        return Ok(ty);
    }
    let Kind::Object(fields, _) = &object.kind else {
        return Err(finding(
            at,
            "object_type",
            "member access requires an object; normalize optional/null alternatives explicitly",
        ));
    };
    let mut ty = fields.get(key).cloned().ok_or_else(|| {
        finding(
            at,
            "unknown_field",
            &format!("{key:?} is not a declared object member"),
        )
    })?;
    ty.missing |= object.missing;
    Ok(ty)
}

fn array_item(ty: &Type, at: &str) -> Result<Type> {
    match &ty.kind {
        Kind::Array(item) => Ok(*item.clone()),
        Kind::Union(values) => Ok(Type::union(
            values
                .iter()
                .map(|value| array_item(value, at))
                .collect::<Result<_>>()?,
        )),
        _ => Err(finding(
            at,
            "collection_type",
            "operation requires a non-null list",
        )),
    }
}

fn assignable(source: &Type, target: &Type) -> bool {
    if source.missing && !target.missing {
        return false;
    }
    match (&source.kind, &target.kind) {
        (Kind::Never, _)
        | (_, Kind::Any)
        | (Kind::Null, Kind::Null)
        | (Kind::Boolean, Kind::Boolean)
        | (Kind::String, Kind::String)
        | (Kind::Integer, Kind::Integer | Kind::Number)
        | (Kind::Number, Kind::Number) => true,
        (Kind::Union(values), _) => values.iter().all(|value| {
            assignable(
                &Type {
                    missing: source.missing,
                    ..value.clone()
                },
                target,
            )
        }),
        (_, Kind::Union(values)) => values.iter().any(|value| {
            assignable(
                source,
                &Type {
                    missing: target.missing,
                    ..value.clone()
                },
            )
        }),
        (Kind::Literal(a), Kind::Literal(b)) => a == b,
        (Kind::Literal(v), Kind::Null) => v.is_null(),
        (Kind::Literal(v), Kind::Boolean) => v.is_boolean(),
        (Kind::Literal(v), Kind::String) => v.is_string(),
        (Kind::Literal(v), Kind::Integer) => v.as_i64().is_some() || v.as_u64().is_some(),
        (Kind::Literal(v), Kind::Number) => v.is_number(),
        (Kind::Array(a), Kind::Array(b)) => assignable(a, b),
        (Kind::Object(a, extra_a), Kind::Object(b, extra_b)) => {
            b.iter().all(|(key, expected)| {
                a.get(key).map_or_else(
                    || expected.missing && assignable(extra_a, expected),
                    |value| assignable(value, expected),
                )
            }) && a.iter().all(|(key, value)| {
                b.contains_key(key)
                    || assignable(
                        &Type {
                            missing: false,
                            ..value.clone()
                        },
                        extra_b,
                    )
            }) && assignable(extra_a, extra_b)
        }
        _ => false,
    }
}
