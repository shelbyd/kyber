use crate::{
    refactorings::{parser::*, Refactoring},
    ContentRegion, EditorContext,
};
use std::{collections::*, ops::Range};

pub struct Script {
    top_levels: Vec<TopLevel>,
}

impl Refactoring for Script {
    fn applies_to(&self, contents: &EditorContext) -> bool {
        self.exec(&contents.contents_ref()).is_ok()
    }

    fn name(&self) -> String {
        self.directive_value("name")
    }

    fn description(&self) -> String {
        self.directive_value("description")
    }
}

impl Script {
    pub fn new(top_levels: Vec<TopLevel>) -> Self {
        Script { top_levels }
    }

    fn directive_value(&self, directive_name: &str) -> String {
        self.top_levels
            .iter()
            .filter_map(|t| match t {
                TopLevel::Directive(d) if d.name == directive_name => Some(d.value.clone()),
                _ => None,
            })
            .next()
            .unwrap()
    }

    fn exec(&self, contents: &[ContentRegion<&str>]) -> Result<(), ()> {
        let mut scope = HashMap::new();

        let all_contents = contents.iter().map(|r| r.text).collect::<String>();

        for tl in &self.top_levels {
            match tl {
                TopLevel::Assignment(ident, expr) => {
                    let mut selected = selected(contents);
                    let mut current_contents = &all_contents[..];

                    let found = loop {
                        let found = self.range(expr, &scope, current_contents)?;
                        if found.start > selected.end {
                            return Err(());
                        }
                        if overlaps(&found, &selected) {
                            break found;
                        }
                        let offset = found.end;
                        current_contents = &current_contents[offset..];
                        selected = (selected.start - offset)..(selected.end - offset);
                    };

                    scope.insert(ident.to_string(), found);
                }

                TopLevel::Expr(_) => {}
                TopLevel::Directive(_) => {}
                unhandled => {
                    unimplemented!("unhandled: {:?}", unhandled);
                }
            }
        }

        Ok(())
    }

    fn range(
        &self,
        expr: &Expr,
        scope: &HashMap<String, Range<usize>>,
        contents: &str,
    ) -> Result<Range<usize>, ()> {
        let range = match expr {
            Expr::StringLiteral(s) => {
                let start = contents.find(s).ok_or(())?;
                let end = start + s.len();
                start..end
            }

            Expr::Regex(re) => {
                let mat = re.find(contents).ok_or(())?;
                mat.range()
            }

            Expr::Concatenate(left, right) => {
                let left_range = self.range(left, scope, contents)?;

                let offset = left_range.end;
                let rest = &contents[left_range.end..];
                let right_range = self.range(right, scope, rest)?;

                if right_range.start != 0 {
                    let sub = self.range(expr, scope, rest)?;
                    return Ok((sub.start + offset)..(sub.end + offset));
                }

                let start = left_range.start;
                let end = left_range.end + (right_range.end - right_range.start);
                start..end
            }

            Expr::Binding(_, e) => return self.range(e, scope, contents),

            unhandled => {
                unimplemented!("unhandled: {:?}", unhandled);
            }
        };
        Ok(range)
    }
}

fn selected(contents: &[ContentRegion<&str>]) -> Range<usize> {
    let mut start = 0;
    for r in contents {
        if r.selected {
            let end = start + r.text.len();
            return start..end;
        }

        start += r.text.len();
    }
    0..0
}

fn overlaps(larger: &Range<usize>, smaller: &Range<usize>) -> bool {
    larger.start <= smaller.start && larger.end >= smaller.end && smaller.start < larger.end
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{refactorings::parser::parse, ContentRegion};

    fn context(regions: &[&str]) -> EditorContext {
        EditorContext {
            contents: regions
                .iter()
                .enumerate()
                .map(|(i, text)| ContentRegion {
                    text: text.to_string(),
                    selected: i % 2 == 1,
                })
                .collect(),
        }
    }

    #[cfg(test)]
    mod applies_to {
        use super::*;

        #[test]
        fn single_char() {
            let script = parse(r#"let region = "t";"#).unwrap();

            assert!(script.applies_to(&context(&["t"])));
            assert!(!script.applies_to(&context(&["u"])));
        }

        #[test]
        fn anywhere_in_string() {
            let script = parse(r#"let region = "test";"#).unwrap();

            assert!(script.applies_to(&context(&["test"])));
            assert!(script.applies_to(&context(&["t", "", "est"])));
            assert!(!script.applies_to(&context(&["test", "", ""])));
            assert!(!script.applies_to(&context(&["r", "", "est"])));
            assert!(!script.applies_to(&context(&["t", "", "t"])));
        }

        #[test]
        fn with_selected() {
            let script = parse(r#"let region = "test";"#).unwrap();

            assert!(script.applies_to(&context(&["t", "es", "t"])));
        }

        #[test]
        fn concatenated() {
            let script = parse(r#"let region = "te" .. "st";"#).unwrap();

            assert!(script.applies_to(&context(&["test"])));
        }

        #[test]
        fn binding() {
            let script = parse(r#"let region = foo:("te" .. "st");"#).unwrap();

            assert!(script.applies_to(&context(&["test"])));
        }

        #[test]
        fn regex() {
            let script = parse(r#"let region = /s+/;"#).unwrap();

            assert!(script.applies_to(&context(&["s"])));
        }

        #[test]
        fn multiple_tries_for_concat() {
            let script = parse(r#"let region = "te" .. "st";"#).unwrap();

            assert!(script.applies_to(&context(&["tet", "", "est"])));
        }

        #[test]
        fn end_to_end() {
            let script = parse(
                r#"let region =
                a:(/[\w_]+/ .. /\s+/) ..
                "!=" ..
                b:(/\s+/ .. /[\w_]+/);"#,
            )
            .unwrap();

            assert!(!script.applies_to(&context(&["extern crate rand; { not_a_winner != false }"])));
            assert!(script.applies_to(&context(&["not_a_winner ", "!=", " false"])));
        }

        #[test]
        fn multiple_instances() {
            let script = parse(
                r#"let region =
                a:(/[\w_]+/ .. /\s+/) ..
                "!=" ..
                b:(/\s+/ .. /[\w_]+/);"#,
            )
            .unwrap();

            assert!(script.applies_to(&context(&["foo != bar; not_a_winner ", "!=", " false"])));
        }
    }

    #[test]
    fn overlaps_() {
        assert!(overlaps(&(0..4), &(3..3)));
        assert!(!overlaps(&(0..4), &(4..4)));
    }

    #[test]
    fn selected_() {
        assert_eq!(selected(&context(&["test"]).contents_ref()), 0..0);
        assert_eq!(selected(&context(&["test", "some"]).contents_ref()), 4..8);
        assert_eq!(
            selected(&context(&["test", "some", "stuff"]).contents_ref()),
            4..8
        );
        assert_eq!(
            selected(&context(&["test", "", "stuff"]).contents_ref()),
            4..4
        );
    }
}
