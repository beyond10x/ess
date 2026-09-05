//! Pure reference execution. No stage result or caller mutation escapes on failure.

use std::collections::BTreeSet;

use serde_json::{Map, Value};

use super::finding;
use super::{Condition, Expr, IntegerOp, Overflow, Refused, Scope};

type Result<T> = std::result::Result<T, Refused>;

pub(super) struct Context<'a> {
    pub input: &'a Value,
    pub item: Option<&'a Value>,
}

impl Context<'_> {
    pub fn value(&self, expr: &Expr, at: &str) -> Result<Option<Value>> {
        let value = match expr {
            Expr::Null => Value::Null,
            Expr::Boolean { value } => Value::Bool(*value),
            Expr::String { value } => Value::String(value.clone()),
            Expr::Integer { value } => Value::Number((*value).into()),
            Expr::Read { scope, path } => {
                let mut value = match scope {
                    Scope::Input => Some(self.input),
                    Scope::Item => self.item,
                };
                for key in path {
                    let Some(parent) = value else {
                        return Ok(None);
                    };
                    let object = parent.as_object().ok_or_else(|| {
                        error(at, "object_value", "member access encountered a nonobject")
                    })?;
                    value = object.get(key);
                }
                return Ok(value.cloned());
            }
            Expr::Field { object, name } => {
                let Some(value) = self.value(object, &format!("{at}/object"))? else {
                    return Ok(None);
                };
                return Ok(value
                    .as_object()
                    .ok_or_else(|| {
                        error(at, "object_value", "member access encountered a nonobject")
                    })?
                    .get(name)
                    .cloned());
            }
            Expr::Record { fields } => {
                let mut out = Map::new();
                for (key, expr) in fields {
                    if let Some(value) =
                        self.value(expr, &super::path(&format!("{at}/fields"), key))?
                    {
                        out.insert(key.clone(), value);
                    }
                }
                Value::Object(out)
            }
            Expr::List { items } => {
                let out = items
                    .iter()
                    .enumerate()
                    .map(|(index, expr)| self.present(expr, &format!("{at}/items/{index}")))
                    .collect::<Result<_>>()?;
                Value::Array(out)
            }
            Expr::Fallback {
                value,
                fallback,
                on_null,
            } => {
                return match self.value(value, &format!("{at}/value"))? {
                    None => self.value(fallback, &format!("{at}/fallback")),
                    Some(value) if *on_null && value.is_null() => {
                        self.value(fallback, &format!("{at}/fallback"))
                    }
                    value => Ok(value),
                };
            }
            Expr::Choose {
                condition,
                then_value,
                else_value,
            } => {
                return if self.condition(condition, &format!("{at}/condition"))? {
                    self.value(then_value, &format!("{at}/then_value"))
                } else {
                    self.value(else_value, &format!("{at}/else_value"))
                };
            }
            Expr::Arithmetic {
                operation,
                left,
                right,
                overflow,
            } => self.arithmetic(*operation, left, right, *overflow, at)?,
            Expr::Map { list, value } => self.map(list, value, at)?,
            Expr::DistinctCount {
                list,
                condition,
                key,
            } => self.count(list, condition, key, at)?,
        };
        Ok(Some(value))
    }

    fn present(&self, expr: &Expr, at: &str) -> Result<Value> {
        self.value(expr, at)?
            .ok_or_else(|| error(at, "missing_value", "required expression value is absent"))
    }

    fn arithmetic(
        &self,
        operation: IntegerOp,
        left: &Expr,
        right: &Expr,
        overflow: Overflow,
        at: &str,
    ) -> Result<Value> {
        let a = integer(&self.present(left, &format!("{at}/left"))?, at)?;
        let b = integer(&self.present(right, &format!("{at}/right"))?, at)?;
        let value = match (operation, overflow) {
            (IntegerOp::Add, Overflow::Reject) => a.checked_add(b),
            (IntegerOp::Multiply, Overflow::Reject) => a.checked_mul(b),
            (IntegerOp::Add, Overflow::Wrap) => Some(a.wrapping_add(b)),
            (IntegerOp::Multiply, Overflow::Wrap) => Some(a.wrapping_mul(b)),
        }
        .ok_or_else(|| {
            error(
                at,
                "integer_overflow",
                "arithmetic result exceeds signed 64-bit representation",
            )
        })?;
        Ok(Value::Number(value.into()))
    }

    fn map(&self, list: &Expr, value: &Expr, at: &str) -> Result<Value> {
        let values = self.present(list, &format!("{at}/list"))?;
        let items = values
            .as_array()
            .ok_or_else(|| error(at, "collection_value", "mapping requires a list"))?;
        let out = items
            .iter()
            .map(|item| {
                Context {
                    input: self.input,
                    item: Some(item),
                }
                .present(value, &format!("{at}/value"))
            })
            .collect::<Result<_>>()?;
        Ok(Value::Array(out))
    }

    fn count(&self, list: &Expr, condition: &Condition, key: &Expr, at: &str) -> Result<Value> {
        let values = self.present(list, &format!("{at}/list"))?;
        let items = values
            .as_array()
            .ok_or_else(|| error(at, "collection_value", "counting requires a list"))?;
        let mut keys = BTreeSet::new();
        for item in items {
            let scope = Context {
                input: self.input,
                item: Some(item),
            };
            if scope.condition(condition, &format!("{at}/condition"))? {
                let key = scope.present(key, &format!("{at}/key"))?;
                keys.insert(
                    key.as_str()
                        .ok_or_else(|| {
                            error(at, "count_key", "distinct category key is not a string")
                        })?
                        .to_owned(),
                );
            }
        }
        let count = i64::try_from(keys.len()).map_err(|_| {
            error(
                at,
                "integer_overflow",
                "category count exceeds signed 64-bit representation",
            )
        })?;
        Ok(Value::Number(count.into()))
    }

    pub fn condition(&self, condition: &Condition, at: &str) -> Result<bool> {
        match condition {
            Condition::Present { value } => {
                Ok(self.value(value, &format!("{at}/value"))?.is_some())
            }
            Condition::Boolean { value } => self
                .present(value, &format!("{at}/value"))?
                .as_bool()
                .ok_or_else(|| error(at, "boolean_value", "condition is not a boolean")),
            Condition::Equal { left, right } => {
                let a = self.value(left, &format!("{at}/left"))?;
                let b = self.value(right, &format!("{at}/right"))?;
                match (a, b) {
                    (Some(a), Some(b)) => {
                        for value in [&a, &b] {
                            if value.is_number() {
                                integer(value, at)?;
                            }
                        }
                        Ok(a == b)
                    }
                    _ => Ok(false),
                }
            }
            Condition::Greater { left, right } => {
                Ok(integer(&self.present(left, &format!("{at}/left"))?, at)?
                    > integer(&self.present(right, &format!("{at}/right"))?, at)?)
            }
            Condition::StartsWith { value, prefix } => {
                let value = self.present(value, &format!("{at}/value"))?;
                let prefix = self.present(prefix, &format!("{at}/prefix"))?;
                let (Some(value), Some(prefix)) = (value.as_str(), prefix.as_str()) else {
                    return Err(error(
                        at,
                        "string_value",
                        "prefix comparison requires strings",
                    ));
                };
                Ok(value.starts_with(prefix))
            }
            Condition::All { conditions } => {
                for (index, condition) in conditions.iter().enumerate() {
                    if !self.condition(condition, &format!("{at}/conditions/{index}"))? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            Condition::Any { conditions } => {
                for (index, condition) in conditions.iter().enumerate() {
                    if self.condition(condition, &format!("{at}/conditions/{index}"))? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            Condition::Not { condition } => {
                Ok(!self.condition(condition, &format!("{at}/condition"))?)
            }
        }
    }
}

fn integer(value: &Value, at: &str) -> Result<i64> {
    value.as_i64().ok_or_else(|| {
        error(
            at,
            "integer_representation",
            "operation requires an exact integral token representable as signed 64-bit",
        )
    })
}

fn error(at: &str, rule: &str, detail: &str) -> Refused {
    Refused(vec![finding(at, rule, detail)])
}
