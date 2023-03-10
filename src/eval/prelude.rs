use std::rc::Rc;

use crate::{
    ann::Ann,
    expr::Expr,
    ops::{
        arithmetic::{add_float, add_int, mul, sub},
        eq::{eq, gt, lt},
        io::{file_read_as_string, write, writeln},
        process::exit,
    },
};

use super::env::Env;

// #TODO use typeclasses (== traits) for overloading
// #TODO make Env::top() -> in fact it's bottom (of the stack)
// #TODO alternative Env::prelude()

pub fn setup_prelude(env: Env) -> Env {
    let mut env = env;

    // num

    // #TODO forget the mangling, implement with a dispatcher function, multi-function.
    env.insert(
        "+",
        Ann::with_type(Expr::ForeignFunc(Rc::new(add_int)), Expr::symbol("Int")),
    );
    env.insert(
        "+$$Int$$Int",
        Ann::with_type(Expr::ForeignFunc(Rc::new(add_int)), Expr::symbol("Int")),
    );
    env.insert(
        "+$$Float$$Float",
        // #TODO add the proper type: (Func Float Float Float)
        // #TODO even better: (Func (Many Float) Float)
        Ann::with_type(Expr::ForeignFunc(Rc::new(add_float)), Expr::symbol("Float")),
    );
    env.insert("-", Expr::ForeignFunc(Rc::new(sub)));
    env.insert("*", Expr::ForeignFunc(Rc::new(mul)));

    // eq

    env.insert("=", Expr::ForeignFunc(Rc::new(eq)));
    env.insert(">", Expr::ForeignFunc(Rc::new(gt)));
    env.insert("<", Expr::ForeignFunc(Rc::new(lt)));

    // io

    env.insert("write", Expr::ForeignFunc(Rc::new(write)));
    env.insert("write$$String", Expr::ForeignFunc(Rc::new(write)));
    env.insert("writeln", Expr::ForeignFunc(Rc::new(writeln)));
    env.insert("writeln$$String", Expr::ForeignFunc(Rc::new(writeln)));
    env.insert(
        "File:read_as_string",
        Expr::ForeignFunc(Rc::new(file_read_as_string)),
    );
    env.insert(
        "File:read_as_string$$String",
        Expr::ForeignFunc(Rc::new(file_read_as_string)),
    );

    // process
    env.insert("exit", Expr::ForeignFunc(Rc::new(exit)));
    env.insert("exit$$", Expr::ForeignFunc(Rc::new(exit)));

    env
}
