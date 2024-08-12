use crate::op::gen::*;

#[derive(Debug, PartialEq)]
pub enum ParseError {
    ParamBeforeOp(String),
    UnpackError,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            ParseError::ParamBeforeOp(param) => write!(f, "Param {:?} occurred before first operation. What should it apply to? We can't guess!", param),
            ParseError::UnpackError => write!(f, "Failed to unpack arguments"),
        }
    }
}
impl std::error::Error for ParseError {}
impl From<ParseError> for std::io::Error {
    fn from(p: ParseError) -> Self {
        Self::other(p)
    }
}

pub fn parse<T>(args: impl IntoIterator<Item = T>) -> Result<Vec<Op>, ParseError>
where
    T: AsRef<str>,
{
    let mut output = Vec::<(OpCode, Vec<String>)>::new();
    for arg in args {
        if let Some(oc) = to_opcode(arg.as_ref()) {
            output.push((oc, vec![]));
        } else if let Some(step) = output.last_mut() {
            step.1.push(arg.as_ref().to_owned())
        } else {
            return Err(ParseError::ParamBeforeOp(arg.as_ref().to_owned()));
        }
    }
    output
        .into_iter()
        .map(|(oc, params)| {
            Op::from_code_and_params(oc, params).map_err(|_| ParseError::UnpackError)
        })
        .collect()
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::op::gen;
    use crate::op::ops;

    #[test]
    fn to_opcode() {
        assert_eq!(gen::to_opcode("foo"), None);
        assert_eq!(gen::to_opcode("--import"), Some(OpCode::Import));
    }

    #[test]
    fn parse_empty() {
        assert_eq!(parse([] as [&str; 0]), Ok(vec![]))
    }

    #[test]
    fn parse_one_import() {
        assert_eq!(
            parse(["--import", "hello", "world"]),
            Ok(vec![Op::Import(ops::import::Import(vec![
                "hello".to_owned(),
                "world".to_owned(),
            ])),])
        )
    }

    #[test]
    fn parse_two_imports() {
        assert_eq!(
            parse(["--import", "x", "--import", "y"]),
            Ok(vec![
                Op::Import(ops::import::Import(vec!["x".to_owned()])),
                Op::Import(ops::import::Import(vec!["y".to_owned()])),
            ])
        )
    }

    #[test]
    fn parse_merge() {
        assert_eq!(parse(["--merge"]), Ok(vec![Op::Merge(ops::merge::Merge)]));
        assert_eq!(parse(["--merge", "an arg"]), Err(ParseError::UnpackError));
    }

    #[test]
    fn parse_mixed_ops() {
        assert_eq!(
            parse(["--import", "x", "y", "--merge", "--empty"]),
            Ok(vec![
                Op::Import(ops::import::Import(vec!["x".to_owned(), "y".to_owned()])),
                Op::Merge(ops::merge::Merge),
                Op::Empty(ops::empty::Empty),
            ])
        )
    }
}
