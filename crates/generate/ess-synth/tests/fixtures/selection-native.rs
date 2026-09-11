use selection_system::{choose, SelectionFailureCause};
use selection_types::core::{Arrived, Domain, Leg, Source};
fn leg(id: &str, domain: Option<Domain>, source: Option<Source>) -> Option<Leg> { Some(Leg { id:id.into(),domain,source,from:"remote".into() }) }
#[test]
fn mixed_occurrences_and_fallbacks() {
    use Domain::{Internal as I, External as E}; use Source::Webrtc as W;
    let rows = [
        (vec![],None,None),
        (vec![None,leg("x",None,None)],Some("x"),Some("x")),
        (vec![leg("x",Some(E),None),leg("y",Some(I),None)],Some("y"),Some("x")),
        (vec![leg("x",Some(E),Some(W))],Some("x"),Some("x")),
        (vec![leg("x",Some(E),Some(W)),leg("y",Some(E),None)],Some("x"),Some("y")),
        (vec![leg("x",Some(I),None),leg("y",Some(E),Some(W))],Some("x"),Some("y")),
        (vec![leg("x",Some(E),Some(W)),leg("y",Some(E),Some(W))],Some("x"),Some("y")),
        (vec![leg("x",Some(E),None),leg("y",Some(E),None)],Some("x"),Some("x")),
        (vec![None,leg("",None,None),leg("z",None,None)],Some("z"),Some("z")),
        (vec![leg("",Some(I),None),leg("y",Some(E),None)],Some(""),Some("y")),
        (vec![leg("",Some(E),None),leg("y",None,None)],Some("y"),Some("")),
        (vec![leg("x",Some(I),None),leg("y",Some(I),None),leg("z",Some(E),None)],Some("x"),Some("z")),
        (vec![leg("same",Some(E),Some(W)),leg("same",Some(E),None)],Some("same"),Some("same")),
    ];
    for (index,(data,agent,external)) in rows.into_iter().enumerate() {let result=choose(&Arrived{data}).unwrap();assert_eq!(result.agent_id.as_deref(),agent,"agent row {index}");assert_eq!(result.external_id.as_deref(),external,"external row {index}");}
}
#[test]
fn oversized_tail_refuses_before_returning_input() {let data=vec![leg("x",Some(Domain::Internal),None);65];let failure=choose(&Arrived{data}).unwrap_err();assert_eq!(failure.cause,SelectionFailureCause::Resource);assert_eq!(failure.input,0);}
#[test]
fn examined_oversized_text_tail_is_not_hidden_by_early_match() {let data=vec![leg("x",Some(Domain::Internal),None),leg(&"x".repeat(4097),None,None)];let failure=choose(&Arrived{data}).unwrap_err();assert_eq!(failure.cause,SelectionFailureCause::Resource);assert_eq!(failure.index,Some(1));}
