use projection_types::core::{Arrived, Body, Choice, SourceTag, Wrapped, Wrapper};

#[test]
fn declared_nominal_conversion_preserves_the_terminal_value() {
    let body = Body { status: SourceTag("ready".into()), nested: None };
    let event = Arrived {
        data: body.clone(), partial: None, choice: Choice::Gone(SourceTag("gone".into())),
        wrapped: Wrapped(Wrapper { body }),
    };
    let input = projection_system::project(&event);
    assert_eq!(input.text.0, "ready");
}
