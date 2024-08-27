mod lex;
mod errors;
use crate::lex::{lext_text,LexToken};
use crate::errors::{Cursor,make_cursor,Diagnostics};  

use std::fs::File;
use std::io::{self, Read};
use std::path::Path;


fn main() -> io::Result<()> {
    // Specify the path to the file you want to lex
    let path = Path::new("sample.txt");
    
    // Open the file
    let mut file = File::open(&path)?;

    // Read the file content into a string
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    // Create a Cursor from the content
    let diag = Diagnostics::new();
    let cursor = make_cursor(&content,&diag);

    // Lex the content
    let mut remaining = cursor;
    while !remaining.fragment().is_empty() {
        match lext_text(remaining) {
            Ok((new_remaining, token)) => {
                println!("{:?}", token);  // Print the lexed token
                remaining = new_remaining;  // Update the remaining input
            }
            Err(err) => {
                eprintln!("Error lexing: {:?}", err);
                break;
            }
        }
    }

    Ok(())
}

