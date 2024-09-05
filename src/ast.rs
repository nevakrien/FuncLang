#![allow(dead_code,unused_imports)]

use crate::token::{TokenSlice,LexToken,LexTag,BinaryOp};
use crate::errors::{UserSideError,UserSideWarning};
use nom_locate::LocatedSpan;
use std::collections::VecDeque;

#[derive(Debug, PartialEq)]
pub struct GrammerNode<'a, 'b> {
	pub base: GrammerNodeBase<'a, 'b>,
	pub error: Option<Box<UserSideError<'a>>>,
    pub warning: Option<Box<UserSideWarning<'a>>>,
}

//this is used to essentially move the error from the token to the AST
#[derive(Debug, PartialEq,Clone)]
pub struct SmallLexToken<'a> {
    pub span: LocatedSpan<&'a str>,
    pub tag: LexTag,
}

#[derive(Debug, PartialEq)]
pub enum GrammerNodeBase<'a, 'b> {
/* 
the 'b lifetime is only there from the unprocessed original input
everything else that has it is basically referrig to a node that may not be fully processed
so when we are done with processing we can move the ast to a diffrent struct and drop that slice memory

this matters for 2 things:

1. when runing the code as an interpeter 
	after error/warning reporting we can drop the 'b slice
	while still keeping the source code refrences for reporting assert fails

2. when doing parallel compilation not all slices are avilble at every thread
	this means we will need to do some copying when merging outputs from 2 threads
	so now we can merge the 2 vectors and then drop them 
	while not invalidating the diagnostic data lifetime

*/

	//non terminal
	Unprocessed(TokenSlice<'a,'b>),
	KeyWord(KeyWord<'a>),
	Paren(ParenExpr<'a,'b>),

	ControlBlock(Block<'a,'b>),
	Else(Else<'a,'b>),

	//hybrid
	Sequence(VecDeque<GrammerNode<'a, 'b>>),
	Val(Value<'a,'b>), //some values such as parthesis or varibles can be non terminal
	

	//terminal
	Return(Return<'a,'b>),
	Declare(Assign<'a,'b>),
	Function(FuncDef<'a,'b>),
}

//some argonomics
impl<'a, 'b> GrammerNode<'a, 'b> {

    pub fn new(base: GrammerNodeBase<'a, 'b>) -> Self {
        Self {
            base,
            error: None,
            warning: None,
        }
    }

    pub fn with_error(mut self, error: UserSideError<'a>) -> Self {
        self.error = Some(Box::new(error));
        self
    }

    pub fn with_warning(mut self, warning: UserSideWarning<'a>) -> Self {
        self.warning = Some(Box::new(warning));
        self
    }
}

impl<'a, 'b> From<GrammerNodeBase<'a, 'b>> for GrammerNode<'a, 'b> {
    fn from(base: GrammerNodeBase<'a, 'b>) -> Self {
        GrammerNode::new(base)
    }
}


impl<'a, 'b> From<Vec<GrammerNode<'a, 'b>>> for GrammerNodeBase<'a, 'b> {
    fn from(nodes: Vec<GrammerNode<'a, 'b>>) -> Self {
        GrammerNodeBase::Sequence(VecDeque::from(nodes))
    }
}

impl<'a, 'b> From<Vec<GrammerNode<'a, 'b>>> for GrammerNode<'a, 'b> {
    fn from(nodes: Vec<GrammerNode<'a, 'b>>) -> Self {
        GrammerNodeBase::Sequence(VecDeque::from(nodes)).into()
    }
}


impl<'a> From<LexToken<'a>> for SmallLexToken<'a> {
    fn from(base: LexToken<'a>) -> Self {
        SmallLexToken{
        	span: base.span,
        	tag: base.tag
        }
    }
}
//internal data

