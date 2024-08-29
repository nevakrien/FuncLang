use crate::token::{StaticCursor,Cursor,TokenSlice,LexToken,LexTag,BinaryOp};
use crate::ast::{ParExp,ParData};
use crate::errors::{UserSideError};

use nom::bytes::complete::take;
use nom::bytes::complete::take_till;

use nom::sequence::preceded;
use nom::Offset;
use nom::InputTake;
use nom::InputLength;

use nom::{Err::Error};

use crate::errors::TResult;
use nom_locate::LocatedSpan;
use nom::combinator::map_res;

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


// fn gather_till_parclose<'a,'b>(input:Cursor<'a,'b>,closer : char , vec : &mut Vec<ParExp<'a,'b>>) -> (Cursor<'a,'b>,Option<LexToken<'a>>) {
// 	// gather_till_parclose(input,closer,vec)
// 	fn is_par<'a>(x:&LexToken<'a>) -> bool{
// 		match x.tag() {
// 			LexTag::Delimiter(_) => true,
// 			_ => false
// 		}
// 	}
// 	let (input,res)=take_till::<_, _, ()>(is_par)(input).unwrap(); //this function never fails...
// 	vec.push(ParExp::Leaf(res.strip_diag()));
	
// 	match input.input_len(){
// 		0 => 	 (input,None),
// 		_ => {
// 			let (input,end) = input.take_split(1);
// 			match end.last().unwrap().tag(){
// 				LexTag::Delimiter(c) => {
// 					if c == closer {
// 						(input,Some(end))
// 					}
// 					else{

// 					}
// 				}
// 				_ => unreachable!()
// 			}
// 		},
// 	}
// }

// fn get_last_par<'a,'b>(par_tok:LexToken<'a>,pars: &ParExp<'a,'b>) -> LexToken<'a>{
// 	match pars {
// 		ParExp::Leaf(x) => x.last().unwrap().clone(),
// 		ParExp::Exp(pd) => match pd.inner.last() {
// 			None => par_tok,
// 			Some(p) => get_last_par(par_tok,p)
// 		}
// 	}
// }

// fn handle_par<'a,'b>(input:Cursor<'a,'b>,par_tok:LexToken<'a>,par_char: char) -> TResult<'a,'b,Option<ParExp<'a,'b>>>{
// 	if is_opener(par_char) {
// 		let mut parts = Vec::new();
// 		let (input,close)=gather_till_parclose(input,get_closer(par_char),&mut parts);
		
// 		let errorered = close.is_none();

// 		let ans =ParExp::Exp(ParData{
// 			start:par_tok.clone(),
// 			inner:parts,
// 			end: close
// 		});

// 		if errorered {
// 			let last = get_last_par(par_tok.clone(),&ans);
// 			input.report_error(UserSideError::UnclosedPar(
//                par_tok.span(),last.span())
//             );
// 		}
// 		Ok((input,Some(ans)))

// 	}
// 	else {
// 		input.report_error(UserSideError::ExtraPar(
//                par_tok.span()
//             ));
// 		Ok((input,None))
// 	}
// }

fn gather_till_del<'a,'b>(input:Cursor<'a,'b>) -> (Cursor<'a,'b>,TokenSlice<'a,'b>) {
	// gather_till_parclose(input,closer,vec)
	fn is_par<'a>(x:&LexToken<'a>) -> bool{
		match x.tag() {
			LexTag::Delimiter(_) => true,
			_ => false
		}
	}
	let diag = input.diag;
	let (input,res) = take_till::<_, _, ()>(is_par)(input.strip_diag()).unwrap(); //this function never fails...
	(input.add_diag(diag),res)
}

fn handle_open_par<'a,'b>(input:Cursor<'a,'b>,par_tok:LexToken<'a>,close_char: char)-> (Cursor<'a,'b>,ParExp<'a,'b>){
	let diag = input.diag;
	let original_input = input.clone();
	let mut input = input.strip_diag();

	let mut count = 1;
	let mut parts = Vec::<ParExp>::new();
	let mut head = &mut parts;
	let mut cur_tok = par_tok.clone();

	while input.input_len() > 0 && count > 0{
		let (input2,next) = gather_till_del(input);
		input = input2;
		head.push(ParExp::Leaf(next.into()));

		let (input, ret) = take(1usize)(input).unwrap_or({break;});

		match ret[0].tag(){
			LexTag::Delimiter(c) => {
				if c == close_char {
					count-=1;
				}

				// match handle_par(input,)

			}
			_ => unreachable!(),
		}
	}

	if count > 0 {
		diag.report_error(UserSideError::UnclosedPar(
			par_tok.span(),
			cur_tok.span()
	    ));

		let ans =ParExp::Exp(ParData{
			start:par_tok,
			inner:parts,
			end: None,
		});
		(input,ans)
	}
	else{
		let ans =ParExp::Exp(ParData{
			start:par_tok,
			inner:parts,
			end: Some(cur_tok),
		});
		(input,ans)
	}	
}

fn handle_par<'a,'b>(input:Cursor<'a,'b>,par_tok:LexToken<'a>,par_char: char)-> (Cursor<'a,'b>,Option<ParExp<'a,'b>>){
	if is_opener(par_char) {
		let (input,res) = handle_open_par(input,par_tok,get_closer(par_char));
		(input,Some(res))
	}
	else{ 
		input.report_error(
			UserSideError::ExtraPar(
            	par_tok.span()
        	)
		);
		(input,None)
	}
}

fn par<'a,'b>(input:Cursor<'a,'b>) -> TResult<'a,'b,Option<ParExp<'a,'b>>>{
	let (input, ret) = take(1usize)(input)?;
	match ret[0].tag() {
		LexTag::Delimiter(c) => Ok(handle_par(input,ret[0].clone(),c)),//Ok((input,ret)),
		_ => Err(Error(()))
	}
}