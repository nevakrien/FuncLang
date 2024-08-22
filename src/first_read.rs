/*
this file handles the inial lexing on a very basic level before we convert to utf8
unicode utf8 has some nice gurntees for us on asci seperators

since we are using refrences there are some pretty big limits on what we can do
*/

use std::io::Write;

pub struct RawToken<'a>{
    code:&'a [u8],
    line:usize,
}

//this includes strings and chars
pub struct ParenStatment<'a> {
        content: Vec<BasicType<'a>>,
        code: &'a [u8],
        closed : bool,       
}

// #[allow(dead_code)]
pub enum BasicType<'a> {
    Comment(RawToken<'a>),
    Statment(RawToken<'a>),
    Remainer(RawToken<'a>),
    

    Parens(ParenStatment<'a>),
    
    ErrorClosingParens(&'a u8), //we use the location in printing the error

    End(),

}
pub struct Gatherer<'a> {
    line_num :usize,
    current_index: usize,
    pub full_code : &'a [u8],
}

impl<'a> Gatherer<'a> {
    /// Creates a new `Gatherer` with the provided full code.
    pub fn new(full_code: &'a [u8]) -> Self {
        Self {
            line_num: 1,
            current_index: 0,
            full_code,
        }
    }

    pub fn next(&mut self,err_file: Option<impl Write>) -> Option<BasicType<'a>>{
        if !self.get_valid_start() {
            return None;
        }

        let start=self.current_index;
        let line=self.line_num;

        let is_last=!self.skip_non_keybytes();      
        let code=&self.full_code[start..self.current_index];
        
        if code.len()>0 || is_last {
            let token = RawToken{code,line};
            
            if is_last {
                return Some(BasicType::Remainer(
                    token
                ));
            }

            return match self.full_code[self.current_index] {
                b';' => {
                    self.current_index+=1;
                    Some(BasicType::Statment(token))
                },
                _ => Some(BasicType::Remainer(token)),
            }
        }
        todo!();

    }

    fn get_valid_start(&mut self) -> bool{
        // Move the current index past all non-key bytes.
        while self.current_index < self.full_code.len() {
            let byte = self.full_code[self.current_index];
            // Stop if we encounter a key byte.
            if matches!(byte, b' ' | b'\t' | b'\n') {
                return true;
            }
            // If a newline is encountered, increment the line number.
            if byte == b'\n' {
                self.line_num += 1;
            }
            self.current_index += 1;
        }
        return false;

    }

    /// Skips over non-key bytes (anything not `{`, `}`, `;`, `#`) and returns the remaining slice.
    fn skip_non_keybytes (&mut self) -> bool{
        // Move the current index past all non-key bytes.
        while self.current_index < self.full_code.len() {
            let byte = self.full_code[self.current_index];
            // Stop if we encounter a key byte.
            if matches!(byte, b'{' | b'}' | b'(' | b')'| b'"' | b'\'' | b';' | b'#') {
                return true;
            }
            // If a newline is encountered, increment the line number.
            if byte == b'\n' {
                self.line_num += 1;
            }
            self.current_index += 1;
        }
        return false;
    }

    //api for external code to consume from us
    pub fn line(&self) -> usize {self.line_num}
    pub fn index(&self) -> usize {self.current_index}
    pub fn consume(&mut self,amount: usize)  {
        for i in 0..amount {
            if self.full_code[i]==b'\n' {
                self.line_num+=1;
            }
        }
        self.current_index+=amount;
    }
}

