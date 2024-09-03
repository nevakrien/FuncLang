#![allow(dead_code)] //this module is being consumed
use nom::{InputIter, InputLength, InputTake, Needed, Slice};
use nom_locate::LocatedSpan;
use std::iter::Enumerate;
use std::ops::{Range, RangeFrom, RangeFull, RangeTo};
use std::slice::Iter;
use core::ops::Index;
use nom::Offset;
use nom::UnspecializedInput;

use crate::errors::{UserSideError};//UserSideWarning


#[derive(Debug, PartialEq)]
pub struct LexToken<'a> {
    pub span: LocatedSpan<&'a str>,
    pub tag: LexTag,
    pub error: Option<Box<UserSideError<'a>>>,
}

impl<'a> LexToken<'a> {
    pub fn new(span: LocatedSpan<&'a str>,tag : LexTag) -> Self {
        LexToken { span: span, tag:tag , error:None}
    }
     pub fn err_new(span: LocatedSpan<&'a str>,tag : LexTag, error:UserSideError<'a>) -> Self {
        LexToken { span: span, tag:tag , error:Some(Box::new(error))}
    }
}

impl<'a> InputLength for LexToken<'a> {
    fn input_len(&self) -> usize {
        1
    }
}


#[derive(Debug, PartialEq, Clone)]
pub enum LexTag {
    // Your LexTag variants here
    Comment(),
    Word(),
    Atom(),
    Float(f64),
    Int(i64),
    Delimiter(char),
    Ender(char),
    Op(BinaryOp),
    String(char),
    PoisonString(char),
    Unknowen(),
}

#[derive(Debug, PartialEq, Clone)]
pub enum BinaryOp {
    // Your BinaryOp variants here
    Pipe,
    Dot,
    Dots,
    DoubleDots,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Exp,
    FatArrow,
    SmallArrow,
    SingleOr,
    Or,
    And,
    Xor,
    OneEqul,
    TwoEqul,
    NotEqual,
    SmallerEqual,
    Smaller,
    Bigger,
    BiggerEqual,
}


// Define the TokenSlice struct with a generic diagnostic type `D` that defaults to `()`.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenSlice<'a, 'b, D = () > {
    tokens: &'b [LexToken<'a>],
    pub diag: D,
}

impl<'a, 'b, D> TokenSlice<'a, 'b, D> {
    pub fn new(tokens: &'b [LexToken<'a>], diag: D) -> Self {
        TokenSlice { tokens, diag }
    }
    pub fn last(&self) -> Option<&LexToken<'a>> {
        self.tokens.last()
    }

}

impl<'a, 'b, D: Clone> TokenSlice<'a, 'b, D> {
    pub fn take_err(&self, count: usize) -> Result<(Self, Self), ()> {
        if self.input_len() >= count {
            let (taken, remaining) = self.take_split(count);
            Ok((taken, remaining))
        } else {
            Err(())
        }
    }
}


// // Implement methods to convert between types with and without diagnostics
// impl<'a, 'b> TokenSlice<'a, 'b, &Diagnostics<'a>> {
//     pub fn strip_diag(self) -> TokenSlice<'a, 'b> {
//         TokenSlice {
//             tokens: self.tokens,
//             diag: (),
//         }
//     }
//     pub fn report_error(&self,error: UserSideError<'a>) {
//     	self.diag.report_error(error);
//     }

//     pub fn report_warning(&self, warning: UserSideWarning<'a>) {
//         self.diag.report_warning(warning);
//     }
// }


// impl<'a, 'b> TokenSlice<'a, 'b,()> {
//     pub fn add_diag(self, diag: &'a Diagnostics<'a>) -> TokenSlice<'a, 'b, &'a Diagnostics<'a>> {
//         TokenSlice {
//             tokens: self.tokens,
//             diag,
//         }
//     }
// }

impl<'a, 'b, D> Index<usize> for TokenSlice<'a, 'b, D>
{
    type Output = LexToken<'a>;

