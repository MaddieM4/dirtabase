mod cli;
use std::env::args;
use std::io::stdout;

fn main() {
    let behavior = cli::parse(args());
    cli::execute(behavior, &mut stdout());
}
