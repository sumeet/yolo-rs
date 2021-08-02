#![feature(box_syntax)]

use crate::parser::Expr;
use std::collections::HashMap;

mod parser;

fn main() {
    let input = r#"
+ 1 1 3 4 5 (* 2 100 (* 100 100))
"#;

    println!("--- PARSED ---");
    let expr = parser::parse_exprs(&mut input.chars(), false);
    println!("{:?}", expr);

    println!();
    println!();
    
    let mut interpreter = BasicInterpreter::new();

    println!("--- EVAL ---");
    println!("{:?}", interpreter.eval(expr));
}

fn functions() -> HashMap<Value, Box<dyn Fn(Value) -> InterpResult>>  {
    let mut funcs : HashMap<Value, Box<dyn Fn(Value) -> InterpResult>> = HashMap::new();
    funcs.insert(Value::String("+".into()), box |val| {
        let mut acc = 0;
        for v in val.into_list()? {
            let n = v.as_num()?;
            acc += n;
        }
        Ok(Value::Num(acc))
    });
    funcs.insert(Value::String("*".into()), box |val| {
        let mut acc = 1;
        for v in val.into_list()? {
            let n = v.as_num()?;
            acc *= n;
        }
        Ok(Value::Num(acc))
    });
    funcs
}

type InterpResult = Result<Value, Box<dyn std::error::Error>>;

struct BasicInterpreter {
    functions: HashMap<Value, Box<dyn Fn(Value) -> InterpResult>>,
}

impl BasicInterpreter {
    fn new() -> Self {
        Self {
            functions: functions(),
        }
    }
    fn eval(&mut self, expr: Expr) -> InterpResult {
        match expr {
            Expr::Word(w) => {
                if let Ok(num) = w.parse() {
                    Ok(Value::Num(num))
                } else {
                    Ok(Value::String(w))
                }
            }
            Expr::List(val) => {
                let vals = val.into_iter().map(|v| self.eval(v)).collect::<Result<Vec<Value>, Box<dyn std::error::Error>>>()?;
                if vals.is_empty() {
                    return Err("list was empty. for an empty list, use the list constructor".into())
                }
                let mut vals = vals.into_iter();
                let first_value = vals.next().unwrap();
                if let Some(func) = self.functions.get(&first_value) {
                    func(Value::List(vals.collect()))
                } else {
                    Err(format!("function {:?} not found", first_value).into())
                }
            }
        }
    }

}

#[derive(PartialEq, Eq, Hash, Debug)]
enum Value {
    Num(usize),
    String(String),
    List(Vec<Value>),
}

impl Value {
    fn into_list(self) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
        match self {
            Self::List(n) => Ok(n),
            _ => Err(format!("expected List but got {:?}", self).into())
        }
    }

    fn as_num(&self) -> Result<usize, Box<dyn std::error::Error>> {
        match self {
            Self::Num(n) => Ok(*n),
            _ => Err(format!("expected Number but got {:?}", self).into())
        }
    }
}
