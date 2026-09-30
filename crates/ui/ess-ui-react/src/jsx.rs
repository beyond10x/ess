//! JSX text: elements with props and children, indented for reading.

/// One JSX element being built.
pub(crate) struct El {
    name: String,
    props: Vec<(String, String)>,
    children: Vec<String>,
}

impl El {
    pub(crate) fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            props: Vec::new(),
            children: Vec::new(),
        }
    }

    /// `data-ui-path="<path>"`: a canonical path is always safe inside a JSX string attribute.
    pub(crate) fn path(mut self, path: &str) -> Self {
        self.props
            .push(("data-ui-path".to_owned(), format!("\"{path}\"")));
        self
    }

    /// `data-ui-path="<path>"` unless an enclosing frame already carries this path.
    pub(crate) fn path_if(self, carry: bool, path: &str) -> Self {
        if carry {
            self.path(path)
        } else {
            self
        }
    }

    /// `name={<expression>}`.
    pub(crate) fn expr(mut self, name: &str, value: impl Into<String>) -> Self {
        self.props
            .push((name.to_owned(), format!("{{{}}}", value.into())));
        self
    }

    /// `name={<expression>}` when present.
    pub(crate) fn opt(self, name: &str, value: Option<String>) -> Self {
        match value {
            Some(value) => self.expr(name, value),
            None => self,
        }
    }

    pub(crate) fn child(mut self, child: impl Into<String>) -> Self {
        self.children.push(child.into());
        self
    }

    pub(crate) fn children(mut self, children: impl IntoIterator<Item = String>) -> Self {
        self.children.extend(children);
        self
    }

    pub(crate) fn render(self) -> String {
        let mut out = format!("<{}", self.name);
        let inline = self.props.len() <= 1
            && self.props.iter().all(|(_, value)| !value.contains('\n'))
            && self.children.is_empty();
        if inline {
            for (name, value) in &self.props {
                out.push(' ');
                out.push_str(name);
                out.push('=');
                out.push_str(value);
            }
            out.push_str(" />");
            return out;
        }
        for (name, value) in &self.props {
            out.push_str("\n  ");
            out.push_str(name);
            out.push('=');
            out.push_str(&indent(value, 2));
        }
        if self.children.is_empty() {
            out.push_str("\n/>");
        } else {
            out.push_str(if self.props.is_empty() { ">" } else { "\n>" });
            for child in &self.children {
                out.push_str("\n  ");
                out.push_str(&indent(child, 2));
            }
            out.push_str("\n</");
            out.push_str(&self.name);
            out.push('>');
        }
        out
    }
}

/// Indents every line after the first by `spaces`.
pub(crate) fn indent(text: &str, spaces: usize) -> String {
    let pad = " ".repeat(spaces);
    text.replace('\n', &format!("\n{pad}"))
}

/// Several siblings as one expression.
pub(crate) fn fragment(children: Vec<String>) -> String {
    match children.len() {
        0 => "null".to_owned(),
        1 => children.into_iter().next().unwrap_or_default(),
        _ => {
            let mut out = "<>".to_owned();
            for child in children {
                out.push_str("\n  ");
                out.push_str(&indent(&child, 2));
            }
            out.push_str("\n</>");
            out
        }
    }
}
