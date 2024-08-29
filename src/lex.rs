use nom::bytes::complete::{is_a,take_till,take_while,take_while1,tag};
use nom::sequence::{pair,preceded};
use nom::combinator::recognize;
use nom::character::complete::{digit1,one_of,anychar};

use nom::multi::fold_many0;
use nom::branch::alt;	

// use nom::character::complete::char as nom_char;

use crate::errors::{Cursor,CResult,strip_reporting,UserSideError};
use nom::combinator::{opt};
// use nom::bytes::complete::is_not;
use nom::InputTake;
use nom::Offset;

use crate::errors::{Diagnostics,make_cursor};

use crate::token::{LexToken,BinaryOp,LexTag};

#[no_mangle]
//#[allow(dead_code)]
pub fn lex_full_text<'a>(input: &'a str,diag :&'a Diagnostics<'a>) -> Vec<LexToken<'a>> {
    let mut cursor = make_cursor(input,diag);
    let mut ans = Vec::new();
    loop {
        match lext_text(cursor) {
            Ok((new_cursor, token)) => {
                ans.push(token);
                cursor=new_cursor;
            }
            Err(_) => {
                break;
            }            
        }
    }
    ans
}

#[no_mangle]
//#[allow(dead_code)]
pub fn lext_text<'a>(input: Cursor<'a>) -> CResult<'a,LexToken<'a>>{
	//TODO add extra token
	//order is from most common to least
	alt((
		lex_word,
        lex_atom,
        lex_ender,
		lex_delimiter,
		lex_operator,
		lex_comment,
		lex_number,
        lex_string,
        lex_unknowen,
	))(skip_whitespace(input))
}
fn lex_unknowen<'a>(input: Cursor<'a>) -> CResult<'a,LexToken<'a>>{
    let (input,x)=recognize(pair(anychar,take_while(|c:char| !c.is_ascii())))(input)?;
    input.extra.diag.report_error(UserSideError::UnokwenToken(strip_reporting(x.clone())));
    let ans = strip_reporting(x).map_extra(|()| LexTag::Unknowen());
    Ok((input,LexToken::new(ans)))
}

fn lex_atom<'a>(input: Cursor<'a>) -> CResult<'a,LexToken<'a>> {
    let (input,s) = recognize(preceded(
        one_of("%:"),
        pair(
            take_while1(|c:char| c.is_alphabetic()   || c=='_'),
            take_while( |c:char| c.is_alphanumeric() || c=='_')
        )
    ))(input)?;
    let ans = strip_reporting(s).map_extra(|()| {
        LexTag::Atom()
    });
    Ok((input, LexToken::new(ans)))
}

fn skip_whitespace<'a>(input: Cursor<'a>) -> Cursor<'a> {
	fn typed_take_whitespace<'a>(input: Cursor<'a>) -> CResult<'a,Cursor<'a>>{
		take_while( |c:char| c.is_whitespace())(input)
	}
	let s=opt(typed_take_whitespace)(input);
	let(ans,_)=s.unwrap();
	return ans;
}

fn lex_delimiter<'a>(input: Cursor<'a>) -> CResult<'a,LexToken<'a>>{
	let(input,token) = recognize(one_of("{}[]()"))(input)?;
	let ans = strip_reporting(token.clone()).map_extra(|()| {
		LexTag::Delimiter(token.fragment().chars().next().unwrap())
	});
	Ok((input,LexToken::new(ans)))
}

fn lex_ender<'a>(input: Cursor<'a>) -> CResult<'a,LexToken<'a>>{
    let(input,token) = recognize(one_of(";,"))(input)?;
    let ans = strip_reporting(token.clone()).map_extra(|()| {
        LexTag::Ender(token.fragment().chars().next().unwrap())
    });
    Ok((input,LexToken::new(ans)))
}

