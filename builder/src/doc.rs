use crate::context::Context;
use crate::op::{Op, OpCode};
use strum::IntoEnumIterator;

#[allow(dead_code)]
pub struct OpDoc {
    flag: &'static str,
    args: &'static str,
    short: &'static str,
    examples: Vec<ExamplePipeline>,
}

#[allow(dead_code)]
pub struct ExamplePipeline {
    as_txt: Vec<&'static str>,
    as_ops: Vec<Op>,
    as_ctx: &'static dyn Fn(&mut Context) -> std::io::Result<()>,
}

impl OpCode {
    pub fn doc(&self) -> OpDoc {
        match self {
            OpCode::Empty => OpDoc {
                flag: "--empty",
                args: "",
                short: "Push an empty archive to the stack.",
                examples: vec![ExamplePipeline {
                    as_txt: vec!["--empty"],
                    as_ops: vec![Op::Empty],
                    as_ctx: &|ctx: &mut Context| {
                        ctx.empty()?;
                        Ok(())
                    },
                }],
            },
            OpCode::Import => OpDoc {
                flag: "--import",
                args: " base [target...]",
                short: "Copy directories into the DB as archives.",
                examples: vec![ExamplePipeline {
                    as_txt: vec!["--import", ".", "dir1"],
                    as_ops: vec![Op::Import {
                        base: ".".into(),
                        targets: vec!["dir1".into()],
                    }],
                    as_ctx: &|ctx: &mut Context| {
                        ctx.import(".", ["dir1"])?;
                        Ok(())
                    },
                }],
            },
        }
    }
}

pub fn usage() -> String {
    let mut sections: Vec<&str> = vec![];
    sections.push("Usage: dirtabase [op...]\n\n");
    sections.push("Valid ops:\n\n");

    for oc in OpCode::iter() {
        let doc = oc.doc();
        sections.extend([
            doc.flag,
            ": ",
            doc.short,
            "\n    Usage: ",
            doc.flag,
            doc.args,
            "\n\n",
        ]);
    }
    sections.concat()
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::logger::Logger;
    use ::ark::*;
    use indoc::indoc;
    use rusty_fork::rusty_fork_test;
    use std::io::Result;

    #[test]
    fn test_usage() {
        assert!(
            usage().starts_with(indoc! {"
            Usage: dirtabase [op...]

            Valid ops:

            --empty: Push an empty archive to the stack.
                Usage: --empty

            --import: Copy directories into the DB as archives.
                Usage: --import base [target...]
        "}),
            "Got: {:?}",
            usage()
        )
    }

    #[test]
    fn test_flags() {
        for oc in OpCode::iter() {
            let flag = oc.doc().flag;
            assert_eq!(OpCode::from_arg(flag), Some(oc));
        }
    }

    #[derive(Debug, PartialEq)]
    struct ExResults {
        stack_after: Vec<Digest>,
    }

    fn try_example(ex: impl Fn(&mut Context) -> Result<()>) -> Result<ExResults> {
        // Set up playground
        let playground = tempfile::tempdir()?;
        let db = DB::new(playground.path().join(".dirtabase_db"))?;
        let mut log = Logger::new_vec();
        let mut ctx = Context::new(&db, &mut log);
        Ark::scan("../fixture")?.write(playground.path())?;

        // Execute
        let original_dir = std::env::current_dir()?;
        std::env::set_current_dir(playground.path())?;
        let err = ex(&mut ctx);
        std::env::set_current_dir(original_dir)?;
        err?;

        // Turn into ExResults
        Ok(ExResults {
            stack_after: ctx.stack,
        })
    }

    rusty_fork_test! {

        #[test]
        fn test_examples() {
            for oc in OpCode::iter() {
                let examples = oc.doc().examples;
                assert!(examples.len() > 0, "Opcode {:?} needs examples", oc);

                for example in examples {
                    assert_eq!(
                        crate::op::parse_pipeline(example.as_txt),
                        Ok(example.as_ops.clone()),
                        "Example for {:?} has mismatching text and ops",
                        oc
                    );
                    assert_eq!(
                        try_example(example.as_ctx).expect("Example failed in as_ctx"),
                        try_example(|ctx| {
                            for op in &example.as_ops {
                                ctx.apply(&op)?;
                            }
                            Ok(())
                        })
                        .expect("Example failed in as_ops"),
                        "Example for {:?} has mismatching ctx and ops results",
                        oc
                    );
                }
            }
        }

    }
}
