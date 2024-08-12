#![allow(dead_code, unused_imports)]

mod archive;
mod attr;
mod cli;
mod digest;
mod enc;
mod label;
mod logger;
mod op;
mod storage;
mod stream;

use std::env::args;
use std::io;
use std::process::ExitCode;

fn main() -> ExitCode {
    let (mut stdout, mut stderr) = (io::stdout(), io::stderr());
    let mut logger = logger::Logger::new(&mut stdout, &mut stderr);

    let behavior = cli::parse(args().skip(1));
    cli::execute(behavior, &mut logger)
}
