use std::str::Chars;

use crate::range::{Range, Ranged};

use self::{error::LexicalError, token::Token};

pub mod error;
pub mod token;

// https://en.wikipedia.org/wiki/Lexical_analysis

// #TODO lex_all, lex_single
// #TODO introduce SemanticToken, with extra semantic information, _after_ parsing.
// #TODO use annotations before number literals to set the type?
// #TODO use (doc_comment ...) for doc-comments.
// #TODO support `\ ` for escaped space in symbols.
// #TODO can the lexer be just a function?
// #TODO implement PutBackIterator
// #TODO use Range literal for ranges?
// #TODO no need to keep iterator as state in Lexer!
// #TODO accept IntoIterator

/// Returns true if ch is considered whitespace.
/// The `,` character is considered whitespace, in the Lisp tradition.
fn is_whitespace(ch: char) -> bool {
    ch.is_whitespace() || ch == ','
}

fn is_delimiter(ch: char) -> bool {
    ch == '(' || ch == ')'
}

fn is_eol(ch: char) -> bool {
    ch == '\n'
}

// #TODO stateful lexer vs buffer

// #Insight Rust's `Peekable` iterator is not used, as multiple-lookahead is
// required to scan e.g. signed-numbers. Additionally, the 'put_back' interface
// seems more intuitive and ergonomic.

/// The Lexer performs the lexical analysis stage of the compilation pipeline.
/// The input text is scanned into lexemes and then evaluated into lexical tokens.
/// The tokens are associated with ranges (ranges within the input text).
pub struct Lexer<'a> {
    chars: Chars<'a>,
    index: usize,
    lookahead: Vec<char>,
}

impl<'a> Lexer<'a> {
    /// Makes a new Lexer with the given input text.
    pub fn new(input: &'a str) -> Self {
        Self {
            chars: input.chars(),
            index: 0,
            lookahead: Vec::new(),
        }
    }

    /// Returns the input text as a String.
    pub fn input(&self) -> String {
        self.chars.clone().collect()
    }

    // #TODO peek_char() (next+put_back)

    fn next_char(&mut self) -> Option<char> {
        self.index += 1;

        if let Some(ch) = self.lookahead.pop() {
            return Some(ch);
        }

        self.chars.next()
    }

    fn put_back_char(&mut self, ch: char) {
        self.lookahead.push(ch);
        self.index -= 1;
    }

    // #TODO add unit tests
    // #TODO try to reuse in more lexers!
    fn scan_lexeme(&mut self) -> Ranged<String> {
        let mut text = String::new();

        let start = self.index;

        let mut char;

        loop {
            char = self.next_char();

            let Some(ch) = char  else {
                break;
            };

            // #TODO maybe whitespace does not need put_back, but need to adjust range.
            if is_whitespace(ch) || is_delimiter(ch) || is_eol(ch) {
                self.put_back_char(ch);
                break;
            }

            text.push(ch);
        }

        let range = start..self.index;

        Ranged(text, range)
    }

    fn lex_comment(&mut self) -> Result<Ranged<Token>, Ranged<LexicalError>> {
        let mut text = String::from(";");

        let start = self.index - 1; // adjust for leading `;`

        let mut char;

        loop {
            char = self.next_char();

            let Some(ch) = char  else {
                break;
            };

            if ch == '\n' {
                break;
            }

            text.push(ch);
        }

        let range = Range {
            start,
            end: self.index - 1, // adjust for the trailing '\n'
        };

        Ok(Ranged(Token::Comment(text), range))
    }

    // #TODO the lexer should keep the Number token as String.
    fn lex_number(&mut self) -> Result<Ranged<Token>, Ranged<LexicalError>> {
        let Ranged(lexeme, range) = self.scan_lexeme();

        // Ignore `_`, it is considered a number separator.
        // #Insight do _not_ consider `,` as number separator, bad idea!
        let mut lexeme = lexeme.replace('_', "");

        // #TODO support radix-8 -> no, leave for arbitrary radix.
        // #TODO support arbitrary radix https://github.com/golang/go/issues/28256
        let mut radix = 10;

        if lexeme.starts_with("0x") {
            lexeme = lexeme.replace("0x", "");
            radix = 16
        } else if lexeme.starts_with("0b") {
            lexeme = lexeme.replace("0b", "");
            radix = 2
        }

        // #TODO more detailed Number error!
        // #TODO error handling not enough, we need to add context, check error_stack

        let n = i64::from_str_radix(&lexeme, radix)
            .map_err(|err| Ranged(LexicalError::NumberError(err), range.clone()))?;

        Ok(Ranged(Token::Number(n), range))
    }

