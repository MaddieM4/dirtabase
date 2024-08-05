use indoc::indoc;
use std::io::Write;
use crate::archive::core::Triad;
use crate::op::{Op,perform as perform_op};

const USAGE: &'static str = indoc! {"
    usage: dirtabase [--help|--version|pipeline...]

    A pipeline is made of one or more operations:

    # Import external files into database
     --import dir1 dir2 ... dirN
"};

#[derive(PartialEq,Debug)]
pub struct PipelineStep(Op, Vec<String>);

/// What we decide to do based on CLI arguments
#[derive(PartialEq,Debug)]
pub enum Behavior {
    Help,
    Version,
    Pipeline(Vec<PipelineStep>),
}

pub fn parse<S>(args: impl Iterator<Item=S>) -> Behavior where S: AsRef<str> {
    for arg in args {
        match arg.as_ref() {
            "--version" => return Behavior::Version,
            "--help" => return Behavior::Help,
            _ => (),
        }
    }
    Behavior::Help
}


pub fn execute(behavior: Behavior, stdout: &mut impl Write) {
    match behavior {
        Behavior::Help => write!(stdout, "{}", USAGE),
        Behavior::Version => write!(stdout, "{}\n", env!("CARGO_PKG_VERSION")),
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
    fn execute_pipeline_import() {
        let mut stdout: Vec<u8> = vec![];
        execute(Behavior::Pipeline(vec![
            PipelineStep(Op::Import, vec!["./fixture".to_owned()]),
        ]), &mut stdout);
        assert_eq!(String::from_utf8(stdout).unwrap(), indoc! {"
            --- Import ---
            json-plain-d6467585a5b63a42945759efd8c8a21dfd701470253339477407653e48a3643a
        "});
    }

}
