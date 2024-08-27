use crate::token::{StaticCursor,Cursor,TokenSlice,LexToken,LexTag,BinaryOp};
use crate::ast::ParExp;
use nom::bytes::complete::take;
use nom::IResult;

use nom::combinator::recognize;
use nom::sequence::preceded;


use nom::Err;
use nom::error::Error;

fn dumb_sketch<'a,'b>(input:Cursor<'a,'b>) ->  IResult<Cursor<'a,'b>,Cursor<'a,'b>>{
	recognize(preceded(par,par))(input)
}

fn par<'a,'b>(input:Cursor<'a,'b>) -> IResult<Cursor<'a,'b>,Cursor<'a,'b>> {
	let (input, ret) = take(1usize)(input)?;
	match ret[0].tag() {
		LexTag::Delimiter(c) => Ok((input,ret)),
		_ => Err(Err::Error(Error::new(input, nom::error::ErrorKind::Tag))),
	}
}