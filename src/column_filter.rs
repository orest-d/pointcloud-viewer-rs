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
    tokens: Vec<String>,
    interpretation: Interpretation,
    operator:Operator
}

impl ColumnFilter{
    pub fn new()->Self{
        ColumnFilter{
            tokens:Vec::new(),
            interpretation: Interpretation::Contains,
            operator:Operator::And,
        }
    }
    pub fn from_text(text: &str, interpretation: Interpretation, operator: Operator)->ColumnFilter{
        ColumnFilter{
            tokens: text.split_whitespace().map(|x| x.to_string()).collect(),
            interpretation: interpretation,
            operator:operator
        }
    }
}