use crate::parser::{Expr, Word, ExprRef, WordRef};
use std::collections::HashMap;
use std::str::{from_utf8};
use std::iter::once;

#[derive(Debug)]
pub enum EvalOutput<'a> {
    Ref(ExprRef<'a>),
    Owned(Expr),
}

pub type EvalResult<'a> = Result<EvalOutput<'a>, Box<dyn std::error::Error>>;

pub struct Interpreter {
    storage: HashMap<Word, Expr>,
}

fn grab_an_expr(exprs: &'b mut impl Iterator<Item = ExprRef<'a>>) -> Result<ExprRef<'a>, Box<dyn std::error::Error>> {
    Ok(exprs.next().ok_or("tried to pop an expr from an empty list")?)
}

impl Interpreter {
    pub fn new() -> Self {
        Self { storage: HashMap::new() }
    }

    pub fn eval(&'_ mut self, mut exprs: impl Iterator<Item = ExprRef<'a>> + 'a)  -> Result<Expr, Box<dyn std::error::Error>> {
        let grabbed = grab_an_expr(&mut exprs)?;
        let name = grabbed.as_word()?.to_owned();
        let x = self.call_builtin(name.as_ref(), exprs)?;
        Ok(match x {
           EvalOutput::Ref(r) => r.to_owned(),
           EvalOutput::Owned(owned) => owned,
        })
    }

    pub fn call_builtin(&'_ mut self, name: WordRef<'_>, exprs: impl Iterator<Item = ExprRef<'a>>) -> EvalResult<'_> {
        match name {
            // b".define" => builtins::define(self, exprs),
            // b".@" => builtins::dedef(self, exprs),
            // b".print-ascii" => builtins::print_ascii(self, exprs),
            // b".exec-all" => builtins::exec_all(self, exprs),
            // b".chain" => builtins::chain(self, exprs),
            _ => return Err(format!("builtin {} not found", from_utf8(name).unwrap()).into()),
        }
    }
}

mod builtins {
    use super::*;
    use itertools::Itertools;

    // returns the word defined
    pub fn define(interp: &'a mut Interpreter, exprs: impl Iterator<Item = ExprRef<'a>>) -> EvalResult<'a> {
        // TODO: this can be done without allocations...
        let exprs = exprs.collect_vec();
        if let [ExprRef::Word(name), to] = exprs.as_slice() {
            interp.storage.insert(name.to_vec(), to.to_owned());
            Ok(EvalOutput::Ref(ExprRef::Word(name)))
        } else {
            Err(format!("invalid arguments for define: {:?}", exprs).into())
        }
    }

    pub fn dedef(interp: &'a mut Interpreter, exprs: impl Iterator<Item = ExprRef<'a>>) -> EvalResult<'a> {
        // TODO: this can be done without allocations...
        let exprs = exprs.collect_vec();
        if let [ExprRef::Word(name)] = exprs.as_slice() {
            let value = interp.storage.get(*name)
                .ok_or_else(|| format!("name {:?} not found", from_utf8(name).unwrap()))?;
            Ok(EvalOutput::Ref(value.as_ref()))
        } else {
            Err(format!("invalid arguments for dedef: {:?}", exprs).into())
        }
    }

    pub fn exec_all(interp: &'a mut Interpreter, mut exprs: impl Iterator<Item = ExprRef<'a>>) -> EvalResult<'a> {
        let first = exprs.next().ok_or("tried to exec-all empty expr list")?;
        let mut res = interp.eval(first.as_list()?.iter().map(|expr| expr.as_ref()))?;
        for expr in exprs {
            res = interp.eval(expr.as_list()?.iter().map(|expr| expr.as_ref()))?;
        }
        Ok(EvalOutput::Owned(res))
    }

    // TODO: should the arg stack just be global?
    pub fn chain(interp: &'a mut Interpreter, mut exprs: impl Iterator<Item = ExprRef<'a>>) -> EvalResult<'a> {
        let first = exprs.next().ok_or("tried to chain an empty list")?;
        let mut args = interp.eval(first.as_list()?.iter().map(|expr| expr.as_ref()))?;

        for next_to_evaluate in exprs {
            let next_to_evaluate = next_to_evaluate.as_list()?.iter().map(|expr| expr.as_ref());
            args = interp.eval(next_to_evaluate.chain(once(args.as_ref())))?;
        }
        todo!()
    }

    pub fn print_ascii(_: &'a mut Interpreter, exprs: impl Iterator<Item = ExprRef<'a>>) -> EvalResult<'a> {
        // TODO: this can be done without allocations...
        let exprs = exprs.collect_vec();
        if let [ExprRef::Word(word)] = exprs.as_slice() {
            println!("{}", from_utf8(word)?);
            Ok(EvalOutput::Ref(ExprRef::Word(word)))
        } else {
            Err(format!("invalid arguments for print_ascii: {:?}", exprs).into())
        }
    }
}