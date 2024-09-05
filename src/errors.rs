// #![allow(dead_code)] //this module is being consumed
use  nom_locate::LocatedSpan;
use crate::token::{LexToken};

#[derive(Debug,PartialEq,Clone)]
pub enum UserSideError<'a> {
	OverflowError(LocatedSpan<&'a str>),
	IntOverflowError(LocatedSpan<&'a str>,u64),
	UnokwenToken(LocatedSpan<&'a str>),
	UnclosedString(LocatedSpan<&'a str>,char),

	Compound(Vec<UserSideError<'a>>),

	MissingFuncName(LocatedSpan<&'a str>),
	EmptyFuncDef(LocatedSpan<&'a str>),
	UnexpectedNameTok(LexToken<'a>),
	ReservedName(LocatedSpan<&'a str>),

	UnexpectedTokens(Vec<LocatedSpan<&'a str>>),
	
	UnclosedPar(LocatedSpan<&'a str>,LocatedSpan<&'a str>),//start found
	ExtraPar(LocatedSpan<&'a str>),


}

pub fn combine_errors<'a>(
    err1: Option<Box<UserSideError<'a>>>,
    err2: Option<Box<UserSideError<'a>>>,
) -> Option<Box<UserSideError<'a>>> {
    match (err1, err2) {
        (None, None) => None,
        (Some(e), None) | (None, Some(e)) => Some(e),
        (Some(e1), Some(e2)) => Some(Box::new(UserSideError::Compound(match (*e1, *e2) {
            (UserSideError::Compound(mut v1), UserSideError::Compound(mut v2)) => {
                v1.append(&mut v2);
                v1
            }
            (UserSideError::Compound(mut v), e) | (e, UserSideError::Compound(mut v)) => {
                v.push(e);
                v
            }
            (e1, e2) => vec![e1, e2],
        }))),
    }
}


#[allow(dead_code)]
#[derive(Debug,PartialEq,Clone)]
pub enum UserSideWarning<'a> {
	UnusedVar(LocatedSpan<&'a str>), //for now not actually implemented
}

