use crate::token::{TokenSlice,LexToken,LexTag};
use crate::ast::{GrammerNode,GrammerNodeBase,ParenExpr,Value,KeyWord};
use nom::bytes::complete::take_while;
use nom::IResult;
// use crate::errors::{UserSideError};

use nom::bytes::complete::take;
use nom::bytes::complete::take_till;
use nom::InputLength;


use nom::{Err::Error};

// // use crate::errors::TResult;


fn is_opener(c:char) -> bool {
	match c {
		'{' => true,
		'[' => true,
		'(' => true,

		')' => false,
		']' => false,
		'}' => false,

		_ => unreachable!()
	}
}

// fn get_closer(c:char) -> char {
// 	match c {
// 		'{' => '}',
// 		'[' => ']',
// 		'(' => ')',

// 		_ => unreachable!()
// 	}
// }

pub type RawResult<'a,'b> = nom::IResult<TokenSlice<'a,'b>, GrammerNode<'a,'b>,()>;

#[allow(dead_code)]
pub fn parse<'a,'b>(input:TokenSlice<'a,'b>) -> GrammerNode<'a,'b> {
	let start :GrammerNode<'a,'b> = GrammerNodeBase::Unprocessed(input).into();
	start
}

fn parse_outer_keyword<'a,'b>(input:TokenSlice<'a,'b>) -> IResult<TokenSlice<'a,'b>,KeyWord<'a>,()>{
	let (input,word_slice) = take(1usize)(input)?;
	let word_token = &word_slice[0];
	match word_token.tag {
		LexTag::Word() => {},
		_ => {return Err(Error(()));}
	};

	let keyword = match *word_token.span.fragment() {
		"import" => KeyWord::Import(word_token.span),
		"def" => KeyWord::FuncDec(word_token.span),
		_ => {return Err(Error(()));},
	};

	Ok((input,keyword))
}

fn is_outer_keyword<'a,'b>(token:&LexToken<'a>) -> bool{
	match token.tag {
		LexTag::Word() => {},
		_ => {return false}
	};

	match *token.span.fragment() {
		"def" | "import"=> true,
		_ => false
	}
}

#[allow(dead_code)]
fn parse_outer_scope<'a,'b>(input:TokenSlice<'a,'b>) -> RawResult<'a,'b> {
	let (input,word) = parse_outer_keyword(input)?;
	let (input,remainder) = take_till(is_outer_keyword)(input)?;

	let word = GrammerNodeBase::KeyWord(word).into();
	let remainder = GrammerNodeBase::Unprocessed(remainder).into();

	Ok((input,vec![word,remainder].into()))
}

// fn parse_del<'a,'b>(input:TokenSlice<'a,'b>) -> RawResult<'a,'b> {
// 	let (input,del_slice) = take(1usize)(input)?;
// 	let del = &del_slice[0];

// 	let c : char = match del.tag { 
// 		LexTag::Delimiter(c) => c,
// 		_ => {return Err(Error(()));}
// 	};

// 	let ans = if is_opener(c) {
// 		ParenExpr{
// 			start:Some(del.span),
// 			inner: None,
// 			end: None,
// 		}
// 	} else {
// 		ParenExpr{
// 			start: None,
// 			inner: None,
// 			end: Some(del.span),
// 		}
// 	};

// 	let mut ans_node:GrammerNode<'a,'b> = GrammerNodeBase::Val(Value::Paren(ans)).into();
// 	ans_node.error=del.error.clone();
// 	Ok((input,ans_node))
// }
#[cfg(test)]
use crate::lex_full_text;

#[test]
#[no_mangle]
fn test_parse_outer() {
    let input_str = r#"
        def (aaa) {
        	(other + content) / "string"
        	"unfinished string
        }
        import
        def ;
        def { 
        	ssas 
        	match if
        import something

    "#;

    let lexed = lex_full_text(input_str);
    let input = TokenSlice::new(&lexed);

    let(input,first) = parse_outer_scope(input).unwrap();
    let(input,second) = parse_outer_scope(input).unwrap();
    let(input,third) = parse_outer_scope(input).unwrap();
    let(input,forth) = parse_outer_scope(input).unwrap();
    let(input,fith) = parse_outer_scope(input).unwrap();

    assert!(parse_outer_scope(input).is_err());
    
    match first.base {
    	GrammerNodeBase::Sequence(v) => {
    		assert!(v.len()==2);
    		match &v[1].base {
    			GrammerNodeBase::Unprocessed(s) => {
    				println!("{:?}",s);

    				assert!(s[s.input_len() - 2].error.is_some())
    			}
    			_ => unreachable!()
    		}
    	},
    	_ => unreachable!()
    }

    match second.base {
    	GrammerNodeBase::Sequence(v) => {
    		assert!(v.len()==2);
    		match &v[1].base {
    			GrammerNodeBase::Unprocessed(s) => {
    				println!("{:?}",s);
    				assert!(s.input_len()==0);
    			}
    			_ => unreachable!()
    		}
    	},
    	_ => unreachable!()
    }

    match fith.base {
    	GrammerNodeBase::Sequence(v) => {
    		assert!(v.len()==2);
    		match &v[1].base {
    			GrammerNodeBase::Unprocessed(s) => {
    				println!("{:?}",s);
    				assert!(s.input_len()==1);
    			}
    			_ => unreachable!()
    		}
    	},
    	_ => unreachable!()
    }
}	