use std::collections::HashMap;

use nom::{
    branch::alt,
    bytes::complete::{escaped_transform, tag, take_till1, take_while, take_while_m_n},
    character::complete::char,
    combinator::map,
    error::{context, ContextError, ParseError},
    multi::separated_list0,
    number::complete::double,
    sequence::{delimited, preceded, separated_pair},
    IResult,
};

#[derive(Debug, PartialEq)]
pub enum JsonValue {
    String(String),
    Bool(bool),
    Null,
    Number(f64), // All parsed to floating point numbers
    Object(HashMap<String, JsonValue>),
    Array(Vec<JsonValue>),
}

/// whitespace
/// nom::character::complete::multispace0
fn parse_whitespace<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&str, &str, E> {
    take_while(|ch| ch == ' ' || ch == '\n' || ch == '\r' || ch == '\t')(input)
}

/// number
/// nom::number::complete::double
fn parse_number<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&str, f64, E> {
    double(input)
}

/// string
/// alt：空字符，非空字符
fn parse_string<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    input: &'a str,
) -> IResult<&str, String, E> {
    context(
        "string",
        alt((
            map(tag("\"\""), |_| "".to_owned()),
            delimited(tag("\""), parse_str, tag("\"")),
        )),
    )(input)
}

/// normal string value
/// take_till：与take_while相反，take_til是截止条件
/// 必须使用take_till1，至少有一个
fn normal_str<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&str, &str, E> {
    take_till1(|ch: char| ch == '\\' || ch == '"' || ch.is_ascii_control())(input)
}

/// escapable characters
/// 4 hex digits 是 &str，其他的也只能用tag了，不能用char
fn escapable<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    input: &'a str,
) -> IResult<&str, char, E> {
    context(
        "escape",
        alt((
            char('"'),
            char('\\'),
            char('/'),
            map(char('b'), |_| '\u{0008}'),
            map(char('f'), |_| '\u{000C}'),
            map(char('n'), |_| '\n'),
            map(char('r'), |_| '\r'),
            map(char('t'), |_| '\t'),
            hex_char,
        )),
    )(input)
}

/// 4 hex digits
/// preceded：
/// peek：不消耗输入
fn hex_char<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    input: &'a str,
) -> IResult<&str, char, E> {
    context(
        "hex_char",
        preceded(
            tag("u"),
            map(
                take_while_m_n(4, 4, |ch: char| ch.is_ascii_hexdigit() || ch == 'u'),
                |s: &str| std::char::from_u32(u32::from_str_radix(s, 16).unwrap()).unwrap(),
            ),
        ),
    )(input)
}

/// str
/// escaped：
/// - normal 常规字符判断函数
/// - control 控制字符
/// - escapade 可转义字符
fn parse_str<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    input: &'a str,
) -> IResult<&str, String, E> {
    escaped_transform(normal_str, '\\', escapable)(input)
}

/// bool
/// map 映射结果，总固定一个结果，可以使用 `value(true, tag("true"))` 简写
/// map 实际是 `Result::map` 的函数包装，延迟parse
/// value 也是，返回的值有value clone而来
fn parse_bool<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&str, bool, E> {
    alt((map(tag("false"), |_| false), map(tag("true"), |_| true)))(input)
}

fn parse_null<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&str, JsonValue, E> {
    map(tag("null"), |_| JsonValue::Null)(input)
}

fn parse_value<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    input: &'a str,
) -> IResult<&str, JsonValue, E> {
    context(
        "value",
        delimited(
            parse_whitespace,
            alt((
                map(parse_string, JsonValue::String),
                map(parse_bool, JsonValue::Bool),
                map(parse_number, JsonValue::Number),
                parse_null,
                map(parse_object, JsonValue::Object),
                map(parse_array, JsonValue::Array),
            )),
            parse_whitespace,
        ),
    )(input)
}

fn parse_array<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    input: &'a str,
) -> IResult<&str, Vec<JsonValue>, E> {
    context(
        "array",
        delimited(
            char('['),
            separated_list0(
                char(','),
                delimited(parse_whitespace, parse_value, parse_whitespace),
            ),
            char(']'),
        ),
    )(input)
}

