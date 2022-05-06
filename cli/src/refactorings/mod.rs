use crate::EditorContext;

use serde::Serialize;

mod parser;
mod script;

pub trait Refactoring {
    fn applies_to(&self, context: &EditorContext) -> bool;
    fn perform(&self, context: &EditorContext) -> Result<Vec<Mutation>, String>;

    fn id(&self) -> String;
    fn name(&self) -> String;
    fn description(&self) -> String;
}

const EXTRACT_NOT_EQ: &str = include_str!("./rust/extract_not_eq.kyb");

pub fn all() -> impl Iterator<Item = Box<dyn Refactoring>> {
    [Box::new(parser::parse(EXTRACT_NOT_EQ).unwrap()) as Box<dyn Refactoring>].into_iter()
}

#[derive(Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "snake_case")]
pub enum Mutation {
    Delete(usize),
    Backspace(usize),
    Insert(String),
}
