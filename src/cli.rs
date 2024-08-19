use crate::context::Context;
use crate::doc::usage;
use crate::logger::Logger;
use arkive::types::DB;
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
    use crate::test_tools::*;
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
            vec!["--import".into(), ".".into(), "fixture".into()],
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
             + Can cache? false
             + Is in cache? false
            8c958951d9f61be6a7b1ec48611710efc3d12ee71f3dc6ac34251afe4a95378e
        "},
                ""
            )
        );
    }

    #[test]
    fn test_pipeline_caching() {
        let db = DB::new_temp().expect("Temp DB");
        let mut logger = Logger::new_vec();
        let res = cli(
            vec![
                "--download".into(),
                REPRODUCIBLE_URL.into(),
                REPRODUCIBLE_DIGEST.into(),
                "--download".into(),
                REPRODUCIBLE_URL.into(),
                REPRODUCIBLE_DIGEST.into(),
            ],
            &db,
            &mut logger,
        );

        assert!(res.is_ok());
        assert_eq!(
            logger.recorded(),
            (
                indoc! {"
            ================================================================
            Download
            ================================================================
             + Can cache? true
             + Is in cache? false
            460f3d82bf451fbebd1958fe4714e2a82a6570dda19e0d6f39cd7504adca6088
            ================================================================
            Download
            ================================================================
             + Can cache? true
             + Is in cache? true
            460f3d82bf451fbebd1958fe4714e2a82a6570dda19e0d6f39cd7504adca6088
            460f3d82bf451fbebd1958fe4714e2a82a6570dda19e0d6f39cd7504adca6088
        "},
                ""
            )
        );
    }
}
