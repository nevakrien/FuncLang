use crate::token::{TokenSlice,LexToken,LexTag,BinaryOp};
pub struct ParData<'a,'b> {
	pub inner:Vec<ParExp<'a,'b>>,
	pub start: LexToken<'a>,
	pub end: Option<LexToken<'a>>
}

impl<'a,'b> ParData<'a,'b> {
	pub fn get_char(&self) -> char {
		match self.start.tag(){
			LexTag::Delimiter(c) => c,
			_ => unreachable!()
		}
	}
}

pub enum ParExp<'a,'b> {
	Leaf(TokenSlice<'a,'b>),
	Exp(ParData<'a,'b>),
	// Unclosed(LocatedSpan<&'a str>,char),
}