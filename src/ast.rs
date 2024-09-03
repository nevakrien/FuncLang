#![allow(dead_code,unused_imports)]

use crate::token::{TokenSlice,LexToken,LexTag,BinaryOp};
use crate::errors::{UserSideError,UserSideWarning};
use nom_locate::LocatedSpan;

pub enum KeyWord<'a>{
	Nil(LocatedSpan<&'a str>),
	
	Return(LocatedSpan<&'a str>),
	FuncDec(LocatedSpan<&'a str>),
	Lamda(LocatedSpan<&'a str>),
	
	If(LocatedSpan<&'a str>),
	Else(LocatedSpan<&'a str>),
	
	Cond(LocatedSpan<&'a str>),
	Match(LocatedSpan<&'a str>),
}

pub enum GrammerNode<'a, 'b> {
	//non terminal
	Unprocessed(TokenSlice<'a,'b>),
	KeyWord(KeyWord<'a>),
	ControlBlock(Block<'a,'b>),
	Else(Else<'a,'b>),

	//hybrid
	Sequence(Vec<GrammerNode<'a, 'b>>),
	Val(Value<'a,'b>), //some values such as parthesis or varibles can be non terminal
	

	//terminal
	Return(Return<'a,'b>),
	Declare(Assign<'a,'b>),
	Function(FuncDef<'a,'b>),

	
	//poison
	Error(UserSideError<'a>),

	//warnings
	Warning(UserSideWarning<'a>),
}

pub enum Value<'a,'b>{
	Basic(LexToken<'a>),//not all tags are valid. up to the programer to make sure only the right types are here
	Nil(KeyWord<'a>),

	Var(Varible<'a>),
	Paren(ParenExpr<'a,'b>),
	Op(BinaryOpNode<'a,'b>),
	
	Call(FunctionCall<'a,'b>),

	If(If<'a,'b>),
	Func(Lamda<'a,'b>),
	// need to implement Match and Cond

}

pub struct Varible<'a> {
    pub name: LocatedSpan<&'a str>,
    pub count: usize, //if the var is shadowed starts at 0 signifiying unknowen shadowing then assined a value
}

pub struct FunctionCall<'a,'b>{
	pub piped : Option<Box<Value<'a,'b>>>,
	pub func : Func<'a,'b>,
	pub par : Box<ParenExpr<'a,'b>>,
}

//this is unified for implicit returns and explicit returns
pub struct Return<'a,'b>{
	pub word: Option<KeyWord<'a>>,//return
	pub value: Option<Value<'a,'b>>,
	pub ender: Option<LocatedSpan<&'a str>>, //;
}

pub struct BinaryOpNode<'a, 'b> {
    pub left: Option<Box<GrammerNode<'a, 'b>>>, // Left-hand operand, might be `None` if missing
    pub operator: LexToken<'a>, // The binary operator itself
    pub right: Option<Box<GrammerNode<'a, 'b>>>, // Right-hand operand, might be `None` if missing
}

pub struct Assign<'a, 'b> {
    pub left: Varible<'a>,
    pub operator: LocatedSpan<&'a str>, //=
    pub right: Option<Box<Value<'a, 'b>>>, // Right-hand operand, might be `None` if missing
    pub ender: Option<Box<LocatedSpan<&'a str>>>, //specifcly the ; at the end
}


//for {... ?(})
pub struct ParenExpr<'a,'b>{ 
	pub start: Option<LocatedSpan<&'a str>>, //this can be non for things like def ) {...}
	pub inner: Option<Box<GrammerNode<'a,'b>>>,
	pub end: Option<LocatedSpan<&'a str>>,//stays None untill we fined a closer (if we even find it)
}

//ifs and lamda have the exact same syntax... other than the keyword. shared logic here
pub struct Block<'a,'b>{ 
	pub start: Option<Box<ParenExpr<'a,'b>>>, //note we use specifcly () braces here
	pub body: Option<Box<ParenExpr<'a,'b>>>, //note we use specifcly {} braces here
}

pub struct If<'a,'b>{
	pub word : KeyWord<'a>,
	pub body: Block<'a,'b>,
	pub else_block : Option<Box<Else<'a,'b>>> //if not there implicitly returns nil
}

pub struct Else<'a,'b>{
	pub word : KeyWord<'a>,
	pub body: ParenExpr<'a,'b>,
}

//add match and cond

pub enum Func<'a,'b>{
	Defed(&'b FuncDef<'a,'b>),
	Vared(Varible<'a>),
	Anon(Lamda<'a,'b>),
}

pub struct Lamda<'a,'b>{
	pub word : KeyWord<'a>,
	pub body: Block<'a,'b>,
}

pub struct FuncDef<'a,'b>{
	pub word : KeyWord<'a>,
	pub name : Option<LocatedSpan<&'a str>>,
	pub body: Block<'a,'b>,
}