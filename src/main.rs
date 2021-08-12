#![feature(box_syntax)]

use crate::interp::Interpreter;

mod parser;
mod interp;

fn main() {
    println!("--- CODE ---");
    let code =
r#".block
(define my-guy mwahaha-this-is-a-value)
(print-ascii my-guy)
(. print-ascii (@ my-guy))
"#;
    println!("{}", code);
    let expr = parser::parse_exprs(&mut code.as_bytes().into_iter().copied());

    println!("--- PARSED ---");
    println!("{:?}", expr);

    let mut interpreter = Interpreter::new();
    println!("--- EVAL ---");
    let res = interpreter.eval(expr).unwrap();
    println!("--- RESULT ---");
    println!("{:?}", res);
}
