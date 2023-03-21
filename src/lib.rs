mod parser;

use nom::{
    error::{convert_error, VerboseError},
    Err,
};
use parser::parse_root;

pub use parser::JsonValue;

pub fn parse(s: &str) -> Result<JsonValue, String> {
    match parse_root::<VerboseError<&str>>(s) {
        Err(Err::Incomplete(_)) | Err(Err::Failure(_)) => Err("failure".to_owned()),
        Err(Err::Error(err)) => Err(convert_error(s, err)),
        Ok((rest, _)) if rest.len() > 0 => Err("错误".to_owned()),
        Ok((_, rst)) => Ok(rst),
    }
}
