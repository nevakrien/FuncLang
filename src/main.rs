use std::env;
use std::str::FromStr;
use std::time::Instant;

mod lex;
mod errors;
mod token;

mod parse;
mod ast;

mod reporting;

use crate::lex::lex_full_text;
use crate::reporting::print_errors_to_stdout;

use std::fs::File;
use std::io::{self, Read, stdout, Write};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let start = Instant::now();
    
    // Get the number of iterations from the command line arguments, defaulting to 1
    let args: Vec<String> = env::args().collect();
    let iterations = if args.len() > 1 {
        usize::from_str(&args[1]).unwrap_or(1)
    } else {
        1
    };


    // Run the benchmark
    for i in 1..=iterations {
        // Start the timer

        // Run the sample
        run_on_sample()?;

        // Stop the timer
        let duration = start.elapsed();

        // Print the duration
        println!("Iteration {}: Time elapsed  is: {:?}", i, duration);
    }

    Ok(())
}

fn run_on_sample() -> Result<(), Box<dyn std::error::Error>> {
    // Specify the path to the file you want to lex
    let path = Path::new("sample.txt");
    
    // Open the file
    let mut file = File::open(&path)?;

    // Read the file content into a string
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    let code = &content;

    let mut errors = Vec::new();

    // Create a Cursor from the content
    for token in lex_full_text(code) {
        println!("{:?}", token);
        if let Some(e) = token.error {
            errors.push(*e);
        }
    }


    print_errors_to_stdout(&errors,code)?;
    stdout().flush()?;

    Ok(())
}
