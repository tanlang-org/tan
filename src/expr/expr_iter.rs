use std::mem;

use crate::ann::Ann;

use super::Expr;

impl Ann<Expr> {
    pub fn iter(&self) -> ExprIter<'_> {
        ExprIter {
            children: std::slice::from_ref(self),
            parent: None,
        }
    }
}

// #Insight
// The iterator is implemented as a separate struct, for flexibility.

// #TODO support in-order, pre-order, post-order
// #TODO implement owned iterator
// #TODO implement mutable iterator
// #TODO https://aloso.github.io/2021/03/09/creating-an-iterator

// #TODO is this really DFS?
/// A depth-first Expr iterator.
#[derive(Default)]
pub struct ExprIter<'a> {
    children: &'a [Ann<Expr>],
    parent: Option<Box<ExprIter<'a>>>,
}

impl<'a> Iterator for ExprIter<'a> {
    type Item = &'a Ann<Expr>;

    // #TODO this does not traverse Array, Dict, etc.
    fn next(&mut self) -> Option<Self::Item> {
        let expr = self.children.get(0);

        match expr {
            None => match self.parent.take() {
                Some(parent) => {
                    // continue with the parent expr
                    *self = *parent;
                    self.next()
                }
                None => None,
            },
            Some(Ann(Expr::List(children), ..)) => {
                self.children = &self.children[1..];
                // iterate the sub-trees
                *self = ExprIter {
                    children: children.as_slice(),
                    parent: Some(Box::new(mem::take(self))),
                };
                // self.next()
                expr
            }
            _ => {
                // let x = self.children.get(0);
                self.children = &self.children[1..];
                expr
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{lexer::Lexer, parser::Parser};

    #[test]
    fn expr_iter_performs_depth_first_iteration() {
        let input = "(quot (1 2 3 (4 5) (6 (+ 7 8)) 9 10))";

        let mut lexer = Lexer::new(input);
        let tokens = lexer.lex().unwrap();

        let mut parser = Parser::new(tokens);
        let expr = parser.parse().unwrap();

        let expr = &expr[0];

        let terms: Vec<String> = expr.iter().map(|ax| ax.0.to_string()).collect();
        let expected_terms = vec![
            "(quot (1 2 3 (4 5) (6 (+ 7 8)) 9 10))",
            "quot",
            "(1 2 3 (4 5) (6 (+ 7 8)) 9 10)",
            "1",
            "2",
            "3",
            "(4 5)",
            "4",
            "5",
            "(6 (+ 7 8))",
            "6",
            "(+ 7 8)",
            "+",
            "7",
            "8",
            "9",
            "10",
        ];
        assert_eq!(terms, expected_terms);
    }
}
