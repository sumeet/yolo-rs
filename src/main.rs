#![feature(box_syntax)]

use crate::interp::Interpreter;

mod parser;
mod interp;

fn main() {
    println!("--- PARSED ---");
    let expr = parser::parse_exprs(&mut r#"
.
(define my-guy mwahaha-this-is-a-value)
(print-ascii my-guy)
(apply print-ascii (@ my-guy))
    "#.as_bytes().into_iter().copied());
    println!("{:?}", expr);

    println!();
    println!();
    
    let mut interpreter = Interpreter::new();

    println!("--- EVAL ---");
    let res = interpreter.eval(expr).unwrap();
    println!();
    println!("--- RESULT ---");
    println!("{:?}", res);
}
