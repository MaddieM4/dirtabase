//! This file is autogenerated. See build.rs for how!
use crate::op::helpers::{Config, FromArgs, Stack, Transform};
use std::io::Result;
use std::path::Path;

#[derive(Debug, PartialEq)]
pub enum OpCode {
    Empty,
    Import,
    Export,
    Merge,
    Filter,
}

pub fn to_opcode(arg: impl AsRef<str>) -> Option<OpCode> {
    match arg.as_ref() {
        "--empty" => Some(OpCode::Empty),
        "--import" => Some(OpCode::Import),
        "--export" => Some(OpCode::Export),
        "--merge" => Some(OpCode::Merge),
        "--filter" => Some(OpCode::Filter),
        _ => None,
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Op {
    Empty(crate::op::ops::empty::Empty),
    Import(crate::op::ops::import::Import),
    Export(crate::op::ops::export::Export),
    Merge(crate::op::ops::merge::Merge),
    Filter(crate::op::ops::filter::Filter),
}

impl Op {
    pub fn from_code_and_params(oc: OpCode, params: Vec<String>) -> Result<Op> {
        Ok(match oc {
            OpCode::Empty => Op::Empty(crate::op::ops::empty::Empty::from_args(params)?),
            OpCode::Import => Op::Import(crate::op::ops::import::Import::from_args(params)?),
            OpCode::Export => Op::Export(crate::op::ops::export::Export::from_args(params)?),
            OpCode::Merge => Op::Merge(crate::op::ops::merge::Merge::from_args(params)?),
            OpCode::Filter => Op::Filter(crate::op::ops::filter::Filter::from_args(params)?),
        })
    }
}

impl Transform for &Op {
    fn transform<P>(self, cfg: &Config<P>, stack: Stack) -> Result<Stack>
    where
        P: AsRef<Path>,
    {
        match self {
            Op::Empty(t) => t.transform(cfg, stack),
            Op::Import(t) => t.transform(cfg, stack),
            Op::Export(t) => t.transform(cfg, stack),
            Op::Merge(t) => t.transform(cfg, stack),
            Op::Filter(t) => t.transform(cfg, stack),
        }
    }
}
