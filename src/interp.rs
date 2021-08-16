use crate::parser::{Expr, Word, WordRef};
use std::collections::HashMap;
use std::str::{from_utf8, FromStr};
use anyhow::anyhow;
use num_bigint::BigUint;

pub struct Interpreter {
    storage: HashMap<Word, Expr>,
    stack: Vec<Expr>,
}

fn grab_an_expr(exprs: &mut impl Iterator<Item = Expr>) -> anyhow::Result<Expr> {
    Ok(exprs.next().ok_or_else(|| anyhow!("tried to pop an expr from an empty list"))?)
}

impl Interpreter {
    pub fn new() -> Self {
        Self { storage: HashMap::new(), stack: vec![] }
    }

    pub fn eval(&mut self, mut exprs: impl Iterator<Item = Expr>)  -> anyhow::Result<()> {
        let grabbed = grab_an_expr(&mut exprs)?;
        let name = grabbed.into_word()?;
        self.stack.extend(exprs);

        let definition = self.storage.get(&name)
            .map(|expr| expr.to_owned())
            .ok_or_else(|| anyhow!("{} not found", from_utf8(&name).unwrap()));

        if let Ok(definition) = definition {
            let list = definition.into_list()?;
            return Ok(self.eval(list.into_iter())?)
        }
        Ok(self.call_builtin(&name)?)
    }

    pub fn pop_expr(&mut self) -> anyhow::Result<Expr> {
        self.stack.pop().ok_or(anyhow!("stack was empty"))
    }

    pub fn call_builtin(&mut self, name: WordRef<'b>) -> anyhow::Result<()> {
        match name {
            b"." => builtins::exec_all(self),
            b".define" => builtins::define(self),
            b".@" => builtins::dedef(self),
            b".+-u" => builtins::plus_unsigned(self),
            b".>-u" => builtins::gt_unsigned(self),
            b".<-u" => builtins::lt_unsigned(self),
            b".write" => builtins::write(self),
            b".while" => builtins::r#while(self),

            // expr stuff
            b".append" => builtins::append(self),

            // temp functions until i get bootstrapped
            b".temp.u" => {
                let w = self.stack.pop();
                let w = w.ok_or_else(|| anyhow!("expected a word"))?;
                let w = w.into_word()?;
                self.stack.push(Expr::Word(BigUint::from_str(from_utf8(&w)?)?.to_bytes_le().to_vec()));
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
        interp.storage.get(&w).ok_or_else(|| anyhow!("couldn't find {} in storage", from_utf8(&w).unwrap()))?;
        Ok(())
    }

    pub fn exec_all(interp: &mut Interpreter) -> anyhow::Result<()> {
        let list = interp.pop_expr()?.into_list()?;
        for el in list {
            interp.eval(el.into_list()?.into_iter())?;
        }
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
        let rhs = interp.pop_expr()?.into_word()?;
        let lhs = interp.pop_expr()?.into_word()?;
        let sum = BigUint::from_bytes_le(&lhs) + BigUint::from_bytes_le(&rhs);
        interp.stack.push(Expr::Word(sum.to_bytes_le()));
        Ok(())
    }

    pub fn lt_unsigned(interp: &mut Interpreter) -> anyhow::Result<()> {
        let rhs = interp.pop_expr()?.into_word()?;
        let lhs = interp.pop_expr()?.into_word()?;
        let bool = (BigUint::from_bytes_le(&lhs) < BigUint::from_bytes_le(&rhs)) as u8;
        interp.stack.push(Expr::Word(vec![bool]));
        Ok(())
    }

    pub fn gt_unsigned(interp: &mut Interpreter) -> anyhow::Result<()> {
        let rhs = interp.pop_expr()?.into_word()?;
        let lhs = interp.pop_expr()?.into_word()?;
        let bool = (BigUint::from_bytes_le(&lhs) > BigUint::from_bytes_le(&rhs)) as u8;
        interp.stack.push(Expr::Word(vec![bool]));
        Ok(())
    }

    pub fn r#while(interp: &mut Interpreter) -> anyhow::Result<()> {
        let block = interp.pop_expr()?.into_list()?;
        // cond needs to push a boolean onto the stack
        let cond = interp.pop_expr()?.into_list()?;

        let should_continue = |interp: &mut Interpreter| -> anyhow::Result<_> {
            interp.eval(cond.clone().into_iter())?;
            Ok(is_truthy(interp.pop_expr()?.as_ref().as_word()?))
        };

        let compute_block = |interp: &mut Interpreter| -> anyhow::Result<_> {
            interp.eval(block.clone().into_iter())?;
            Ok(())
        };
        while should_continue(interp)? {
            compute_block(interp)?;
        }
        Ok(())
    }

    pub fn append(interp: &mut Interpreter) -> anyhow::Result<()> {
        let to_append = interp.pop_expr()?;
        let mut list = interp.pop_expr()?.into_list()?;
        list.push(to_append);
        interp.stack.push(Expr::List(list));
        Ok(())
    }
}

fn is_truthy(w: WordRef) -> bool {
    w.iter().any(|&b| b != 0)
}