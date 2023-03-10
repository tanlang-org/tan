// #Insight
// What we call reserved_symbol is maybe similar to lisp's 'special form'?

// #TODO consider using `name` instead of `symbol`?

use std::fmt;

/// Returns true if `sym` is reserved.
pub fn is_reserved_symbol(sym: &str) -> bool {
    // #TODO think about `Func`.
    matches!(
        sym,
        "do" | "ann"
            | "let"
            | "if"
            | "for"
            | "for_each"
            | "eval"
            | "quot"
            | "use" // #TODO consider `using`
            | "Char"
            | "Func"
            | "Macro"
            | "List"
            | "Array"
            | "Dict"
    )
}

/// The`Break` is thrown when a pass processor cannot synchronize
/// to continue processing to detect more errors. Processing is stopped immediately.
/// Typically signals non-recoverable errors or end of input.
#[derive(Debug)]
pub struct Break {}

impl std::error::Error for Break {}

impl fmt::Display for Break {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Break")
    }
}