fn lex_operator<'a>(input: Cursor<'a>) -> CResult<'a, LexToken<'a>> {
    let (input, token) = alt((
        // 1. Single-char operators with no associated double-char version
        recognize(one_of("+/.%")),

        // 2. Multi-character operators
        recognize(tag("|>")),
        recognize(tag("**")),
        recognize(tag("&&")),
        recognize(tag("||")),
        recognize(tag("^^")),
        recognize(tag("==")),
        recognize(tag("!=")),
        recognize(tag("<=")),
        recognize(tag(">=")),
        recognize(tag("=>")),
        recognize(tag("->")),
        recognize(tag("::")),

        // 3. Remaining single-char operators
        recognize(one_of("-=*<>|:^")),
    ))(input)?;

    let op_tag = match *token.fragment() {
        // Single-char operators 
        "+" => LexTag::Op(BinaryOp::Add),
        "-" => LexTag::Op(BinaryOp::Sub),
        "*" => LexTag::Op(BinaryOp::Mul),
        "/" => LexTag::Op(BinaryOp::Div),

        "::" => LexTag::Op(BinaryOp::DoubleDots),
        ":" => LexTag::Op(BinaryOp::Dots),

        "%" => LexTag::Op(BinaryOp::Mod),

        "**" | "^" => LexTag::Op(BinaryOp::Exp),


        "<" => LexTag::Op(BinaryOp::Smaller),
        ">" => LexTag::Op(BinaryOp::Bigger),
        "=" => LexTag::Op(BinaryOp::OneEqul),
        
        "." => LexTag::Op(BinaryOp::Dot),
        "|" => LexTag::Op(BinaryOp::SingleOr),

        // Multi-char operators
        "|>" => LexTag::Op(BinaryOp::Pipe),
        "&&" => LexTag::Op(BinaryOp::And),
        "||" => LexTag::Op(BinaryOp::Or),
        "^^" => LexTag::Op(BinaryOp::Xor),
        "==" => LexTag::Op(BinaryOp::TwoEqul),
        "!=" => LexTag::Op(BinaryOp::NotEqual),
        "<=" => LexTag::Op(BinaryOp::SmallerEqual),
        ">=" => LexTag::Op(BinaryOp::BiggerEqual),
        "=>" => LexTag::Op(BinaryOp::FatArrow),
        "->" => LexTag::Op(BinaryOp::SmallArrow),
        
        _ => unreachable!(),
    };

    let ans = strip_reporting(token.clone()).map_extra(|()| op_tag);
    Ok((input, LexToken::new(ans)))
}

fn lex_comment<'a>(input: Cursor<'a>) -> CResult<'a,LexToken<'a>>{
	recognize(preceded(
		is_a("#"),
		take_till(|c| c=='\n'),
		// take_while(|c| c=='\n')
	))(input)	
	.map(|(i,x)| 
		(i,LexToken::new(strip_reporting(x).map_extra(|()| LexTag::Comment())))
	)
}


fn lex_word<'a>(input: Cursor<'a>) -> CResult<'a,LexToken<'a>> {
	recognize(pair(
		take_while1(|c:char| c.is_alphabetic()   || c=='_'),
		take_while( |c:char| c.is_alphanumeric() || c=='_')
	))(input)
	.map(|(i,x)| 
		(i,LexToken::new(strip_reporting(x).map_extra(|()| LexTag::Word())))
	)		
}

fn skip_to_str_end(input: &str, del: char) -> Result<usize, usize> {
    assert!(del.is_ascii());

    let mut chars = input.chars().peekable();
    let mut count = 0;

    while let Some(c) = chars.next() {
        // Increment count for each character processed
        count += 1;

        if c == '\\' {
            // If a backslash is found, skip the next character (escape sequence)
            if let Some(c2) = chars.next() {
                if c2== '\n'{
                    return Err(count);
                }
                count += 1; // Count the escaped character as well
                continue;
            } else {
                // If backslash is the last character, return an error with it
                return Err(count);
            }
        } else if c == del {
            // If the delimiter is found and it's not escaped, return the count
            return Ok(count);
        } else if c== '\n'{
            return Err(count);
        }
    }

    // If end of string is reached without finding an unescaped delimiter, return an error with the last character
    Err(count)
}

fn lex_string<'a>(input: Cursor<'a>) -> CResult<'a,LexToken<'a>> {
    let original_input = input.clone();

    let (input,del) = one_of("\"'")(input)?;
    match skip_to_str_end(input.fragment(),del) {
        Ok(u) => {
            let (input,ans) = original_input.take_split(u+1);//original del dels
            Ok((input,LexToken::new(ans.map_extra(|_| {
                LexTag::String(del)
            }))))    
        }
        Err(u) => {
            let (input,ans) = original_input.take_split(u+1);
            input.extra.diag.report_error(UserSideError::UnclosedString(
                strip_reporting(ans.clone()),del
            ));

            Ok((input,LexToken::new(ans.map_extra(|_| {
                LexTag::String(del)
            }))))
        }
    }
}

