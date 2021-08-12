use std::collections::VecDeque;

pub type Word = Vec<u8>;
pub type List = VecDeque<Expr>;

#[derive(Debug, Clone)]
pub enum Expr {
    Word(Word),
    List(List),
}

impl Expr {
    pub fn empty_list() -> Self {
        Self::List(List::new())
    }

    pub fn into_word(self) -> Result<Word, Box<dyn std::error::Error>> {
        match self {
            Expr::Word(w) => Ok(w),
            otherwise => Err(format!("expected Word but got {:?}", otherwise).into())
        }
    }

    pub fn into_list(self) -> Result<List, Box<dyn std::error::Error>> {
        match self {
            Expr::List(l) => Ok(l),
            otherwise => Err(format!("expected List but got {:?}", otherwise).into())
        }
    }

    pub fn as_mut_list(&mut self) -> Result<&mut List, Box<dyn std::error::Error>> {
        match self {
            Expr::List(l) => Ok(l),
            otherwise => Err(format!("expected List but got {:?}", otherwise).into())
        }
    }
}

pub fn parse_exprs(cs: &mut impl Iterator<Item = u8>) -> Expr {
    parse_exprs_rec(cs, false)
}

fn parse_exprs_rec(cs: &mut impl Iterator<Item = u8>, is_inside_list: bool) -> Expr {
    let mut exprs = List::new();
    let mut current_string : Option<Word> = None;
    loop {
        match cs.next() {
            Some(c) => {
                match c {
                    b'(' => {
                        exprs.push_back(parse_exprs_rec(cs, true));
                    }
                    b')' => {
                        if is_inside_list {
                            exprs.extend(current_string.take().map(Expr::Word));
                            break;
                        } else {
                            panic!("got a ) and wasn't expecting one");
                        }
                    }
                    c if (c as char).is_whitespace() => {
                        exprs.extend(current_string.take().map(Expr::Word));
                    }
                    c => {
                        match current_string.as_mut() {
                            None => current_string = Some(vec![c]),
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
