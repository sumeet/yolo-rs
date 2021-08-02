#[derive(Debug)]
pub enum Expr {
    Word(String),
    List(Vec<Expr>),
}

pub fn parse_exprs(cs: &mut impl Iterator<Item = char>, is_inside_list: bool) -> Expr {
    let mut exprs = vec![];
    let mut current_string : Option<String> = None;
    loop {
        match cs.next() {
            Some(c) => {
                match c {
                    '(' => {
                        exprs.push(parse_exprs(cs, true));
                    }
                    ')' => {
                        if is_inside_list {
                            exprs.extend(current_string.take().map(Expr::Word));
                            break;
                        } else {
                            panic!("got a ) and wasn't expecting one");
                        }
                    }
                    c if c.is_whitespace() => {
                        exprs.extend(current_string.take().map(Expr::Word));
                    }
                    c => {
                        match current_string.as_mut() {
                            None => current_string = Some(c.into()),
                            Some(s) => s.push(c),
                        }
                    }
                }

            }
            None => {
                if is_inside_list {
                    panic!("was expecting a ) but reached end of stream")
                } else {
                    exprs.extend(current_string.take().map(Expr::Word));
                    break;
                }
            }
        }
    }
    Expr::List(exprs)
}
