use indoc::indoc;
use std::io::Write;

const USAGE: &'static str = indoc! {"
    usage: dirtabase [--help|--version|pipeline...]

    A pipeline is made of one or more operations:

    # Import external files into database
     --import dir1 dir2 ... dirN
"};

/// What we decide to do based on CLI arguments
#[derive(PartialEq,Debug)]
pub enum Behavior {
    Help,
    Version,
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
    }.expect("Failed to execute");
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

}
