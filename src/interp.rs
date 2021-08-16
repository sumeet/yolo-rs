use crate::parser::{Expr, Word, ExprRef, WordRef};
use std::collections::HashMap;
use std::str::{from_utf8, FromStr};
use std::iter::once;
use anyhow::anyhow;

#[derive(Debug)]
pub enum EvalOutput<'a> {
    Ref(ExprRef<'a>),
    Owned(Expr),
}

pub type EvalResult<'a> = anyhow::Result<EvalOutput<'a>>;

pub struct Interpreter {
    storage: HashMap<Word, Expr>,
}

fn grab_an_expr(exprs: &'b mut impl Iterator<Item = ExprRef<'a>>) -> anyhow::Result<ExprRef<'a>> {
    Ok(exprs.next().ok_or_else(|| anyhow!("tried to pop an expr from an empty list"))?)
}

impl Interpreter {
    pub fn new() -> Self {
        Self { storage: HashMap::new() }
    }

    pub fn eval(&'a mut self, mut exprs: impl Iterator<Item = ExprRef<'a>> + 'a)  -> anyhow::Result<Expr> {
        let grabbed = grab_an_expr(&mut exprs)?;
        let name = grabbed.as_word()?.to_owned();
        Ok(match self.call_builtin(&name, exprs)? {
           EvalOutput::Ref(r) => r.to_owned(),
           EvalOutput::Owned(owned) => owned,
        })
    }

    pub fn call_builtin(&'a mut self, name: WordRef<'b>, mut exprs: impl Iterator<Item = ExprRef<'a>> + 'a) -> EvalResult<'a> {
        match name {
            b".define" => builtins::define(self, exprs),
            b".@" => builtins::dedef(self, exprs),
            b".+-u" => builtins::plus_unsigned(self, exprs),
            b".>-u" => builtins::gt_unsigned(self, exprs),
            b".<-u" => builtins::lt_unsigned(self, exprs),
            b".print-ascii" => builtins::print_ascii(self, exprs),
            b".exec-all" => builtins::exec_all(self, Box::new(exprs)),
            b".chain" => builtins::chain(self, Box::new(exprs)),
            b".while" => builtins::r#while(self, Box::new(exprs)),

            // expr stuff
            b".append" => builtins::append(self, exprs),

            // temp functions until i get bootstrapped
            b".temp.u64" => {
                let w = exprs.next();
                let w = w.ok_or_else(|| anyhow!("expected a word"))?;
                let w = w.as_word()?;
                Ok(EvalOutput::Owned(Expr::Word(u64::from_str(from_utf8(&w)?)?.to_ne_bytes().to_vec())))
            }
            b".temp.print-u64" => {
                let w = exprs.next();
                let expr_ref = w.ok_or_else(|| anyhow!("expected a word"))?;
                let w = expr_ref.as_word()?;
                println!("{}", u64::from_ne_bytes(w.try_into()?));
                Ok(EvalOutput::Ref(expr_ref))
            }
            _ => Err(anyhow!("builtin {} not found", from_utf8(name).unwrap())),
        }
    }
}

mod builtins {
    use super::*;
    use itertools::Itertools;
    use num_bigint::BigUint;

