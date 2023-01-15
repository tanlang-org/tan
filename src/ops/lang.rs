use crate::{error::Error, eval::env::Env, expr::Expr};

pub fn ann(args: &[Expr], _env: &Env) -> Result<Expr, Error> {
    if args.len() != 1 {
        return Err(Error::invalid_arguments("`ann` requires one argument"));
    }

    // #TODO support multiple arguments.

    let _expr = args.first().unwrap();

    // #TODO aargh, no access to annotations!
            
    Ok(Expr::One)
}