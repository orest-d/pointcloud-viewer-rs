#![allow(dead_code)]
use anyhow::*;

#[derive(Debug, Clone, PartialOrd, PartialEq, Copy)]
pub enum Interpretation{
    Contains,
    Prefix,
    Postfix
}

#[derive(Debug, Clone, PartialOrd, PartialEq, Copy)]
pub enum Operator{
    And, Or
}

#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub struct ColumnFilter{
    text:String,
    interpretation: Interpretation,
    operator:Operator
}

impl ColumnFilter{
    pub fn new()->Self{
        ColumnFilter{
            text:"".to_owned(),
            interpretation: Interpretation::Contains,
            operator:Operator::And,
        }
    }
}