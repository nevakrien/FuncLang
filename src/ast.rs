use crate::token::{TokenSlice,LexToken,LexTag,BinaryOp};
pub enum ParExp<'a,'b> {
	Leaf(TokenSlice<'a,'b>),
	Exp(Vec<ParExp<'a,'b>>,char),
}