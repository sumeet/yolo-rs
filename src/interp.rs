use crate::parser::{Expr, Word};
use std::collections::HashMap;
use std::collections::hash_map::RandomState;
use std::str::{from_utf8, from_utf8_unchecked};

pub type EvalResult = Result<Expr, Box<dyn std::error::Error>>;

pub struct Interpreter {
    storage: HashMap<Word, Expr>,
}

fn grab_a_word(expr: &mut Expr) -> Result<Word, Box<dyn std::error::Error>> {
    expr.as_mut_list()?.pop_front().ok_or("tried to pop a word from an empty list")?.into_word()
}

impl Interpreter {
    pub fn new() -> Self {
        Self { storage: HashMap::new()}
    }

    pub fn eval(&mut self, mut expr: Expr) -> EvalResult {
        let w = grab_a_word(&mut expr)?;
        match w.as_slice() {
            b"print_ascii" => builtins::print_ascii(self, expr),
            b"." => builtins::eval_all(self, expr),
            _ => return Err(format!("builtin {} not found", from_utf8(&w).unwrap()).into()),
        }
    }
}

mod builtins {
    use super::*;

    pub fn eval_all(interp: &mut Interpreter, expr: Expr) -> EvalResult {
        Ok(Expr::List(expr.into_list()?.into_iter().map(|expr| {
            interp.eval(expr)
        }).collect::<Result<_, _>>()?))
    }

    pub fn print_ascii(_: &mut Interpreter, mut expr: Expr) -> EvalResult {
        let w = grab_a_word(&mut expr)?;
        println!("{}", from_utf8(&w)?);
        Ok(Expr::empty_list())
    }
}