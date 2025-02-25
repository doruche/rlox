#![allow(unused)]

use crate::common::Value;

#[derive(Debug)]
pub enum FlowSignal {
    Normal,
    Break,
    Continue,
    Return(Value),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FlowContext {
    Loop,
    Function,
    Block,
}