use crate::token::{Cursor,TokenSlice,LexToken,LexTag};
use crate::ast::{ParExp,ParData};
use crate::errors::{UserSideError};

use nom::bytes::complete::take;
use nom::bytes::complete::take_till;
use nom::combinator::map;

use nom::InputLength;

use nom::{Err::Error};

use crate::errors::TResult;


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

fn get_closer(c:char) -> char {
	match c {
		'{' => '}',
		'[' => ']',
		'(' => ')',

		_ => unreachable!()
	}
}

//never errors
fn gather_till_del<'a,'b>(input:Cursor<'a,'b>) -> TResult<'a,'b,TokenSlice<'a,'b>> {
	// gather_till_parclose(input,closer,vec)
	fn is_par<'a>(x:&LexToken<'a>) -> bool{
		match x.tag() {
			LexTag::Delimiter(_) => true,
			_ => false
		}
	}
	map(take_till(is_par),|c:Cursor<'a,'b>| c.strip_diag())(input)
}

/*
currently we are not really catching misshaped parthesis well
if its all of the same type its fine but as soon as u start mixing them errors will look funny.
its probably best to rewrite the whole thing to be less effishent but more functional
that way we can pattern match for common expressions

this would be fairly expensive because we are going to be cloning a vector back and forth way too match but I belive its worth it.

*/
//never errors
fn handle_open_par<'a,'b>(mut input:Cursor<'a,'b>,par_tok:LexToken<'a>,close_char: char)-> TResult<'a,'b,ParExp<'a,'b>>{
	let mut open = true;
	let mut cur_tok = par_tok.clone();
	let mut parts = Vec::<ParExp>::new();

	while input.input_len() > 0 && open{
		let (input2,next) = gather_till_del(input)?;//never errors

		let ret = match input2.take_err(1usize) {
			Err(_)  => {
				input=input2;
				cur_tok=next.clone().last().unwrap().clone();
				parts.push(ParExp::Leaf(next.into()));
				break;
			},
			Ok((input3,ret)) => {
				input = input3; 
				cur_tok = ret[0].clone(); 
				ret
			} 
		};

		parts.push(ParExp::Leaf(next.into()));


		match ret[0].tag(){
			LexTag::Delimiter(c) => {
				if c == close_char {
					open=false;
					break;
				}
				let (inp,op) = handle_par(input.clone(),ret[0].clone(),c)?;
				input=inp;
				match op{
					None => {},
					Some(x) => {parts.push(x);}
				}

			}
			_ => unreachable!(),
		}
	}

	if open {
		input.report_error(UserSideError::UnclosedPar(
			par_tok.span(),
			cur_tok.span()
	    ));

		let ans =ParExp::Exp(ParData{
			start:par_tok,
			inner:parts,
			end: None,
		});
		Ok((input,ans))
	}
	else{
		let ans =ParExp::Exp(ParData{
			start:par_tok,
			inner:parts,
			end: Some(cur_tok),
		});
		Ok((input,ans))

	}	
}

//never errors
fn handle_par<'a,'b>(input:Cursor<'a,'b>,par_tok:LexToken<'a>,par_char: char)-> TResult<'a,'b,Option<ParExp<'a,'b>>>{
	if is_opener(par_char) {
		let (input,res) = handle_open_par(input,par_tok,get_closer(par_char))?;
		Ok((input,Some(res)))
	}
	else{ 
		input.report_error(
			UserSideError::ExtraPar(
            	par_tok.span()
        	)
		);
		Ok((input,None))
	}
}

fn par<'a,'b>(input:Cursor<'a,'b>) -> TResult<'a,'b,Option<ParExp<'a,'b>>>{
	let (input, ret) = take(1usize)(input)?;
	match ret[0].tag() {
		LexTag::Delimiter(c) => handle_par(input,ret[0].clone(),c),//Ok((input,ret)),
		_ => Err(Error(()))
	}
}