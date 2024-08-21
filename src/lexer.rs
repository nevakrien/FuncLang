use nom::{
    bytes::complete::{take_while1, take_till},
    combinator::{recognize, opt},
    sequence::pair,
    IResult,
};

fn lex_word(input: &str) -> IResult<&str, &str> {
    fn is_initial_char(c: char) -> bool {
        c.is_alphabetic() || c == '_'
    }

    fn is_alphanumeric_or_underscore(c: char) -> bool {
        c.is_alphanumeric() || c == '_'
    }

    recognize(pair(
        take_while1(is_initial_char), // First character: alphabetic or underscore
        opt(take_while1(is_alphanumeric_or_underscore)), // Rest: alphanumeric or underscore
    ))(input)
}

fn lex_delimited_word(input: &str) -> IResult<&str, &str> {
    let (input, _) = take_till(|c: char| c.is_alphanumeric() || c == '_')(input)?;
    lex_word(input)
}

#[test]
fn test_lex_valid_word_with_no_delimiter() {
    let input = "abc_12";
    let result = lex_delimited_word(input);
    assert!(result.is_ok());
    let (remaining, word) = result.unwrap();
    assert_eq!(word, "abc_12");
    assert_eq!(remaining, "");
}

#[test]
fn test_lex_valid_word_with_multiple_spaces() {
    let input = "      abc_12     ";
    let result = lex_delimited_word(input);
    assert!(result.is_ok());
    let (remaining, word) = result.unwrap();
    assert_eq!(word, "abc_12");
    assert_eq!(remaining, "     ");
}

#[test]
fn test_lex_invalid_word_starting_with_number() {
    let input = " 123abc ";
    let result = lex_delimited_word(input);
    assert!(result.is_err());
}

#[test]
fn test_lex_word_with_comma() {
    let input = ",abc_12,";
    let result = lex_delimited_word(input);
    assert!(result.is_ok());
    let (remaining, word) = result.unwrap();
    assert_eq!(word, "abc_12");
    assert_eq!(remaining, ",");
}

#[test]
fn test_lex_word_with_parenthesis() {
    let input = ")word_123(";
    let result = lex_delimited_word(input);
    assert!(result.is_ok());
    let (remaining, word) = result.unwrap();
    assert_eq!(word, "word_123");
    assert_eq!(remaining, "(");
}

#[test]
fn test_lex_word_with_underscore_only() {
    let input = " _abc123 ";
    let result = lex_delimited_word(input);
    assert!(result.is_ok());
    let (remaining, word) = result.unwrap();
    assert_eq!(word, "_abc123");
    assert_eq!(remaining, " ");
}

#[test]
fn test_lex_word_with_only_one_valid_character() {
    let input = " _ ";
    let result = lex_delimited_word(input);
    assert!(result.is_ok());
    let (remaining, word) = result.unwrap();
    assert_eq!(word, "_");
    assert_eq!(remaining, " ");
}
