#![allow(dead_code)] //this module is being consumed
use core::cell::RefCell;


#[derive(Debug,PartialEq)]
pub enum UserSideError {
	BadNumPostfix(),
}

#[derive(Debug,PartialEq)]
pub enum UserSideWarning {
	UnusedVar(),
}

#[derive(Debug,PartialEq)]
pub struct Diagnostics{
	errors: RefCell<Vec<UserSideError>>,
	warnings: RefCell<Vec<UserSideWarning>>
}

impl Diagnostics{
	pub fn new() -> Self{
		Diagnostics{
			errors: RefCell::new(vec![]),
			warnings: RefCell::new(vec![]),
		}
	}
}

#[derive(Clone, Debug)]
pub struct Reporter<'a>(&'a RefCell<Vec<UserSideError>>,&'a RefCell<Vec<UserSideWarning>>);

impl<'a> Reporter<'a> {
	pub fn new(storage : &'a Diagnostics)-> Self{
		Reporter(&storage.errors, &storage.warnings)
	}
    pub fn report_error(&self, error: UserSideError) {
        self.0.borrow_mut().push(error);
    }

    pub fn report_warning(&self, warning: UserSideWarning) {
        self.1.borrow_mut().push(warning);
    }
}

pub type Cursor<'a> = nom_locate::LocatedSpan<&'a str, Reporter<'a>>;
pub type CResult<'a , T> = nom::IResult<Cursor<'a>, T>;
