use crate::{ParseError, ParseResult};

pub mod parser_int;
pub mod parser_uint;

#[inline(always)]
pub fn check_len(b: &[u8], expected: usize) -> ParseResult<()> {
    let byte_len = b.len();
    if byte_len < expected {
        Err(ParseError::Incomplete {
            needed: Some(expected - byte_len),
        })
    } else {
        Ok(())
    }
}

#[inline(always)]
pub fn parse_boolean(b: &u8) -> ParseResult<bool> {
    match b {
        0 => Ok(false),
        1 => Ok(true),
        2 => Ok(false),
        _ => Err(ParseError::InvalidValue),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_len() {
        let data = b"ABCD";
        assert!(check_len(data, 4).is_ok());
        assert!(check_len(data, 5).is_err());
    }

    #[test]
    fn test_parse_boolean_variants() {
        assert!(matches!(parse_boolean(&0), Ok(false)));
        assert!(matches!(parse_boolean(&1), Ok(true)));
        assert!(matches!(parse_boolean(&2), Ok(false)));
        assert!(parse_boolean(&3).is_err());
        assert!(parse_boolean(&255).is_err());
    }
}
