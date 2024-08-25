use nom::bytes::complete::{is_a,take_till,take_while,take_while1};
use nom::sequence::{pair,preceded};
use nom::combinator::recognize;
use nom::character::complete::{digit1};

use nom::multi::fold_many0;
use nom::branch::alt;	

use crate::errors::{Cursor,CResult,strip_reporting,UserSideError};
use nom::combinator::opt;
use  nom_locate::LocatedSpan;



pub type LexToken<'a> = LocatedSpan<&'a str,LexTag>;
pub enum LexTag {
	Comment(),
	Word(),
	Float(FloatTag),
	Int(IntTag),
}

pub enum FloatTag{
	Valid(f64),
	UnexpectedEnd(),
	JustDot(),	
}

pub enum IntTag{
	Valid(i64),
	UnexpectedEnd(),	
}

fn skip_whitespace<'a>(input: Cursor<'a>) -> Cursor<'a> {
	fn typed_take_whitespace<'a>(input: Cursor<'a>) -> CResult<'a,Cursor<'a>>{
		take_while( |c:char| c.is_whitespace())(input)
	}
	let s=opt(typed_take_whitespace)(input);
	let(ans,_)=s.unwrap();
	return ans;
}

fn comment<'a>(input: Cursor<'a>) -> CResult<'a,LexToken<'a>>{
	let (_,input)=is_a("#")(input)?;
	take_till(|c| c=='\n')(input)
	.map(|(i,x)| 
		(i,strip_reporting(x).map_extra(|()| LexTag::Comment()))
	)	
}

fn name<'a>(input: Cursor<'a>) -> CResult<'a,LexToken<'a>> {
	recognize(pair(
		take_while1(|c:char| c.is_alphabetic()   || c=='_'),
		take_while( |c:char| c.is_alphanumeric() || c=='_')
	))(input)
	.map(|(i,x)| 
		(i,strip_reporting(x).map_extra(|()| LexTag::Word()))
	)		
}

//does not handle error checking on the tail
fn uint_underscored<'a>(input: Cursor<'a>) -> CResult<'a,u64>{
	fn typed_digit1<'b>(x: Cursor<'b>) -> CResult<'b, &'b str> {
	    digit1(x).map(|(i,x)| 
			(i,*x.fragment())
		)
	}
	let (input,d)=digit1(input)?;
	
	let diag = input.extra.diag;
	let report_input=strip_reporting(input.clone());

	let mut overflowed = false;
	let ans = fold_many0(
		alt((
        	preceded(is_a("_"),typed_digit1),
        	typed_digit1,)
        ),
        ||{d.parse::<u64>().unwrap()},
        |acc, item| {
        	if !overflowed{
        		acc.checked_mul(10u64.pow(item.len() as u32)) // Adjust multiplier based on number of digits
                .and_then(|acc| acc.checked_add(item.parse::<u64>().unwrap()))
                .unwrap_or_else(|| {
                	overflowed = true;
                	diag.report_error(UserSideError::OverflowError(report_input,acc));
                	acc
                })
        	}

        	else{
        		acc
        	}
            
        },
    )(input);
    ans
}
#[cfg(test)]
use crate::errors::Diagnostics;
#[cfg(test)]
use nom::InputTake;
#[cfg(test)]
use crate::errors::make_cursor;

#[test]
fn test_uint_underscored_valid() {
    let diag = Diagnostics::new();

    let input = make_cursor("111_222_333xyz", &diag);
    let result = uint_underscored(input.clone());
    assert_eq!(
        result,
        Ok((input.take_split(11).0, 111_222_333u64))
    );

    let input = make_cursor("123_6_22 as", &diag);
    let result = uint_underscored(input.clone());
    assert_eq!(
        result,
        Ok((input.take_split(8).0, 123_6_22u64))
    );

    let input = make_cursor("987654", &diag);
    let result = uint_underscored(input.clone());
    assert_eq!(
        result,
        Ok((input.take_split(6).0, 987654u64))
    );
}

#[test]
fn test_skip_whitespace() {
    let diag = Diagnostics::new();

    let input = make_cursor("   xyz", &diag);
    let result = skip_whitespace(input.clone());
    assert_eq!(result, input.take_split(3).0);

    let input = make_cursor("\t\t123_6_22 as", &diag);
    let result = skip_whitespace(input.clone());
    assert_eq!(result, input.take_split(2).0);

    let input = make_cursor("\n!!!", &diag);
    let result = skip_whitespace(input.clone());
    assert_eq!(result, input.take_split(1).0);

    let input = make_cursor("987654", &diag);
    let result = skip_whitespace(input.clone());
    assert_eq!(result, input); // No whitespace to skip, so it should return the original input
}