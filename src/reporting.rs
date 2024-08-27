use ariadne::{Report, ReportKind, Label, Color, Source,CharSet};
use nom_locate::LocatedSpan;
use std::ops::Range;

use crate::errors::{UserSideError, Diagnostics};
use crate::lex::lex_full_text;

impl<'a> UserSideError<'a> {
    pub fn to_ariadne_report(&self, source: &'a str) -> Report<'a, Range<usize>> {
        match self {
            UserSideError::OverflowError(span) => handle_overflow_error(*span, source),
            UserSideError::IntOverflowError(span, value) => handle_int_overflow_error(*span, *value, source),
            UserSideError::UnclosedString(span, ch) => handle_unclosed_string(*span, *ch, source),
        }
    }
}

// Utility function to get the line info including byte range
fn get_line_info(source: &str, offset: usize) -> (&str, Range<usize>, usize) {
    let start = source[..offset].rfind('\n').map_or(0, |pos| pos + 1);
    let end = source[offset..].find('\n').map_or(source.len(), |pos| offset + pos);
    let char_offset = offset - start;

    (&source[start..end], start..end, char_offset)
}

// Function to create a report for OverflowError
fn handle_overflow_error<'a>(span: LocatedSpan<&'a str>, source: &'a str) -> Report<'a, Range<usize>> {
    let (line, line_range, _) = get_line_info(source, span.location_offset());
    let byte_offset = span.location_offset();
    let span_len = span.fragment().len();

    Report::build(ReportKind::Error, (), byte_offset)
        .with_config(ariadne::Config::default().with_char_set(CharSet::Unicode))
        .with_message("Overflow error")
        .with_label(
            Label::new(byte_offset..byte_offset + span_len)
                .with_message("Too large to parse properly")
                .with_color(Color::Red),
        )
        .with_label(
            Label::new(line_range)
                .with_message(line)
                .with_color(Color::White),
        )
        .finish()
}

// Function to create a report for IntOverflowError
fn handle_int_overflow_error<'a>(span: LocatedSpan<&'a str>, value: u64, source: &'a str) -> Report<'a, Range<usize>> {
    let (line, line_range, _) = get_line_info(source, span.location_offset());
    let byte_offset = span.location_offset();
    let span_len = span.fragment().len();

    Report::build(ReportKind::Error, (), byte_offset)
        .with_config(ariadne::Config::default().with_char_set(CharSet::Unicode))
        .with_message(format!("Integer overflow error with value {}", value))
        .with_label(
            Label::new(byte_offset..byte_offset + span_len)
                .with_message("This number does not fit into int. Try a float")
                .with_color(Color::Red),
        )
        .with_label(
            Label::new(line_range)
                .with_message(line)
                .with_color(Color::White),
        )
        .finish()
}

// Function to create a report for UnclosedString
fn handle_unclosed_string<'a>(span: LocatedSpan<&'a str>, ch: char, source: &'a str) -> Report<'a, Range<usize>> {
    let (line, line_range, _) = get_line_info(source, span.location_offset());
    let byte_offset = span.location_offset();
    let span_len = span.fragment().len();

    Report::build(ReportKind::Error, (), byte_offset)
        .with_config(ariadne::Config::default().with_char_set(CharSet::Unicode))
        .with_message("Unclosed string error")
        .with_label(
            Label::new(byte_offset..byte_offset + span_len)
                .with_message(format!("Expected closing '\\{}'", ch))
                .with_color(Color::Red),
        )
        .with_label(
            Label::new(line_range)
                .with_message(line)
                .with_color(Color::White),
        )
        .finish()
}

pub fn print_errors_to_stdout<'a>(errors: &[UserSideError<'a>], source: &'a str) -> Result<(), std::io::Error>{
    for error in errors {
        let report = error.to_ariadne_report(source);
        report.eprint(Source::from(source))?;
    }
    Ok(())
}


// #[cfg(test)]
pub fn gather_errors_to_buffer<'a>(errors: &[UserSideError<'a>], source: &'a str) -> String {
    let mut buffer = Vec::new();

    for error in errors {
        let report = error.to_ariadne_report(source);
        report.write(Source::from(source), &mut buffer).unwrap();
    }

    String::from_utf8(buffer).unwrap()
}
#[test]
fn test_print() {
    let source_code = ", 丐, 丑 \n\nmore::stuff\n\n\" unclosed string";//"let x = 9223372036854775808;  aaa  :ww \n922337203685477580822\"unterminated string;\n ";
    let diag = Diagnostics::new();
    for token in lex_full_text(source_code, &diag) {
        println!("{:?}", token);
    }

    for error in diag.errors.borrow().iter(){
        println!("{:?}", error);
    }

    let buffer = gather_errors_to_buffer(&diag.errors.borrow(), source_code);

    // Print all errors at once
    println!("got {} errors:\n{}", diag.errors.borrow().len(), buffer);
}
