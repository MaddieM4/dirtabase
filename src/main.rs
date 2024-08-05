#![allow(dead_code,unused_imports)]

mod archive;
mod attr;
mod cli;
mod digest;
mod label;
mod op;
mod storage;
mod stream;

use std::env::args;
use std::io::stdout;

fn main() {
    let behavior = cli::parse(args().skip(1));
    cli::execute(behavior, &mut stdout());
}
