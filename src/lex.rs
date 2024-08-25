use nom::bytes::complete::{is_a,take_till,take_while,take_while1};
use nom::sequence::{pair,preceded,terminated};
use nom::combinator::recognize;
use nom::character::complete::{digit1};

use nom::multi::fold_many0;
use nom::branch::alt;	

use crate::errors::{Cursor,CResult,strip_reporting,UserSideError};
use nom::combinator::opt;
use  nom_locate::LocatedSpan;



pub type LexToken<'a> = LocatedSpan<&'a str,LexTag>;

#[derive(Debug,PartialEq,Clone)]
pub enum LexTag {
	Comment(),
	Word(),
	Float(FloatTag),
	Int(IntTag),
}

#[derive(Debug,PartialEq,Clone)]
pub enum FloatTag{
	Valid(f64),
	UnexpectedEnd(),
	JustDot(),	
}

#[derive(Debug,PartialEq,Clone)]
pub enum IntTag{
	Valid(i64),
	UnexpectedEnd(),	
}

pub fn lext_text<'a>(input: Cursor<'a>) -> CResult<'a,LexToken<'a>>{
	//TODO add extra token
	alt((comment,name))(skip_whitespace(input))
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
	let (input,_)=is_a("#")(input)?;
	terminated(
		take_till(|c| c=='\n'),
		take_while(|c| c=='\n')
	)(input)	
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

#[test]
fn test_lex_basic_string_with_names_and_comments() {
    let diag = Diagnostics::new();

    let input = make_cursor("name1 # This is a comment\nname2 # Another comment", &diag);
    let result = lext_text(input);

    assert!(result.is_ok());

    let (remaining, token1) = result.unwrap();
    assert_eq!(token1.extra, LexTag::Word());
    assert_eq!(token1.fragment(), &"name1");

    let result = lext_text(remaining);
    let (remaining, token2) = result.unwrap();
    assert_eq!(token2.extra, LexTag::Comment());
    
    let result = lext_text(remaining);
    let (remaining, token3) = result.unwrap();
    assert_eq!(token3.extra, LexTag::Word());
    assert_eq!(token3.fragment(), &"name2");

    let result = lext_text(remaining);
    let (remaining, token4) = result.unwrap();
    assert_eq!(token4.extra, LexTag::Comment());
    
    // Make sure no tokens left
    assert_eq!(remaining.fragment(), &"");
}

#[test]
fn test_lex_invalid_token_error() {
    let diag = Diagnostics::new();

    let input = make_cursor("name1 { name2", &diag);
    let result = lext_text(input);

    assert!(result.is_ok());

    let (remaining, token1) = result.unwrap();
    assert_eq!(token1.extra, LexTag::Word());
    assert_eq!(token1.fragment(), &"name1");

    let result = lext_text(remaining);
    
    // This should fail because `{` is an invalid token
    assert!(result.is_err());
}