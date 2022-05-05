use crate::EditorContext;

mod parser;
mod script;

pub trait Refactoring {
    fn applies_to(&self, context: &EditorContext) -> bool;
    fn name(&self) -> String;
    fn description(&self) -> String;
}

const EXTRACT_NOT_EQ: &str = include_str!("./rust/extract_not_eq.kyb");

pub fn all() -> impl Iterator<Item = Box<dyn Refactoring>> {
    [Box::new(parser::parse(EXTRACT_NOT_EQ).unwrap()) as Box<dyn Refactoring>].into_iter()
}
