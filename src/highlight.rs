#![allow(dead_code)]
use crate::column_filter::Operator;
use crate::pointdata::PointData;
//use std::cmp::Ordering::*;

trait HighlightFilter{
    fn filter(&self, data: &PointData) -> Vec<bool>;
}

#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub enum HighlightFilterVariants{
    Selection(String, String),
    GreaterThan(String, f64),
    LessThan(String, f64),
    Band(String, f64, f64),
    Empty
}

impl HighlightFilter for HighlightFilterVariants{
    fn filter(&self, data: &PointData) -> Vec<bool>{
        match self{
            HighlightFilterVariants::Selection(column, value) =>{
                if data.data.contains_key(column){
                    if let Ok(value) = value.parse::<f64>(){
                        data.data[column].iter().map(|x| *x==value).collect()
                    }
                    else{
                        Vec::new()
                    }
                }
                else{
                    if data.aux.contains_key(column){
                        data.aux[column].iter().map(|x| x==value).collect()
                    }
                    else{
                        Vec::new()
                    }
                }
            },
            HighlightFilterVariants::LessThan(column, value) =>{
                if data.data.contains_key(column){
                    data.data[column].iter().map(|x| *x<*value).collect()
                }
                else{
                    if data.aux.contains_key(column){
                        data.aux[column].iter().map(|x| {
                            if let Ok(x)=x.parse::<f64>(){
                                x<*value
                            }
                            else{
                                false
                            }
                        }).collect()
                    }
                    else{
                        Vec::new()
                    }
                }
            },
            HighlightFilterVariants::GreaterThan(column, value) =>{
                if data.data.contains_key(column){
                    data.data[column].iter().map(|x| *x>*value).collect()
                }
                else{
                    if data.aux.contains_key(column){
                        data.aux[column].iter().map(|x| {
                            if let Ok(x)=x.parse::<f64>(){
                                x>*value
                            }
                            else{
                                false
                            }
                        }).collect()
                    }
                    else{
                        Vec::new()
                    }
                }
            }
            HighlightFilterVariants::Band(column, value, width) =>{
                if data.data.contains_key(column){
                    data.data[column].iter().map(|x| {(*x>=(*value-0.5*width)) && (*x<=(*value+0.5*width))}).collect()
                }
                else{
                    if data.aux.contains_key(column){
                        if data.aux.contains_key(column){
                            data.aux[column].iter().map(|x| {
                                if let Ok(x)=x.parse::<f64>(){
                                    (x>=(*value-0.5*width)) && (x<=(*value+0.5*width))
                                }
                                else{
                                    false
                                }
                            }).collect()
                        }
                        else{
                            Vec::new()
                        }
                    }
                    else{
                        Vec::new()
                    }
                }
            },
            HighlightFilterVariants::Empty =>{
                Vec::new()
            }
        }
    }
}    




#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub struct CombinedHighlightFilter{
    pub operator:Operator,
    pub filters:Vec<HighlightFilterVariants>
}

impl CombinedHighlightFilter{
    pub fn new()->Self{
        CombinedHighlightFilter{
            operator:Operator::And,
            filters:Vec::new()
        }
    }
}

impl HighlightFilter for CombinedHighlightFilter{
    fn filter(&self, data: &PointData) -> Vec<bool>{
        match self.operator{
            Operator::And => {
                if self.filters.is_empty(){
                    Vec::new()
                }
                else{
                    let mut acc = self.filters[0].filter(data);
                    for f in self.filters.iter().skip(1){
                        acc = acc.iter().zip(f.filter(data).iter()).map(|(a,b)| *a && *b).collect::<Vec<bool>>();
                    }
                    acc
                }
            },
            Operator::Or=>{
                if self.filters.is_empty(){
                    Vec::new()
                }
                else{
                    let mut acc = self.filters[0].filter(data);
                    for f in self.filters.iter().skip(1){
                        acc = acc.iter().zip(f.filter(data).iter()).map(|(a,b)| *a || *b).collect::<Vec<bool>>();
                    }
                    acc
                }
            }
        }
    }
}