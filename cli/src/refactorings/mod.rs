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

pub fn all() -> impl Iterator<Item = Box<dyn Refactoring>> {
    [
        include_str!("./rust/extract_not_eq.kyb"),
        include_str!("./rust/replace_eq_false.kyb"),
        include_str!("./rust/remove_surrounding_parens.kyb"),
        include_str!("./rust/remove_double_not.kyb"),
    ]
    .into_iter()
    .map(|s| Box::new(parser::parse(s).unwrap()) as Box<dyn Refactoring>)
}

#[derive(Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "snake_case")]
pub enum Mutation {
    Delete(usize),
    Backspace(usize),
    Insert(String),
}
