use ariadne::{Color, Fmt, Label, Report, ReportKind, Source};
use nom::{
    bytes::complete::take_while1,
    character::complete::{char, digit1, one_of},
    combinator::{opt, recognize, all_consuming},
    sequence::{pair, preceded},
    IResult,
};
use thiserror::Error;
use std::fmt;

#[derive(Debug, PartialEq)]
enum Number {
    Int(i64),
    Float(f64),
}

#[derive(Debug, Error)]
enum ParseError {
    #[error("Nom error: {0}")]
    NomError(nom::Err<nom::error::Error<String>>),
}

#[derive(Debug)]
struct NumberParseError {
    src: String,
    span: (usize, usize),
    nom_error: ParseError,
}

impl fmt::Display for NumberParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.nom_error)
    }
}

fn parse_number(input: &str) -> IResult<&str, Number, nom::error::Error<&str>> {
    let (input, _) = opt(one_of("+-"))(input)?;
    let (input, digits) = recognize(pair(
        digit1,
        opt(pair(char('_'), take_while1(|c: char| c.is_digit(10)))),
    ))(input)?;
    let (input, fractional) = opt(preceded(
        char('.'),
        recognize(pair(digit1, opt(pair(char('_'), take_while1(|c: char| c.is_digit(10)))))),
    ))(input)?;

    let cleaned_digits: String = digits.chars().filter(|&c| c != '_').collect();

    if let Some(frac) = fractional {
        let cleaned_frac: String = frac.chars().filter(|&c| c != '_').collect();
        let num_str = format!("{}.{}", cleaned_digits, cleaned_frac);
        let number = num_str.parse::<f64>().map_err(|_| {
            nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Float))
        })?;
        Ok((input, Number::Float(number)))
    } else {
        let number = cleaned_digits.parse::<i64>().map_err(|_| {
            nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Digit))
        })?;
        Ok((input, Number::Int(number)))
    }
}

fn parse_and_report(input: &str) -> Result<Number, NumberParseError> {
    let clean_input = input.to_string();
    let parse_result: Result<Number, ParseError> = all_consuming(parse_number)(&clean_input)
        .map(|(_, number)| number)
        .map_err(|e| {
            let converted_err = e.map_input(|input| input.to_string());
            ParseError::NomError(converted_err)
        });

    match parse_result {
        Ok(number) => Ok(number),
        Err(e) => {
            Err(NumberParseError {
                src: clean_input,
                span: (0, input.len()),
                nom_error: e,
            })
        }
    }
}

fn report_error(error: NumberParseError) {
    let source = Source::from(&error.src);
    let report = Report::build(ReportKind::Error, "input", error.span.0)
        .with_message("Failed to parse number")
        .with_label(Label::new(("input", error.span.0..error.span.1))
            .with_message(format!("{}", error.nom_error))
            .with_color(Color::Red))
        .finish();

    report.eprint(("input", source)).unwrap();
}

#[test]
fn test_invalid_numbers() {
    println!("Testing invalid number inputs...");

    if let Err(e) = parse_and_report("12_34_") {
        report_error(e);
    } else {
        panic!("Expected an error for input '12_34_'");
    }

    if let Err(e) = parse_and_report("123a45") {
        report_error(e);
    } else {
        panic!("Expected an error for input '123a45'");
    }

    if let Err(e) = parse_and_report("3.14.15") {
        report_error(e);
    } else {
        panic!("Expected an error for input '3.14.15'");
    }
}

#[test]
fn test_valid_numbers() {
    assert_eq!(parse_and_report("12345").unwrap(), Number::Int(12345));
    assert_eq!(parse_and_report("10_000").unwrap(), Number::Int(10_000));
    assert_eq!(parse_and_report("3.1415").unwrap(), Number::Float(3.1415));
}
