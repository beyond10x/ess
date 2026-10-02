//! The checked, fail-closed subset of expressions used by client row filters.
//!
//! A filter is deliberately separate from display expressions: unknown display text is literal,
//! while an unknown predicate must never reveal rows. Both the checker and terminal use this
//! parser. The React runtime implements the same grammar and value comparisons.

use serde_yaml::Value;

/// A parsed filter expression. Bare words are string literals; dotted names are paths.
#[derive(Debug, Clone, PartialEq)]
pub enum Predicate {
    /// A scalar literal.
    Literal(Value),
    /// A path whose first segment is a root.
    Path(Vec<String>),
    /// A bracketed list.
    List(Vec<Self>),
    /// Boolean negation.
    Not(Box<Self>),
    /// A comparison or boolean connective.
    Binary(Operator, Box<Self>, Box<Self>),
}

/// The operators admitted in a filter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    /// Equality, following UI expression comparison semantics.
    Equal,
    /// Inequality.
    NotEqual,
    /// Membership in a literal list.
    In,
    /// Boolean conjunction.
    And,
    /// Boolean disjunction.
    Or,
}

/// Parse the complete expression. The caller checks [`Predicate::boolean`] and its roots.
pub fn parse(text: &str) -> Option<Predicate> {
    let tokens = tokenize(text)?;
    let mut parser = Parser {
        tokens: &tokens,
        at: 0,
    };
    let result = parser.or()?;
    (parser.at == tokens.len()).then_some(result)
}

impl Predicate {
    /// Whether the top-level form always produces a boolean.
    pub fn boolean(&self) -> bool {
        matches!(self, Self::Not(_) | Self::Binary(..))
    }

    /// Every path, including those inside nested expressions and lists.
    pub fn paths(&self) -> Vec<&[String]> {
        let mut out = Vec::new();
        self.visit_paths(&mut out);
        out
    }

    fn visit_paths<'a>(&'a self, out: &mut Vec<&'a [String]>) {
        match self {
            Self::Path(path) => out.push(path),
            Self::List(items) => items.iter().for_each(|item| item.visit_paths(out)),
            Self::Not(inner) => inner.visit_paths(out),
            Self::Binary(_, left, right) => {
                left.visit_paths(out);
                right.visit_paths(out);
            }
            Self::Literal(_) => {}
        }
    }

    /// The roots admitted by both read filters and dynamic navigation filters.
    pub fn allowed_roots(&self) -> bool {
        self.paths()
            .iter()
            .all(|path| ["row", "params", "state", "shell", "args"].contains(&path[0].as_str()))
    }

    /// Evaluate a checked predicate against its owner scope, resolving each complete path.
    /// Missing paths resolve to null, as in other UI expressions.
    pub fn evaluate(&self, resolve: &impl Fn(&[String]) -> Value) -> Value {
        match self {
            Self::Literal(value) => value.clone(),
            Self::Path(path) => resolve(path),
            Self::List(items) => {
                Value::Sequence(items.iter().map(|item| item.evaluate(resolve)).collect())
            }
            Self::Not(inner) => Value::Bool(!truthy(&inner.evaluate(resolve))),
            Self::Binary(op, left, right) => {
                let left = left.evaluate(resolve);
                let value = match op {
                    Operator::And => truthy(&left) && truthy(&right.evaluate(resolve)),
                    Operator::Or => truthy(&left) || truthy(&right.evaluate(resolve)),
                    Operator::Equal => equal(&left, &right.evaluate(resolve)),
                    Operator::NotEqual => !equal(&left, &right.evaluate(resolve)),
                    Operator::In => right
                        .evaluate(resolve)
                        .as_sequence()
                        .is_some_and(|items| items.iter().any(|item| equal(&left, item))),
                };
                Value::Bool(value)
            }
        }
    }
}

