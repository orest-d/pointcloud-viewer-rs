#![allow(dead_code)]

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
    operator:Operator,
    case_sensitive:bool
}

impl ColumnFilter{
    pub fn new()->Self{
        ColumnFilter{
            tokens:Vec::new(),
            interpretation: Interpretation::Contains,
            operator:Operator::And,
            case_sensitive:true
        }
    }
    pub fn from_text(text: &str, interpretation: Interpretation, operator: Operator, case_sensitive:bool)->ColumnFilter{
        let tokens = if case_sensitive{
            text.split_whitespace().map(|x| x.to_string()).collect()
        } else{
            text.split_whitespace().map(|x| x.to_lowercase()).collect()
        };
        ColumnFilter{
            tokens:tokens,
            interpretation: interpretation,
            operator:operator,
            case_sensitive:case_sensitive
        }
    }
    pub fn filter(&self, column:&str)->bool{
        let column = if self.case_sensitive{
            column.to_owned()
        }
        else{
            column.to_lowercase()
        };

        let matching:fn(&str,&str)->bool = match self.interpretation{
            Interpretation::Contains => |column, token| column.contains(token),
            Interpretation::Prefix => |column, token| column.starts_with(token),
            Interpretation::Postfix => |column, token| column.ends_with(token)
        };

        match self.operator{
            Operator::And => self.tokens.iter().all(|x| matching(&column,x)),
            Operator::Or => self.tokens.iter().any(|x| matching(&column,x)),
        }
    }
}