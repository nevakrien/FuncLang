mod lex;
mod errors;
use crate::lex::lex_full_text;
use crate::errors::{Diagnostics};  

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
    for token in lex_full_text(&content,&diag) {
        println!("{:?}", token);
    }

    Ok(())
}
    