use anyhow::anyhow;
use std::fmt::{Debug, Formatter};
use std::str::from_utf8;
use smallvec::{SmallVec,smallvec};

pub type Bytes = SmallVec<[u8; 16]>;
pub type List = Vec<Expr>;
pub type BytesRef<'a> = &'a [u8];
pub type ListRef<'a> = &'a [Expr];

#[derive(Clone)]
pub enum Expr {
    Bytes(Bytes),
    List(List),
}

// from https://docs.rs/encoding8/0.3.1/src/encoding8/ascii/mod.rs.html#121-123
pub fn is_control(b: u8) -> bool {
    const DEL: u8 = 127;
    b < 32 || b == DEL
}


impl Debug for Expr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Bytes(w) => {
                if let Ok(s) = from_utf8(w) {
                    for c in s.chars() {
                        if is_control(c as u8) {
                            write!(f, "0x{:02x}", c as u8)?;
                        } else {
                            write!(f, "{}", c)?;
                        }
                    }
                } else {
                    write!(f, "{:?}", w)?;
                }
            }
            Expr::List(l) => {
                write!(f, "(")?;
                if let Some((last, init)) = l.split_last() {
                    for expr in init {
                        expr.fmt(f)?;
                        write!(f, " ")?;
                    }
                    last.fmt(f)?;
                }
                write!(f, ")")?;
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum ExprRef<'a> {
    Bytes(BytesRef<'a>),
    List(ListRef<'a>),
}

impl ExprRef<'_> {
    pub fn to_owned(&self) -> Expr {
        match self {
            ExprRef::Bytes(w) => Expr::Bytes((*w).into()),
            ExprRef::List(l) => Expr::List(l.to_vec()),
        }
    }
}

impl Expr {
    pub fn as_ref(&self) -> ExprRef<'_> {
        match self {
            Expr::Bytes(w) => ExprRef::Bytes(w),
            Expr::List(l) => ExprRef::List(l),
        }
    }

    pub fn into_list(self) -> anyhow::Result<List> {
        match self {
            Expr::List(l) => Ok(l),
            _ => Err(anyhow!("expected List but got {:?}", self)),
        }
    }

    pub fn into_bytes(self) -> anyhow::Result<Bytes> {
        match self {
            Expr::Bytes(w) => Ok(w),
            _ => Err(anyhow!("expected Bytes but got {:?}", self)),
        }
    }
}

pub fn parse_exprs(cs: &mut impl Iterator<Item = u8>) -> List {
    parse_exprs_rec(cs, false)
}

// TODO: this could be an iterator of Exprs?
fn parse_exprs_rec(cs: &mut impl Iterator<Item = u8>, is_inside_list: bool) -> List {
    let mut exprs = List::new();
    let mut current_string: Option<Bytes> = None;
    loop {
        match cs.next() {
            Some(c) => match c {
                b'(' => {
                    exprs.push(Expr::List(parse_exprs_rec(cs, true)));
                }
                b')' => {
                    if is_inside_list {
                        exprs.extend(current_string.take().map(Expr::Bytes));
                        break;
                    } else {
                        panic!("got a ) and wasn't expecting one");
                    }
                }
                c if (c as char).is_whitespace() => {
                    exprs.extend(current_string.take().map(Expr::Bytes));
                }
                c => match current_string.as_mut() {
                    None => current_string = Some(smallvec![c]),
                    Some(s) => s.push(c),
                },
            },
            None => {
                if is_inside_list {
                    panic!("was expecting a ) but reached end of stream")
                } else {
                    exprs.extend(current_string.take().map(Expr::Bytes));
                    break;
                }
            }
        }
    }
    exprs
}