    fn index(&self, index: usize) -> &LexToken<'a> {
        &self.tokens[index]
    }

}

// Implement InputLength for TokenSlice
impl<'a, 'b, D> InputLength for TokenSlice<'a, 'b, D> {
    fn input_len(&self) -> usize {
        self.tokens.len()
    }
}

// Implement InputTake for TokenSlice
impl<'a, 'b, D : Clone> InputTake for TokenSlice<'a, 'b, D> {
    fn take(&self, count: usize) -> Self {
        TokenSlice {
            tokens: &self.tokens[..count],
            diag: self.diag.clone(),
        }
    }

    fn take_split(&self, count: usize) -> (Self, Self) {
        let (prefix, suffix) = self.tokens.split_at(count);
        (
            TokenSlice {
                tokens: suffix,
                diag: self.diag.clone(),
            },
            TokenSlice {
                tokens: prefix,
                diag: self.diag.clone(),
            },
        )
    }
}

impl<'a, 'b, D> Offset for TokenSlice<'a, 'b, D> {
    fn offset(&self, second: &Self) -> usize {
        let first_ptr = self.tokens.as_ptr();
        let second_ptr = second.tokens.as_ptr();

        // Calculate the offset in terms of the number of tokens
        second_ptr as usize - first_ptr as usize
    }
}


// Implement InputIter for TokenSlice
impl<'a, 'b, D> InputIter for TokenSlice<'a, 'b, D> {
    type Item = &'b LexToken<'a>;
    type Iter = Enumerate<Iter<'b, LexToken<'a>>>;
    type IterElem = Iter<'b, LexToken<'a>>;

    fn iter_indices(&self) -> Self::Iter {
        self.tokens.iter().enumerate()
    }

    fn iter_elements(&self) -> Self::IterElem {
        self.tokens.iter()
    }

    fn position<P>(&self, pred: P) -> Option<usize>
    where
        P: Fn(Self::Item) -> bool,
    {
        self.tokens.iter().position(pred)
    }

    fn slice_index(&self, count: usize) -> Result<usize, Needed> {
        if self.tokens.len() >= count {
            Ok(count)
        } else {
            Err(Needed::new(count))
        }
    }
}

// Implement Slice for TokenSlice
impl<'a, 'b, D> Slice<Range<usize>> for TokenSlice<'a, 'b, D>
where
    D: Clone,
{
    fn slice(&self, range: Range<usize>) -> Self {
        TokenSlice {
            tokens: &self.tokens[range],
            diag: self.diag.clone(),
        }
    }
}

impl<'a, 'b, D> Slice<RangeTo<usize>> for TokenSlice<'a, 'b, D>
where
    D: Clone,
{
    fn slice(&self, range: RangeTo<usize>) -> Self {
        TokenSlice {
            tokens: &self.tokens[..range.end],
            diag: self.diag.clone(),
        }
    }
}

impl<'a, 'b, D> Slice<RangeFrom<usize>> for TokenSlice<'a, 'b, D>
where
    D: Clone,
{
    fn slice(&self, range: RangeFrom<usize>) -> Self {
        TokenSlice {
            tokens: &self.tokens[range.start..],
            diag: self.diag.clone(),
        }
    }
}

impl<'a, 'b, D> Slice<RangeFull> for TokenSlice<'a, 'b, D>
where
    D: Clone,
{
    fn slice(&self, _: RangeFull) -> Self {
        TokenSlice {
            tokens: self.tokens,
            diag: self.diag.clone(),
        }
    }
}


impl<'a, 'b, D> Iterator for TokenSlice<'a, 'b, D> {
    type Item = &'b LexToken<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.tokens.is_empty() {
            None
        } else {
            let (first, rest) = self.tokens.split_first()?;
            self.tokens = rest;
            Some(first)
        }
    }
}


impl<'a, 'b, D> UnspecializedInput for TokenSlice<'a, 'b, D> {}
