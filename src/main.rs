const BLAH: &str = "(()";

#[derive(Debug)]
enum Expr {
    Word(String),
    List(Vec<Expr>),
}

fn parse_exprs(cs: &mut impl Iterator<Item = char>) -> Vec<Expr> {
    let mut exprs = vec![];
    let mut current_string : Option<String> = None;
    while let Some(c) = cs.next() {
        match c {
            '(' => {
                exprs.push(Expr::List(parse_exprs(cs)));
            }
            ')' => {
                match current_string.take() {
                    None => (),
                    Some(finished_string) => {
                        exprs.push(Expr::Word(finished_string));
                    }
                }
                break;
            }
            c if c.is_whitespace() => {
                match current_string.take() {
                    None => (),
                    Some(finished_string) => {
                        exprs.push(Expr::Word(finished_string));
                    }
                }
            }
            c => {
                match current_string.as_mut() {
                    None => current_string = Some(c.into()),
                    Some(s) => s.push(c),
                }
            }
        }
    }
    exprs
}

fn main() {
    println!("{:?}", parse_exprs(&mut BLAH.chars()));
}