fn parse_object<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    input: &'a str,
) -> IResult<&str, HashMap<String, JsonValue>, E> {
    context(
        "object",
        delimited(
            char('{'),
            map(
                separated_list0(
                    tag(","),
                    separated_pair(
                        delimited(parse_whitespace, parse_string, parse_whitespace), // key
                        char(':'),                                                   // :
                        parse_value,                                                 // value
                    ),
                ),
                |list| {
                    list.into_iter()
                        .map(|(key, val)| (key.to_owned(), val))
                        .collect()
                },
            ),
            char('}'),
        ),
    )(input)
}

/// JSON root element
/// only object or array
pub fn parse_root<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    input: &'a str,
) -> IResult<&str, JsonValue, E> {
    delimited(
        parse_whitespace,
        alt((
            map(parse_object, JsonValue::Object),
            map(parse_array, JsonValue::Array),
        )),
        parse_whitespace,
    )(input)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use nom::{
        error::{convert_error, Error},
        Finish,
    };

    use crate::parser::{
        hex_char, normal_str, parse_array, parse_bool, parse_object, parse_str, parse_string,
        parse_value, JsonValue,
    };

    #[test]
    fn test_normal_str() {
        assert_eq!(normal_str::<Error<&str>>("abc\n"), Ok(("\n", "abc")));
        assert_eq!(normal_str::<Error<&str>>("abc\\n"), Ok(("\\n", "abc")));
    }

    #[test]
    fn test_hex_char() {
        assert_eq!(hex_char::<Error<&str>>("u1234abc"), Ok(("abc", '\u{1234}')));
    }

    #[test]
    fn test_str() {
        assert_eq!(
            parse_str::<Error<&str>>(r#"abc\n \u1234"#),
            Ok(("", "abc\n \u{1234}".to_owned()))
        )
    }

    #[test]
    fn test_string() {
        assert_eq!(
            parse_string::<Error<&str>>(r#""""#),
            Ok(("", "".to_owned()))
        );
        assert_eq!(
            parse_string::<Error<&str>>(r#""abc\n \u1234""#),
            Ok(("", "abc\n \u{1234}".to_owned()))
        );
    }

    #[test]
    fn test_bool() {
        assert_eq!(parse_bool::<Error<&str>>("false"), Ok(("", false)));
        assert_eq!(parse_bool::<Error<&str>>("true"), Ok(("", true)));
    }

    #[test]
    fn test_value() {
        assert_eq!(
            parse_value::<Error<&str>>("null"),
            Ok(("", JsonValue::Null))
        );
        assert_eq!(
            parse_value::<Error<&str>>("true"),
            Ok(("", JsonValue::Bool(true)))
        );
        assert_eq!(
            parse_value::<Error<&str>>("false"),
            Ok(("", JsonValue::Bool(false)))
        );
        assert_eq!(
            parse_value::<Error<&str>>("\"\""),
            Ok(("", JsonValue::String("".to_owned())))
        );
    }

    #[test]
    fn test_array() {
        assert_eq!(parse_array::<Error<&str>>(r#"[]"#), Ok(("", vec![])));
        assert_eq!(
            parse_array::<Error<&str>>(r#"["string",   null, 0 , false, [  null]    ,{}]"#),
            Ok((
                "",
                vec![
                    JsonValue::String("string".to_owned()),
                    JsonValue::Null,
                    JsonValue::Number(0.),
                    JsonValue::Bool(false),
                    JsonValue::Array(vec![JsonValue::Null]),
                    JsonValue::Object(HashMap::new())
                ],
            ))
        );
    }

    #[test]
    fn test_object() {
        let mut rst = HashMap::new();
        rst.insert("a".to_owned(), JsonValue::Null);
        rst.insert("b".to_owned(), JsonValue::Array(vec![]));
        rst.insert("c".to_owned(), JsonValue::Object(HashMap::new()));
        assert_eq!(
            parse_object::<Error<&str>>(r#"{"a": null, "b": [] , "c" :{} }"#),
            Ok(("", rst))
        );
    }

    #[test]
    fn test_unclosed_array() {
        println!(
            "{}",
            convert_error("[,]", parse_array("[,]").finish().err().unwrap())
        );
    }
}
