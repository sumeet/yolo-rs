use crate::parser::{Expr, Word, ExprRef};
use std::collections::HashMap;
use std::str::{from_utf8};

#[derive(Debug)]
pub enum EvalOutput<'a> {
    Ref(ExprRef<'a>),
    Owned(Expr),
}

pub type EvalResult<'a> = Result<EvalOutput<'a>, Box<dyn std::error::Error>>;

pub struct Interpreter {
    storage: HashMap<Word, Expr>,
}

fn grab_an_expr(exprs: &'a mut impl Iterator<Item = &'a Expr>) -> Result<ExprRef<'a>, Box<dyn std::error::Error>> {
    Ok(exprs.next().ok_or("tried to pop an expr from an empty list")?.as_ref())
}

impl Interpreter {
    pub fn new() -> Self {
        Self { storage: HashMap::new() }
    }

    // TODO: this could take an iterator of exprs
    pub fn eval(&'a mut self, expr: &'a Expr) -> Result<Expr, Box<dyn std::error::Error>> {
        let expr_ref = expr.as_ref();
        let expr_list = expr_ref.as_list()?;
        let mut exprs = expr_list.iter();

        let grabbed = grab_an_expr(&mut exprs)?;
        let name = grabbed.as_word()?;
        let rest = &expr_list[1..];
        Ok(match self.call_builtin(name, rest)? {
            EvalOutput::Ref(r) => r.to_owned(),
            EvalOutput::Owned(owned) => owned,
        })
    }

    pub fn call_builtin(&'a mut self, name: &'a Word, exprs: &'a [Expr]) -> EvalResult<'a> {
        match name.as_slice() {
            b".define" => builtins::define(self, exprs),
            b".@" => builtins::dedef(self, exprs),
            b".print-ascii" => builtins::print_ascii(self, exprs),
            b".exec-all" => builtins::exec_all(self, exprs),
            b".call" => builtins::call(self, exprs),
            _ => return Err(format!("builtin {} not found", from_utf8(name).unwrap()).into()),
        }
    }
}

mod builtins {
    use super::*;
    use core::slice::SlicePattern;

    // returns the word defined
    pub fn define(interp: &'a mut Interpreter, exprs: &'a [Expr]) -> EvalResult<'a> {
        if let [Expr::Word(name), to] = exprs.as_slice() {
            interp.storage.insert(name.clone(), to.clone());
            Ok(EvalOutput::Ref(ExprRef::Word(name)))
        } else {
            Err(format!("invalid arguments for define: {:?}", exprs).into())
        }
    }

    pub fn dedef(interp: &'a mut Interpreter, exprs: &'a [Expr]) -> EvalResult<'a> {
        if let [Expr::Word(name)] = exprs.as_slice() {
            let value = interp.storage.get(name)
                .ok_or_else(|| format!("name {:?} not found", from_utf8(name).unwrap()))?;
            Ok(EvalOutput::Ref(value.as_ref()))
        } else {
            Err(format!("invalid arguments for dedef: {:?}", exprs).into())
        }
    }

    pub fn exec_all(interp: &'a mut Interpreter, exprs: &'a [Expr]) -> EvalResult<'a> {
        let (last, init) = exprs.split_last().ok_or("tried to exec-all empty expr list")?;
        for expr in init {
            interp.eval(expr)?;
        }
        Ok(EvalOutput::Owned(interp.eval(last)?))
    }

    pub fn call(interp: &'a mut Interpreter, exprs: &'a [Expr]) -> EvalResult<'a> {
        if let [Expr::Word(name), Expr::List(args)] = exprs.as_slice() {
            interp.call_builtin(name, args)
        } else {
            Err(format!("invalid arguments for call: {:?}", exprs).into())
        }
    }

    pub fn print_ascii(_: &'a mut Interpreter, exprs: &'a [Expr]) -> EvalResult<'a> {
        if let [Expr::Word(word)] = exprs.as_slice() {
            println!("{}", from_utf8(word)?);
            Ok(EvalOutput::Ref(ExprRef::Word(word)))
        } else {
            Err(format!("invalid arguments for print_ascii: {:?}", exprs).into())
        }
    }
}