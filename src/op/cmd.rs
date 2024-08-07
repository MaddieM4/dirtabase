use crate::storage::traits::*;
use crate::archive::core::Triad;
use std::process::Command;
use tempfile::tempdir;
use std::io::{Error, Result};

fn _err<T>(text: &'static str) -> Result<T> {
    Err(Error::other(text))
}

pub fn cmd_impure(store: &impl Storage, mut triads: Vec<Triad>, params: Vec<String>) -> Result<Vec<Triad>> {
    if params.len() != 1 {
        return _err("--cmd-impure only takes 1 argument")
    }
    let command = &params[0];

    // Extract to temporary directory
    let t = triads
        .pop()
        .ok_or(Error::other("Need an archive to work on"))?;
    let dir = tempdir()?;
    crate::stream::archive::source(store, t, crate::stream::osdir::sink(&dir))?;

    // Run the command
    // Equivalent to: bash -o pipefail -e -c '...'
    println!("--- [{}] ---", command);
    let status = Command::new("bash")
        .arg("-o").arg("pipefail")
        .arg("-e")
        .arg("-c")
        .arg(command)
        .current_dir(&dir)
        .status()?;

    if !&status.success() {
        return Err(Error::other(format!("Command {:?} failed with status {:?}", command, status.code().unwrap())))
    }

    let reimport = crate::stream::osdir::source(&dir, crate::stream::archive::sink(store))?;
    triads.push(reimport);
    Ok(triads)
}
