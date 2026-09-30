//! The binding-expression grammar of `expressions.forms`, enough to say whether a string is an
//! expression.
//!
//! An expression is paths over the schema's roots (`state.<name>`, `row.<field>`, …, a segment
//! optionally called: `actor.may(<command>)`), the function forms (`same_as(<section>)`,
//! `matches(params)`), literals, and the operators `==`, `!=`, `in [..]`, `not`, `and`, `or`.
//! A string is an expression when the whole of it parses in that grammar **and** it reads at
//! least one path or function form: its value exists only at render time. A string that does not
//! parse (`UNMAPPED: …`, a sentence with punctuation), or that parses to literals alone
//! (`Terms and conditions` is the literals `Terms`, `conditions` joined by `and`), is a literal.

use std::collections::BTreeSet;
use std::sync::OnceLock;

use crate::schema;

/// `true` when `text` is a binding expression rather than a literal.
pub(crate) fn is_expression(text: &str) -> bool {
    let Some(tokens) = tokenize(text) else {
        return false;
    };
    let mut parser = Parser {
        tokens: &tokens,
        at: 0,
        reads: false,
    };
    parser.or().is_some() && parser.at == tokens.len() && parser.reads
}

/// The roots and function forms `expressions.forms` names.
struct Forms {
    roots: BTreeSet<String>,
    functions: BTreeSet<String>,
}

fn forms() -> &'static Forms {
    static FORMS: OnceLock<Forms> = OnceLock::new();
    FORMS.get_or_init(|| {
        let mut forms = Forms {
            roots: BTreeSet::new(),
            functions: BTreeSet::new(),
        };
        for form in schema::expression_forms() {
            let head: String = form
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect();
            match form[head.len()..].chars().next() {
                Some('.') => {
                    forms.roots.insert(head);
                }
                Some('(') => {
                    forms.functions.insert(head);
                }
                _ => {} // `operators`: the grammar below
            }
        }
        forms
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Word(String),
    Quoted,
    Open,
    Close,
    OpenList,
    CloseList,
    Comma,
    Equal,
    NotEqual,
}

fn is_word_char(character: char) -> bool {
    character.is_ascii_alphanumeric() || "_.-".contains(character)
}

fn tokenize(text: &str) -> Option<Vec<Token>> {
    let mut tokens = Vec::new();
    let mut characters = text.char_indices().peekable();
    while let Some((start, character)) = characters.next() {
        let token = match character {
            c if c.is_whitespace() => continue,
            '(' => Token::Open,
            ')' => Token::Close,
            '[' => Token::OpenList,
            ']' => Token::CloseList,
            ',' => Token::Comma,
            '=' | '!' => {
                characters.next_if(|(_, next)| *next == '=')?;
                if character == '=' {
                    Token::Equal
                } else {
                    Token::NotEqual
                }
            }
            '\'' | '"' => {
                characters.find(|(_, next)| *next == character)?;
                Token::Quoted
            }
            c if is_word_char(c) => {
                let mut end = start + c.len_utf8();
                while let Some((at, next)) = characters.next_if(|(_, next)| is_word_char(*next)) {
                    end = at + next.len_utf8();
                }
                Token::Word(text[start..end].to_owned())
            }
            _ => return None,
        };
        tokens.push(token);
    }
    Some(tokens)
}

struct Parser<'a> {
    tokens: &'a [Token],
    at: usize,
    /// Whether a path or function form has been read.
    reads: bool,
}

impl Parser<'_> {
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.at)
    }

    fn eat(&mut self, token: &Token) -> bool {
        let found = self.peek() == Some(token);
        if found {
            self.at += 1;
        }
        found
    }

    fn keyword(&mut self, word: &str) -> bool {
        let found = matches!(self.peek(), Some(Token::Word(w)) if w == word);
        if found {
            self.at += 1;
        }
        found
    }

    fn or(&mut self) -> Option<()> {
        self.and()?;
        while self.keyword("or") {
            self.and()?;
        }
        Some(())
    }

    fn and(&mut self) -> Option<()> {
        self.unary()?;
        while self.keyword("and") {
            self.unary()?;
        }
        Some(())
    }

    fn unary(&mut self) -> Option<()> {
        if self.keyword("not") {
            return self.unary();
        }
        self.primary()?;
        if self.eat(&Token::Equal) || self.eat(&Token::NotEqual) || self.keyword("in") {
            self.primary()?;
        }
        Some(())
    }

    /// Items separated by commas up to `close`, the opening token already read.
    fn items(&mut self, close: &Token) -> Option<()> {
        if self.eat(close) {
            return Some(());
        }
        loop {
            self.or()?;
            if self.eat(close) {
                return Some(());
            }
            if !self.eat(&Token::Comma) {
                return None;
            }
        }
    }

    fn primary(&mut self) -> Option<()> {
        let token = self.peek()?.clone();
        self.at += 1;
        match token {
            Token::Open => {
                self.or()?;
                self.eat(&Token::Close).then_some(())
            }
            Token::OpenList => self.items(&Token::CloseList),
            Token::Quoted => Some(()),
            Token::Word(word) => {
                if ["and", "or", "not", "in"].contains(&word.as_str()) {
                    return None;
                }
                let forms = forms();
                let path = word.split_once('.').is_some_and(|(root, rest)| {
                    forms.roots.contains(root) && rest.split('.').all(|segment| !segment.is_empty())
                });
                let function = forms.functions.contains(&word);
                if path || function {
                    self.reads = true;
                }
                if self.eat(&Token::Open) {
                    // Only a path segment or a function form is called.
                    if !(path || function) {
                        return None;
                    }
                    return self.items(&Token::Close);
                }
                // A function form is always called.
                (!function).then_some(())
            }
            Token::Close | Token::CloseList | Token::Comma | Token::Equal | Token::NotEqual => None,
        }
    }
}
