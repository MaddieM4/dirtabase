use arkive::Digest;
use hex::FromHexError;
use serde::Serialize;
use strum_macros::EnumIter;

#[derive(Debug, PartialEq)]
pub enum ParseError {
    MissingArg { oc: OpCode, name: &'static str },
    TooManyArgs { oc: OpCode, excess: usize },
    ArgBeforeFirstOp(String),
    InvalidDigest(String, FromHexError),
}
impl From<ParseError> for std::io::Error {
    fn from(pe: ParseError) -> Self {
        Self::other(match pe {
            ParseError::MissingArg { oc, name } => format!("Op {:?} missing arg {}", oc, name),
            ParseError::TooManyArgs { oc, excess } => {
                format!("Op {:?} given {} too many arguments", oc, excess)
            }
            ParseError::ArgBeforeFirstOp(arg) => {
                format!("Arg {:?} given before any operations", arg)
            }
            ParseError::InvalidDigest(arg, err) => {
                format!(
                    "Arg {:?} could not be parsed as a hex digest: {:?}",
                    arg, err
                )
            }
        })
    }
}

#[derive(Debug, PartialEq, Clone, Copy, EnumIter)]
pub enum OpCode {
    Empty,
    Import,
    Export,
    Merge,
    Prefix,
    Rename,
    Filter,
    Download,
    DownloadImpure,
    CmdImpure,
}

#[derive(Debug, PartialEq, Clone, Serialize)]
pub enum Op {
    Empty,
    Import { base: String, targets: Vec<String> },
    Export(String),
    Merge,
    Prefix(String),
    Filter(String),
    Rename(String, String),
    Download(String, Digest),
    DownloadImpure(String),
    CmdImpure(String),
}

impl OpCode {
    pub fn to_op(&self, args: Vec<String>) -> Result<Op, ParseError> {
        let mut it = args.into_iter();
        match self {
            Self::Empty => {
                no_further_params(self, &mut it)?;
                Ok(Op::Empty)
            }
            Self::Import => {
                let base = consume_param(self, "base", &mut it)?;
                Ok(Op::Import {
                    base: base,
                    targets: it.collect(),
                })
            }
            Self::Export => {
                let dest = consume_param(self, "dest", &mut it)?;
                no_further_params(self, &mut it)?;
                Ok(Op::Export(dest))
            }
            Self::Merge => {
                no_further_params(self, &mut it)?;
                Ok(Op::Merge)
            }
            Self::Prefix => {
                let prefix = consume_param(self, "prefix", &mut it)?;
                no_further_params(self, &mut it)?;
                Ok(Op::Prefix(prefix))
            }
            Self::Filter => {
                let pattern = consume_param(self, "pattern", &mut it)?;
                no_further_params(self, &mut it)?;
                Ok(Op::Filter(pattern))
            }
            Self::Rename => {
                let pattern = consume_param(self, "pattern", &mut it)?;
                let replacement = consume_param(self, "replacement", &mut it)?;
                no_further_params(self, &mut it)?;
                Ok(Op::Rename(pattern, replacement))
            }
            Self::Download => {
                let url = consume_param(self, "url", &mut it)?;
                let hash = consume_param(self, "hash", &mut it)?;
                no_further_params(self, &mut it)?;
                let digest =
                    Digest::from_hex(&hash).map_err(|e| ParseError::InvalidDigest(hash, e))?;
                Ok(Op::Download(url, digest))
            }
            Self::DownloadImpure => {
                let url = consume_param(self, "url", &mut it)?;
                no_further_params(self, &mut it)?;
                Ok(Op::DownloadImpure(url))
            }
            Self::CmdImpure => {
                let cmd = consume_param(self, "cmd", &mut it)?;
                no_further_params(self, &mut it)?;
                Ok(Op::CmdImpure(cmd))
            }
        }
    }