    fn lex_symbol(&mut self) -> Result<Ranged<Token>, Ranged<LexicalError>> {
        let Ranged(lexeme, range) = self.scan_lexeme();

        let token = match lexeme.as_str() {
            "if" => Token::If,
            "using" => Token::Using,
            _ => Token::Symbol(lexeme),
        };

        Ok(Ranged(token, range))
    }

    // #TODO support multi-line strings
    fn lex_string(&mut self) -> Result<Ranged<Token>, Ranged<LexicalError>> {
        let mut text = String::new();

        let start = self.index - 1; // adjust for leading '"'

        let mut char;

        loop {
            char = self.next_char();

            let Some(ch) = char  else {
                break;
            };

            if ch == '"' {
                break;
            }

            text.push(ch);
        }

        let mut range = start..self.index;

        if char != Some('"') {
            range.end -= 1;
            return Err(Ranged(LexicalError::UnterminatedStringError, range));
        }

        Ok(Ranged(Token::String(text), range))
    }

    fn lex_annotation(&mut self) -> Result<Ranged<Token>, Ranged<LexicalError>> {
        let mut text = String::new();

        let start = self.index - 1; // adjust for leading '#'

        let mut nesting = 0;

        // #TODO only allow one level of nesting?

        let mut char;

        loop {
            char = self.next_char();

            let Some(ch) = char  else {
                break;
            };

            if ch == '(' {
                nesting += 1;
            } else if ch == ')' {
                nesting -= 1;
            } else if nesting == 0 && (is_whitespace(ch) || is_eol(ch)) {
                // #TODO maybe whitespace does not need put_back, but need to adjust range.
                self.put_back_char(ch);
                break;
            }

            text.push(ch);
        }

        let range = start..self.index;

        if nesting != 0 {
            return Err(Ranged(LexicalError::UnterminatedAnnotationError, range));
        }

        Ok(Ranged(Token::Annotation(text), range))
    }

    // #TODO consider passing into array of chars or something more general.
    pub fn lex(&mut self) -> Result<Vec<Ranged<Token>>, Ranged<LexicalError>> {
        let mut tokens: Vec<Ranged<Token>> = Vec::new();

        let mut char;

        loop {
            char = self.next_char();

            let Some(ch) = char  else {
                break;
            };

            match ch {
                '(' => {
                    tokens.push(Ranged(Token::LeftParen, self.index..self.index));
                }
                ')' => {
                    tokens.push(Ranged(Token::RightParen, self.index..self.index));
                }
                ';' => {
                    tokens.push(self.lex_comment()?);
                }
                '"' => {
                    tokens.push(self.lex_string()?);
                }
                '-' => {
                    // #TODO support for `--` line comments!

                    let char1 = self.next_char();

                    let Some(ch1) = char1 else {
                        let range = (self.index-1)..(self.index-1);
                        return Err(Ranged(LexicalError::UnexpectedEol, range));
                    };

                    if ch1.is_numeric() {
                        // Negative number
                        self.put_back_char(ch1);
                        self.put_back_char(ch);
                        tokens.push(self.lex_number()?);
                    } else {
                        // #TODO lint warning for this!
                        // Symbol
                        self.put_back_char(ch1);
                        self.put_back_char(ch);
                        tokens.push(self.lex_symbol()?);
                    }
                }
                '#' => {
                    tokens.push(self.lex_annotation()?);
                }
                _ if is_whitespace(ch) => {
                    // Consume whitespace
                }
                _ if ch.is_numeric() => {
                    self.put_back_char(ch);
                    tokens.push(self.lex_number()?);
                }
                _ => {
                    self.put_back_char(ch);
                    tokens.push(self.lex_symbol()?);
                }
            }
        }

        Ok(tokens)
    }
}

#[cfg(test)]
mod tests {
    use std::num::IntErrorKind;

    use crate::{
        lexer::{Lexer, LexicalError, Token},
        range::Ranged,
        util::format::format_pretty_error,
    };

    fn read_input(filename: &str) -> String {
        std::fs::read_to_string(format!("tests/fixtures/{filename}")).unwrap()
    }

    #[test]
    fn lex_handles_an_empty_string() {
        let input = read_input("empty.tan");
        let tokens = Lexer::new(&input).lex();

        let tokens = tokens.unwrap();

        assert_eq!(tokens.len(), 0);
    }

