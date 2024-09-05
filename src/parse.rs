use crate::token::{TokenSlice,LexToken,LexTag};
use crate::ast::{GrammerNode,GrammerNodeBase,ParenExpr,Value,KeyWord,Block};
use nom::IResult;
use crate::errors::{UserSideError};
use nom_locate::LocatedSpan;

use nom::sequence::tuple;
use nom::combinator::opt;

use nom::bytes::complete::take;
use nom::bytes::complete::{take_till,take_while};
use nom::InputLength;
use crate::ast::FuncDef;

use nom::{Err::Error};

// // use crate::errors::TResult;
fn is_paren<'a>(x:&LexToken<'a>) -> bool {
	match x.tag {
		LexTag::Delimiter(_) => true,
		_ => false,
	}
}

// enum AssumeResult<'a,'b> {
// 	None,
// 	Wrong(TokenSlice<'a,'b>,TokenSlice<'a,'b>),
// 	Valid(TokenSlice<'a,'b>,TokenSlice<'a,'b>),
// }

// fn is_opener(c:char) -> bool {
// 	match c {
// 		'{' => true,
// 		'[' => true,
// 		'(' => true,

// 		')' => false,
// 		']' => false,
// 		'}' => false,

// 		_ => unreachable!()
// 	}
// }

// fn get_closer(c:char) -> char {
// 	match c {
// 		'{' => '}',
// 		'[' => ']',
// 		'(' => ')',

// 		_ => unreachable!()
// 	}
// }
pub type TResult<'a,'b,T> = nom::IResult<TokenSlice<'a,'b>, T, ()>;
pub type GResult<'a,'b> = nom::IResult<TokenSlice<'a,'b>, GrammerNode<'a,'b>,()>;
pub type RawResult<'a,'b> = nom::IResult<TokenSlice<'a,'b>, TokenSlice<'a,'b>,()>;

#[allow(dead_code)]
pub fn parse<'a,'b>(input:TokenSlice<'a,'b>) -> GResult<'a,'b> {
	// let start :GrammerNode<'a,'b> = GrammerNodeBase::Unprocessed(input).into();
	// start
	if input.input_len() == 0 {
		return return Err(Error(()));
	}
	match parse_outer_scope(input.clone()) {
		Err(_) => Ok((TokenSlice::new(&[]),GrammerNodeBase::Unprocessed(input).into())),
		Ok((input,res)) => {
			let x = handle_outer(res);
			todo!()
		}//Some(handle_outer(input,res)),
	}
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

struct OuterExp<'a,'b> {
	pub keyword : KeyWord<'a>,
	pub body : TokenSlice<'a,'b>,
}

fn parse_outer_scope<'a,'b>(input:TokenSlice<'a,'b>) -> TResult<'a,'b,OuterExp<'a,'b>> {
	let (input,word) = parse_outer_keyword(input)?;
	let (input,remainder) = take_till(is_outer_keyword)(input)?;

	Ok((input,OuterExp{keyword: word,body: remainder}))	
}

fn handle_outer<'a,'b>(outer: OuterExp<'a,'b>) -> GrammerNode<'a,'b> {
	match outer.keyword.clone() {
		KeyWord::Import(_) => todo!(),
		KeyWord::FuncDec(_) => {

			let (input,name,error) = match outer.body.take_err(1usize) {
				Ok((input,res)) => match res[0].tag {
					LexTag::Word() => {
						if match_keyword(res[0].span).is_none() {
							(input,Some(res[0].span),None)
						} else {
							let error = UserSideError::ReservedName(res[0].span);
							(input,None,Some(error))
						}
					},
					LexTag::Delimiter(_) => {
						let error = UserSideError::MissingFuncName(outer.keyword.get_span());
						(outer.body,None,Some(error))
					}, 
					_ => {
						let error = UserSideError::UnexpectedNameTok(res[0].clone());
						(input,None,Some(error))
					}
				}

				Err(_) => {
					let node :GrammerNode<'a,'b> = GrammerNodeBase::KeyWord(outer.keyword.clone()).into();
					return node.with_error(
						UserSideError::EmptyFuncDef(outer.keyword.get_span())
					);
				}
			};

			// let ans = FuncDef{
			// 	keyword:outer.keyword,
			// 	name: name,
			// };
			todo!()
			// GrammerNodeBase::Function(FuncDef{
			// 	word:keyword,
			// 	name:opt(parse_word)
			// })
		}
		_ => unreachable!()
	}
}

fn parse_assumed_paren<'a,'b>(input:TokenSlice<'a,'b>) -> GResult<'a,'b> {
	
	let (input,(extra,del)) = tuple((
		// take_while(is_comment),
		take_till(is_paren),
		// take_while(is_comment),
		opt(take(1usize))
		)
	)(input)?;
	let error = match extra.input_len(){
		0 => None,
		_ => Some(UserSideError::UnexpectedTokens(extra.spans()))
	};
	// let ans = GrammerNodeBase::Par(ParenExpr{

	// })
	todo!()
}
// enum AssumeWord<'a> {
// 	// WrongTag(LexToken<'a>),
// 	KeyWord(KeyWord<'a>),
// 	Name(LexToken<'a>),
// }

// fn parse_word<'a,'b>(input:TokenSlice<'a,'b>) -> TResult<'a,'b,AssumeWord<'a>> {
// 	let (input,slice) = take(1usize)(input)?;
// 	let x = slice[0].clone();
// 	let ans = match x.tag {
// 		LexTag::Word() => match match_keyword(x.span) {
// 			None => AssumeWord::Name(x),
// 			Some(kw) => AssumeWord::KeyWord(kw),

// 		},
// 		// _ => AssumeWord::WrongTag(x)
// 		_ => {return Err(Error(()));}
// 	};
// 	Ok((input,ans))
// }

fn match_keyword<'a>(x:LocatedSpan<&'a str>) -> Option<KeyWord<'a>> {
	match *x.fragment() {
		"null" | "nil" => Some(KeyWord::Nil(x)),
	
		"import" => Some(KeyWord::Import(x)),

		"return" => Some(KeyWord::Return(x)),
		"def" => Some(KeyWord::FuncDec(x)),
		"fn" | "lamda" => Some(KeyWord::Lamda(x)),
		
		"if" => Some(KeyWord::If(x)),
		"else" => Some(KeyWord::Else(x)),
		
		"cond" => Some(KeyWord::Cond(x)),
		"match" => Some(KeyWord::Match(x)),
		_ => None,
	}
}



// fn assume_block<'a,'b>(input:TokenSlice<'a,'b>) -> GrammerNode<'a,'b> {

// }

// fn parse_del<'a,'b>(input:TokenSlice<'a,'b>) -> GResult<'a,'b> {
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
    let(input,_third) = parse_outer_scope(input).unwrap();
    let(input,_forth) = parse_outer_scope(input).unwrap();
    let(input,fith) = parse_outer_scope(input).unwrap();

    assert!(parse_outer_scope(input).is_err());
    
    {
    	let s = first.body;
    	// println!("{:?}",s);
    	assert!(s[s.input_len() - 2].error.is_some())
    }

    assert!(second.body.input_len()==0);
    assert!(fith.body.input_len()==1);

}	