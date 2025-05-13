use crate::parser::Parser;
use std::fmt::Write;

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum OlpError {
    #[error("StringParseError on char. Already extracted: {0:?}")]
    StringParseError(Vec<String>),
    #[error("Character '{0}' was not expected.")]
    InvalidChar(char),
    #[error("No more characters to consume.")]
    IteratorExhaustion,
}

pub(crate) fn into_olp_string(olp: Vec<String>) -> String {
    if olp.is_empty() {
        return "".to_string();
    }
    let mut olp_s = "(".to_string();
    for elem in olp {
        let _ = write!(olp_s, "\"\"{}\"\" ", elem);
    }
    olp_s.push(')');
    olp_s
}

/// Parse an olp string.
/// ```rust
/// use org_roamers::sqlite::SqliteConnection;
///
/// assert_eq!(
///     SqliteConnection::parse_olp(
///         "(\"VLIW\" \"Instruction\" )".to_string()
///     ).unwrap(),
///     vec!["VLIW".to_string(), "Instruction".to_string()]
/// );
pub fn parse_olp(olp: String) -> anyhow::Result<Vec<String>> {
    let mut parser = Parser::new(&olp);
    let whitespace = |parser: &mut Parser| {
        let mut attempt = parser.attempt();
        attempt.consume_whitespace();
        parser.sync(attempt);
    };

    whitespace(&mut parser);

    let mut attempt = parser.attempt();
    if let None = attempt.consume_char('(') {
        return Err(OlpError::InvalidChar('(').into());
    }
    parser.sync(attempt);

    let mut paths = vec![];

    loop {
        let mut attempt = parser.attempt();
        match attempt.consume_string() {
            Some(path) => paths.push(path),
            None => {
                whitespace(&mut parser);
                let mut attempt = parser.attempt();
                if let Some(_) = attempt.consume_char(')') {
                    return Ok(paths);
                } else {
                    break;
                }
            }
        }
        parser.sync(attempt);
        whitespace(&mut parser);
    }

    Err(OlpError::StringParseError(paths).into())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_olp_parser_correct() {
        const OLP: &'static str = "(\"This is a test\" \"How about that\")";
        let res = parse_olp(OLP.to_string());
        assert_eq!(
            res.unwrap(),
            vec!["This is a test".to_string(), "How about that".to_string()]
        );
    }

    #[test]
    fn test_olp_deserialize_serialize() {
        let olp = "(\"test\" \"other\")";
        let arr = parse_olp(olp.to_string()).unwrap();
        assert_eq!(arr, vec!["test".to_string(), "other".to_string()]);
        let s = into_olp_string(arr);
        assert_eq!(s, "(\"\"test\"\" \"\"other\"\" )");
    }
}