fn after_dot_to_float(digits: u64) -> f64 {
    if digits == 0 {
        return 0.0;
    }

    // Calculate the number of digits using logarithm
    let num_digits = (digits as f64).log10().floor() as u32 + 1;

    // Calculate the divisor
    let divisor = 10_u64.pow(num_digits);

    // Convert to fraction
    digits as f64 / divisor as f64
}

fn lex_number<'a>(input: Cursor<'a>) -> CResult<'a, LexToken<'a>>{
	let (remaining_input, sign_char) = opt(one_of("+-"))(input.clone())?;
    let (remaining_input, value) = uint_underscored(remaining_input)?;
    let (remaining_input,dot) = opt(is_a("."))(remaining_input)?;

    let sign = match sign_char {
        Some('-') => -1i64,
        _ => 1i64, 
    };

    

    match dot {
    	None => {

    		let consumed_len = input.offset(&remaining_input);
    		let (_, consumed_token) = input.take_split(consumed_len);
    		let token_base = strip_reporting(consumed_token);
			
			let signed_value = sign*value.try_into().unwrap_or_else(|_| {
    			input.extra.diag.report_error(
    				UserSideError::IntOverflowError(
            			token_base.clone(), 
            			value
            		)
            	);
   				i64::MAX //probably not used
			});

			let lex_token = token_base.map_extra(|()| {
			    LexTag::Int(signed_value)
			});    

    		Ok((remaining_input, LexToken::new(lex_token)))
    	}

    	Some(_) => {
            let mut fval=value as f64;

    		let remaining_input=match uint_underscored(remaining_input.clone()){
    			Err(_) => {
    				remaining_input
    			},
    			Ok((remaining_input,after_dot)) => {
    				
    				fval+=after_dot_to_float(after_dot);
                    remaining_input
    			}
    		};
            fval*=sign as f64;

            let consumed_len = input.offset(&remaining_input);
            let (_, consumed_token) = input.take_split(consumed_len);
            let token_base = strip_reporting(consumed_token);
            
            let lex_token = token_base.map_extra(|()| {
                LexTag::Float(fval)
            });    

            Ok((remaining_input, LexToken::new(lex_token)))
    	}
    }
}

fn uint_underscored<'a>(input: Cursor<'a>) -> CResult<'a,u64>{
	//rust needs some help on figuring out typing so...
    fn typed_digit1<'b>(x: Cursor<'b>) -> CResult<'b, &'b str> {
	    digit1(x).map(|(i,x)| 
			(i,*x.fragment())
		)
	}
    let report_input=strip_reporting(input.clone());
    
    let (input,d)=digit1(input)?;
	let diag = input.extra.diag;
	

	let mut overflowed = false;
    let first_val = {
        d.parse::<u64>().unwrap_or_else(|_| {
            overflowed = true; 
            i64::MAX.try_into().unwrap() //chosen to avoid double overflow reporting
        })
    };
	let (input,ans) = fold_many0(
		alt((
        	preceded(is_a("_"),typed_digit1),
        	typed_digit1,)
        ),
        ||{first_val}, 
        |acc, item| {
        	if !overflowed{
        		acc.checked_mul(10u64.pow(item.len() as u32)) // Adjust multiplier based on number of digits
                .and_then(|acc| acc.checked_add(item.parse::<u64>()
                    .unwrap_or_else(|_| {
                        overflowed = true; 
                        i64::MAX.try_into().unwrap()
                    })
                )) 
                .unwrap_or_else(|| {
                	overflowed = true;
                    i64::MAX.try_into().unwrap()
                })
        	}

        	else{
        		acc
        	}
            
        },
    )(input).unwrap();
    if overflowed {
        // Calculate the start and end of the substring in `report_input`
        let start = report_input.location_offset();
        let end = input.location_offset();
        
        // Slice out the relevant part of the original input
        let relevant_input = report_input.take(end - start);
        
        diag.report_error(UserSideError::OverflowError(relevant_input));
    }
    Ok((input,ans))
}



