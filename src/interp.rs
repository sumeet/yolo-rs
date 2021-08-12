use crate::parser::{Expr, Word, List};
use std::collections::HashMap;
use std::str::{from_utf8};

pub type EvalResult = Result<Expr, Box<dyn std::error::Error>>;

pub struct Interpreter {
    storage: HashMap<Word, Expr>,
}

fn grab_an_expr(expr: &mut Expr) -> EvalResult {
    Ok(expr.as_mut_list()?.pop_front().ok_or("tried to pop an expr from an empty list")?)
}

fn grab_a_word(expr: &mut Expr) -> Result<Word, Box<dyn std::error::Error>> {
    grab_an_expr(expr)?.into_word()
}

fn grab_a_list(expr: &mut Expr) -> Result<List, Box<dyn std::error::Error>> {
    grab_an_expr(expr)?.into_list()
}

impl Interpreter {
    pub fn new() -> Self {
        Self { storage: HashMap::new()}
    }

    pub fn eval(&mut self, mut expr: Expr) -> EvalResult {
        let w = grab_a_word(&mut expr)?;
        match w.as_slice() {
            b"define" => builtins::define(self, expr),
            b"@" => builtins::dedef(self, expr),
            b"print-ascii" => builtins::print_ascii(self, expr),
            // TODO: should .block return the last instead of returning all?
            b".block" => builtins::eval_all(self, expr),
            b"." => builtins::apply(self, expr),
            _ => return Err(format!("builtin {} not found", from_utf8(&w).unwrap()).into()),
        }
    }
}

mod builtins {
    use super::*;

    // returns the word defined
    pub fn define(interp: &mut Interpreter, mut expr: Expr) -> EvalResult {
        let name = grab_a_word(&mut expr)?;
        let value = grab_an_expr(&mut expr)?;
        interp.storage.insert(name.clone(), value);
        Ok(Expr::Word(name))
    }

    pub fn dedef(interp: &mut Interpreter, mut expr: Expr) -> EvalResult {
        let name = grab_a_word(&mut expr)?;
        let found = interp.storage.get(&name)
            .ok_or_else(|| format!("def {:?} not found", from_utf8(&name).unwrap()))?;
        Ok(found.clone())
    }

    pub fn eval_all(interp: &mut Interpreter, expr: Expr) -> EvalResult {
        Ok(Expr::List(expr.into_list()?.into_iter().map(|expr| {
            interp.eval(expr)
        }).collect::<Result<_, _>>()?))
    }

    pub fn apply(interp: &mut Interpreter, mut expr: Expr) -> EvalResult {
        let func_to_apply = grab_a_word(&mut expr)?;
        let mut args = eval_all(interp, expr)?.into_list()?;
        args.push_front(Expr::Word(func_to_apply));
        interp.eval(Expr::List(args))
    }

    pub fn print_ascii(_: &mut Interpreter, mut expr: Expr) -> EvalResult {
        let w = grab_a_word(&mut expr)?;
        println!("{}", from_utf8(&w)?);
        Ok(Expr::empty_list())
    }
}