    #[test]
    fn lex_returns_tokens() {
        let input = "((+ 1   25 399)  )";
        let tokens = Lexer::new(input).lex();

        let tokens = tokens.unwrap();

        dbg!(&tokens);

        assert_eq!(tokens.len(), 8);
        assert!(matches!(tokens[0].as_ref(), Token::LeftParen));
        assert!(matches!(tokens[2].as_ref(), Token::Symbol(x) if x == "+"));
        assert_eq!(tokens[2].1.start, 2);
        assert!(matches!(tokens[3].as_ref(), Token::Number(..)));
        assert_eq!(tokens[3].1.start, 4);
        // #TODO add more assertions.
    }

    #[test]
    fn lex_parses_comments() {
        let input = "; This is a comment\n;; Another comment\n(write \"hello\")";
        let tokens = Lexer::new(input).lex();

        let tokens = tokens.unwrap();

        assert!(matches!(tokens[0].as_ref(), Token::Comment(x) if x == "; This is a comment"));
        assert!(matches!(tokens[1].as_ref(), Token::Comment(x) if x == ";; Another comment"));
    }

    #[test]
    fn lex_parses_annotations() {
        let input = "
            #deprecated
            #(inline 'always)
            (let #public (add x y) (+ x y))
        ";
        let tokens = Lexer::new(input).lex();

        let tokens = tokens.unwrap();

        assert!(matches!(tokens[0].as_ref(), Token::Annotation(x) if x == "deprecated"));
        assert!(matches!(tokens[1].as_ref(), Token::Annotation(x) if x == "(inline 'always)"));
    }

    #[test]
    fn lex_handles_number_separators() {
        let input = "(+ 1 3_000)";
        let tokens = Lexer::new(input).lex().unwrap();

        // dbg!(&tokens);

        assert!(matches!(tokens[3].as_ref(), Token::Number(n) if n == &3000));
    }

    #[test]
    fn lex_handles_signed_numbers() {
        let input = read_input("signed_number.tan");
        let tokens = Lexer::new(&input).lex();

        let tokens = tokens.unwrap();

        assert!(matches!(tokens[3].as_ref(), Token::Number(n) if n == &-123));
        assert!(matches!(tokens[7].as_ref(), Token::Symbol(s) if s == "-variable"));
    }

    #[test]
    fn lex_reports_unexpected_eol() {
        let input = "(let a -";
        let result = Lexer::new(input).lex();

        assert!(result.is_err());

        let err = result.unwrap_err();

        eprintln!("{}", format_pretty_error(&err, input, None));

        assert!(matches!(err.0, LexicalError::UnexpectedEol));
    }

    #[test]
    fn lex_handles_numbers_with_radix() {
        let input = "(let a 0xfe)";
        let tokens = Lexer::new(input).lex().unwrap();

        assert!(matches!(tokens[3].as_ref(), Token::Number(n) if n == &254));

        let input = "(let a 0b1010)";
        let tokens = Lexer::new(input).lex().unwrap();

        assert!(matches!(tokens[3].as_ref(), Token::Number(n) if n == &10));

        let input = "(let a 0b00000)";
        let tokens = Lexer::new(input).lex().unwrap();

        assert!(matches!(tokens[3].as_ref(), Token::Number(n) if n == &0));
    }

    #[test]
    fn lex_reports_number_errors() {
        let input = "(+ 1 3$%99)";
        let result = Lexer::new(input).lex();

        assert!(result.is_err());

        let err = result.unwrap_err();

        assert!(matches!(err.0, LexicalError::NumberError(..)));

        eprintln!("{}", format_pretty_error(&err, input, None));

        if let Ranged(LexicalError::NumberError(pie), range) = err {
            assert_eq!(pie.kind(), &IntErrorKind::InvalidDigit);
            assert_eq!(range.start, 5);
            assert_eq!(range.end, 10);
        }
    }

    #[test]
    fn lex_reports_unterminated_strings() {
        let input = r##"(write "Hello)"##;
        let tokens = Lexer::new(input).lex();

        let result = tokens;

        assert!(result.is_err());

        let err = result.unwrap_err();

        assert!(matches!(err.0, LexicalError::UnterminatedStringError));

        eprintln!("{}", format_pretty_error(&err, input, None));

        assert_eq!(err.1.start, 7);
        assert_eq!(err.1.end, 14);
    }

    #[test]
    fn lex_reports_unterminated_annotations() {
        let input = r##"
        #deprecated
        #(inline true
        (write "Hello)
        "##;
        let tokens = Lexer::new(input).lex();

        let result = tokens;

        assert!(result.is_err());

        let err = result.unwrap_err();

        assert!(matches!(err.0, LexicalError::UnterminatedAnnotationError));

        eprintln!("{}", format_pretty_error(&err, input, None));

        assert_eq!(err.1.start, 29);
    }
}
