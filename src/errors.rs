#![allow(dead_code)] //this module is being consumed
use core::cell::RefCell;
use  nom_locate::LocatedSpan;

#[derive(Debug,PartialEq)]
pub enum UserSideError<'a> {
	BadNumPostfix(LocatedSpan<&'a str>),
	OverflowError(LocatedSpan<&'a str>),
	IntOverflowError(LocatedSpan<&'a str>,u64),

	UnclosedString(LocatedSpan<&'a str>,char),

}

#[derive(Debug,PartialEq)]
pub enum UserSideWarning<'a> {
	UnusedVar(LocatedSpan<&'a str>),
}

#[derive(Debug,PartialEq)]
pub struct Diagnostics<'a>{
	pub errors: RefCell<Vec<UserSideError<'a>>>,
	pub warnings: RefCell<Vec<UserSideWarning<'a>>>
}

impl<'a> Diagnostics<'a>{
	pub fn new() -> Self{
		Diagnostics{
			errors: RefCell::new(vec![]),
			warnings: RefCell::new(vec![]),
		}
	}

	pub fn report_error(&self, error: UserSideError<'a>) {
        self.errors.borrow_mut().push(error);
    }

    pub fn report_warning(&self, warning: UserSideWarning<'a>) {
        self.warnings.borrow_mut().push(warning);
    }
}

#[derive(Debug,PartialEq,Clone)]
pub struct Extra<'a,T:Clone> {
	pub diag: &'a Diagnostics<'a>,
	pub tag: T,
}

impl<'a, T: Clone> Extra<'a, T> {
    pub fn map_tag<U: Clone, F>(self, f: F) -> Extra<'a, U>
    where
        F: FnOnce(T) -> U,
    {
        Extra {
            diag: self.diag,
            tag: f(self.tag),
        }
    }
}

pub type Cursor<'a,T=()> = LocatedSpan<&'a str, Extra<'a,T>>;
pub type CResult<'a , O,T=()> = nom::IResult<Cursor<'a,T>, O>;

pub fn make_cursor<'a>(code:&'a str,diag: &'a Diagnostics<'a>,) -> Cursor<'a> {
	Cursor::new_extra(code, Extra{diag,tag:()})
}

pub fn strip_reporting<'a,T :Clone>(x:Cursor<'a,T>) -> LocatedSpan<&'a str, T> {
	x.map_extra(|extra:Extra<'a,T>| extra.tag)
}

pub fn add_reporting<'a,T :Clone>(x:LocatedSpan<&'a str, T>,diag: &'a Diagnostics<'a>) ->  Cursor<'a,T>{
	x.map_extra(|tag:T| Extra{diag:diag,tag:tag})
}