use nom::bytes::complete::{is_a,take_till,take_while,take_while1};
use nom::sequence::{pair,preceded};
use nom::combinator::recognize;
use nom::character::complete::{digit1};

use nom::multi::fold_many0;
use nom::branch::alt;	

use crate::errors::{Cursor,CResult};

pub enum LexToken<'a> {
	Comment(&'a str),
	Word(&'a str),
	Float(FloatToken<'a>),
	Int(IntToken<'a>),
}

pub enum FloatToken<'a>{
	Valid(f64),
	UnexpectedEnd(Cursor<'a>),
	JustDot(Cursor<'a>),	
}

pub enum IntToken<'a>{
	Valid(f64),
	UnexpectedEnd(Cursor<'a>),	
}

fn skip_whitespace<'a>(input: Cursor<'a>) -> Cursor<'a> {
	fn _skip_whitespace<'a>(input: Cursor<'a>) -> CResult<'a,Cursor<'a>>{
		take_while( |c:char| c.is_whitespace())(input)
	}

	match _skip_whitespace(input.clone()){
		Err(_) => input,
		Ok((ans,_)) => ans,
	}
}

fn comment<'a>(input: Cursor<'a>) -> CResult<'a,LexToken<'a>>{
	let (_,input)=is_a("#")(input)?;
	take_till(|c| c=='\n')(input)
	.map(|(i,x)| 
		(i,LexToken::Comment(*x.fragment()))
	)	
}

fn name<'a>(input: Cursor<'a>) -> CResult<'a,LexToken<'a>> {
	recognize(pair(
		take_while1(|c:char| c.is_alphabetic()   || c=='_'),
		take_while( |c:char| c.is_alphanumeric() || c=='_')
	))(input)
	.map(|(i,x)| 
		(i,LexToken::Word(*x.fragment()))
	)		
}

//does not handle error checking on the tail
fn uint_underscored<'a>(input: Cursor<'a>) -> CResult<'a,u64>{
	fn typed_digit1<'b>(x: Cursor<'b>) -> CResult<'b, &'b str> {
	    digit1(x).map(|(i,x)| 
			(i,*x.fragment())
		)
	}
	let (input,d)=recognize(digit1)(input)?;

	let ans = fold_many0(
		alt((
        	preceded(is_a("_"),typed_digit1),
        	typed_digit1,)
        ),
        ||{d.parse::<u64>().unwrap()},
        |acc, item| {
            acc.checked_mul(10u64.pow(item.len() as u32)) // Adjust multiplier based on number of digits
                .and_then(|acc| acc.checked_add(item.parse::<u64>().unwrap()))
                .unwrap_or_else(|| {
                    panic!("Overflow occurred while parsing digits")
                })
        },
    )(input);
    ans
}
#[cfg(test)]
use crate::errors::Diagnostics;
#[cfg(test)]
use crate::errors::Reporter;
#[cfg(test)]
use nom::InputTake;

#[test]
fn test_uint_underscored_valid() {
    let diag = Diagnostics::new();
    let reporter = Reporter::new(&diag);

    let input = Cursor::new_extra("111_222_333xyz", reporter.clone());
    let result = uint_underscored(input.clone());
    assert_eq!(
        result,
        Ok((input.take_split(11).0, 111_222_333u64))
    );

    let input = Cursor::new_extra("123_6_22 as", reporter.clone());
    let result = uint_underscored(input.clone());
    assert_eq!(
        result,
        Ok((input.take_split(8).0, 123_6_22u64))
    );

    let input = Cursor::new_extra("987654", reporter.clone());
    let result = uint_underscored(input.clone());
    assert_eq!(
        result,
        Ok((input.take_split(6).0, 987654u64))
    );
}