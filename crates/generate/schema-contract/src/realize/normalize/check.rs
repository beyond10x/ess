//! Structural checking reuses the qualified type plan; schema refinements run at stage boundaries.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::{json, Value};

use super::{Binary64Step, Condition, Expr, Finding, NumberPath, Scope, Stage, Types};
use crate::realize::{finding, InputIdentity, Node, Shape, UnionMode};

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

pub(super) fn input_numbers(
    paths: &[Vec<NumberPath>],
    input: &Types,
    root: &str,
    at: &str,
    found: &mut Vec<Finding>,
) {
    if paths.is_empty() {
        return;
    }
    let ty = match from_node(input, &input.definitions[root], 0) {
        Ok(ty) => ty,
        Err(error) => {
            found.push(error);
            return;
        }
    };
    let mut seen = BTreeSet::new();
    for (index, path) in paths.iter().enumerate() {
        let at = format!("{at}/{index}");
        if !seen.insert(path) {
            found.push(finding(
                &at,
                "duplicate_numeric_path",
                "numeric input path is declared twice",
            ));
        } else if let Err(error) = numeric_path(&ty, path, &at) {
            found.push(error);
        }
    }
}

fn numeric_path(input: &Type, path: &[NumberPath], at: &str) -> Result<()> {
    limit(path.len(), at)?;
    let mut ty = input.clone();
    for (index, segment) in path.iter().enumerate() {
        let at = format!("{at}/{index}");
        ty = match segment {
            NumberPath::Field { name } => member(&ty.without_null(), name, &at)?,
            NumberPath::Items => array_item(&ty.without_null(), &at)?,
        };
    }
    let mut ty = ty.without_null();
    ty.missing = false;
    if ty.kind == Kind::Never || !assignable(&ty, &Type::new(Kind::Number)) {
        return Err(finding(
            at,
            "numeric_input_type",
            "binary64 decoding requires a declared numeric leaf",
        ));
    }
    Ok(())
}

pub(super) fn input_captures(
    paths: &[Vec<NumberPath>],
    numbers: &[Vec<NumberPath>],
    input: &Types,
    root: &str,
    at: &str,
    found: &mut Vec<Finding>,
) {
    if paths.is_empty() {
        return;
    }
    let ty = match from_node(input, &input.definitions[root], 0) {
        Ok(ty) => ty,
        Err(error) => {
            found.push(error);
            return;
        }
    };
    for (index, path) in paths.iter().enumerate() {
        let location = format!("{at}/{index}");
        if let Some(earlier) = paths[..index].iter().position(|other| other == path) {
            found.push(finding(
                &location,
                "duplicate_capture_path",
                &format!("capture path duplicates {at}/{earlier}"),
            ));
        } else if let Some(earlier) = paths[..index]
            .iter()
            .position(|other| overlaps(path, other))
        {
            found.push(finding(
                &location,
                "overlapping_capture_path",
                &format!("capture path overlaps {at}/{earlier}"),
            ));
        } else if let Some(numeric) = numbers.iter().position(|other| overlaps(path, other)) {
            let numeric_at = at.replacen("/raw_json_inputs/", "/binary64_inputs/", 1);
            found.push(finding(
                &location,
                "input_policy_overlap",
                &format!("capture path overlaps {numeric_at}/{numeric}"),
            ));
        } else if let Err(error) = capture_path(&ty, path, &location) {
            found.push(error);
        }
    }
}

fn overlaps(left: &[NumberPath], right: &[NumberPath]) -> bool {
    left.starts_with(right) || right.starts_with(left)
}

fn capture_path(input: &Type, path: &[NumberPath], at: &str) -> Result<()> {
    limit(path.len(), at)?;
    let mut ty = input.clone();
    for (index, segment) in path.iter().enumerate() {
        let at = format!("{at}/{index}");
        ty = match segment {
            NumberPath::Field { name } => member(&ty.without_null(), name, &at)?,
            NumberPath::Items => array_item(&ty.without_null(), &at)?,
        };
    }
    let mut ty = ty.without_null();
    ty.missing = false;
    if ty.kind == Kind::Never || !assignable(&ty, &Type::new(Kind::String)) {
        return Err(finding(
            at,
            "capture_input_type",
            "raw JSON capture requires a declared string leaf",
        ));
    }
    Ok(())
}