#[derive(Debug, PartialEq,Clone)]
pub enum KeyWord<'a>{
	Nil(LocatedSpan<&'a str>),
	
	Import(LocatedSpan<&'a str>),

	Return(LocatedSpan<&'a str>),
	FuncDec(LocatedSpan<&'a str>),
	Lamda(LocatedSpan<&'a str>),
	
	If(LocatedSpan<&'a str>),
	Else(LocatedSpan<&'a str>),
	
	Cond(LocatedSpan<&'a str>),
	Match(LocatedSpan<&'a str>),
}

impl<'a> KeyWord<'a> {
    pub fn get_span(&self) -> LocatedSpan<&'a str> {
        match self {
            KeyWord::Nil(span) 
            | KeyWord::Import(span)
            | KeyWord::Return(span)
            | KeyWord::FuncDec(span)
            | KeyWord::Lamda(span)
            | KeyWord::If(span)
            | KeyWord::Else(span)
            | KeyWord::Cond(span)
            | KeyWord::Match(span) => *span,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Value<'a,'b>{
	Basic(SmallLexToken<'a>),//not all tags are valid. up to the programer to make sure only the right types are here
	Nil(KeyWord<'a>),

	Var(Varible<'a>),
	Paren(ParenExpr<'a,'b>),
	Op(BinaryOpNode<'a,'b>),
	
	Call(FunctionCall<'a,'b>),

	If(If<'a,'b>),
	Func(Lamda<'a,'b>),
	// need to implement Match and Cond

}

#[derive(Debug, PartialEq)]
pub struct Varible<'a> {
    pub name: LocatedSpan<&'a str>,
    pub count: usize, //if the var is shadowed starts at 0 signifiying unknowen shadowing then assined a value
}

#[derive(Debug, PartialEq)]
pub struct FunctionCall<'a,'b>{
	pub piped : Option<Box<Value<'a,'b>>>,
	pub func : Func<'a,'b>,
	pub par : Box<ParenExpr<'a,'b>>,
}

//this is unified for implicit returns and explicit returns
#[derive(Debug, PartialEq)]
pub struct Return<'a,'b>{
	pub word: Option<KeyWord<'a>>,//return
	pub value: Option<Value<'a,'b>>,
	pub ender: Option<LocatedSpan<&'a str>>, //;
}

#[derive(Debug, PartialEq)]
pub struct BinaryOpNode<'a, 'b> {
    pub left: Option<Box<GrammerNode<'a, 'b>>>, // Left-hand operand, might be `None` if missing
    pub operator: SmallLexToken<'a>, // The binary operator itself
    pub right: Option<Box<GrammerNode<'a, 'b>>>, // Right-hand operand, might be `None` if missing
}

#[derive(Debug, PartialEq)]
pub struct Assign<'a, 'b> {
    pub left: Varible<'a>,
    pub operator: LocatedSpan<&'a str>, //=
    pub right: Option<Box<Value<'a, 'b>>>, // Right-hand operand, might be `None` if missing
    pub ender: Option<Box<LocatedSpan<&'a str>>>, //specifcly the ; at the end
}


//for {... ?(})
#[derive(Debug, PartialEq)]
pub struct ParenExpr<'a,'b>{ 
	pub start: Option<LocatedSpan<&'a str>>, //this can be non for things like def ) {...}
	pub body: Option<Box<GrammerNode<'a,'b>>>,
	pub end: Option<LocatedSpan<&'a str>>,//stays None untill we fined a closer (if we even find it)
}

//ifs and lamda have the exact same syntax... other than the keyword. shared logic here
#[derive(Debug, PartialEq)]
pub struct Block<'a,'b>{ 
	pub start: Box<ParenExpr<'a,'b>>, //note we use specifcly () braces here
	pub body: Box<ParenExpr<'a,'b>>, //note we use specifcly {} braces here
}

#[derive(Debug, PartialEq)]
pub struct If<'a,'b>{
	pub keyword : KeyWord<'a>,
	pub body: Block<'a,'b>,
	pub else_block : Option<Box<Else<'a,'b>>> //if not there implicitly returns nil
}

#[derive(Debug, PartialEq)]
pub struct Else<'a,'b>{
	pub keyword : KeyWord<'a>,
	pub body: ParenExpr<'a,'b>,
}

//add match and cond

#[derive(Debug, PartialEq)]
pub enum Func<'a,'b>{
	Defed(&'b FuncDef<'a,'b>),
	Vared(Varible<'a>),
	Anon(Lamda<'a,'b>),
}

#[derive(Debug, PartialEq)]
pub struct Lamda<'a,'b>{
	pub keyword : KeyWord<'a>,
	pub body: Block<'a,'b>,
}

#[derive(Debug, PartialEq)]
pub struct FuncDef<'a,'b>{
	pub keyword : KeyWord<'a>,
	pub name : Option<LocatedSpan<&'a str>>,
	pub body: Block<'a,'b>,
}