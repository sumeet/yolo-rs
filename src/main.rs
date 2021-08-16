#![feature(box_syntax)]
#![feature(slice_pattern)]
#![feature(in_band_lifetimes)]
#![feature(generators)]

use crate::interp::Interpreter;

mod parser;
mod interp;

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
    (.define .u-incr (. (
        (.temp.u 1)
        (.+-u)
    )))
    (.temp.u 2)
    (.u-incr)
    (.temp.print-u)
)
"#;
    println!("{}", code);
    let exprs = parser::parse_exprs(&mut code.as_bytes().into_iter().copied());

    println!("--- PARSED ---");
    println!("{:?}", exprs);

    let mut interpreter = Interpreter::new();
    println!("--- EVAL ---");
    interpreter.eval(exprs.into_iter()).unwrap();
}
