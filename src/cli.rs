use crate::archive::core::Triad;
use crate::logger::Logger;
use crate::op::{perform as perform_op, Op};
use indoc::indoc;
use std::io::Write;
use std::process::ExitCode;

// TODO: Generate this based on docs from src/op/ops/*.rs
const USAGE: &'static str = indoc! {"
    usage: dirtabase [--help|--version|pipeline...]

    A pipeline is made of one or more operations:

    # Put an empty archive on the top of the stack
     --empty

    # Import external files into database (each param becomes an archive).
     --import dir1 dir2 ... dirN

    # Export files from the DB to the operating system.
    # Consumes the last N archives on the stack, where N is the number of params.
     --export .

    # Merge all archives on the stack into one, consuming them.
     --merge

    # Filter an archive, keeping only the files where the path matches the pattern.
     --filter '^/hello'
     --filter 'x|y'

    # Rename entries in an archive with a regex find and replace.
     --replace 'foe' 'friend'
     --replace '\\.([a-z]*)$' '.${1}.old'

    # Rename entries in an archive, restricted to changing the START of paths.
     --prefix 'overly/nested/' ''

    # Unpack an archive to a tempdir, run a command there, and reimport the directory.
     --cmd-impure 'echo \"some text\" > file.txt'
"};

/// What we decide to do based on CLI arguments
#[derive(PartialEq, Debug)]
pub enum Behavior {
    Help,
    Version,
    UnexpectedArg(String),
    Pipeline(Vec<String>),
}

pub fn parse<S>(args: impl Iterator<Item = S>) -> Behavior
where
    S: AsRef<str>,
{
    let mut pipeline_args: Vec<String> = vec![];
    for arg in args {
        match arg.as_ref() {
            "--version" => return Behavior::Version,
            "--help" => return Behavior::Help,
            other => pipeline_args.push(other.to_owned()),
        }
    }

    if pipeline_args.is_empty() {
        Behavior::Help
    } else {
        Behavior::Pipeline(pipeline_args)
    }
}

pub fn execute<OUT, ERR>(behavior: Behavior, log: &mut Logger<OUT, ERR>) -> ExitCode
where
    OUT: std::io::Write,
    ERR: std::io::Write,
{
    let result = match behavior {
        Behavior::Help => write!(log.stdout, "{}", USAGE),
        Behavior::Version => write!(log.stdout, "{}\n", env!("CARGO_PKG_VERSION")),
        Behavior::UnexpectedArg(a) => write!(log.stdout, "Unexpected argument: {}\n", a),
        Behavior::Pipeline(args) => execute_pipeline(args, log),
    };
    match result {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            write!(log.stdout, "Failed to execute: {:?}\n", e)
                .expect("Failed to print failure msg");
            ExitCode::from(1)
        }
    }
}

fn execute_pipeline<OUT, ERR>(
    steps: Vec<String>,
    _log: &mut Logger<OUT, ERR>,
) -> std::io::Result<()>
where
    OUT: std::io::Write,
    ERR: std::io::Write,
{
    let store = crate::storage::new("./.dirtabase_db")?;
    let cfg = crate::op::helpers::Config::new(&store);
    cfg.ctx().parse_apply(steps)?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_empty() {
        assert_eq!(parse(Vec::<String>::new().iter()), Behavior::Help);
    }

    #[test]
    fn parse_help() {
        assert_eq!(parse(vec!["--help"].iter()), Behavior::Help);
    }

    #[test]
    fn parse_version() {
        assert_eq!(parse(vec!["--version"].iter()), Behavior::Version);
    }

    #[test]
    fn parse_conflict() {
        assert_eq!(parse(vec!["--help", "--version"].iter()), Behavior::Help);
        assert_eq!(parse(vec!["--version", "--help"].iter()), Behavior::Version);
    }

    #[test]
    fn parse_pipelines() {
        assert_eq!(
            parse(vec!["--import", "foo", "bar"].iter()),
            Behavior::Pipeline(vec!["--import".into(), "foo".into(), "bar".into()])
        );
    }

    #[test]
    fn execute_help() {
        let mut stdout: Vec<u8> = vec![];
        let mut stderr: Vec<u8> = vec![];
        let mut log = Logger::new(&mut stdout, &mut stderr);
        execute(Behavior::Help, &mut log);
        assert_eq!(&String::from_utf8(stdout).unwrap(), USAGE);
    }

    #[test]
    fn execute_version() {
        let mut stdout: Vec<u8> = vec![];
        let mut stderr: Vec<u8> = vec![];
        let mut log = Logger::new(&mut stdout, &mut stderr);
        execute(Behavior::Version, &mut log);
        assert_eq!(
            String::from_utf8(stdout).unwrap(),
            env!("CARGO_PKG_VERSION").to_owned() + "\n"
        );
    }

    #[test]
    fn execute_unexpected_arg() {
        let mut stdout: Vec<u8> = vec![];
        let mut stderr: Vec<u8> = vec![];
        let mut log = Logger::new(&mut stdout, &mut stderr);
        execute(Behavior::UnexpectedArg("xyz".into()), &mut log);
        assert_eq!(
            String::from_utf8(stdout).unwrap(),
            "Unexpected argument: xyz\n"
        );
    }

    #[test]
    fn execute_pipeline_import() {
        let mut stdout: Vec<u8> = vec![];
        let mut stderr: Vec<u8> = vec![];
        let mut log = Logger::new(&mut stdout, &mut stderr);
        execute(
            Behavior::Pipeline(vec!["--import".into(), "./fixture".into()]),
            &mut log,
        );
        assert_eq!(
            String::from_utf8(stdout).unwrap(),
            indoc! {"
            --- Import ---
            json-plain-90d0cf810af44cbf7a5d24a9cca8bad6e3724606b28880890b8639da8ee6f7e4
        "}
        );
    }

    #[test]
    fn execute_pipeline_export() {
        let mut stdout: Vec<u8> = vec![];
        let mut stderr: Vec<u8> = vec![];
        let mut log = Logger::new(&mut stdout, &mut stderr);
        let dir = tempfile::tempdir().expect("Failed to create temporary directory");
        execute(
            Behavior::Pipeline(vec![
                "--import".into(),
                "./fixture".into(),
                "--export".into(),
                dir.path().to_str().unwrap().into(),
            ]),
            &mut log,
        );
        assert_eq!(
            String::from_utf8(stdout).unwrap(),
            indoc! {"
            --- Import ---
            json-plain-90d0cf810af44cbf7a5d24a9cca8bad6e3724606b28880890b8639da8ee6f7e4
            --- Export ---
        "}
        );
        assert!(dir.path().join("dir1/dir2/nested.txt").exists());
    }
}
