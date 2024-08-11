#![allow(dead_code, unused_imports)]

mod archive;
mod attr;
mod cli;
mod digest;
mod enc;
mod label;
mod op;
mod storage;
mod stream;

use std::env::args;
use std::io::stdout;
use std::process::ExitCode;

fn main() -> ExitCode {
    let behavior = cli::parse(args().skip(1));
    cli::execute(behavior, &mut stdout())
}
