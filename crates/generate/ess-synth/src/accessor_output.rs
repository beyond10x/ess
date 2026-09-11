//! Bounded accessor emission; legacy output never consumes this account.
use std::fmt::{self, Write};

const LIMIT: usize = 2 * 1024 * 1024;

#[derive(Default)]
pub(crate) struct Output {
    text: String,
    full: bool,
}
impl Write for Output {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        if self.full || self.text.len().saturating_add(text.len()) > LIMIT {
            self.full = true;
            return Err(fmt::Error);
        }
        self.text.push_str(text);
        Ok(())
    }
}
impl Output {
    pub fn push_str(&mut self, text: &str) {
        let _ = self.write_str(text);
    }
    pub fn push(&mut self, ch: char) {
        let _ = self.write_char(ch);
    }
    pub fn check(&self) -> Result<(), String> {
        if self.full {
            Err("accessor generated source exceeds 2097152 bytes".into())
        } else {
            Ok(())
        }
    }
    pub fn finish(self) -> Result<String, String> {
        self.check()?;
        Ok(self.text)
    }
}

pub(crate) fn failure(
    ir: &ess_compiler::EssIr,
    target: crate::Target,
    plan: &crate::SynthesisPlan,
    source: &str,
    reason: String,
) -> crate::TargetFailure {
    crate::TargetFailure::new(
        ir,
        target,
        plan,
        vec![crate::TargetFailureCause::new(
            crate::TargetFailureCode::AccessorResource,
            vec![source.into()],
            reason,
        )],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounded_writer_refuses_before_appending_over_limit() {
        let mut output = Output::default();
        output.push_str(&"x".repeat(LIMIT));
        assert!(output.check().is_ok());
        output.push('x');
        assert!(output.check().is_err());
        assert_eq!(output.text.len(), LIMIT);
    }

    #[test]
    fn new_resource_cause_selects_failure_three_even_without_accessor_ir() {
        use ess_compiler::{resolve::compile_locating, source::SourceMap};
        use ess_domain::{spec::RawSpecFile, system::Source, Specification};
        let text = "format: ess/1\nsystem: example\nversion: v1\ndomain: example.core\n";
        let spec = Specification::assemble([(
            Source::new("test.yaml"),
            RawSpecFile::parse(text).unwrap(),
        )])
        .unwrap();
        let mut sources = SourceMap::new();
        sources.insert("test.yaml", text);
        let ir = compile_locating(&spec, &sources, &["test.yaml"]).unwrap();
        let plan = crate::synthesize_for(&ir, crate::Target::Rust)
            .unwrap()
            .plan;
        for target in [
            crate::Target::Rust,
            crate::Target::Go,
            crate::Target::Web,
            crate::Target::Clap,
        ] {
            let failure = failure(
                &ir,
                target,
                &plan,
                "binding.example",
                "bounded output".into(),
            );
            let json = serde_json::to_value(failure).unwrap();
            assert_eq!(json["format"], "ess-target-failure/3");
            assert_eq!(json["causes"][0]["code"], "accessor-resource");
        }
    }
}