#[test]
#[no_mangle]
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
#[no_mangle]
fn test_lex_int_with_signs() {
    let diag = Diagnostics::new();

    // Test positive number with explicit plus sign
    let input = make_cursor("+1234", &diag);
    let result = lex_number(input.clone());
    assert_eq!(
        result,
        Ok((input.take_split(5).0, LexToken::new(strip_reporting(input.take_split(5).1).map_extra(|()| LexTag::Int(1234)))))
    );

    // Test negative number
    let input = make_cursor("-5678", &diag);
    let result = lex_number(input.clone());
    assert_eq!(
        result,
        Ok((input.take_split(5).0, LexToken::new(strip_reporting(input.take_split(5).1).map_extra(|()| LexTag::Int(-5678)))))
    );

    // Test number without sign (implicit positive)
    let input = make_cursor("9876", &diag);
    let result = lex_number(input.clone());
    assert_eq!(
        result,
        Ok((input.take_split(4).0, LexToken::new(strip_reporting(input.take_split(4).1).map_extra(|()| LexTag::Int(9876)))))
    );
}

#[test]
#[no_mangle]
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
#[no_mangle]
fn test_lex_basic_string_with_names_and_comments() {
    let diag = Diagnostics::new();

    let input = make_cursor("name1 # This is a comment\nname2 # Another comment", &diag);
    let result = lext_text(input);

    assert!(result.is_ok());

    let (remaining, token1) = result.unwrap();
    assert_eq!(token1.inner.extra, LexTag::Word());
    assert_eq!(token1.inner.fragment(), &"name1");

    let result = lext_text(remaining);
    let (remaining, token2) = result.unwrap();
    assert_eq!(token2.inner.extra, LexTag::Comment());
    
    let result = lext_text(remaining);
    let (remaining, token3) = result.unwrap();
    assert_eq!(token3.inner.extra, LexTag::Word());
    assert_eq!(token3.inner.fragment(), &"name2");

    let result = lext_text(remaining);
    let (remaining, token4) = result.unwrap();
    assert_eq!(token4.inner.extra, LexTag::Comment());
    
    // Make sure no tokens left
    assert_eq!(remaining.fragment(), &"");
}
#[test]
#[no_mangle]
fn test_lex_empty() {
    let diag = Diagnostics::new();

    let input = make_cursor("", &diag);
    let result = lext_text(input);
    assert!(result.is_err());
}

#[test]
#[no_mangle]
fn test_lex_invalid_token_error() {
    let diag = Diagnostics::new();

    let input = make_cursor("name1 🏳️‍⚧️ name2", &diag);
    let result = lext_text(input);

    assert!(result.is_ok());

    let (remaining, token1) = result.unwrap();
    assert_eq!(token1.inner.extra, LexTag::Word());
    assert_eq!(token1.inner.fragment(), &"name1");

    let result = lext_text(remaining);
    
    // This should fail because `🏳️‍⚧️` is an invalid token (for now)
    let (remaining, token2) = result.unwrap();
    assert_eq!(token2.inner.extra, LexTag::Unknowen());

    let (_, token3)=lext_text(remaining).unwrap(); 
    //depending on if the unknowen handeling is good or not. token3 can be anything
    match token3.inner.extra {
        LexTag::Unknowen() | LexTag::Word() => {},
        _ => unreachable!("Unexpected tag"),
    };
}

#[cfg(test)]
#[no_mangle]
fn assert_operator(input: &str, expected_tag: LexTag) {
    let diag = Diagnostics::new();
    let cursor = make_cursor(input, &diag);
    let result = lex_operator(cursor);

    assert!(result.is_ok(), "Failed to parse operator: {}", input);
    let (_, token) = result.unwrap();
    assert_eq!(token.inner.extra, expected_tag, "Expected {:?} but got {:?}", expected_tag, token.inner.extra);
}

