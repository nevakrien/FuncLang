use nom::{InputIter, InputLength, InputTake, Needed, Slice};
use nom_locate::LocatedSpan;
use std::iter::Enumerate;
use std::ops::{Range, RangeFrom, RangeFull, RangeTo};
use std::slice::Iter;

#[derive(Debug, PartialEq, Clone)]
pub struct LexToken<'a> {
    pub inner: LocatedSpan<&'a str, LexTag>,
}

impl<'a> LexToken<'a> {
    pub fn new(span: LocatedSpan<&'a str, LexTag>) -> Self {
        LexToken { inner: span }
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

// Newtype wrapper around a slice of LexToken
#[derive(Debug, Clone, PartialEq)]
pub struct TokenSlice<'a, 'b> {
    tokens: &'b [LexToken<'a>],
}

impl<'a, 'b> TokenSlice<'a, 'b> {
    pub fn new(tokens: &'b [LexToken<'a>]) -> Self {
        TokenSlice { tokens }
    }
}

// Implementing InputLength for TokenSlice
impl<'a, 'b> InputLength for TokenSlice<'a, 'b> {
    fn input_len(&self) -> usize {
        self.tokens.len()
    }
}

// Implementing InputTake for TokenSlice
impl<'a, 'b> InputTake for TokenSlice<'a, 'b> {
    fn take(&self, count: usize) -> Self {
        TokenSlice {
            tokens: &self.tokens[..count],
        }
    }

    fn take_split(&self, count: usize) -> (Self, Self) {
        let (prefix, suffix) = self.tokens.split_at(count);
        (
            TokenSlice { tokens: suffix },
            TokenSlice { tokens: prefix },
        )
    }
}

// Implementing InputIter for TokenSlice
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

// Implementing Slice for TokenSlice
impl<'a, 'b> Slice<Range<usize>> for TokenSlice<'a, 'b> {
    fn slice(&self, range: Range<usize>) -> Self {
        TokenSlice {
            tokens: &self.tokens[range],
        }
    }
}

impl<'a, 'b> Slice<RangeTo<usize>> for TokenSlice<'a, 'b> {
    fn slice(&self, range: RangeTo<usize>) -> Self {
        TokenSlice {
            tokens: &self.tokens[..range.end],
        }
    }
}

impl<'a, 'b> Slice<RangeFrom<usize>> for TokenSlice<'a, 'b> {
    fn slice(&self, range: RangeFrom<usize>) -> Self {
        TokenSlice {
            tokens: &self.tokens[range.start..],
        }
    }
}

impl<'a, 'b> Slice<RangeFull> for TokenSlice<'a, 'b> {
    fn slice(&self, _: RangeFull) -> Self {
        TokenSlice {
            tokens: self.tokens,
        }
    }
}



