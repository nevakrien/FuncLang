use codespan_reporting::diagnostic::{Label};
use codespan_reporting::diagnostic::Diagnostic as PrintDiagnostic;
use codespan_reporting::files::SimpleFile;
use codespan_reporting::term::{
    self,
    termcolor::{ColorChoice, StandardStream, Buffer},
};
use nom_locate::LocatedSpan;
use std::error::Error;

use crate::errors::{UserSideError, Diagnostics};
use crate::lex::lex_full_text;

impl<'a> UserSideError<'a> {
    pub fn to_codespan_diagnostic(&self) -> PrintDiagnostic<()> {
        match self {
            UserSideError::OverflowError(span) => handle_overflow_error(span),
            UserSideError::IntOverflowError(span, value) => {
                handle_int_overflow_error(span, *value)
            }
            UserSideError::UnclosedString(span, ch) => handle_unclosed_string(span, *ch),
            UserSideError::UnokwenToken(span) => handle_unkowen_token_error(span),
        }
    }
}

fn handle_unkowen_token_error(span: &LocatedSpan<&str>) -> PrintDiagnostic<()> {
    let start = span.location_offset();
    let end = start + span.fragment().len();

    PrintDiagnostic::error()
        .with_message("Unokwen Token")
        .with_labels(vec![Label::primary((), start..end)])
}

// Function to create a diagnostic for OverflowError
fn handle_overflow_error(span: &LocatedSpan<&str>) -> PrintDiagnostic<()> {
    let start = span.location_offset();
    let end = start + span.fragment().len();

    PrintDiagnostic::error()
        .with_message("Overflow error")
        .with_labels(vec![Label::primary((), start..end)
            .with_message("Too large to parse properly")])
}

// Function to create a diagnostic for IntOverflowError
fn handle_int_overflow_error(span: &LocatedSpan<&str>, value: u64) -> PrintDiagnostic<()> {
    let start = span.location_offset();
    let end = start + span.fragment().len();

    PrintDiagnostic::error()
        .with_message(format!("Integer overflow with value {}", value))
        .with_labels(vec![Label::primary((), start..end)
            .with_message("This number does not fit into an integer. Try using a float.")])
}

// Function to create a diagnostic for UnclosedString
fn handle_unclosed_string(span: &LocatedSpan<&str>, ch: char) -> PrintDiagnostic<()> {
    let start = span.location_offset();
    let end = start + span.fragment().len();

    PrintDiagnostic::error()
        .with_message("Unclosed string")
        .with_labels(vec![Label::primary((), start..end)
            .with_message(format!("Expected closing '{}'", ch))])
        .with_notes(vec![
            "Strings must be closed with matching quotation marks.".to_string(),
        ])
}

// Function to print errors to standard output
pub fn print_errors_to_stdout<'a>(
    errors: &[UserSideError<'a>],
    source: &'a str,
) -> Result<(), Box<dyn Error>> {
    let file = SimpleFile::new("source", source);
    let writer = StandardStream::stderr(ColorChoice::Auto);
    let config = term::Config::default();

    for error in errors {
        let diagnostic = error.to_codespan_diagnostic();
        term::emit(&mut writer.lock(), &config, &file, &diagnostic)?;
    }

    Ok(())
}

// Function to gather errors into a string buffer
pub fn gather_errors_to_buffer<'a>(errors: &[UserSideError<'a>], source: &'a str) -> String {
    let file = SimpleFile::new("source", source);
    let mut buffer = Buffer::ansi();
    let config = term::Config::default();

    for error in errors {
        let diagnostic = error.to_codespan_diagnostic();
        term::emit(&mut buffer, &config, &file, &diagnostic).unwrap();
    }

    String::from_utf8(buffer.into_inner()).unwrap()
}

#[test]
fn test_print() {
    let source_code = "let x = 9223372036854775808;  🏳️‍⚧️ aaa  :ww \n922337203685477580822\"unterminated string;\n ";
    let diag = Diagnostics::new();
    
    // Simulate lexing process and error collection
    for token in lex_full_text(source_code, &diag) {
        println!("{:?}", token);
    }
    
    for error in diag.borrow_errors().iter() {
        println!("{:?}", error);
    }
    
    // Print errors to stdout
    // print_errors_to_stdout(&diag.borrow_errors(), source_code).unwrap();
    
    // Gather errors into buffer and print
    let buffer = gather_errors_to_buffer(&diag.borrow_errors(), source_code);
    println!("Collected Errors:\n{}", buffer);
}