#[test]
#[no_mangle]
fn test_single_char_operators() {
    assert_operator("+", LexTag::Op(BinaryOp::Add));
    assert_operator("-", LexTag::Op(BinaryOp::Sub));
    assert_operator("*", LexTag::Op(BinaryOp::Mul));
    assert_operator("/", LexTag::Op(BinaryOp::Div));
    assert_operator("%", LexTag::Op(BinaryOp::Mod));
    assert_operator(".", LexTag::Op(BinaryOp::Dot));
    assert_operator("^", LexTag::Op(BinaryOp::Exp));
    assert_operator("|", LexTag::Op(BinaryOp::SingleOr));


    assert_operator("<", LexTag::Op(BinaryOp::Smaller));
    assert_operator(">", LexTag::Op(BinaryOp::Bigger));
    assert_operator("=", LexTag::Op(BinaryOp::OneEqul));
}

#[test]
#[no_mangle]
fn test_multi_char_operators() {
    assert_operator("|>", LexTag::Op(BinaryOp::Pipe));
    assert_operator("**", LexTag::Op(BinaryOp::Exp));
    assert_operator("&&", LexTag::Op(BinaryOp::And));
    assert_operator("||", LexTag::Op(BinaryOp::Or));
    assert_operator("==", LexTag::Op(BinaryOp::TwoEqul));
    assert_operator("!=", LexTag::Op(BinaryOp::NotEqual));
    assert_operator("<=", LexTag::Op(BinaryOp::SmallerEqual));
    assert_operator(">=", LexTag::Op(BinaryOp::BiggerEqual));
    assert_operator("=>", LexTag::Op(BinaryOp::FatArrow));
    assert_operator("->", LexTag::Op(BinaryOp::SmallArrow));
}

#[test]
#[no_mangle]
fn test_lex_string() {
    let diag = Diagnostics::new();

    // Test a valid string with escaped characters
    let input = make_cursor("\"Hello, world!\\n\" junk", &diag);
    let result = lex_string(input.clone());
    assert!(result.is_ok(), "Failed to parse valid string");
    let (remaining, token) = result.unwrap();
    assert_eq!(token.inner.extra, LexTag::String('"'));
    assert_eq!(token.inner.fragment(), &"\"Hello, world!\\n\"");
    assert_eq!(remaining.fragment().len(), 5, "Unexpected characters remaining after parsing a valid string");

    // Test an unclosed string which should still return a token and log an error
    let input = make_cursor("\"Unclosed string example", &diag);
    let result = lex_string(input.clone());
    assert!(result.is_ok(), "Should return a token despite being unclosed");
    let (_, token) = result.unwrap();
    assert_eq!(token.inner.extra, LexTag::String('"'));
    assert_eq!(token.inner.fragment(), &"\"Unclosed string example");
    assert!(!diag.borrow_errors().is_empty(), "Should log an error for unclosed string");

    // Test a single character string
    let input = make_cursor("'a'", &diag);
    let result = lex_string(input.clone());
    assert!(result.is_ok(), "Failed to parse single character string");
    let (remaining, token) = result.unwrap();
    assert_eq!(token.inner.extra, LexTag::String('\''));
    assert_eq!(token.inner.fragment(), &"'a'");
    assert_eq!(remaining.fragment().len(), 0, "Unexpected characters remaining after parsing a single character string");

    // Test a string with special escaped characters
    let input = make_cursor("\"Escaped \\\" quote\"", &diag);
    let result = lex_string(input.clone());
    assert!(result.is_ok(), "Failed to parse string with escaped quote");
    let (remaining, token) = result.unwrap();
    assert_eq!(token.inner.extra, LexTag::String('"'));
    assert_eq!(token.inner.fragment(), &"\"Escaped \\\" quote\"");
    assert_eq!(remaining.fragment().len(), 0, "Unexpected characters remaining after parsing a string with escaped quote");

    // Test a string containing newlines and tabs
    let input = make_cursor("\"Line1\\nLine2\\tTabbed\"", &diag);
    let result = lex_string(input.clone());
    assert!(result.is_ok(), "Failed to parse string with newlines and tabs");
    let (remaining, token) = result.unwrap();
    assert_eq!(token.inner.extra, LexTag::String('"'));
    assert_eq!(token.inner.fragment(), &"\"Line1\\nLine2\\tTabbed\"");
    assert_eq!(remaining.fragment().len(), 0, "Unexpected characters remaining after parsing a string with newlines and tabs");
}

