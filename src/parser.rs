pub type Word = Vec<u8>;
pub type List = Vec<Expr>;
pub type WordRef<'a> = &'a [u8];
pub type ListRef<'a> = &'a [Expr];

#[derive(Debug, Clone)]
pub enum Expr {
    Word(Word),
    List(List),
}

#[derive(Debug)]
pub enum ExprRef<'a> {
    Word(WordRef<'a>),
    List(ListRef<'a>),
}

impl ExprRef<'_> {
    pub fn to_owned(&self) -> Expr {
        match self {
            ExprRef::Word(w) => {
                Expr::Word(w.to_vec())
            },
            ExprRef::List(l) => Expr::List(l.to_vec()),
        }
    }

    pub fn as_word(&self) -> Result<WordRef, Box<dyn std::error::Error>> {
        match self {
            Self::Word(w) => Ok(w),
            otherwise => Err(format!("expected Word but got {:?}", otherwise).into())
        }
    }

    pub fn as_list(&self) -> Result<ListRef, Box<dyn std::error::Error>> {
        match self {
            Self::List(l) => Ok(l),
            otherwise => Err(format!("expected List but got {:?}", otherwise).into())
        }
    }
}

impl Expr {
    pub fn as_ref(&self) -> ExprRef<'_> {
        match self {
            Expr::Word(w) => ExprRef::Word(w),
            Expr::List(l) => ExprRef::List(l),
        }
    }
}

pub fn parse_exprs(cs: &mut impl Iterator<Item = u8>) -> Expr {
    parse_exprs_rec(cs, false)
}

// TODO: this could be an iterator of Exprs?
fn parse_exprs_rec(cs: &mut impl Iterator<Item = u8>, is_inside_list: bool) -> Expr {
    let mut exprs = List::new();
    let mut current_string : Option<Word> = None;
    loop {
        match cs.next() {
            Some(c) => {
                match c {
                    b'(' => {
                        exprs.push(parse_exprs_rec(cs, true));
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
