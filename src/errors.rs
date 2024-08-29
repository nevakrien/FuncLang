// #![allow(dead_code)] //this module is being consumed
#[cfg(feature = "safe_mode")]
use core::cell::RefCell;
#[cfg(feature = "safe_mode")]
use core::cell::Ref;

#[cfg(feature = "unsafe_mode")]
use core::cell::UnsafeCell;


use  nom_locate::LocatedSpan;

#[derive(Debug,PartialEq)]
pub enum UserSideError<'a> {
	OverflowError(LocatedSpan<&'a str>),
	IntOverflowError(LocatedSpan<&'a str>,u64),
	UnokwenToken(LocatedSpan<&'a str>),
	UnclosedString(LocatedSpan<&'a str>,char),

	UnclosedPar(LocatedSpan<&'a str>,LocatedSpan<&'a str>),//start found
	ExtraPar(LocatedSpan<&'a str>),


}

#[allow(dead_code)]
#[derive(Debug,PartialEq)]
pub enum UserSideWarning<'a> {
	UnusedVar(LocatedSpan<&'a str>), //for now not actually implemented
}


#[derive(Debug)]
pub struct Diagnostics<'a> {
    #[cfg(feature = "safe_mode")]
    errors: RefCell<Vec<UserSideError<'a>>>,
    #[cfg(feature = "safe_mode")]
    warnings: RefCell<Vec<UserSideWarning<'a>>>,

    #[cfg(feature = "unsafe_mode")]
    errors: UnsafeCell<Vec<UserSideError<'a>>>,
    #[cfg(feature = "unsafe_mode")]
    warnings: UnsafeCell<Vec<UserSideWarning<'a>>>,
}

impl<'a> PartialEq for Diagnostics<'a> {
    fn eq(&self, other: &Self) -> bool {
        #[cfg(feature = "safe_mode")]
        {
            // Correctly dereference Ref<T> to &T before comparison
            (*self.errors.borrow()).eq(&*other.errors.borrow()) &&
            (*self.warnings.borrow()).eq(&*other.warnings.borrow())
        }

        #[cfg(feature = "unsafe_mode")]
        {
            // In unsafe_mode, directly access the UnsafeCell contents
            unsafe {
                (*self.errors.get()).eq(&*other.errors.get()) &&
                (*self.warnings.get()).eq(&*other.warnings.get())
            }
        }
    }
}



impl<'a> Diagnostics<'a>{
	pub fn new() -> Self {
        Diagnostics {
            #[cfg(feature = "safe_mode")]
            errors: RefCell::new(vec![]),
            #[cfg(feature = "safe_mode")]
            warnings: RefCell::new(vec![]),

            #[cfg(feature = "unsafe_mode")]
            errors: UnsafeCell::new(vec![]),
            #[cfg(feature = "unsafe_mode")]
            warnings: UnsafeCell::new(vec![]),
        }
    }

    #[cfg(feature = "safe_mode")]
	pub fn borrow_errors(&self) -> Ref<Vec<UserSideError<'a>>>{
		self.errors.borrow()
	}

	#[cfg(feature = "safe_mode")]
	pub fn borrow_warnings(&self) -> Ref<Vec<UserSideWarning<'a>>> {
		self.warnings.borrow()
	}

    #[cfg(feature = "unsafe_mode")]
    pub fn borrow_errors(&self) -> &Vec<UserSideError<'a>> {
        unsafe { &*self.errors.get() }
    }

    #[cfg(feature = "unsafe_mode")]
    pub fn borrow_warnings(&self) -> &Vec<UserSideWarning<'a>> {
       unsafe { &*self.warnings.get() }
    }

    pub fn report_error(&self, error: UserSideError<'a>) {
        #[cfg(feature = "safe_mode")]
        {
            self.errors.borrow_mut().push(error);
        }
        #[cfg(feature = "unsafe_mode")]
        {
            unsafe { (*self.errors.get()).push(error); }
        }
    }

    pub fn report_warning(&self, warning: UserSideWarning<'a>) {
        #[cfg(feature = "safe_mode")]
        {
            self.warnings.borrow_mut().push(warning);
        }
        #[cfg(feature = "unsafe_mode")]
        {
            unsafe { (*self.warnings.get()).push(warning); }
        }
    }
}

#[derive(Debug,PartialEq,Clone)]
pub struct Extra<'a,T:Clone> {
	pub diag: &'a Diagnostics<'a>,
	pub tag: T,
}

// impl<'a, T: Clone> Extra<'a, T> {
//     pub fn map_tag<U: Clone, F>(self, f: F) -> Extra<'a, U>
//     where
//         F: FnOnce(T) -> U,
//     {
//         Extra {
//             diag: self.diag,
//             tag: f(self.tag),
//         }
//     }
// }

pub type Cursor<'a,T=()> = LocatedSpan<&'a str, Extra<'a,T>>;
pub type CResult<'a , O,T=()> = nom::IResult<Cursor<'a,T>, O>;

//#[allow(dead_code)] //will use for the parser
pub type TResult<'a,'b,O , E=()> = nom::IResult<crate::token::Cursor<'a,'b>, O, E>;

pub fn make_cursor<'a>(code:&'a str,diag: &'a Diagnostics<'a>,) -> Cursor<'a> {
	Cursor::new_extra(code, Extra{diag,tag:()})
}

pub fn strip_reporting<'a,T :Clone>(x:Cursor<'a,T>) -> LocatedSpan<&'a str, T> {
	x.map_extra(|extra:Extra<'a,T>| extra.tag)
}

// pub fn add_reporting<'a,T :Clone>(x:LocatedSpan<&'a str, T>,diag: &'a Diagnostics<'a>) ->  Cursor<'a,T>{
// 	x.map_extra(|tag:T| Extra{diag:diag,tag:tag})
// }