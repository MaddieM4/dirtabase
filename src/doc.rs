use crate::context::Context;
use crate::op::{Op, OpCode};
use crate::test_tools::*;
use arkive::Digest;
use std::path::Path;
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
                    as_txt: vec!["--import", ".", "fixture"],
                    as_ops: vec![Op::Import {
                        base: ".".into(),
                        targets: vec!["fixture".into()],
                    }],
                    as_ctx: &|ctx: &mut Context| {
                        ctx.import(".", ["fixture"])?;
                        Ok(())
                    },
                }],
            },
            OpCode::Export => OpDoc {
                flag: "--export",
                args: " dest",
                short: "Output an archive to an OS directory.",
                examples: vec![ExamplePipeline {
                    as_txt: vec!["--import", ".", "fixture", "--export", "./out"],
                    as_ops: vec![
                        Op::Import {
                            base: ".".into(),
                            targets: vec!["fixture".into()],
                        },
                        Op::Export("./out".into()),
                    ],
                    as_ctx: &|ctx: &mut Context| {
                        ctx.import(".", ["fixture"])?.export("./out")?;
                        assert!(Path::new("./out/fixture/dir1/dir2/nested.txt").exists());
                        Ok(())
                    },
                }],
            },
            OpCode::Merge => OpDoc {
                flag: "--merge",
                args: "",
                short: "Merge all archives on the stack into one.",
                examples: vec![ExamplePipeline {
                    as_txt: vec![
                        "--import", ".", "fixture", "src", "--merge", "--export", "./out",
                    ],
                    as_ops: vec![
                        Op::Import {
                            base: ".".into(),
                            targets: vec!["fixture".into(), "src".into()],
                        },
                        Op::Merge,
                        Op::Export("./out".into()),
                    ],
                    as_ctx: &|ctx: &mut Context| {
                        ctx.import(".", ["fixture", "src"])?
                            .merge()?
                            .export("./out")?;
                        assert!(Path::new("./out/fixture/dir1/dir2/nested.txt").exists());
                        assert!(Path::new("./out/src/doc.rs").exists());
                        Ok(())
                    },
                }],
            },
            OpCode::Prefix => OpDoc {
                flag: "--prefix",
                args: " prefix",
                short: "Add a prefix to all paths in the top archive on the stack.",
                examples: vec![ExamplePipeline {
                    as_txt: vec![
                        "--import", ".", "fixture", "--prefix", "foo", "--export", "./out",
                    ],
                    as_ops: vec![
                        Op::Import {
                            base: ".".into(),
                            targets: vec!["fixture".into()],
                        },
                        Op::Prefix("foo".into()),
                        Op::Export("./out".into()),
                    ],
                    as_ctx: &|ctx: &mut Context| {
                        ctx.import(".", ["fixture"])?
                            .prefix("foo")?
                            .export("./out")?;
                        assert!(Path::new("./out/foo/fixture/dir1/dir2/nested.txt").exists());
                        Ok(())
                    },
                }],
            },
            OpCode::Filter => OpDoc {
                flag: "--filter",
                args: " pattern",
                short: "Exclude files and directories where the path doesn't match a regex.",
                examples: vec![ExamplePipeline {
                    as_txt: vec![
                        "--import", ".", "fixture", "--filter", "root", "--export", "./out",
                    ],
                    as_ops: vec![
                        Op::Import {
                            base: ".".into(),
                            targets: vec!["fixture".into()],
                        },
                        Op::Filter("root".into()),
                        Op::Export("./out".into()),
                    ],
                    as_ctx: &|ctx: &mut Context| {
                        ctx.import(".", ["fixture"])?
                            .filter("root")?
                            .export("./out")?;
                        assert!(!Path::new("./out/fixture/dir1/dir2/nested.txt").exists());
                        assert!(Path::new("./out/fixture/file_at_root.txt").exists());
                        Ok(())
                    },
                }],
            },
            OpCode::Download => OpDoc {
                flag: "--download",
                args: " url digest",
                short: "Download a file and verify the archive hash.",
                examples: vec![ExamplePipeline {
                    as_txt: vec![
                        "--download",
                        REPRODUCIBLE_URL,
                        REPRODUCIBLE_DIGEST,
                        "--export",
                        "out",
                    ],
                    as_ops: vec![
                        Op::Download(
                            REPRODUCIBLE_URL.into(),
                            Digest::from_hex(REPRODUCIBLE_DIGEST).expect("Invalid hex digest"),
                        ),
                        Op::Export("out".into()),
                    ],
                    as_ctx: &|ctx: &mut Context| {
                        ctx.download(REPRODUCIBLE_URL, REPRODUCIBLE_DIGEST)?
                            .export("out")?;
                        assert!(Path::new("./out/reproducible.txt").exists());
                        Ok(())
                    },
                }],
            },
            OpCode::DownloadImpure => OpDoc {
                flag: "--download-impure",
                args: " url",
                short: "Download a file without verifying its hash.",
                examples: vec![ExamplePipeline {
                    as_txt: vec!["--download-impure", REPRODUCIBLE_URL, "--export", "out"],
                    as_ops: vec![
                        Op::DownloadImpure(REPRODUCIBLE_URL.into()),
                        Op::Export("out".into()),
                    ],
                    as_ctx: &|ctx: &mut Context| {
                        ctx.download_impure(REPRODUCIBLE_URL)?.export("out")?;
                        assert!(Path::new("./out/reproducible.txt").exists());
                        Ok(())
                    },
                }],
            },
            OpCode::CmdImpure => OpDoc {
                flag: "--cmd-impure",
                args: " cmd",
                short: "Run a command within the top archive on the stack..",
                examples: vec![ExamplePipeline {
                    as_txt: vec!["--empty", "--cmd-impure", "touch grass", "--export", "out"],
                    as_ops: vec![
                        Op::Empty,
                        Op::CmdImpure("touch grass".into()),
                        Op::Export("out".into()),
                    ],
                    as_ctx: &|ctx: &mut Context| {
                        ctx.empty()?.cmd_impure("touch grass")?.export("out")?;
                        assert!(Path::new("./out/grass").exists());
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
            "\n    Examples:\n",
        ]);
        for example in doc.examples {
            sections.push("      dirtabase");
            for arg in example.as_txt {
                sections.extend([" ", arg]);
            }
            sections.push("\n\n");
        }
    }
    sections.concat()
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::logger::Logger;
    use arkive::*;
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
                Examples:
                  dirtabase --empty

            --import: Copy directories into the DB as archives.
                Usage: --import base [target...]
                Examples:
                  dirtabase --import . fixture
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
        Ark::scan("fixture")?.write(playground.path().join("fixture"))?;
        Ark::scan("src")?.write(playground.path().join("src"))?;

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
