use crate::token::{Cursor,TokenSlice,LexToken,LexTag};
use crate::ast::{};
use crate::errors::{UserSideError};

use nom::bytes::complete::take;
use nom::bytes::complete::take_till;
use nom::combinator::map;

use nom::InputLength;

use nom::{Err::Error};

use crate::errors::TResult;


fn is_opener(c:char) -> bool {
	match c {
		'{' => true,
		'[' => true,
		'(' => true,

		')' => false,
		']' => false,
		'}' => false,

		_ => unreachable!()
	}
}

fn get_closer(c:char) -> char {
	match c {
		'{' => '}',
		'[' => ']',
		'(' => ')',

		_ => unreachable!()
	}
}

