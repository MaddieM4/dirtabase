use indoc::indoc;
use std::io::Write;
use crate::archive::core::Triad;
use crate::op::{Op,perform as perform_op};

const USAGE: &'static str = indoc! {"
    usage: dirtabase [--help|--version|pipeline...]

    A pipeline is made of one or more operations:

    # Import external files into database
     --import dir1 dir2 ... dirN

    # Export files from the DB to the operating system.
    # Consumes the last N items of the stack, where N is the number of params.
     --export .

    # Merge all archives on the stack into one, consuming them.
     --merge

    # Filter an archive, keeping only the files where the path matches the pattern.
     --filter '^/hello'
     --filter 'x|y'

    # Rename entries in an archive with a regex find and replace.
     --replace 'foe' 'friend'
     --replace '\\.([a-z]*)$' '.${1}.old'
"};

#[derive(PartialEq,Debug)]
pub struct PipelineStep(Op, Vec<String>);

/// What we decide to do based on CLI arguments
#[derive(PartialEq,Debug)]
pub enum Behavior {
    Help,
    Version,
    UnexpectedArg(String),
    Pipeline(Vec<PipelineStep>),
}

pub fn parse<S>(args: impl Iterator<Item=S>) -> Behavior where S: AsRef<str> {
    let mut pipeline: Vec<PipelineStep> = vec![];

    for arg in args {
        match arg.as_ref() {
            "--version" => return Behavior::Version,
            "--help" => return Behavior::Help,
            "--import" => pipeline.push(PipelineStep(Op::Import, vec![])),
            "--export" => pipeline.push(PipelineStep(Op::Export, vec![])),
            "--merge" => pipeline.push(PipelineStep(Op::Merge, vec![])),
            "--filter" => pipeline.push(PipelineStep(Op::Filter, vec![])),
            "--replace" => pipeline.push(PipelineStep(Op::Replace, vec![])),

            other => if pipeline.is_empty() {
                return Behavior::UnexpectedArg(other.to_owned())
            } else {
                let index = pipeline.len() - 1;
                let current_pipeline = &mut pipeline[index];
                current_pipeline.1.push(other.to_owned())
            },
        }
    }

    if pipeline.is_empty() {
        Behavior::Help
    } else {
        Behavior::Pipeline(pipeline)
    }
}


pub fn execute(behavior: Behavior, stdout: &mut impl Write) {
    match behavior {
        Behavior::Help => write!(stdout, "{}", USAGE),
        Behavior::Version => write!(stdout, "{}\n", env!("CARGO_PKG_VERSION")),
        Behavior::UnexpectedArg(a) => write!(stdout, "Unexpected argument: {}\n", a),
        Behavior::Pipeline(steps) => execute_pipeline(steps, stdout),
    }.expect("Failed to execute");
}

fn execute_pipeline(steps: Vec<PipelineStep>, stdout: &mut impl Write) -> std::io::Result<()> {
    let store = crate::storage::simple::storage("./.dirtabase_db")?;
    let mut triads: Vec<Triad> = vec![];
    for step in steps {
        let (op, params) = (step.0, step.1);
        triads = perform_op(op, &store, triads, params)?;
        write!(stdout, "--- {:?} ---\n", op)?;
        for t in &triads {
            write!(stdout, "{}\n", t)?;
        }
    }
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
    fn parse_unexpected_arg() {
        assert_eq!(parse(vec!["xyz"].iter()), Behavior::UnexpectedArg("xyz".to_owned()));
    }

    #[test]
    fn parse_pipelines() {
        assert_eq!(parse(vec!["--import", "foo", "bar"].iter()), Behavior::Pipeline(vec![
            PipelineStep(Op::Import, vec!["foo".to_owned(), "bar".to_owned()]),
        ]));
        assert_eq!(parse(vec![
            "--import", "foo", "bar",
            "--filter", "some|regex",
            "--export", "dir1", "dir2",
        ].iter()), Behavior::Pipeline(vec![
            PipelineStep(Op::Import, vec!["foo".to_owned(), "bar".to_owned()]),
            PipelineStep(Op::Filter, vec!["some|regex".to_owned()]),
            PipelineStep(Op::Export, vec!["dir1".to_owned(), "dir2".to_owned()]),
        ]));
    }

    #[test]
    fn execute_help() {
        let mut stdout: Vec<u8> = vec![];
        execute(Behavior::Help, &mut stdout);
        assert_eq!(&String::from_utf8(stdout).unwrap(), USAGE);
    }

    #[test]
    fn execute_version() {
        let mut stdout: Vec<u8> = vec![];
        execute(Behavior::Version, &mut stdout);
        assert_eq!(String::from_utf8(stdout).unwrap(), env!("CARGO_PKG_VERSION").to_owned() + "\n");
    }

    #[test]
    fn execute_unexpected_arg() {
        let mut stdout: Vec<u8> = vec![];
        execute(Behavior::UnexpectedArg("xyz".into()), &mut stdout);
        assert_eq!(String::from_utf8(stdout).unwrap(), "Unexpected argument: xyz\n");
    }


    #[test]
    fn execute_pipeline_import() {
        let mut stdout: Vec<u8> = vec![];
        execute(Behavior::Pipeline(vec![
            PipelineStep(Op::Import, vec!["./fixture".into()]),
        ]), &mut stdout);
        assert_eq!(String::from_utf8(stdout).unwrap(), indoc! {"
            --- Import ---
            json-plain-d6467585a5b63a42945759efd8c8a21dfd701470253339477407653e48a3643a
        "});
    }

    #[test]
    fn execute_pipeline_export() {
        let dir = tempfile::tempdir().expect("Failed to create temporary directory");
        let mut stdout: Vec<u8> = vec![];
        execute(Behavior::Pipeline(vec![
            PipelineStep(Op::Import, vec!["./fixture".into()]),
            PipelineStep(Op::Export, vec![dir.path().to_str().unwrap().into()]),
        ]), &mut stdout);
        assert_eq!(String::from_utf8(stdout).unwrap(), indoc! {"
            --- Import ---
            json-plain-d6467585a5b63a42945759efd8c8a21dfd701470253339477407653e48a3643a
            --- Export ---
        "});
        assert!(dir.path().join("dir1/dir2/nested.txt").exists());
    }

}