/// Keep a row only for a valid, admitted predicate that evaluates to true.
pub fn keeps(text: &str, resolve: &impl Fn(&[String]) -> Value) -> bool {
    parse(text).is_some_and(|predicate| {
        predicate.boolean()
            && predicate.allowed_roots()
            && predicate.evaluate(resolve) == Value::Bool(true)
    })
}

fn truthy(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(value) => *value,
        Value::Number(value) => value
            .as_f64()
            .is_some_and(|value| value != 0.0 && !value.is_nan()),
        Value::String(value) => !value.is_empty(),
        Value::Sequence(value) => !value.is_empty(),
        Value::Mapping(_) => true,
        Value::Tagged(value) => truthy(&value.value),
    }
}

fn text(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::Bool(value) => value.to_string(),
        Value::Number(value) => value.as_f64().map_or_else(
            || value.to_string(),
            |value| ryu_js::Buffer::new().format(value).to_owned(),
        ),
        Value::String(value) => value.clone(),
        Value::Sequence(items) => items.iter().map(text).collect::<Vec<_>>().join(","),
        Value::Mapping(_) => "[object Object]".to_owned(),
        Value::Tagged(value) => text(&value.value),
    }
}

fn equal(left: &Value, right: &Value) -> bool {
    if let (Value::Sequence(left), Value::Sequence(right)) = (left, right) {
        let mut left: Vec<_> = left.iter().map(canonical).collect();
        let mut right: Vec<_> = right.iter().map(canonical).collect();
        left.sort();
        right.sort();
        return left == right;
    }
    if left.is_mapping() || right.is_mapping() {
        return canonical(left) == canonical(right);
    }
    text(left) == text(right)
}

fn canonical(value: &Value) -> String {
    match value {
        Value::Number(_) => text(value),
        Value::Sequence(items) => format!(
            "[{}]",
            items.iter().map(canonical).collect::<Vec<_>>().join(",")
        ),
        Value::Mapping(items) => {
            let sorted: std::collections::BTreeMap<_, _> = items
                .iter()
                .map(|(key, value)| (text(key), value))
                .collect();
            format!(
                "{{{}}}",
                sorted
                    .iter()
                    .map(|(key, value)| format!(
                        "{}:{}",
                        serde_json::to_string(key).unwrap_or_default(),
                        canonical(value)
                    ))
                    .collect::<Vec<_>>()
                    .join(",")
            )
        }
        _ => serde_json::to_string(value).unwrap_or_default(),
    }
}

/// Walk a JSON-shaped value as the browser does, including field projection through lists.
pub fn path(value: &Value, segments: &[&str]) -> Value {
    let Some((first, rest)) = segments.split_first() else {
        return value.clone();
    };
    if let Value::Sequence(items) = value {
        let projected = if *first == "length" {
            Value::Number(items.len().into())
        } else {
            Value::Sequence(items.iter().map(|item| path(item, &[*first])).collect())
        };
        return path(&projected, rest);
    }
    path(value.get(*first).unwrap_or(&Value::Null), rest)
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Word(String),
    String(String),
    Open,
    Close,
    List,
    EndList,
    Comma,
    Equal,
    NotEqual,
}

fn tokenize(text: &str) -> Option<Vec<Token>> {
    let mut chars = text.char_indices().peekable();
    let mut tokens = Vec::new();
    while let Some((start, c)) = chars.next() {
        tokens.push(match c {
            c if c.is_whitespace() => continue,
            '(' => Token::Open,
            ')' => Token::Close,
            '[' => Token::List,
            ']' => Token::EndList,
            ',' => Token::Comma,
            '=' | '!' => {
                chars.next_if(|(_, c)| *c == '=')?;
                if c == '=' {
                    Token::Equal
                } else {
                    Token::NotEqual
                }
            }
            '\'' | '"' => {
                let (end, _) = chars.find(|(_, next)| *next == c)?;
                Token::String(text[start + 1..end].to_owned())
            }
            c if c.is_ascii_alphanumeric() || "_-".contains(c) => {
                let mut end = start + c.len_utf8();
                while let Some((at, c)) =
                    chars.next_if(|(_, c)| c.is_ascii_alphanumeric() || "_.-".contains(*c))
                {
                    end = at + c.len_utf8();
                }
                Token::Word(text[start..end].to_owned())
            }
            _ => return None,
        });
    }
    Some(tokens)
}

