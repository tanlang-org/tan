use crate::ann::Ann;

use super::Expr;

impl Ann<Expr> {
    // #TODO this is some kind of map-reduce, try to use some kind of interator.
    // #TODO alternatively, this implements some kind of visitor pattern.

    /// Transforms the expression by recursively applying the `f` mapping
    /// function.
    pub fn transform<F>(self, f: &F) -> Self
    where
        F: Fn(Self) -> Self,
    {
        match self {
            Ann(Expr::List(terms), ann) => {
                let terms = terms.into_iter().map(|t| t.transform(f)).collect();
                let list = Ann(Expr::List(terms), ann);
                f(list)
            }
            _ => f(self),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{ann::Ann, api::parse_string, expr::Expr};

    pub fn identity_fn(expr: Ann<Expr>) -> Ann<Expr> {
        expr
    }

    #[test]
    fn transform_with_identity_function() {
        let input = "(quot (1 2 3 (4 5) (6 (+ 7 8)) 9 10))";

        let expr = parse_string(input).unwrap();

        let expr_string = expr.0.to_string();

        let expr_transformed = expr.transform(&identity_fn);

        assert_eq!(expr_string, expr_transformed.0.to_string());
    }
}
