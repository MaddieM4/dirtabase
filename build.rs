use std::fs::File;
use std::io::{Result, Write};

// Produces a list of operation names like "cmd_impure".
// This is based on the "pub mod" list in `src/op/ops.rs`.
fn list_ops() -> Result<Vec<String>> {
    let pattern = r"//\!  - ([a-z_]*)";
    let re = regex::Regex::new(pattern).expect("Failed to compile regex");
    let haystack = std::fs::read_to_string("src/op/ops.rs")?;

    Ok(re
        .captures_iter(&haystack)
        .map(|c| {
            let (_, [module]) = c.extract();
            module.to_owned()
        })
        .collect())
}

fn titlecase(modname: &str) -> String {
    let pattern = r"(?:^|_)([a-z])";
    let re = regex::Regex::new(pattern).expect("Failed to compile regex");
    re.replace_all(modname, |caps: &regex::Captures| {
        dbg!(caps);
        (&caps[1]).to_uppercase()
    })
    .into()
}

fn flag(modname: &str) -> String {
    "--".to_owned() + &str::replace(modname, "_", "-")
}

// I apologize that this is a readability nightmare. Having gen.rs readable
// directly and checked into source control should hopefully help.
fn write_genrs(mut f: impl Write) -> Result<()> {
    let ops = list_ops()?;

    write!(
        f,
        "{}",
        "//! This file is autogenerated. See build.rs for how!
use crate::op::helpers::{Config, FromArgs, Stack, Transform};
use crate::op::ops as x;
use std::io::Result;
use std::path::Path;

#[derive(Debug, PartialEq)]
pub enum OpCode {
"
    )?;
    for op in ops.iter() {
        write!(f, "    {},\n", titlecase(op))?
    }
    write!(
        f,
        "{}",
        "}

pub fn to_opcode(arg: impl AsRef<str>) -> Option<OpCode> {
    match arg.as_ref() {
"
    )?;

    for op in ops.iter() {
        write!(
            f,
            "        \"{}\" => Some(OpCode::{}),\n",
            flag(op),
            titlecase(op)
        )?;
    }

    write!(
        f,
        "{}",
        "        _ => None,
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Op {
"
    )?;
    for op in ops.iter() {
        write!(f, "    {}(x::{}::{}),\n", titlecase(op), op, titlecase(op))?;
    }
    write!(
        f,
        "{}",
        "}

impl Op {
    #[rustfmt::skip]
    pub fn from_code_and_params(oc: OpCode, params: Vec<String>) -> Result<Op> {
        Ok(match oc {
"
    )?;
    for op in ops.iter() {
        write!(
            f,
            "            OpCode::{} => Op::{}(x::{}::{}::from_args(params)?),\n",
            titlecase(op),
            titlecase(op),
            op,
            titlecase(op)
        )?;
    }
    write!(
        f,
        "{}",
        "        })
    }
}

impl Transform for &Op {
    fn transform<P>(self, cfg: &Config<P>, stack: Stack) -> Result<Stack>
    where
        P: AsRef<Path>,
    {
        match self {
"
    )?;
    for op in ops.iter() {
        write!(
            f,
            "            Op::{}(t) => t.transform(cfg, stack),\n",
            titlecase(op)
        )?;
    }
    write!(
        f,
        "{}",
        "        }
    }
}
"
    )?;

    Ok(())
}

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=src/op/ops.rs");
    write_genrs(File::create("src/op/gen.rs")?)
}