struct Parser<'a> {
    tokens: &'a [Token],
    at: usize,
}
impl Parser<'_> {
    fn eat(&mut self, token: &Token) -> bool {
        if self.tokens.get(self.at) == Some(token) {
            self.at += 1;
            true
        } else {
            false
        }
    }
    fn word(&mut self, word: &str) -> bool {
        self.eat(&Token::Word(word.to_owned()))
    }
    fn or(&mut self) -> Option<Predicate> {
        let mut left = self.and()?;
        while self.word("or") {
            left = Predicate::Binary(Operator::Or, Box::new(left), Box::new(self.and()?));
        }
        Some(left)
    }
    fn and(&mut self) -> Option<Predicate> {
        let mut left = self.unary()?;
        while self.word("and") {
            left = Predicate::Binary(Operator::And, Box::new(left), Box::new(self.unary()?));
        }
        Some(left)
    }
    fn unary(&mut self) -> Option<Predicate> {
        if self.word("not") {
            return Some(Predicate::Not(Box::new(self.unary()?)));
        }
        let left = self.primary()?;
        let op = if self.eat(&Token::Equal) {
            Operator::Equal
        } else if self.eat(&Token::NotEqual) {
            Operator::NotEqual
        } else if self.word("in") {
            Operator::In
        } else {
            return Some(left);
        };
        let right = self.primary()?;
        if op == Operator::In && !matches!(right, Predicate::List(_)) {
            return None;
        }
        Some(Predicate::Binary(op, Box::new(left), Box::new(right)))
    }
    fn primary(&mut self) -> Option<Predicate> {
        let token = self.tokens.get(self.at)?.clone();
        self.at += 1;
        match token {
            Token::Open => {
                let inner = self.or()?;
                self.eat(&Token::Close).then_some(inner)
            }
            Token::List => {
                let mut items = Vec::new();
                if !self.eat(&Token::EndList) {
                    loop {
                        items.push(self.or()?);
                        if self.eat(&Token::EndList) {
                            break;
                        }
                        if !self.eat(&Token::Comma) {
                            return None;
                        }
                    }
                }
                Some(Predicate::List(items))
            }
            Token::String(value) => Some(Predicate::Literal(Value::String(value))),
            Token::Word(word) => {
                if ["and", "or", "not", "in"].contains(&word.as_str()) {
                    return None;
                }
                let number_like = word.starts_with(|c: char| c.is_ascii_digit() || c == '-');
                if number_like {
                    let digits = word.strip_prefix('-').unwrap_or(&word);
                    let parts: Vec<_> = digits.split('.').collect();
                    if parts.len() > 2
                        || parts.iter().any(|part| {
                            part.is_empty() || !part.chars().all(|c| c.is_ascii_digit())
                        })
                    {
                        return None;
                    }
                }
                let literal = match word.as_str() {
                    "true" => Some(Value::Bool(true)),
                    "false" => Some(Value::Bool(false)),
                    "null" => Some(Value::Null),
                    _ => word
                        .parse::<f64>()
                        .ok()
                        .filter(|n| n.is_finite())
                        .map(|n| Value::Number(n.into())),
                };
                if let Some(value) = literal {
                    return Some(Predicate::Literal(value));
                }
                if word.contains('.') {
                    let segments: Vec<String> = word.split('.').map(str::to_owned).collect();
                    if segments.iter().any(|s| {
                        s.is_empty()
                            || !s.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_')
                    }) {
                        return None;
                    }
                    Some(Predicate::Path(segments))
                } else {
                    Some(Predicate::Literal(Value::String(word)))
                }
            }
            _ => None,
        }
    }
}
