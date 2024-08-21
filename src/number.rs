use ariadne::{Color, Label, Report, ReportKind, Source};
use nom::{
    bytes::complete::take_while1,
    character::complete::{char, digit1, one_of},
    combinator::{opt, recognize, not, peek, map,consumed},
    sequence::{pair, preceded, tuple},
    branch::alt,
    error::{VerboseError, VerboseErrorKind,ParseError},
    IResult,
};
use std::io::{self, Write, Cursor};

#[derive(Debug, PartialEq)]
pub enum Number {
    Int(i64),
    Float(f64),
}

// Parse the optional sign
fn parse_sign(input: &str) -> IResult<&str, Option<char>, VerboseError<&str>> {
    opt(one_of("+-"))(input)
}

// Parse the main digits with optional underscores
fn parse_digits(input: &str) -> IResult<&str, &str, VerboseError<&str>> {
    recognize(pair(
        digit1,
        opt(pair(char('_'), take_while1(|c: char| c.is_digit(10)))),
    ))(input)
}

// Parse the optional fractional part
fn parse_fractional(input: &str) -> IResult<&str, &str, VerboseError<&str>> {
    preceded(
        char('.'),
        recognize(pair(
            digit1,
            opt(pair(char('_'), take_while1(|c: char| c.is_digit(10)))),
        )),
    )(input)
}

// Ensure the next character is not an alphabet, dot, or underscore
fn check_next_char(input: &str) -> IResult<&str, (), VerboseError<&str>> {
    map(not(peek(one_of("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ._"))), |_| ())(input)
}

// Parse an integer number
fn parse_integer(input: &str) -> IResult<&str, Number, VerboseError<&str>> {
    let (input, (sign, digits, _)) = tuple((parse_sign, parse_digits, check_next_char))(input)?;

    let mut number_str = digits.replace('_', "");
    if let Some(sign) = sign {
        number_str.insert(0, sign);
    }

    number_str
        .parse::<i64>()
        .map(|n| (input, Number::Int(n)))
        .map_err(|_| nom::Err::Error(VerboseError::from_error_kind(input, nom::error::ErrorKind::Digit)))
}

// Parse a floating-point number
fn parse_float(input: &str) -> IResult<&str, Number, VerboseError<&str>> {
    let (input, (sign, digits, fractional, _)) = tuple((parse_sign, parse_digits, parse_fractional, check_next_char))(input)?;

    let mut number_str = digits.replace('_', "");
    number_str.push('.');
    number_str.push_str(&fractional.replace('_', ""));

    if let Some(sign) = sign {
        number_str.insert(0, sign);
    }

    number_str
        .parse::<f64>()
        .map(|n| (input, Number::Float(n)))
        .map_err(|_| nom::Err::Error(VerboseError::from_error_kind(input, nom::error::ErrorKind::Float)))
}

// General number parsing using `alt` to try parsing as float first, then as integer
pub fn parse_number(input: &str) -> IResult<&str, Number, VerboseError<&str>> {
    alt((parse_float, parse_integer))(input)
}

// Parse and report errors
pub fn parse_number_and_report(input: &str) -> Result<Number, VerboseError<&str>> {
    match parse_number(input) {
        Ok((_, number)) => Ok(number),
        Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => Err(e),
        Err(nom::Err::Incomplete(_)) => Err(VerboseError {
            errors: vec![(input, VerboseErrorKind::Context("Incomplete input: more data required"))],
        }),
    }
}


// Report the first error using ariadne
pub fn report_error(error: VerboseError<&str>, original_input: &str, file: impl Write) {
    let source = Source::from(original_input);
    let mut report_builder = Report::build(ReportKind::Error, "input", 0)
        .with_message("Parsing error:");

    if let Some((input, kind)) = error.errors.first() {
        let start = original_input.len() - input.len();
        let end = original_input.len();

        // Highlight the erroneous part in red
        report_builder = report_builder.with_label(
            Label::new(("input", start..end))
                .with_message(format!("{:?}", kind))
                .with_color(Color::Red),
        );

        // Add labels for the non-error parts
        if start > 0 {
            report_builder = report_builder.with_label(
                Label::new(("input", 0..start))
                    .with_color(Color::White),
            );
        }
    }

    let report = report_builder.finish();
    report.write(("input", source), file).unwrap();
}


#[test]
fn test_invalid_numbers() {
    println!("Testing invalid number inputs...");

    let mut buffer = Cursor::new(Vec::new());

    {
        if let Err(e) = parse_number_and_report("12_34_") {
            report_error(e, "12_34_", &mut buffer);
        } else {
            panic!("Expected an error for input '12_34_'");
        }
    }

    {
        if let Err(e) = parse_number_and_report("_22") {
            report_error(e, "_22", &mut buffer);
        } else {
            panic!("Expected an error for input '_22'");
        }
    }

    {
        if let Err(e) = parse_number_and_report("123a45") {
            report_error(e, "123a45", &mut buffer);
        } else {
            panic!("Expected an error for input '123a45'");
        }
    }

    {
        if let Err(e) = parse_number_and_report("3.14.15") {
            report_error(e, "3.14.15", &mut buffer);
        } else {
            panic!("Expected an error for input '3.14.15'");
        }
    }

    {
        if let Err(e) = parse_number_and_report("311a322") {
            report_error(e, "311a322", &mut buffer);
        } else {
            panic!("Expected an error for input '311a322'");
        }
    }

    // Ensure stdout is flushed
    io::stdout().flush().unwrap();
    io::stdout().write_all(&buffer.into_inner()).unwrap();
    io::stdout().flush().unwrap();
}
#[test]
fn test_valid_numbers() {
    assert_eq!(parse_number_and_report("-69").unwrap(), Number::Int(-69));    
    assert_eq!(parse_number_and_report("+69").unwrap(), Number::Int(69));
    assert_eq!(parse_number_and_report("12345").unwrap(), Number::Int(12345));
    assert_eq!(parse_number_and_report("10_000").unwrap(), Number::Int(10_000));
    assert_eq!(parse_number_and_report("3.1415").unwrap(), Number::Float(3.1415));
}
