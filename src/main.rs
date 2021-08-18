#![feature(box_syntax)]
#![feature(slice_pattern)]
#![feature(in_band_lifetimes)]
#![feature(generators)]
#![feature(array_methods)]

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
r#".|> (
    (.define .if (.|> (
        // last element on the stack is the else body to eval,
        // next element on the stack is the if-true body to eval,
        // next element on the stack is the condition to eval (returns true/false),

        // swap the condition to the top of the stack
        (.|> ((.u 2) (.swap)))
        
        // evaluates the condition, which should push true or false onto the stack
        (.)
        // either evaluates the conditional body or drops it
        (.?)
    )))
    
    (.push (.|> ((.u 68) (.u 69) (.u>))))
    (.push (.|> ((.u 735) (.u-print))))
    (.push (.|> ((.u 90) (.u-print))))
    (.if)

    (.if (.|> ((.u 68) (.u 69) (.u>)))
        // then
        (.|> ((.u 735) (.u-print)))
        // else
        (.|> ((.u 90) (.u-print)))
    )
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

    println!("--- STACK ---");
    interpreter.dbg_stack();

    dbg!(std::mem::size_of::<parser::Expr>());
}

fn remove_comments(code: &str) -> String {
    code.lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .join("\n")
}
