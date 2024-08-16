use crate::context::Context;
use crate::doc::usage;
use crate::logger::Logger;
use ark::types::DB;
use std::io::{Result, Write};
use std::process::ExitCode;

pub fn cli(args: Vec<String>, db: &DB, log: &mut Logger) -> Result<()> {
    if args.is_empty() {
        write!(log.stdout, "{}", usage())?;
    }
    Context::new(db, log).parse_apply(args)
}

fn infer_db() -> Result<DB> {
    DB::new("./.dirtabase_db")
}

pub fn real_cli() -> ExitCode {
    let db = infer_db().expect("Could not initialize DB");
    let mut logger = Logger::new_real();
    let args: Vec<String> = std::env::args().skip(1).collect();
    cli(args, &db, &mut logger).expect("Pipeline failed");
    ExitCode::SUCCESS
}

#[cfg(test)]
mod test {
    use super::*;
    use indoc::indoc;

    #[test]
    fn test_no_args() {
        let db = DB::new_temp().expect("Temp DB");
        let mut logger = Logger::new_vec();
        let res = cli(vec![], &db, &mut logger);
        let usage_txt = usage();

        assert!(res.is_ok());
        assert_eq!(logger.recorded(), (usage_txt.as_ref(), "",));
    }

    #[test]
    fn test_pipeline() {
        let db = DB::new_temp().expect("Temp DB");
        let mut logger = Logger::new_vec();
        let res = cli(
            vec!["--import".into(), "..".into(), "fixture".into()],
            &db,
            &mut logger,
        );

        assert!(res.is_ok());
        assert_eq!(
            logger.recorded(),
            (
                indoc! {"
            ================================================================
            Import
            ================================================================
            fb9dde674e4002c7646770fcdee7eb2669de2aa90b216f47331f7bd155d0f787
        "},
                ""
            )
        );
    }
}
