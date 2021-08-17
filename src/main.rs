#![feature(box_syntax)]
#![feature(slice_pattern)]
#![feature(in_band_lifetimes)]
#![feature(generators)]

use crate::interp::Interpreter;
use itertools::Itertools;

mod interp;
mod parser;

fn main() {
    println!("--- CODE ---");
    let code =
// TODO perhaps put this in a package, i.e., this is the "core" language
// the other languages can have a different specification and be part of a different system
// and they can FFI into core.
//
// for example, there could be a type checked language that would call into this one. but you
// wouldn't be able to call it directly because none of these funcs are type checked
//
// another idea, the package icon can go on the item instead of an icon for the function itself
r#". (
    (.define .if (. (
        // last element on the stack is the conditional body,
        // next element on the stack is the condition
        //
        // swap the condition to the top of the stack
        (.u 1)
        (.swap)
        // evaluates the condition, which should push true or false onto the stack
        (.eval)
        // either evaluates the conditional body or drops it
        (.?)
    )))
    (.define .u++ (. (
        (.u 1)
        (.u+)
    )))

    (.u 69)
    (.push (.u-print))
    (.u 0)
    (.?)
)
"#;
    println!("{}", code);
    let code = remove_comments(code);
    let exprs = parser::parse_exprs(&mut code.as_bytes().into_iter().copied());

    println!("--- PARSED ---");
    println!("{:?}", exprs);

    let mut interpreter = Interpreter::new();
    println!("--- EVAL ---");
    interpreter.eval(exprs.into_iter()).unwrap();
}

fn remove_comments(code: &str) -> String {
    code.lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .join("\n")
}
