pub use crate::archive::core::Archive;
pub use crate::op::helpers::*;
pub use std::io::{Error, Result, Write};

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

pub fn download(store: &crate::storage::Store, url: &str) -> Result<crate::digest::Digest> {
    let response = reqwest::blocking::get(url).map_err(|e| Error::other(e))?;
    let digest = store.cas().write(response)?;
    print!(">> Downloaded {}\n>> Digest: {}\n", url, digest.to_hex());
    Ok(digest)
}

pub fn url_filename(given_url: &str) -> Result<String> {
    let parsed_url = url::Url::parse(&given_url).map_err(|e| Error::other(e))?;
    Ok(parsed_url
        .path_segments()
        .ok_or(Error::other("Could not break URL into path segments"))?
        .last()
        .ok_or(Error::other("Could not determine filename"))?
        .to_owned())
}
