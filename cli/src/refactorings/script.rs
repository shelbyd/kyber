use crate::{
    refactorings::{parser::*, Mutation, Refactoring},
    ContentRegion, EditorContext,
};
use std::{collections::*, ops::Range};

#[derive(Debug)]
pub struct Script {
    top_levels: Vec<TopLevel>,
}

impl Refactoring for Script {
    fn applies_to(&self, context: &EditorContext) -> bool {
        self.exec(context).is_ok()
    }

    fn perform(&self, context: &EditorContext) -> Result<Vec<Mutation>, String> {
        self.exec(context)
    }

    fn id(&self) -> String {
        self.directive_value("id")
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

    fn exec(&self, context: &EditorContext) -> Result<Vec<Mutation>, String> {
        let mut result = Vec::new();
        let mut scope = HashMap::new();

        for tl in &self.top_levels {
            match tl {
                TopLevel::Stmt(Stmt::Assignment(ident, expr)) => {
                    scope.insert(ident.to_string(), self.eval(expr, &scope, context)?);
                }

                TopLevel::Stmt(Stmt::Expr(e)) => match self.eval(e, &mut scope, context)? {
                    Value::Mutations(m) => result.extend(m),
                    _ => {}
                },
                TopLevel::Directive(_) => {}
                unhandled => {
                    unimplemented!("unhandled: {:?}", unhandled);
                }
            }
        }

        Ok(result)
    }

    fn eval(
        &self,
        expr: &Expr,
        scope: &HashMap<String, Value>,
        context: &EditorContext,
    ) -> Result<Value, String> {
        match expr {
            Expr::MethodCall(obj, method, args) => {
                let obj = self.eval(obj, scope, context)?;
                match (obj, method.as_str()) {
                    (Value::Range(r, _), "replace") => {
                        let replace_with_expr = args
                            .get(0)
                            .ok_or_else(|| format!("Too few arguments to Range.replace"))?;
                        let replace_with = match self.eval(replace_with_expr, scope, context)? {
                            Value::String(s) => s,
                            unexpected => {
                                return Err(format!("Expected string, found {:?}", unexpected))
                            }
                        };

                        let selected = selected(&context.contents_ref());
                        let (deletes, backspaces) = delete_range(r.clone(), selected)
                            .ok_or_else(|| String::from("Could not mutate range"))?;

                        let mut mutations = Vec::new();
                        if deletes > 0 {
                            mutations.push(Mutation::Delete(deletes));
                        }
                        if backspaces > 0 {
                            mutations.push(Mutation::Backspace(backspaces));
                        }
                        mutations.push(Mutation::Insert(replace_with));
                        Ok(Value::Mutations(mutations))
                    }
                    unhandled => {
                        unimplemented!("unhandled: {:?}", unhandled);
                    }
                }
            }
            Expr::FnCall(func, args) => match func.as_str() {
                "find" => {
                    let contents = context.contents_ref();
                    let all_contents = contents.iter().map(|r| r.text).collect::<String>();
                    let selected = selected(&contents);

                    let expr = args
                        .get(0)
                        .ok_or_else(|| String::from("Too few arguments to find"))?;

                    let mut offset = 0;
                    loop {
                        let (found, bindings) = self.range(expr, &all_contents[offset..])?;
                        let found = (found.start + offset)..(found.end + offset);

                        if found.start > selected.end {
                            return Err(format!("Not found"));
                        }
                        if overlaps(&found, &selected) {
                            return Ok(Value::Range(found, bindings));
                        }
                        offset = found.end;
                    }
                }

                unhandled => {
                    unimplemented!("unhandled: {:?}", unhandled);
                }
            },
            Expr::Ident(i) => scope
                .get(i)
                .cloned()
                .ok_or_else(|| format!("Unknown variable {:?}", i)),
            Expr::StringLiteral(s) => Ok(Value::String(s.clone())),

            Expr::Concatenate(left, right) => {
                match (
                    self.eval(left, scope, context)?,
                    self.eval(right, scope, context)?,
                ) {
                    (Value::String(left), Value::String(right)) => Ok(Value::String(left + &right)),
                    unhandled => {
                        unimplemented!("unhandled: {:?}", unhandled);
                    }
                }
            }

            Expr::DotAccess(obj, prop) => {
                let obj = self.eval(obj, scope, context)?;
                match obj {
                    Value::Range(_, props) => props
                        .get(prop)
                        .map(|s| Value::String(s.clone()))
                        .ok_or_else(|| format!("Region does not have binding {:?}", prop)),
                    unhandled => {
                        unimplemented!("unhandled: {:?}", unhandled);
                    }
                }
            }

            unhandled => {
                unimplemented!("unhandled: {:?}", unhandled);
            }
        }
    }

    fn range(
        &self,
        expr: &Expr,
        contents: &str,
    ) -> Result<(Range<usize>, HashMap<String, String>), String> {
        match expr {
            Expr::StringLiteral(s) => {
                let start = contents
                    .find(s)
                    .ok_or_else(|| format!("Not found {:?}", s))?;
                let end = start + s.len();
                Ok((start..end, HashMap::new()))
            }

            Expr::Regex(re) => {
                let mat = re
                    .find(contents)
                    .ok_or_else(|| format!("No match /{:?}/", re))?;
                Ok((mat.range(), HashMap::new()))
            }

            Expr::Concatenate(left, right) => {
                let (left_range, mut bindings) = self.range(left, contents)?;

                let offset = left_range.end;
                let rest = &contents[left_range.end..];
                let (right_range, right_bindings) = self.range(right, rest)?;

                if right_range.start != 0 {
                    let (sub, bindings) = self.range(expr, rest)?;
                    let new_range = (sub.start + offset)..(sub.end + offset);
                    return Ok((new_range, bindings));
                }

                let start = left_range.start;
                let end = left_range.end + (right_range.end - right_range.start);

                bindings.extend(right_bindings);

                Ok((start..end, bindings))
            }

            Expr::Binding(ident, e) => {
                let (range, mut bindings) = self.range(e, contents)?;
                bindings.insert(ident.to_string(), contents[range.clone()].to_string());
                Ok((range, bindings))
            }

            unhandled => {
                unimplemented!("unhandled: {:?}", unhandled);
            }
        }
    }
}

#[derive(Debug, Clone)]
enum Value {
    Range(Range<usize>, HashMap<String, String>),
    Mutations(Vec<Mutation>),
    String(String),
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

fn delete_range(to_delete: Range<usize>, selected: Range<usize>) -> Option<(usize, usize)> {
    let deletes_needed = to_delete.end.checked_sub(selected.end)?;
    let backspace_needed = selected.start.checked_sub(to_delete.start)?;

    let selected_len = selected.end - selected.start;
    if selected_len > 0 {
        Some((deletes_needed + 1, backspace_needed))
    } else {
        Some((deletes_needed, backspace_needed))
    }
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
            let script = parse(r#"find("t");"#).unwrap();

            assert!(script.applies_to(&context(&["t"])));
            assert!(!script.applies_to(&context(&["u"])));
        }

        #[test]
        fn anywhere_in_string() {
            let script = parse(r#"find("test");"#).unwrap();

            assert!(script.applies_to(&context(&["test"])));
            assert!(script.applies_to(&context(&["t", "", "est"])));
            assert!(!script.applies_to(&context(&["test", "", ""])));
            assert!(!script.applies_to(&context(&["r", "", "est"])));
            assert!(!script.applies_to(&context(&["t", "", "t"])));
        }

        #[test]
        fn with_selected() {
            let script = parse(r#"find("test");"#).unwrap();

            assert!(script.applies_to(&context(&["t", "es", "t"])));
        }

        #[test]
        fn concatenated() {
            let script = parse(r#"find("te" .. "st");"#).unwrap();

            assert!(script.applies_to(&context(&["test"])));
        }

        #[test]
        fn binding() {
            let script = parse(r#"find(foo:("te" .. "st"));"#).unwrap();

            assert!(script.applies_to(&context(&["test"])));
        }

        #[test]
        fn regex() {
            let script = parse(r#"find(/s+/);"#).unwrap();

            assert!(script.applies_to(&context(&["s"])));
        }

        #[test]
        fn multiple_tries_for_concat() {
            let script = parse(r#"find("te" .. "st");"#).unwrap();

            assert!(script.applies_to(&context(&["tet", "", "est"])));
        }

        #[test]
        fn end_to_end() {
            let script = parse(
                r#"find(
                    a:(/[\w_]+/ .. /\s+/) ..
                    "!=" ..
                    b:(/\s+/ .. /[\w_]+/));"#,
            )
            .unwrap();

            assert!(!script.applies_to(&context(&["extern crate rand; { not_a_winner != false }"])));
            assert!(script.applies_to(&context(&["not_a_winner ", "!=", " false"])));
        }

        #[test]
        fn multiple_instances() {
            let script = parse(
                r#"find(
                    a:(/[\w_]+/ .. /\s+/) ..
                    "!=" ..
                    b:(/\s+/ .. /[\w_]+/));"#,
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

    #[cfg(test)]
    mod perform {
        use super::*;

        #[test]
        fn single_char_replacement() {
            let script = parse(r#"let region = find("t"); region.replace("r");"#).unwrap();

            assert_eq!(
                script.perform(&context(&["t"])).unwrap(),
                vec![Mutation::Delete(1), Mutation::Insert("r".to_string())]
            );
        }

        #[test]
        fn replace_with_concat() {
            let script = parse(r#"let region = find("t"); region.replace("r" .. "e");"#).unwrap();

            assert_eq!(
                script.perform(&context(&["t"])).unwrap(),
                vec![Mutation::Delete(1), Mutation::Insert("re".to_string())]
            );
        }

        #[test]
        fn using_binding() {
            let script =
                parse(r#"let region = find("r" .. foo:(/\w+/)); region.replace(region.foo);"#)
                    .unwrap();

            assert_eq!(
                script.perform(&context(&["rate"])).unwrap(),
                vec![Mutation::Delete(4), Mutation::Insert("ate".to_string())]
            );
        }

        #[test]
        fn multiple_occurrences() {
            let script = parse(r#"let region = find("r"); region.replace("t");"#).unwrap();

            assert_eq!(
                script.perform(&context(&["rrr", "", "r"])).unwrap(),
                vec![Mutation::Delete(1), Mutation::Insert("t".to_string())]
            );
        }
    }

    #[test]
    fn delete_range_() {
        assert_eq!(delete_range(0..1, 0..0).unwrap(), (1, 0));
        assert_eq!(delete_range(0..2, 0..0).unwrap(), (2, 0));
        assert_eq!(delete_range(1..2, 1..1).unwrap(), (1, 0));
        assert_eq!(delete_range(0..2, 1..1).unwrap(), (1, 1));
        assert_eq!(delete_range(0..2, 0..2).unwrap(), (1, 0));
        assert_eq!(delete_range(0..4, 0..2).unwrap(), (3, 0));

        assert_eq!(delete_range(1..3, 0..0), None);
    }
}
