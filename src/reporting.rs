use codespan_reporting::diagnostic::{Label};
use codespan_reporting::diagnostic::Diagnostic as PrintDiagnostic;
use codespan_reporting::files::SimpleFile;
use codespan_reporting::term::{
    self,
    termcolor::{ColorChoice, StandardStream, Buffer},
};
use nom_locate::LocatedSpan;
use std::error::Error;

use crate::errors::{UserSideError};
#[cfg(test)]
use crate::lex::lex_full_text;

impl<'a> UserSideError<'a> {
    pub fn to_codespan_diagnostics(&self) -> Vec<PrintDiagnostic<()>> {
        match self {
            UserSideError::OverflowError(span) => vec![handle_overflow_error(span)],
            UserSideError::IntOverflowError(span, value) => {
                vec![handle_int_overflow_error(span, *value)]
            }
            UserSideError::UnclosedString(span, ch) => vec![handle_unclosed_string(span, *ch)],
            UserSideError::UnokwenToken(span) => vec![handle_unkowen_token_error(span)],
            UserSideError::ExtraPar(span) => vec![handle_extra_par_error(span)],
            UserSideError::UnclosedPar(start, end) => vec![handle_unclosed_par_error(start, end)],
            UserSideError::Compound(errors) => handle_compound_error(errors),
        }
    }
}

// Function to create a diagnostic for ExtraPar
fn handle_extra_par_error(span: &LocatedSpan<&str>) -> PrintDiagnostic<()> {
    let start = span.location_offset();
    let end = start + span.fragment().len();

    PrintDiagnostic::error()
        .with_message("Unexpected extra parenthesis")
        .with_labels(vec![Label::primary((), start..end)])
}

// Function to create a diagnostic for UnclosedPar
fn handle_unclosed_par_error(start: &LocatedSpan<&str>, end: &LocatedSpan<&str>) -> PrintDiagnostic<()> {
    let start_offset = start.location_offset();
    let end_offset = end.location_offset() + end.fragment().len();

    PrintDiagnostic::error()
        .with_message("Unclosed parenthesis")
        .with_labels(vec![
            Label::primary((), start_offset..start_offset + start.fragment().len())
                .with_message("Opened here"),
            Label::primary((), end_offset..end_offset)
                .with_message("Expected a closing parenthesis here"),
        ])
}

// Function to create diagnostics for Compound errors
fn handle_compound_error<'a>(errors: &[UserSideError<'a>]) -> Vec<PrintDiagnostic<()>> {
    let mut diagnostics = Vec::new();

    for error in errors {
        let sub_diagnostics = error.to_codespan_diagnostics();
        diagnostics.extend(sub_diagnostics);
    }

    diagnostics
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
#[allow(dead_code)]
pub fn print_errors_to_stdout<'a>(
    errors: &[UserSideError<'a>],
    source: &'a str,
) -> Result<(), Box<dyn Error>> {
    let file = SimpleFile::new("source", source);
    let writer = StandardStream::stderr(ColorChoice::Auto);
    let config = term::Config::default();

    for error in errors {
        for diagnostic in error.to_codespan_diagnostics() {
            term::emit(&mut writer.lock(), &config, &file, &diagnostic)?;
        }
    }

    Ok(())
}

// Function to gather errors into a string buffer
#[allow(dead_code)]
pub fn gather_errors_to_buffer<'a>(errors: &[UserSideError<'a>], source: &'a str) -> String {
    let file = SimpleFile::new("source", source);
    let mut buffer = Buffer::ansi();
    let config = term::Config::default();

    for error in errors {
        for diagnostic in error.to_codespan_diagnostics() {
            term::emit(&mut buffer, &config, &file, &diagnostic).unwrap();
        }
    }

    String::from_utf8(buffer.into_inner()).unwrap()
}

#[test]
fn test_print() {
    let source_code = "let x = 9223372036854775808;  🏳️‍⚧️ aaa  :ww \n922337203685477580822\"unterminated string;\n ";
    let mut errors = Vec::new();

    // Create a Cursor from the content
    for token in lex_full_text(source_code) {
        // println!("{:?}", token);
        if let Some(e) = token.error {
            errors.push(*e);
        }
    }
    
    // Print errors to stdout
    // print_errors_to_stdout(&diag.borrow_errors(), source_code).unwrap();
    
    // Gather errors into buffer and print
    let buffer = gather_errors_to_buffer(&errors, source_code);
    println!("Collected Errors:\n{}", buffer);
}

#[test]
fn test_compound_print() {
    let source_code = "999999999999999999999999999999999999999999999999999999999999999999999999999999999.999999999999999999999999999999999999999999999999999999";
    let mut errors = Vec::new();

    // Create a Cursor from the content
    for token in lex_full_text(source_code) {
        // println!("{:?}", token);
        if let Some(e) = token.error {
            errors.push(*e);
        }
    }
    
    // Print errors to stdout
    // print_errors_to_stdout(&diag.borrow_errors(), source_code).unwrap();
    
    // Gather errors into buffer and print
    let buffer = gather_errors_to_buffer(&errors, source_code);
    println!("Collected Errors:\n{}", buffer);
}
