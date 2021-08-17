use crate::parser::{Expr, ExprRef, Word, WordRef};
use anyhow::{anyhow, bail};
use num_bigint::BigUint;
use std::collections::HashMap;
use std::str::{from_utf8, FromStr};

pub struct Interpreter {
    storage: HashMap<Word, Expr>,
    stack: Vec<Expr>,
}

fn grab_an_expr(exprs: &mut impl Iterator<Item = Expr>) -> anyhow::Result<Expr> {
    Ok(exprs
        .next()
        .ok_or_else(|| anyhow!("tried to pop an expr from an empty list"))?)
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            storage: HashMap::new(),
            stack: vec![],
        }
    }

    pub fn eval(&mut self, mut exprs: impl Iterator<Item = Expr>) -> anyhow::Result<()> {
        let grabbed = grab_an_expr(&mut exprs)?;
        let name = grabbed.into_word()?;
        self.stack.extend(exprs);

        let definition = self
            .storage
            .get(&name)
            .map(|expr| expr.to_owned())
            .ok_or_else(|| anyhow!("{} not found", from_utf8(&name).unwrap()));

        if let Ok(definition) = definition {
            let list = definition.into_list()?;
            return Ok(self.eval(list.into_iter())?);
        }
        Ok(self.call_builtin(&name)?)
    }

    pub fn pop_expr(&mut self) -> anyhow::Result<Expr> {
        self.stack.pop().ok_or(anyhow!("stack was empty"))
    }

    pub fn peek_expr(&self) -> anyhow::Result<ExprRef> {
        self.stack
            .last()
            .map(|expr| expr.as_ref())
            .ok_or(anyhow!("stack was empty"))
    }

    pub fn call_builtin(&mut self, name: WordRef<'b>) -> anyhow::Result<()> {
        match name {
            b".error" => builtins::error(self),
            b"." => builtins::exec_all(self),
            b".?" => builtins::cond(self),
            b".drop" => builtins::drop(self),
            b".dup" => builtins::dup(self),
            b".define" => builtins::define(self),
            b".peek-len" => builtins::length(self),
            b".@" => builtins::dedef(self),
            b".+-u" => builtins::plus_unsigned(self),
            b".>-u" => builtins::gt_unsigned(self),
            b".<-u" => builtins::lt_unsigned(self),
            b".empty-word" => builtins::empty_word(self),
            b".write" => builtins::write(self),

            // expr stuff
            b".append" => builtins::append(self),

            // temp functions until i get bootstrapped
            b".temp.u" => {
                let w = self.stack.pop();
                let w = w.ok_or_else(|| anyhow!("expected a word"))?;
                let w = w.into_word()?;
                self.stack.push(Expr::Word(
                    BigUint::from_str(from_utf8(&w)?)?.to_bytes_le().to_vec(),
                ));
                Ok(())
            }
            b".temp.print-u" => {
                let w = self.stack.pop();
                let w = w.ok_or_else(|| anyhow!("expected a word"))?;
                let w = w.into_word()?;
                println!("{}", BigUint::from_bytes_le(&w));
                Ok(())
            }
            _ => Err(anyhow!("builtin {} not found", from_utf8(name).unwrap())),
        }
    }
}

mod builtins {
    use super::*;
    use num_bigint::BigUint;
    use std::io::Write;

    pub fn define(interp: &mut Interpreter) -> anyhow::Result<()> {
        let definition = interp.pop_expr()?;
        let w = interp.pop_expr()?.into_word()?;
        interp.storage.insert(w, definition);
        Ok(())
    }

    pub fn dedef(interp: &mut Interpreter) -> anyhow::Result<()> {
        let w = interp.pop_expr()?.into_word()?;
        interp
            .storage
            .get(&w)
            .ok_or_else(|| anyhow!("couldn't find {} in storage", from_utf8(&w).unwrap()))?;
        Ok(())
    }

