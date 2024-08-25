use nom::bytes::complete::{is_a,take_till,take_while,take_while1};
use nom::sequence::{pair,preceded,delimited};
use nom::combinator::recognize;
use nom::character::complete::{digit1};

use nom::multi::fold_many0;
use nom::branch::alt;	
use nom::character::complete::one_of;

use crate::errors::{Cursor,CResult,strip_reporting,UserSideError};
use nom::combinator::opt;
use  nom_locate::LocatedSpan;
use nom::InputTake;
use nom::Offset;




pub type LexToken<'a> = LocatedSpan<&'a str,LexTag>;

#[allow(dead_code)]
#[derive(Debug,PartialEq,Clone)]
pub enum LexTag {
	Comment(),
	Word(),
	Float(f64),
	Int(i64),
}

#[allow(dead_code)]
pub fn lext_text<'a>(input: Cursor<'a>) -> CResult<'a,LexToken<'a>>{
	//TODO add extra token
	alt((lex_comment,lext_word))(skip_whitespace(input))
}

fn skip_whitespace<'a>(input: Cursor<'a>) -> Cursor<'a> {
	fn typed_take_whitespace<'a>(input: Cursor<'a>) -> CResult<'a,Cursor<'a>>{
		take_while( |c:char| c.is_whitespace())(input)
	}
	let s=opt(typed_take_whitespace)(input);
	let(ans,_)=s.unwrap();
	return ans;
}

fn lex_comment<'a>(input: Cursor<'a>) -> CResult<'a,LexToken<'a>>{
	delimited(
		is_a("#"),
		take_till(|c| c=='\n'),
		take_while(|c| c=='\n')
	)(input)	
	.map(|(i,x)| 
		(i,strip_reporting(x).map_extra(|()| LexTag::Comment()))
	)
}


fn lext_word<'a>(input: Cursor<'a>) -> CResult<'a,LexToken<'a>> {
	recognize(pair(
		take_while1(|c:char| c.is_alphabetic()   || c=='_'),
		take_while( |c:char| c.is_alphanumeric() || c=='_')
	))(input)
	.map(|(i,x)| 
		(i,strip_reporting(x).map_extra(|()| LexTag::Word()))
	)		
}



fn lex_int<'a>(input: Cursor<'a>) -> CResult<'a, LexToken<'a>> {
    // Check for an optional sign
    let (remaining_input, sign) = opt(one_of("+-"))(input.clone())?;
    let (remaining_input, value) = uint_underscored(remaining_input)?;

    let mut overflowed = false;
    let signed_value = value.try_into().unwrap_or_else(|_| {
        overflowed = true;
        i64::MAX //probably not used
    });

    let signed_value = match sign {
        Some('-') => -signed_value,
        _ => signed_value, 
    };

    // Calculate the length of the consumed input including the sign
    let consumed_len = input.offset(&remaining_input);
    let (_, consumed_token) = input.take_split(consumed_len);
    let token_base = strip_reporting(consumed_token);

    if overflowed {
    	input.extra.diag.report_error(UserSideError::IntOverflowError(
            token_base.clone(), value)
        );
    }

    let lex_token = token_base.map_extra(|()| {
        LexTag::Int(signed_value)
    });    

    Ok((remaining_input, lex_token))
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
                	0//so that we dont have signed overflow causing a double report
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
fn test_lex_int_with_signs() {
    let diag = Diagnostics::new();

    // Test positive number with explicit plus sign
    let input = make_cursor("+1234", &diag);
    let result = lex_int(input.clone());
    assert_eq!(
        result,
        Ok((input.take_split(5).0, strip_reporting(input.take_split(5).1).map_extra(|()| LexTag::Int(1234))))
    );

    // Test negative number
    let input = make_cursor("-5678", &diag);
    let result = lex_int(input.clone());
    assert_eq!(
        result,
        Ok((input.take_split(5).0, strip_reporting(input.take_split(5).1).map_extra(|()| LexTag::Int(-5678))))
    );

    // Test number without sign (implicit positive)
    let input = make_cursor("9876", &diag);
    let result = lex_int(input.clone());
    assert_eq!(
        result,
        Ok((input.take_split(4).0, strip_reporting(input.take_split(4).1).map_extra(|()| LexTag::Int(9876))))
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