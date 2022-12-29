pub mod error;

use crate::{
    ann::Annotated,
    expr::Expr,
    lexer::{token::Token, Lexer},
    range::{Range, Ranged},
};

use self::error::ParseError;

// #TODO no need to keep iterator as state in parser!
// #TODO can the parser be just a function? -> yes, if we use a custom iterator to keep the parsing state.

// #Insight
// The syntax of the language is explicitly designed to _not_ require a lookahead buffer.

// #Insight
// We move the tokens into the parser to simplify the code. The tokens are useless outside the parser.

/// The Parser performs the syntax analysis stage of the compilation pipeline.
/// The input token stream is reduced into and Abstract Syntax Tree (AST).
/// The nodes of the AST are associated with annotations.
pub struct Parser<I>
where
    I: IntoIterator<Item = Ranged<Token>>,
{
    tokens: I::IntoIter,
    buffered_annotations: Option<Vec<Ranged<String>>>,
}

impl<I> Parser<I>
where
    I: IntoIterator<Item = Ranged<Token>>,
{
    pub fn new(tokens: I) -> Self {
        let tokens = tokens.into_iter();

        Self {
            tokens,
            buffered_annotations: None,
        }
    }

    /// Wrap the `expr` with the buffered (prefix) annotations.
    /// The annotations are parsed into an Expr representation.
    fn attach_buffered_annotations(
        &mut self,
        expr: Expr,
    ) -> Result<Annotated<Expr>, Ranged<ParseError>> {
        let Some(annotations) = self.buffered_annotations.take() else {
            return Ok(Annotated::new(expr));
        };

        let mut ann_exprs = Vec::new();

        for Ranged(ann_str, ann_range) in annotations {
            let mut lexer = Lexer::new(&ann_str);

            let Ok(tokens) = lexer.lex() else {
                return Err(Ranged(ParseError::MalformedAnnotationError(ann_str), ann_range));
            };

            let mut parser = Parser::new(tokens);

            let Annotated(ann_expr, ..) = parser.parse()?;

            ann_exprs.push(ann_expr);
        }

        Ok(Annotated(expr, Some(ann_exprs)))
    }

    // #Insight
    // Parsing built-ins as Exprs optimizes runtime evaluation, with more efficient
    // matching.
    pub fn parse_atom(&mut self, token: Ranged<Token>) -> Result<Option<Expr>, Ranged<ParseError>> {
        let Ranged(t, range) = token;

        let expr = match t {
            Token::Comment(..) => None,
            Token::String(s) => Some(Expr::String(s)),
            Token::Symbol(s) => Some(Expr::Symbol(s)),
            Token::Number(n) => Some(Expr::Int(n)),
            Token::Annotation(s) => {
                if self.buffered_annotations.is_none() {
                    self.buffered_annotations = Some(Vec::new());
                }

                self.buffered_annotations
                    .as_mut()
                    .unwrap()
                    .push(Ranged(s, range));

                None
            }
            Token::LeftParen => {
                let list_exprs = self.parse_list(range)?;

                if list_exprs.is_empty() {
                    // `()` == One/Unit/Top
                    Some(Expr::One)
                } else {
                    let head = list_exprs[0].clone();
                    match head {
                        // `if` expression
                        Annotated(Expr::Symbol(s), ..) if s == "if" => {
                            // #TODO detailed checking and error-reporting
                            Some(Expr::If(
                                Box::new(list_exprs[1].clone()),
                                Box::new(list_exprs[2].clone()),
                                // #TODO optional parsing!
                                Some(Box::new(list_exprs[3].clone())),
                            ))
                        }
                        _ => Some(Expr::List(list_exprs)),
                    }
                }
            }
            Token::RightParen => {
                // #TODO custom error for this?
                return Err(Ranged(ParseError::UnexpectedToken(t), range));
            }
            _ => {
                return Err(Ranged(ParseError::UnexpectedToken(t), range));
            }
        };

        Ok(expr)
    }

    // #TODO parse tokens here, to be consistent with parse_atom?
    pub fn parse_list(
        &mut self,
        list_range: Range,
    ) -> Result<Vec<Annotated<Expr>>, Ranged<ParseError>> {
        let mut exprs = Vec::new();

        let mut token: Option<Ranged<Token>>;

        loop {
            token = self.tokens.next();

            let Some(token) = token  else {
                break;
            };

            match token.0 {
                Token::RightParen => {
                    return Ok(exprs);
                }
                _ => {
                    if let Some(e) = self.parse_atom(token)? {
                        let e = self.attach_buffered_annotations(e)?;
                        exprs.push(e);
                    }
                }
            };
        }

        Err(Ranged(ParseError::UnterminatedList, list_range))
    }

    /// Tries to parse at least one expression.
    pub fn parse(&mut self) -> Result<Annotated<Expr>, Ranged<ParseError>> {
        // #TODO can consolidate more with parse_atom

        loop {
            let token = self.tokens.next();

            let Some(token) = token  else {
                // #TODO what should we return on empty tokens list? Never? Error?
                return Ok(Annotated::new(Expr::One));
            };

            let expr = self.parse_atom(token)?;

            if let Some(expr) = expr {
                return self.attach_buffered_annotations(expr);
            }
        }
    }
}