    // returns the word defined
    pub fn define(interp: &'a mut Interpreter, exprs: impl Iterator<Item = ExprRef<'a>>) -> EvalResult<'a> {
        // TODO: this can be done without allocations...
        let exprs = exprs.collect_vec();
        if let [ExprRef::Word(name), to] = exprs.as_slice() {
            interp.storage.insert(name.to_vec(), to.to_owned());
            Ok(EvalOutput::Ref(ExprRef::Word(name)))
        } else {
            Err(anyhow!("invalid arguments for define: {:?}", exprs))
        }
    }

    pub fn dedef(interp: &'a mut Interpreter, exprs: impl Iterator<Item = ExprRef<'a>> + 'a) -> EvalResult<'a> {
        // TODO: this can be done without allocations...
        let exprs = exprs.collect_vec();
        if let [ExprRef::Word(name)] = exprs.as_slice() {
            let value = interp.storage.get(*name)
                .ok_or_else(|| anyhow!("name {:?} not found", from_utf8(name).unwrap()))?;
            Ok(EvalOutput::Ref(value.as_ref()))
        } else {
            Err(anyhow!("invalid arguments for dedef: {:?}", exprs))
        }
    }

    // this must take a Box because this func is mutually recursive with interp.eval(), otherwise
    // the type for the iterator couldn't be computed
    pub fn exec_all(interp: &'a mut Interpreter, mut exprs: Box<dyn Iterator<Item = ExprRef<'a>> + 'a>) -> EvalResult<'a> {
        let first = exprs.next().ok_or_else(|| anyhow!("tried to exec-all empty expr list"))?;
        let mut res = interp.eval(first.as_list()?.iter().map(|expr| expr.as_ref()))?;
         for expr in exprs {
             res = interp.eval(expr.as_list()?.iter().map(|expr| expr.as_ref()))?;
         }
        Ok(EvalOutput::Owned(res))
    }

    // TODO: should the arg stack just be global?
    // this must take a Box because this func is mutually recursive with interp.eval(), otherwise
    // the type for the iterator couldn't be computed
    pub fn chain(interp: &'a mut Interpreter, mut exprs: Box<dyn Iterator<Item = ExprRef<'a>> + 'a>) -> EvalResult<'a> {
        let first = exprs.next().ok_or_else(|| anyhow!("tried to chain an empty list"))?;
        let mut args = interp.eval(first.as_list()?.iter().map(|expr| expr.as_ref()))?;
        for next_to_evaluate in exprs {
            let next_to_evaluate = next_to_evaluate.as_list()?.iter().map(|expr| expr.as_ref());
            args = interp.eval(next_to_evaluate.chain(once(args.as_ref())))?;
        }
        Ok(EvalOutput::Owned(args))
    }

    pub fn print_ascii(_: &'a mut Interpreter, exprs: impl Iterator<Item = ExprRef<'a>>) -> EvalResult<'a> {
        // TODO: this can be done without allocations...
        let exprs = exprs.collect_vec();
        if let [ExprRef::Word(word)] = exprs.as_slice() {
            println!("{}", from_utf8(word)?);
            Ok(EvalOutput::Ref(ExprRef::Word(word)))
        } else {
            Err(anyhow!("invalid arguments for print_ascii: {:?}", exprs))
        }
    }

    pub fn plus_unsigned(_interp: &'a mut Interpreter, exprs: impl Iterator<Item = ExprRef<'a>>) -> EvalResult<'a> {
        // TODO: this can be done without allocations...
        let exprs = exprs.collect_vec();
        if let [ExprRef::Word(lhs), ExprRef::Word(rhs)] = exprs.as_slice() {
            let sum = BigUint::from_bytes_le(lhs) + BigUint::from_bytes_le(rhs);
            Ok(EvalOutput::Owned(Expr::Word(sum.to_bytes_le())))
        } else {
            Err(anyhow!("invalid arguments for plus_unsigned: {:?}", exprs))
        }
    }

    pub fn lt_unsigned(_interp: &'a mut Interpreter, exprs: impl Iterator<Item = ExprRef<'a>>) -> EvalResult<'a> {
        // TODO: this can be done without allocations...
        let exprs = exprs.collect_vec();
        if let [ExprRef::Word(lhs), ExprRef::Word(rhs)] = exprs.as_slice() {
            let bool = (BigUint::from_bytes_le(lhs) < BigUint::from_bytes_le(rhs)) as u8;
            Ok(EvalOutput::Owned(Expr::Word(vec![bool])))
        } else {
            Err(anyhow!("invalid arguments for gt: {:?}", exprs))
        }
    }

    pub fn gt_unsigned(_interp: &'a mut Interpreter, exprs: impl Iterator<Item = ExprRef<'a>>) -> EvalResult<'a> {
        // TODO: this can be done without allocations...
        let exprs = exprs.collect_vec();
        if let [ExprRef::Word(lhs), ExprRef::Word(rhs)] = exprs.as_slice() {
            let bool = (BigUint::from_bytes_le(lhs) > BigUint::from_bytes_le(rhs)) as u8;
            Ok(EvalOutput::Owned(Expr::Word(vec![bool])))
        } else {
            Err(anyhow!("invalid arguments for gt: {:?}", exprs))
        }
    }

    pub fn r#while(interp: &'a mut Interpreter, exprs: Box<dyn Iterator<Item = ExprRef<'a>> + 'a>) -> EvalResult<'a> {
        // TODO: this can be done without allocations...
        let exprs = exprs.collect_vec();
        if let [ExprRef::List(cond), ExprRef::List(block)] = exprs.as_slice() {
            let should_continue = |interp: &mut Interpreter| -> anyhow::Result<_> {
                let exprs_to_eval = cond.iter().map(|expr| expr.as_ref());
                let result = interp.eval(exprs_to_eval)?;
                let result_ref = result.as_ref();
                let b = is_truthy(result_ref.as_word()?);
                Ok((result, b))
            };
            let (condition_result, b) = should_continue(interp)?;
            if !b {
                return Ok(EvalOutput::Owned(condition_result))
            }

            let compute_block = |interp: &mut Interpreter| -> anyhow::Result<_> {
                interp.eval(block.iter().map(|expr| expr.as_ref()))
            };
            let mut result = compute_block(interp)?;
            while should_continue(interp)?.1 {
                result = compute_block(interp)?;
            }
            Ok(EvalOutput::Owned(result))
        } else {
            Err(anyhow!("invalid arguments for gt: {:?}", exprs))
        }
    }

    pub fn append(_interp: &'a mut Interpreter, exprs: impl Iterator<Item = ExprRef<'a>>) -> EvalResult<'a> {
        // TODO: this can be done without allocations...
        let exprs = exprs.collect_vec();
        if let [ExprRef::List(list), expr] = exprs.as_slice() {
            let expr = expr.to_owned();
            let new_list = list.into_iter().chain(once(&expr));
            Ok(EvalOutput::Owned(Expr::List(new_list.map(|expr| expr.to_owned()).collect())))
        } else {
            Err(anyhow!("invalid arguments for append: {:?}", exprs))
        }
    }
}

fn is_truthy(w: WordRef) -> bool {
    w.iter().any(|&b| b != 0)
}