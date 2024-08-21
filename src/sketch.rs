use ariadne::{Color, Label, Report, ReportKind, Source};
use nom::{
    bytes::complete::take_while1,
    character::complete::{char, digit1, one_of},
    combinator::{opt, recognize, all_consuming},
    sequence::{pair, preceded},
    error::{ParseError, ErrorKind},
    IResult,
};
// use thiserror::Error;
use std::io::{self, Write,Cursor};

#[derive(Debug, PartialEq)]
enum Number {
    Int(i64),
    Float(f64),
}


#[derive(Debug)]
// #[error("{reason} at {span:?} in input: {src}")]
struct NumberParseError {
    src: String,
    span: (usize, usize),
    reason: String,
}

impl ParseError<&str> for NumberParseError {
    fn from_error_kind(input: &str, kind: ErrorKind) -> Self {
        NumberParseError {
            src: input.to_string(),
            span: (0, input.len()),
            reason: kind.description().to_string(),
        }
    }

    fn append(input: &str, kind: ErrorKind, mut other: Self) -> Self {
        // Extend the span to cover the new input
        other.span.1 += input.len();

        // Append the new part of the input to the source
        other.src.push_str(input);

        // Append the new error kind to the reason
        other.reason.push_str(&format!(", {}", kind.description()));

        other
    }

    fn from_char(input: &str, _: char) -> Self {
        NumberParseError {
            src: input.to_string(),
            span: (0, input.len()),
            reason: "Unexpected character".to_string(),
        }
    }
}


fn parse_number(input: &str) -> IResult<&str, Number, NumberParseError> {
    //save base for reports
    let original = input;//.to_string(); 

    // Parse the sign, if any
    let (input, sign) = opt(one_of("+-"))(input)?;

    // Parse the main digits and optional underscores
    let (input, digits) = recognize(pair(
        digit1,
        opt(pair(char('_'), take_while1(|c: char| c.is_digit(10)))),
    ))(input)?;

    // Remove underscores from the parsed digits
    let mut cleaned_digits: String = digits.chars().filter(|&c| c != '_').collect();

    // Check for the fractional part, if any
    let (input, fractional) = opt(preceded(
        char('.'),
        recognize(pair(digit1, opt(pair(char('_'), take_while1(|c: char| c.is_digit(10)))))),
    ))(input)?;


    if let Some(fractional) = fractional {
        cleaned_digits.push('.');
        cleaned_digits.extend(fractional.chars().filter(|&c| c != '_'));
        let number = cleaned_digits.parse::<f64>().map_err(|_| {
            nom::Err::Error(NumberParseError {
                src: original.to_string(),
                span: (0, original.len()),
                reason: "Failed to parse as float".to_string(),
            })
        })?;
        
        let number = if sign == Some('-') { -number } else { number };
        Ok((input, Number::Float(number)))
    } else {
        let number = cleaned_digits.parse::<i64>().map_err(|_| {
            nom::Err::Error(NumberParseError {
                src: original.to_string(),
                span: (0, original.len()),
                reason: "Failed to parse as integer".to_string(),
            })
        })?;

        let number = if sign == Some('-') { -number } else { number };
        Ok((input, Number::Int(number)))
    }
}


fn parse_and_report(input: &str) -> Result<Number, NumberParseError> {
    match all_consuming(parse_number)(input) {
        Ok((_, number)) => Ok(number),
        Err(nom::Err::Error(mut e)) | Err(nom::Err::Failure(mut e)) => {
            // Fix src and span in case of Error or Failure
            e.src = input.to_string();
            e.span = (0, input.len());
            Err(e)
        }
        Err(nom::Err::Incomplete(_)) => Err(NumberParseError {
            src: input.to_string(),
            span: (0, input.len()),
            reason: "Incomplete input: more data required".to_string(),
        }),
    }
}


fn report_error(error: NumberParseError,file:impl Write) {
    let source = Source::from(&error.src);
    let report = Report::build(ReportKind::Error, "input", error.span.1)
        .with_message("Failed to parse number")
        .with_label(Label::new(("input", error.span.0..error.span.1))
            // .with_message(format!("{}", error.nom_error))
            .with_color(Color::Red))
        .finish();

    // io::stdout().flush().unwrap();
    report.write(("input", source),file).unwrap();
    // io::stdout().flush().unwrap();
}




#[test]
fn test_invalid_numbers() {
    println!("Testing invalid number inputs...");

    let mut buffer = Cursor::new(Vec::new());

    {
        if let Err(e) = parse_and_report("12_34_") {
            report_error(e, &mut buffer);
        } else {
            panic!("Expected an error for input '12_34_'");
        }
    }

    {
        if let Err(e) = parse_and_report("123a45") {
            report_error(e, &mut buffer);
        } else {
            panic!("Expected an error for input '123a45'");
        }
    }

    {
        if let Err(e) = parse_and_report("3.14.15") {
            report_error(e, &mut buffer);
        } else {
            panic!("Expected an error for input '3.14.15'");
        }
    }

    {
        if let Err(e) = parse_and_report("311 322") {
            report_error(e, &mut buffer);
        } else {
            panic!("Expected an error for input '311 322'");
        }
    }

    // Ensure stdout is flushed
    io::stdout().flush().unwrap();
    io::stdout().write_all(&buffer.into_inner()).unwrap();
    io::stdout().flush().unwrap();

}


#[test]
fn test_valid_numbers() {
    assert_eq!(parse_and_report("-69").unwrap(), Number::Int(-69));    
    assert_eq!(parse_and_report("+69").unwrap(), Number::Int(69));
    assert_eq!(parse_and_report("12345").unwrap(), Number::Int(12345));
    assert_eq!(parse_and_report("10_000").unwrap(), Number::Int(10_000));
    assert_eq!(parse_and_report("3.1415").unwrap(), Number::Float(3.1415));
}
