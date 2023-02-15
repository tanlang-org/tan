use std::{collections::HashMap, fmt};

use crate::{
    ann::Ann,
    error::Error,
    expr::{format_value, Expr},
    lexer::{token::Token, Lexer},
    range::{Range, Ranged},
};

// #TODO no need to keep iterator as state in parser!
// #TODO can the parser be just a function? -> yes, if we use a custom iterator to keep the parsing state.
// #TODO think some more how annotations should be handled.

// #Insight
// The syntax of the language is explicitly designed to _not_ require a lookahead buffer.

// #Insight
// We move the tokens into the parser to simplify the code. The tokens are useless outside the parser.

/// The`NonRecoverableError` is thrown when the parser cannot synchronize
/// to continue parsing to detect more errors. Parsing is stopped immediately.
#[derive(Debug)]
pub struct NonRecoverableError {}

impl std::error::Error for NonRecoverableError {}

impl fmt::Display for NonRecoverableError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "NRE")
    }
}

/// The Parser performs the syntactic analysis stage of the compilation pipeline.
/// The input token stream is reduced into and Abstract Syntax Tree (AST).
/// The nodes of the AST are associated with annotations.
pub struct Parser<I>
where
    I: IntoIterator<Item = Ranged<Token>>,
{
    tokens: I::IntoIter,
    buffered_annotations: Option<Vec<Ranged<String>>>,
    start: usize,
    index: usize,
    lookahead: Vec<Ranged<Token>>,
    errors: Vec<Ranged<Error>>,
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
            start: 0,
            index: 0,
            lookahead: Vec::new(),
            errors: Vec::new(),
        }
    }

    // #TODO unit test
    // #TODO refactor
    fn next_token(&mut self) -> Option<Ranged<Token>> {
        if let Some(token) = self.lookahead.pop() {
            self.index = token.1.end;
            return Some(token);
        }

        if let Some(token) = self.tokens.next() {
            self.index = token.1.end;
            Some(token)
        } else {
            None
        }
    }

    fn put_back_token(&mut self, token: Ranged<Token>) {
        self.index = token.1.start;
        self.lookahead.push(token);
    }

    fn range(&self) -> Range {
        self.start..self.index
    }

    fn push_error(&mut self, error: Error, range: &Range) {
        self.errors.push(Ranged(error, range.clone()));
    }

    /// Wrap the `expr` with the buffered (prefix) annotations. The annotations
    /// are parsed into an Expr representation. Also attaches the range of the
    /// expression as an annotation.
    fn attach_annotations(&mut self, expr: Expr) -> Ann<Expr> {
        // Annotate the expression with the range, by default.
        let mut expr = Ann::with_range(expr, self.range());

        let Some(annotations) = self.buffered_annotations.take() else {
            // No annotations for the expression.
            return expr;
        };

        for Ranged(ann_str, ann_range) in annotations {
            let mut lexer = Lexer::new(&ann_str);

            let Ok(tokens) = lexer.lex() else {
                self.push_error(Error::MalformedAnnotation(ann_str), &ann_range);
                // Ignore the buffered annotations, and continue parsing to find more syntactic errors.
                return expr;
            };

            let mut parser = Parser::new(tokens);

            let ann_expr = parser.parse();

            if let Err(ann_expr_errors) = ann_expr {
                for error in ann_expr_errors {
                    self.push_error(error.0, &error.1);
                }
                // Ignore the buffered annotations, and continue parsing to find more syntactic errors.
                return expr;
            }

            // #TODO temp, support multiple expressions in annotation?
            let ann_expr = ann_expr.unwrap().swap_remove(0);

            let ann_expr = ann_expr.0;

            match &ann_expr {
                Expr::Symbol(sym) => {
                    if sym.is_empty() {
                        // #TODO specialized error needed.
                        self.push_error(Error::MalformedAnnotation(ann_str), &ann_range);
                        // Ignore the buffered annotations, and continue parsing to find more syntactic errors.
                        return expr;
                    }

                    if sym.chars().next().unwrap().is_uppercase() {
                        // Type shorthand: If the annotation starts with uppercase
                        // letter, it's considered type annotations.
                        expr.set_annotation("type", ann_expr);
                    } else {
                        // Bool=true shorthand: If the annotation starts with lowercase
                        // letter, it's considered a boolean flag.
                        expr.set_annotation(sym.clone(), Expr::Bool(true));
                    }
                }
                Expr::List(list) => {
                    if let Some(Ann(Expr::Symbol(sym), _)) = list.first() {
                        expr.set_annotation(sym.clone(), ann_expr);
                    } else {
                        self.push_error(Error::MalformedAnnotation(ann_str), &ann_range);
                        // Ignore the buffered annotations, and continue parsing to find more syntactic errors.
                        return expr;
                    }
                }
                _ => {
                    self.push_error(Error::MalformedAnnotation(ann_str), &ann_range);
                    // Ignore the buffered annotations, and continue parsing to find more syntactic errors.
                    return expr;
                }
            }
        }

        expr
    }

    pub fn parse_expr(&mut self) -> Result<Option<Expr>, NonRecoverableError> {
        let Some(token) = self.next_token() else {
            // #TODO not strictly an error, rename to Exit/Break or something.
            return Err(NonRecoverableError {});
        };

        let Ranged(t, range) = token;

        let expr = match t {
            Token::Comment(..) => None,
            // Token::Char(c) => Some(Expr::Char(c)),
            Token::String(s) => Some(Expr::String(s)),
            Token::Symbol(s) => {
                if s.starts_with(':') {
                    let s = s.strip_prefix(':').unwrap();
                    Some(Expr::KeySymbol(s.to_string()))
                } else if s == "true" {
                    // #TODO consider using (True) for true 'literal'.
                    // #TODO e.g. (let flag (True))
                    // #TODO Bool = True + False = True | False = ~False | False
                    Some(Expr::Bool(true))
                } else if s == "false" {
                    // #TODO consider using False for false 'literal'.
                    Some(Expr::Bool(false))
                } else {
                    Some(Expr::Symbol(s))
                }
            }
            Token::Number(mut lexeme) => {
                // #TODO more detailed Number error!
                // #TODO error handling not enough, we need to add context, check error_stack

                if lexeme.contains('.') {
                    // #TODO support radix for non-integers?
                    // #TODO find a better name for 'non-integer'.
                    match lexeme.parse::<f64>().map_err(Error::MalformedFloat) {
                        Ok(n) => Some(Expr::Float(n)),
                        Err(error) => {
                            self.push_error(error, &range);
                            None
                        }
                    }
                } else {
                    // #TODO support arbitrary radix https://github.com/golang/go/issues/28256
                    let mut radix = 10;

                    if lexeme.starts_with("0x") {
                        lexeme = lexeme.replace("0x", "");
                        radix = 16
                    } else if lexeme.starts_with("0b") {
                        lexeme = lexeme.replace("0b", "");
                        radix = 2
                    } else if lexeme.starts_with("0o") {
                        // #Insight Octal literals are supported for historical reasons.
                        lexeme = lexeme.replace("0o", "");
                        radix = 8
                    }

                    match i64::from_str_radix(&lexeme, radix).map_err(Error::MalformedInt) {
                        Ok(n) => Some(Expr::Int(n)),
                        Err(error) => {
                            self.push_error(error, &range);
                            None
                        }
                    }
                }
            }
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
            Token::Quote => {
                // #Insight we should allow consecutive quotes, emit a linter warning instead!

                let Ok(quot_expr) = self.parse_expr() else {
                    // Parsing the quoted expression failed.
                    // Continue parsing to detect more errors.
                    self.push_error(Error::InvalidQuote, &range);
                    return Ok(None);
                };

                let Some(target) = quot_expr else {
                    self.push_error(Error::InvalidQuote, &range);
                    // It is recoverable error.
                    return Ok(None);
                };

                // #TODO the actual quoting should be handled here?
                // #TODO what about interpolation?

                Some(Expr::List(vec![Expr::symbol("quot").into(), target.into()]))
            }
            Token::LeftParen => {
                self.start = range.start;

                let terms = self.parse_many(Token::RightParen)?;

                if terms.is_empty() {
                    // #TODO do we _really_ want this or just return a list?
                    // `()` == One/Unit/Top
                    Some(Expr::One)
                } else {
                    Some(Expr::List(terms))

                    // #TODO optimize some special forms but in another comptime pass.

                    // let head = list_exprs[0].clone();
                    // match head {
                    //     // #TODO optimize more special forms.

                    //     // #Insight
                    //     // Parsing built-ins as Exprs optimizes runtime evaluation, with more efficient
                    //     // matching.

                    //     // `if` expression
                    //     Ann(Expr::Symbol(s), ..) if s == "if" => {
                    //         // #TODO detailed checking and error-reporting
                    //         Some(Expr::If(
                    //             Box::new(list_exprs[1].clone()),
                    //             Box::new(list_exprs[2].clone()),
                    //             // #TODO optional parsing!
                    //             Some(Box::new(list_exprs[3].clone())),
                    //         ))
                    //     }
                    //     _ => Some(Expr::List(list_exprs)),
                    // }
                }
            }
            Token::LeftBracket => {
                // Syntactic sugar for a List/Array.

                self.start = range.start;

                let args = self.parse_many(Token::RightBracket)?;

                let mut items = Vec::new();

                // #TODO also parse (Array ..)
                // #TODO add error checking!
                // #TODO optimize.
                // #TODO evaluate the list_exprs
                // #TODO list

                for x in args {
                    items.push(x.0);
                }

                Some(Expr::Array(items))
            }
            Token::LeftBrace => {
                // Syntactic sugar for a Dict.

                self.start = range.start;

                let args = self.parse_many(Token::RightBrace)?;

                let mut dict = HashMap::new();

                // #TODO also parse (Dict ..)
                // #TODO add error checking!
                // #TODO optimize.
                // #TODO use Hashable.
                // #TODO evaluate the list_exprs
                // #TODO list

                for pair in args.chunks(2) {
                    let k = pair[0].clone();
                    let v = pair[1].clone();
                    dict.insert(format_value(k.0), v.0);
                }

                Some(Expr::Dict(dict))
            }
            Token::RightParen | Token::RightBracket | Token::RightBrace => {
                // #TODO custom error for this?
                self.push_error(Error::UnexpectedToken(t), &range);
                // Parsing can continue.
                return Ok(None);
            }
        };

        Ok(expr)
    }

    // #TODO rename to `parse_until`?
    pub fn parse_many(&mut self, delimiter: Token) -> Result<Vec<Ann<Expr>>, NonRecoverableError> {
        let mut exprs = Vec::new();

        loop {
            let Some(token) = self.next_token() else {
                self.push_error(Error::UnterminatedList, &self.range());
                return Err(NonRecoverableError {});
            };

            if token.0 == delimiter {
                // Will be annotated upstream.
                return Ok(exprs);
            } else {
                self.put_back_token(token);
                if let Some(e) = self.parse_expr()? {
                    let e = self.attach_annotations(e);
                    exprs.push(e);
                }
            }
        }
    }

    // #Insight
    // The parse function intentionally returns an 'unstructured' vector of
    // expressions instead of something like a do-block or a module. Downstream
    // functions can enforce some structure.dd

    // #Insight
    // The loop in the parser is also useful to skip over comments.

    /// Parses the input tokens into expressions.
    /// The parser tries to return as many errors as possible.
    pub fn parse(&mut self) -> Result<Vec<Ann<Expr>>, Vec<Ranged<Error>>> {
        let mut exprs = Vec::new();

        loop {
            let expr = self.parse_expr();

            let Ok(expr) = expr else {
                // A non-recoverable parse error was detected, stop parsing.
                break;
            };

            if let Some(expr) = expr {
                let expr = self.attach_annotations(expr);

                if self.errors.is_empty() {
                    exprs.push(expr);
                } else {
                    break;
                }
            }
        }

        if self.errors.is_empty() {
            Ok(exprs)
        } else {
            let errors = std::mem::take(&mut self.errors);
            Err(errors)
        }
    }
}
