#![allow(dead_code)]
mod cli;
mod context;
mod doc;
mod logger;
mod op;

fn main() -> std::process::ExitCode {
    crate::cli::real_cli()
}
