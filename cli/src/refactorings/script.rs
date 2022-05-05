use crate::{
    refactorings::{parser::TopLevel, Refactoring},
    EditorContext,
};

pub struct Script {
    top_levels: Vec<TopLevel>,
}

impl Script {
    pub fn new(top_levels: Vec<TopLevel>) -> Self {
        Script { top_levels }
    }
}

impl Refactoring for Script {
    fn applies_to(&self, _context: &EditorContext) -> bool {
        todo!("applies_to")
    }

    fn name(&self) -> String {
        todo!("name")
    }

    fn description(&self) -> String {
        todo!("description")
    }
}