pub(super) fn stage(
    stage: &Stage,
    input: &Types,
    output: &Types,
    at: &str,
    extended: bool,
    found: &mut Vec<Finding>,
) {
    let converted = from_node(input, &input.definitions[stage.input.name()], 0).and_then(|input| {
        from_node(output, &output.definitions[stage.output.name()], 0).map(|output| (input, output))
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
        extended,
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
        Shape::Intersection(terms) if matches!(plan.input, InputIdentity::Model { .. }) && model_enum(terms).is_some() => return Ok(model_enum(terms).expect("checked model enum")),
        Shape::Array { .. } | Shape::Intersection(_) => return Err(finding(&node.pointer, "unsupported_shape", "normalization requires an explicit tuple/intersection mapping; it is not flattened implicitly")),
    };
    Ok(Type::new(kind))
}

fn model_enum(terms: &[Node]) -> Option<Type> {
    let [Node {
        shape: Shape::Union {
            mode: UnionMode::Enum,
            variants,
        },
        ..
    }, Node {
        shape: Shape::String,
        ..
    }] = terms
    else {
        return None;
    };
    let literals = variants
        .iter()
        .map(|node| match &node.shape {
            Shape::Literal(value @ Value::String(_)) => {
                Some(Type::new(Kind::Literal(value.clone())))
            }
            _ => None,
        })
        .collect::<Option<Vec<_>>>()?;
    Some(Type::union(literals))
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
    extended: bool,
}

impl Context<'_> {
    fn value(&self, value: &Expr, at: &str, depth: usize) -> Result<Type> {
        limit(depth, at)?;
        self.version(value, at)?;
        self.expression(value, at, depth)
    }

    fn version(&self, value: &Expr, at: &str) -> Result<()> {
        if !self.extended
            && matches!(
                value,
                Expr::Concat { .. }
                    | Expr::Join { .. }
                    | Expr::IntegerString { .. }
                    | Expr::ConcatLists { .. }
                    | Expr::ItemIndex
                    | Expr::SelectMap { .. }
                    | Expr::Find { .. }
                    | Expr::Binary64ToInteger { .. }
            )
        {
            return Err(finding(
                at,
                "operation_version",
                "operation requires ess-normalization/2",
            ));
        }
        Ok(())
    }

    fn expression(&self, value: &Expr, at: &str, depth: usize) -> Result<Type> {
        match value {
            Expr::Null => Ok(Type::new(Kind::Null)),
            Expr::Boolean { value } => Ok(Type::new(Kind::Literal(json!(value)))),
            Expr::String { value } => Ok(Type::new(Kind::Literal(json!(value)))),
            Expr::Integer { value } => Ok(Type::new(Kind::Literal(json!(value)))),
            Expr::Binary64ToInteger { value, steps, .. } => self.binary64(value, steps, at, depth),
            Expr::Read { scope, path } => self.read(*scope, path, at),
            Expr::Field { object, name } => member(
                &self.value(object, &format!("{at}/object"), depth + 1)?,
                name,
                at,
            ),
            Expr::Record { fields } => self.record(fields, at, depth),
            Expr::List { items } => self.list(items, at, depth),
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
            Expr::Map { list, value } => self.map(list, value, at, depth),
            Expr::DistinctCount {
                list,
                condition,
                key,
            } => self.count(list, condition, key, at, depth),
            Expr::Concat { parts } => {
                for (index, part) in parts.iter().enumerate() {
                    self.string(part, &format!("{at}/parts/{index}"), depth + 1)?;
                }
                Ok(Type::new(Kind::String))
            }
            Expr::Join { list, separator } => self.join(list, separator, at, depth),
            Expr::IntegerString { value } => {
                self.integer(value, &format!("{at}/value"), depth + 1)?;
                Ok(Type::new(Kind::String))
            }
            Expr::ConcatLists { lists } => {
                let mut items = Vec::new();
                for (index, list) in lists.iter().enumerate() {
                    items.push(self.item_type(list, &format!("{at}/lists/{index}"), depth + 1)?);
                }
                Ok(Type::new(Kind::Array(Box::new(Type::union(items)))))
            }
            Expr::ItemIndex => {
                if self.item.is_none() {
                    return Err(finding(
                        at,
                        "item_scope",
                        "no collection index is bound here",
                    ));
                }
                Ok(Type::new(Kind::Integer))
            }
            Expr::SelectMap {
                list,
                condition,
                value,
            } => {
                let ty = self.selected(list, condition, value, at, depth)?;
                Ok(Type::new(Kind::Array(Box::new(ty))))
            }
            Expr::Find {
                list,
                condition,
                value,
                otherwise,
            } => {
                let selected = self.selected(list, condition, value, at, depth)?;
                let at_otherwise = format!("{at}/otherwise");
                let otherwise = self.value(otherwise, &at_otherwise, depth + 1)?;
                required(&otherwise, &at_otherwise)?;
                Ok(Type::union(vec![selected, otherwise]))
            }
        }
    }

    fn map(&self, list: &Expr, value: &Expr, at: &str, depth: usize) -> Result<Type> {
        let item = self.item_type(list, &format!("{at}/list"), depth + 1)?;
        let ty = Context {
            input: self.input,
            item: Some(&item),
            extended: self.extended,
        }
        .value(value, &format!("{at}/value"), depth + 1)?;
        required(&ty, at)?;
        Ok(Type::new(Kind::Array(Box::new(ty))))
    }

    fn binary64(
        &self,
        value: &Expr,
        steps: &[Binary64Step],
        at: &str,
        depth: usize,
    ) -> Result<Type> {
        let at_value = format!("{at}/value");
        if !assignable(
            &self.value(value, &at_value, depth + 1)?,
            &Type::new(Kind::Number),
        ) {
            return Err(finding(
                &at_value,
                "binary64_type",
                "binary64 conversion requires a present non-null numeric value",
            ));
        }
        for (index, step) in steps.iter().enumerate() {
            let (Binary64Step::Multiply { value }
            | Binary64Step::Minimum { value }
            | Binary64Step::Maximum { value }) = step;
            if super::numeric::literal(value).is_none() {
                return Err(finding(
                    &format!("{at}/steps/{index}/value"),
                    "binary64_literal",
                    "expected a finite JSON numeric token",
                ));
            }
        }
        Ok(Type::new(Kind::Integer))
    }

    fn read(&self, scope: Scope, path: &[String], at: &str) -> Result<Type> {
        let mut ty = match scope {
            Scope::Input => self.input,
            Scope::Item => self
                .item
                .ok_or_else(|| finding(at, "item_scope", "no collection item is bound here"))?,
        }
        .clone();
        for key in path {
            ty = member(&ty, key, at)?;
        }
        Ok(ty)
    }

    fn list(&self, items: &[Expr], at: &str, depth: usize) -> Result<Type> {
        let mut types = Vec::new();
        for (index, item) in items.iter().enumerate() {
            let at = format!("{at}/items/{index}");
            let ty = self.value(item, &at, depth + 1)?;
            required(&ty, &at)?;
            types.push(ty);
        }
        Ok(Type::new(Kind::Array(Box::new(Type::union(types)))))
    }

    fn count(
        &self,
        list: &Expr,
        condition: &Condition,
        key: &Expr,
        at: &str,
        depth: usize,
    ) -> Result<Type> {
        let item = self.item_type(list, &format!("{at}/list"), depth + 1)?;
        let scope = Context {
            input: self.input,
            item: Some(&item),
            extended: self.extended,
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

    fn join(&self, list: &Expr, separator: &Expr, at: &str, depth: usize) -> Result<Type> {
        let at_list = format!("{at}/list");
        let item = self.item_type(list, &at_list, depth + 1)?;
        if !assignable(&item, &Type::new(Kind::String)) {
            return Err(finding(
                &at_list,
                "string_type",
                "joining requires present string elements",
            ));
        }
        self.string(separator, &format!("{at}/separator"), depth + 1)?;
        Ok(Type::new(Kind::String))
    }

    fn selected(
        &self,
        list: &Expr,
        condition: &Condition,
        value: &Expr,
        at: &str,
        depth: usize,
    ) -> Result<Type> {
        let item = self.item_type(list, &format!("{at}/list"), depth + 1)?;
        let scope = Context {
            input: self.input,
            item: Some(&item),
            extended: self.extended,
        };
        scope.condition(condition, &format!("{at}/condition"), depth + 1)?;
        let at_value = format!("{at}/value");
        let result = scope.value(value, &at_value, depth + 1)?;
        required(&result, &at_value)?;
        Ok(result)
    }

    fn string(&self, value: &Expr, at: &str, depth: usize) -> Result<()> {
        if assignable(&self.value(value, at, depth)?, &Type::new(Kind::String)) {
            Ok(())
        } else {
            Err(finding(
                at,
                "string_type",
                "operation requires a present non-null string",
            ))
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
