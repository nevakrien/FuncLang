// #![allow(dead_code)] //this module is being consumed
use core::cell::RefCell;
use core::cell::Ref;



use  nom_locate::LocatedSpan;

#[derive(Debug,PartialEq)]
pub enum UserSideError<'a> {
	OverflowError(LocatedSpan<&'a str>),
	IntOverflowError(LocatedSpan<&'a str>,u64),
	UnokwenToken(LocatedSpan<&'a str>),
	UnclosedString(LocatedSpan<&'a str>,char),

	Compound(Vec<UserSideError<'a>>),

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
#[derive(Debug,PartialEq)]
pub enum UserSideWarning<'a> {
	UnusedVar(LocatedSpan<&'a str>), //for now not actually implemented
}



// #[derive(Debug)]
// pub struct Diagnostics<'a> {
//     errors: RefCell<Vec<UserSideError<'a>>>,
//     warnings: RefCell<Vec<UserSideWarning<'a>>>,
// }

// impl<'a> PartialEq for Diagnostics<'a> {
//     fn eq(&self, other: &Self) -> bool {

//         (*self.errors.borrow()).eq(&*other.errors.borrow()) &&
//         (*self.warnings.borrow()).eq(&*other.warnings.borrow())
        
//     }
// }



// impl<'a> Diagnostics<'a>{
// 	pub fn new() -> Self {
//         Diagnostics {
//             errors: RefCell::new(vec![]),
//             warnings: RefCell::new(vec![]),
//         }
//     }

// 	pub fn borrow_errors(&self) -> Ref<Vec<UserSideError<'a>>>{
// 		self.errors.borrow()
// 	}

// 	pub fn borrow_warnings(&self) -> Ref<Vec<UserSideWarning<'a>>> {
// 		self.warnings.borrow()
// 	}

//     pub fn report_error(&self, error: UserSideError<'a>) {
//     	self.errors.borrow_mut().push(error);
//     }

//     pub fn report_warning(&self, warning: UserSideWarning<'a>) {
//     	self.warnings.borrow_mut().push(warning);
//     }
// }

// #[derive(Debug,PartialEq,Clone)]
// pub struct Extra<'a,T:Clone> {
// 	pub diag: &'a Diagnostics<'a>,
// 	pub tag: T,
// }

// // impl<'a, T: Clone> Extra<'a, T> {
// //     pub fn map_tag<U: Clone, F>(self, f: F) -> Extra<'a, U>
// //     where
// //         F: FnOnce(T) -> U,
// //     {
// //         Extra {
// //             diag: self.diag,
// //             tag: f(self.tag),
// //         }
// //     }
// // }

// pub type Cursor<'a,T=()> = LocatedSpan<&'a str, Extra<'a,T>>;
// pub type CResult<'a , O,T=()> = nom::IResult<Cursor<'a,T>, O>;

// //#[allow(dead_code)] //will use for the parser
// pub type TResult<'a,'b,O , E=()> = nom::IResult<crate::token::Cursor<'a,'b>, O, E>;

// pub fn make_cursor<'a>(code:&'a str,diag: &'a Diagnostics<'a>,) -> Cursor<'a> {
// 	Cursor::new_extra(code, Extra{diag,tag:()})
// }

// pub fn strip_reporting<'a,T :Clone>(x:Cursor<'a,T>) -> LocatedSpan<&'a str, T> {
// 	x.map_extra(|extra:Extra<'a,T>| extra.tag)
// }

// // pub fn add_reporting<'a,T :Clone>(x:LocatedSpan<&'a str, T>,diag: &'a Diagnostics<'a>) ->  Cursor<'a,T>{
// // 	x.map_extra(|tag:T| Extra{diag:diag,tag:tag})
// // }