#[test]
#[no_mangle]
fn test_overflow_errors() {
    let diag = Diagnostics::new();

    // Number that is likely too large, causing overflow
    let input_large_float = make_cursor("999999999999999999999999999999999999999999999999999999999999999999999999999999999.999999999999999999999999999999999999999999999999999999", &diag);
    let input_large_int = make_cursor("9223372036854775808", &diag);  // Just beyond the range of i64 for positive numbers

    // Numbers with underscores
    let input_with_underscores = make_cursor("2_33_1", &diag);
    let input_overflow_with_underscores = make_cursor("9999999999_9999999999_9999999999", &diag);

    // Test large float overflow
    let result_large_float = lex_number(input_large_float.clone());
    assert_eq!(diag.borrow_errors().len(), 2, "Expected two overflow errors logged");

    assert!(result_large_float.is_ok(), "Failed to parse large float with overflow");
    let (_, token_large_float) = result_large_float.unwrap();
    assert!(matches!(token_large_float.inner.extra, LexTag::Float(_)), "Expected a float token despite overflow");

    // Test large int overflow
    let result_large_int = lex_number(input_large_int.clone());
    assert!(result_large_int.is_ok(), "Failed to parse large int with overflow");
    let (_, token_large_int) = result_large_int.unwrap();
    assert!(matches!(token_large_int.inner.extra, LexTag::Int(_)), "Expected an int token despite overflow");

    // Test number with underscores
    let result_with_underscores = lex_number(input_with_underscores.clone());
    assert!(result_with_underscores.is_ok(), "Failed to parse number with underscores");
    let (_, token_with_underscores) = result_with_underscores.unwrap();
    assert_eq!(*token_with_underscores.inner.fragment(), "2_33_1", "Parsed value should ignore underscores");


    // Test overflow with underscores
    let result_overflow_with_underscores = lex_number(input_overflow_with_underscores.clone());
    assert!(result_overflow_with_underscores.is_ok(), "Failed to parse overflow number with underscores");
    let (_, token_overflow_with_underscores) = result_overflow_with_underscores.unwrap();
    assert!(matches!(token_overflow_with_underscores.inner.extra, LexTag::Int(_)), "Expected an int token despite overflow");

    // Check diagnostics for errors
    assert_eq!(diag.borrow_errors().len(), 4, "Expected two overflow errors logged");

    for error in diag.borrow_errors().iter() {
        match error {
            UserSideError::OverflowError(_) | UserSideError::IntOverflowError(_,_) => println!("Logged overflow as expected"),
            _ => panic!("Unexpected error type logged"),
        }
    }
}



#[test]
#[no_mangle]
fn test_lex_text_happy_path() {
    let diag = Diagnostics::new();

    // Prepare an input that combines words, operators, strings, numbers, and comments
    let mut remaining = make_cursor("func + 123 / 2.11_2; 1.\"string\" # aa \" {}comment \n %atom :: :atom", &diag);

    // Expected sequence of tokens
    let expected = vec![
        LexTag::Word(),                      // 'func'
        LexTag::Op(BinaryOp::Add),          // '+'
        LexTag::Int(123),                   // '123'
        LexTag::Op(BinaryOp::Div), 
        LexTag::Float(2.112),
        LexTag::Ender(';'),
        LexTag::Float(1.),
        LexTag::String('"'),                // '"string"'
        LexTag::Comment(),                  // '// comment'
        LexTag::Atom(),                     // ':atom'
        LexTag::Op(BinaryOp::DoubleDots),
        LexTag::Atom(),
    ];

    // Test parsing sequence
    for expected_tag in expected {
        let result = lext_text(remaining);
        assert!(result.is_ok(), "Failed to parse expected token: {:?}", expected_tag);
        let (new_remaining, token) = result.unwrap();
        assert_eq!(token.inner.extra, expected_tag, "Parsed token does not match expected. Expected {:?}, found {:?}", expected_tag, token.inner.extra);
        remaining = new_remaining;  // Update remaining input for the next token
    }

    // Check that there are no more tokens left after parsing
    assert_eq!(remaining.fragment().len(), 0, "Unexpected characters remaining after parsing all tokens");
    assert_eq!(diag.borrow_errors().len(), 0, "Unexpected errors");
    assert_eq!(diag.borrow_warnings().len(), 0, "Unexpected warnings");
}