    pub fn error(interp: &mut Interpreter) -> anyhow::Result<()> {
        bail!("error raised from VM: {:?}", interp.pop_expr()?)
    }

    // TODO: this can probably be written in the language if we had a way
    //       to iterate through expressions and eval a single one on
    //       the stack
    pub fn exec_all(interp: &mut Interpreter) -> anyhow::Result<()> {
        let list = interp.pop_expr()?.into_list()?;
        for el in list {
            interp.eval(el.into_list()?.into_iter())?;
        }
        Ok(())
    }

    pub fn eval(interp: &mut Interpreter) -> anyhow::Result<()> {
        let el = interp.pop_expr()?.into_list()?;
        interp.eval(el.into_iter())?;
        Ok(())
    }

    pub fn drop(interp: &mut Interpreter) -> anyhow::Result<()> {
        interp.pop_expr()?;
        Ok(())
    }

    // pub fn swap(interp: &mut Interpreter) -> anyhow::Result<()> {
    //     let n =
    // }

    // this should probably take an operand
    pub fn dup(interp: &mut Interpreter) -> anyhow::Result<()> {
        interp.stack.push(interp.peek_expr()?.to_owned());
        Ok(())
    }

    // doesn't consume the last element of the stack, this peeks
    pub fn length(interp: &mut Interpreter) -> anyhow::Result<()> {
        let el = interp.peek_expr()?;
        let len = match el {
            ExprRef::Word(w) => w.len(),
            ExprRef::List(l) => l.len(),
        };
        interp.stack.push(Expr::Word(len.to_ne_bytes().to_vec()));
        Ok(())
    }

    pub fn cond(interp: &mut Interpreter) -> anyhow::Result<()> {
        let bool = interp.pop_expr()?.into_word()?;
        if is_truthy(bool.as_ref()) {
            self::eval(interp)?;
        } else {
            self::drop(interp)?;
        }
        Ok(())
    }

    pub fn empty_word(interp: &mut Interpreter) -> anyhow::Result<()> {
        interp.stack.push(Expr::Word(vec![]));
        Ok(())
    }

    pub fn write(interp: &mut Interpreter) -> anyhow::Result<()> {
        let w = interp.pop_expr()?.into_word()?;
        let mut out = std::io::stdout();
        out.write_all(&w)?;
        out.flush()?;
        Ok(())
    }

    pub fn plus_unsigned(interp: &mut Interpreter) -> anyhow::Result<()> {
        let rhs = pop_uint(interp)?;
        let lhs = pop_uint(interp)?;
        interp.stack.push(Expr::Word((lhs + rhs).to_bytes_le()));
        Ok(())
    }

    pub fn lt_unsigned(interp: &mut Interpreter) -> anyhow::Result<()> {
        let rhs = pop_uint(interp)?;
        let lhs = pop_uint(interp)?;
        interp.stack.push(Expr::Word(vec![(lhs < rhs) as _]));
        Ok(())
    }

    pub fn gt_unsigned(interp: &mut Interpreter) -> anyhow::Result<()> {
        let rhs = pop_uint(interp)?;
        let lhs = pop_uint(interp)?;
        interp.stack.push(Expr::Word(vec![(lhs > rhs) as _]));
        Ok(())
    }

    pub fn append(interp: &mut Interpreter) -> anyhow::Result<()> {
        let to_append = interp.pop_expr()?;
        let mut list = interp.pop_expr()?.into_list()?;
        list.push(to_append);
        interp.stack.push(Expr::List(list));
        Ok(())
    }

    fn pop_uint(interp: &mut Interpreter) -> anyhow::Result<BigUint> {
        Ok(uint(interp.pop_expr()?.into_word()?))
    }
}

fn is_truthy(w: WordRef) -> bool {
    w.iter().any(|&b| b != 0)
}

fn uint(w: Word) -> BigUint {
    BigUint::from_bytes_le(&w)
}
