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


#[derive(Debug, PartialEq,Clone)]
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
pub struct TokenSlice<'a, 'b > {
    tokens: &'b [LexToken<'a>],
}

impl<'a, 'b> TokenSlice<'a, 'b> {
    pub fn new(tokens: &'b [LexToken<'a>]) -> Self {
        TokenSlice { tokens }
    }
    pub fn last(&self) -> Option<&LexToken<'a>> {
        self.tokens.last()
    }
        
    /// Collects the spans of all tokens in the slice
    pub fn spans(&self) -> Vec<LocatedSpan<&'a str>> {
        self.tokens.iter().map(|token| token.span).collect()
    }
}

impl<'a, 'b> TokenSlice<'a, 'b> {
    pub fn take_err(&self, count: usize) -> Result<(Self, Self), ()> {
        if self.input_len() >= count {
            let (taken, remaining) = self.take_split(count);
            Ok((taken, remaining))
        } else {
            Err(())
        }
    }
}


impl<'a, 'b> Index<usize> for TokenSlice<'a, 'b>
{
    type Output = LexToken<'a>;

    fn index(&self, index: usize) -> &LexToken<'a> {
        &self.tokens[index]
    }

}

// Implement InputLength for TokenSlice
impl<'a, 'b> InputLength for TokenSlice<'a, 'b> {
    fn input_len(&self) -> usize {
        self.tokens.len()
    }
}

// Implement InputTake for TokenSlice
impl<'a, 'b> InputTake for TokenSlice<'a, 'b> {
    fn take(&self, count: usize) -> Self {
        TokenSlice {
            tokens: &self.tokens[..count],
        }
    }

    fn take_split(&self, count: usize) -> (Self, Self) {
        let (prefix, suffix) = self.tokens.split_at(count);
        (
            TokenSlice {
                tokens: suffix,
            },
            TokenSlice {
                tokens: prefix,
            },
        )
    }
}

impl<'a, 'b> Offset for TokenSlice<'a, 'b> {
    fn offset(&self, second: &Self) -> usize {
        let first_ptr = self.tokens.as_ptr();
        let second_ptr = second.tokens.as_ptr();

        // Calculate the offset in terms of the number of tokens
        second_ptr as usize - first_ptr as usize
    }
}


// Implement InputIter for TokenSlice
impl<'a, 'b> InputIter for TokenSlice<'a, 'b> {
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
impl<'a, 'b> Slice<Range<usize>> for TokenSlice<'a, 'b>{
    fn slice(&self, range: Range<usize>) -> Self {
        TokenSlice {
            tokens: &self.tokens[range],
        }
    }
}

impl<'a, 'b> Slice<RangeTo<usize>> for TokenSlice<'a, 'b>{
    fn slice(&self, range: RangeTo<usize>) -> Self {
        TokenSlice {
            tokens: &self.tokens[..range.end],
        }
    }
}

impl<'a, 'b> Slice<RangeFrom<usize>> for TokenSlice<'a, 'b>{
    fn slice(&self, range: RangeFrom<usize>) -> Self {
        TokenSlice {
            tokens: &self.tokens[range.start..],
        }
    }
}

impl<'a, 'b> Slice<RangeFull> for TokenSlice<'a, 'b>{
    fn slice(&self, _: RangeFull) -> Self {
        TokenSlice {
            tokens: self.tokens,
        }
    }
}


impl<'a, 'b> Iterator for TokenSlice<'a, 'b> {
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


impl<'a, 'b> UnspecializedInput for TokenSlice<'a, 'b> {}

use nom::FindToken;

impl<'a, 'b> FindToken<LexToken<'a>> for TokenSlice<'a, 'b> {
    fn find_token(&self, token: LexToken<'a>) -> bool {
        self.tokens.iter().any(|t| *t == token)
    }
}