    pub fn from_arg(arg: &str) -> Option<Self> {
        match arg {
            "--empty" => Some(Self::Empty),
            "--import" => Some(Self::Import),
            "--export" => Some(Self::Export),
            "--merge" => Some(Self::Merge),
            "--prefix" => Some(Self::Prefix),
            "--filter" => Some(Self::Filter),
            "--rename" => Some(Self::Rename),
            "--download" => Some(Self::Download),
            "--download-impure" => Some(Self::DownloadImpure),
            "--cmd-impure" => Some(Self::CmdImpure),
            _ => None,
        }
    }
}

impl Op {
    pub fn to_code(&self) -> OpCode {
        match self {
            Self::Empty => OpCode::Empty,
            Self::Import { .. } => OpCode::Import,
            Self::Export(_) => OpCode::Export,
            Self::Merge => OpCode::Merge,
            Self::Prefix(_) => OpCode::Prefix,
            Self::Filter(_) => OpCode::Filter,
            Self::Rename(_, _) => OpCode::Rename,
            Self::Download(_, _) => OpCode::Download,
            Self::DownloadImpure(_) => OpCode::DownloadImpure,
            Self::CmdImpure(_) => OpCode::CmdImpure,
        }
    }
}

pub fn parse_pipeline<T>(args: impl IntoIterator<Item = T>) -> Result<Vec<Op>, ParseError>
where
    T: AsRef<str>,
{
    let mut ops = Vec::<(OpCode, Vec<String>)>::new();
    for arg in args {
        if let Some(oc) = OpCode::from_arg(arg.as_ref()) {
            ops.push((oc, vec![]))
        } else {
            let latest = ops
                .last_mut()
                .ok_or_else(|| ParseError::ArgBeforeFirstOp(arg.as_ref().into()))?;
            latest.1.push(arg.as_ref().into());
        }
    }
    ops.into_iter().map(|(oc, args)| oc.to_op(args)).collect()
}

fn consume_param<T>(
    oc: &OpCode,
    name: &'static str,
    args: &mut impl Iterator<Item = String>,
) -> Result<T, ParseError>
where
    T: From<String>,
{
    let arg = args.next().ok_or_else(|| ParseError::MissingArg {
        oc: *oc,
        name: name,
    })?;

    Ok(arg.into())
}

fn no_further_params(
    oc: &OpCode,
    args: &mut impl Iterator<Item = String>,
) -> Result<(), ParseError> {
    let c = args.count();
    if c == 0 {
        Ok(())
    } else {
        Err(ParseError::TooManyArgs { oc: *oc, excess: c })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn round_trip() -> Result<(), ParseError> {
        let cases = [(OpCode::Import, vec!["hello", "world"])];
        for (oc, args) in cases {
            let args = args.into_iter().map(|x| x.to_owned()).collect();
            let op = oc.to_op(args)?;
            assert_eq!(op.to_code(), oc);
        }
        Ok(())
    }

    #[test]
    fn oc_from_arg() {
        assert_eq!(OpCode::from_arg("--help"), None);
        assert_eq!(OpCode::from_arg(""), None);
        assert_eq!(OpCode::from_arg("some param"), None);

        assert_eq!(OpCode::from_arg("--import"), Some(OpCode::Import));
    }

    #[test]
    fn parse() {
        assert_eq!(parse_pipeline([] as [&str; 0]), Ok(vec![]));
        assert_eq!(
            parse_pipeline(["--import"]),
            Err(ParseError::MissingArg {
                oc: OpCode::Import,
                name: "base",
            })
        );
        assert_eq!(
            parse_pipeline(["--import", "base"]),
            Ok(vec![Op::Import {
                base: "base".into(),
                targets: vec![]
            },])
        );
        assert_eq!(
            parse_pipeline(["--import", "base", "hello", "world"]),
            Ok(vec![Op::Import {
                base: "base".into(),
                targets: vec!["hello".into(), "world".into(),]
            },])
        );

        assert_eq!(
            parse_pipeline(["--empty", "oh", "no"]),
            Err(ParseError::TooManyArgs {
                oc: OpCode::Empty,
                excess: 2,
            })
        );
        assert_eq!(
            parse_pipeline(["--empty", "--empty", "--empty"]),
            Ok(vec![Op::Empty, Op::Empty, Op::Empty])
        );
    }
}
