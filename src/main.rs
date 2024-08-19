#![allow(dead_code)]
mod behavior;
mod cli;
mod context;
mod doc;
mod logger;
mod op;
pub(crate) mod test_tools;

fn main() -> std::process::ExitCode {
    crate::cli::real_cli()
}
