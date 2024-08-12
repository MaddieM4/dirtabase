pub use crate::archive::core::Archive;
pub use crate::op::helpers::*;
pub use std::io::{Error, Result};
pub use std::path::Path;

#[derive(Debug)]
pub struct UnpackError<const N: usize> {
    op: &'static str,
    arg_names: [&'static str; N],
    n_provided: usize,
}

impl<const N: usize> std::fmt::Display for UnpackError<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "--{} takes {} arguments ({:?}), got {}",
            self.op, N, self.arg_names, self.n_provided,
        )
    }
}
impl<const N: usize> std::error::Error for UnpackError<N> {}
impl<const N: usize> From<UnpackError<N>> for std::io::Error {
    fn from(e: UnpackError<N>) -> Self {
        Self::other(e)
    }
}

pub fn unpack<const N: usize, T, A>(
    op: &'static str,
    args: A,
    arg_names: [&'static str; N],
) -> std::result::Result<[String; N], UnpackError<N>>
where
    T: AsRef<str>,
    A: IntoIterator<Item = T>,
{
    let args: Vec<String> = args.into_iter().map(|s| s.as_ref().to_owned()).collect();
    if args.len() != N {
        return Err(UnpackError {
            op: op,
            arg_names: arg_names,
            n_provided: args.len(),
        });
    } else {
        Ok(args.try_into().unwrap())
    }
}
