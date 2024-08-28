mod lex;
mod errors;
mod token;

// mod parse;
// mod ast;

mod reporting;

use crate::lex::lex_full_text;
use crate::errors::{Diagnostics};  
use crate::reporting::print_errors_to_stdout;

use std::fs::File;
use std::io::{Read,stdout,Write};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Specify the path to the file you want to lex
    let path = Path::new("sample.txt");
    
    // Open the file
    let mut file = File::open(&path)?;

    // Read the file content into a string
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    let code = &content;

    // Create a Cursor from the content
    let diag = Diagnostics::new();
    for token in lex_full_text(code,&diag) {
        println!("{:?}", token);
    }

    for error in diag.borrow_errors().iter(){
        println!("{:?}", error);
    }

    print_errors_to_stdout(&diag.borrow_errors(),code)?;
    stdout().flush()?;

    Ok(())
}
    
// use codespan_reporting::diagnostic::{Diagnostic, Label};
// use codespan_reporting::term;
// use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};
// use codespan_reporting::files::SimpleFile;

// fn main() {
//     let file = SimpleFile::new("sample.txt", "丐, 丑 let x = 1;\nlet y = 🐍;\n");

//     let diagnostic = Diagnostic::error()
//         .with_message("Unexpected emoji")
//         .with_labels(vec![
//             Label::primary((), 29..30)
//                 .with_message("Found an emoji here"),
//         ]);

//     let mut writer = StandardStream::stderr(ColorChoice::Auto);
//     let config = term::Config::default();

//     term::emit(&mut writer, &config, &file, &diagnostic).unwrap();
// }
