#![allow(dead_code)]
use crate::column_filter::Operator;
use crate::pointdata::PointData;
use bitvector::*;
use macroquad::prelude::*;
//use std::cmp::Ordering::*;

const BAND: &str = "↔";

pub trait HighlightFilter {
    fn filter(&self, data: &PointData) -> BitVector;
    fn interface(&mut self, data: &PointData, ui: &mut egui::Ui, id: usize);
}

#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub enum HighlightFilterVariants {
    Selection(String, String),
    GreaterThan(String, f64),
    LessThan(String, f64),
    Band(String, f64, f64),
    Empty,
}

impl HighlightFilterVariants {
    fn delete_button(&mut self, ui: &mut egui::Ui) {
        if ui.button("🗙").clicked() {
            *self = HighlightFilterVariants::Empty;
        }
    }

    fn selection_ui(
        column: &str,
        value: &str,
        data: &PointData,
        ui: &mut egui::Ui,
        id: usize,
    ) -> (String, String) {
        ui.label("=");
        let mut highlight_column = column.to_string();
        let mut highlight_value = value.to_owned();
        egui::ComboBox::from_id_source(format!("Highlight column Selection {}", id))
            .selected_text(column.to_string())
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut highlight_column, "".to_string(), "");
                for column in data.headers.iter() {
                    ui.selectable_value(&mut highlight_column, column.to_string(), column);
                }
            });
        egui::ComboBox::from_id_source(format!("Highlight value Selection {}", id))
            .selected_text(value.to_string())
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut highlight_value, "".to_string(), "");
                for value in data.unique_values(&column).iter() {
                    ui.selectable_value(&mut highlight_value, value.to_string(), value);
                }
            });
        (highlight_column, highlight_value)
    }
    fn threshold_ui(
        less: bool,
        column: &str,
        value: f64,
        data: &PointData,
        ui: &mut egui::Ui,
        id: usize,
    ) -> (bool, String, f64) {
        let is_less = if less {
            !ui.button("<").clicked()
        } else {
            ui.button(">").clicked()
        };

        let mut highlight_column = column.to_string();
        let mut highlight_value = value;
        egui::ComboBox::from_id_source(format!("Highlight threshold {}", id))
            .selected_text(column.to_string())
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut highlight_column, "".to_string(), "");
                for column in data.headers.iter() {
                    ui.selectable_value(&mut highlight_column, column.to_string(), column);
                }
            });
        ui.add(egui::DragValue::new(&mut highlight_value).speed(0.1));
        (is_less, highlight_column, highlight_value)
    }

    fn band_ui(
        column: &str,
        value: f64,
        width: f64,
        data: &PointData,
        ui: &mut egui::Ui,
        id: usize,
    ) -> (String, f64, f64) {
        ui.label(BAND);

        let mut highlight_column = column.to_string();
        let mut highlight_value = value;
        let mut highlight_width = width;
        egui::ComboBox::from_id_source(format!("Highlight band {}", id))
            .selected_text(column.to_string())
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut highlight_column, "".to_string(), "");
                for column in data.headers.iter() {
                    ui.selectable_value(&mut highlight_column, column.to_string(), column);
                }
            });
        ui.add(egui::DragValue::new(&mut highlight_value).speed(0.1));
        ui.end_row();
        ui.label("");
        ui.label("Width:");
        ui.add(
            egui::DragValue::new(&mut highlight_width)
                .speed(0.1)
                .clamp_range(0.0..=f64::MAX),
        );

        (highlight_column, highlight_value, highlight_width)
    }
}

impl HighlightFilter for HighlightFilterVariants {
    fn filter(&self, data: &PointData) -> BitVector {
        let mut bv = BitVector::new(data.len());
        //        println!(" filter {:?} BV:{} DATA:{}",self, bv.capacity(), data.len());
        match self {
            HighlightFilterVariants::Selection(column, value) => {
                if data.data.contains_key(column) {
                    if let Ok(value) = value.parse::<f64>() {
                        for (i, x) in data.data[column].iter().enumerate() {
                            if *x == value {
                                bv.insert(i);
                            }
                        }
                    }
                } else {
                    if data.aux.contains_key(column) {
                        for (i, x) in data.aux[column].iter().enumerate() {
                            if x == value {
                                bv.insert(i);
                            }
                        }
                    }
                }
            }
            HighlightFilterVariants::LessThan(column, value) => {
                if data.data.contains_key(column) {
                    for (i, x) in data.data[column].iter().enumerate() {
                        if *x < *value {
                            bv.insert(i);
                        }
                    }
                } else {
                    if data.aux.contains_key(column) {
                        for (i, x) in data.aux[column].iter().enumerate() {
                            if let Ok(x) = x.parse::<f64>() {
                                if x < *value {
                                    bv.insert(i);
                                }
                            }
                        }
                    }
                }
            }
            HighlightFilterVariants::GreaterThan(column, value) => {
                if data.data.contains_key(column) {
                    for (i, x) in data.data[column].iter().enumerate() {
                        if *x > *value {
                            bv.insert(i);
                        }
                    }
                } else {
                    if data.aux.contains_key(column) {
                        for (i, x) in data.aux[column].iter().enumerate() {
                            if let Ok(x) = x.parse::<f64>() {
                                if x > *value {
                                    bv.insert(i);
                                }
                            }
                        }
                    }
                }
            }
            HighlightFilterVariants::Band(column, value, width) => {
                if data.data.contains_key(column) {
                    for (i, x) in data.data[column].iter().enumerate() {
                        if *x >= *value - 0.5 * width && *x <= *value + 0.5 * width {
                            bv.insert(i);
                        }
                    }
                } else {
                    if data.aux.contains_key(column) {
                        for (i, x) in data.aux[column].iter().enumerate() {
                            if let Ok(x) = x.parse::<f64>() {
                                if x >= *value - 0.5 * width && x <= *value + 0.5 * width {
                                    bv.insert(i);
                                }
                            }
                        }
                    }
                }
            }
            HighlightFilterVariants::Empty => {}
        }
        //       println!(" - return {:?} BV:{} DATA:{}",self,bv.capacity(),data.len());
        bv
    }

    fn interface(&mut self, data: &PointData, ui: &mut egui::Ui, id: usize) {
        match self {
            HighlightFilterVariants::Selection(column, value) => {
                let (new_column, new_value) = HighlightFilterVariants::selection_ui(column, value, data, ui, id);
                *self = HighlightFilterVariants::Selection(new_column, new_value);
                self.delete_button(ui);
            }
            HighlightFilterVariants::LessThan(column, value) => {
                let (is_less, new_column, new_value) = HighlightFilterVariants::threshold_ui(
                    true,
                    &column.to_string(),
                    *value,
                    data,
                    ui,
                    id,
                );
                *self = if is_less {
                    HighlightFilterVariants::LessThan(new_column, new_value)
                } else {
                    HighlightFilterVariants::GreaterThan(new_column, new_value)
                };
                self.delete_button(ui);
            }
            HighlightFilterVariants::GreaterThan(column, value) => {
                let (is_less, new_column, new_value) = HighlightFilterVariants::threshold_ui(
                    false,
                    &column.to_string(),
                    *value,
                    data,
                    ui,
                    id,
                );
                *self = if is_less {
                    HighlightFilterVariants::LessThan(new_column, new_value)
                } else {
                    HighlightFilterVariants::GreaterThan(new_column, new_value)
                };
                self.delete_button(ui);
            }
            HighlightFilterVariants::Band(column, value, width) => {
                let (new_column, new_value, new_width) = HighlightFilterVariants::band_ui(
                    &column.to_string(),
                    *value,
                    *width,
                    data,
                    ui,
                    id,
                );
                *self = HighlightFilterVariants::Band(new_column, new_value, new_width);
                self.delete_button(ui);
            }
            HighlightFilterVariants::Empty => {}
        }
    }
}

#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub struct CombinedHighlightFilter {
    pub operator: Operator,
    pub filters: Vec<HighlightFilterVariants>,
}

impl CombinedHighlightFilter {
    pub fn new() -> Self {
        CombinedHighlightFilter {
            operator: Operator::And,
            filters: Vec::new(),
        }
    }
    fn tidy(&mut self) {
        let mut i = 0;
        while i < self.filters.len() {
            if self.filters[i] == HighlightFilterVariants::Empty {
                self.filters.remove(i);
            } else {
                i += 1;
            }
        }
    }
}

impl HighlightFilter for CombinedHighlightFilter {
    fn filter(&self, data: &PointData) -> BitVector {
        match self.operator {
            Operator::And => {
                let mut bv = BitVector::new(data.len());
                for i in 0..data.len() {
                    bv.insert(i);
                }
                for f in self.filters.iter() {
                    bv.intersection_inplace(&f.filter(data));
                }
                bv
            }
            Operator::Or => {
                let mut bv = BitVector::new(data.len());
                for f in self.filters.iter() {
                    bv.union_inplace(&f.filter(data));
                }
                bv
            }
        }
    }
    fn interface(&mut self, data: &PointData, ui: &mut egui::Ui, id: usize) {
        for (i, f) in self.filters.iter_mut().enumerate() {
            f.interface(data, ui, 103 * id + i);
            ui.end_row();
        }
        self.tidy();
        ui.label("Add");
        ui.horizontal(|ui| {
            if ui.button("=").clicked() {
                self.filters.push(HighlightFilterVariants::Selection(
                    "".to_string(),
                    "".to_string(),
                ))
            }
            if ui.button("<").clicked() {
                self.filters
                    .push(HighlightFilterVariants::LessThan("".to_string(), 0.0))
            }
            if ui.button(">").clicked() {
                self.filters
                    .push(HighlightFilterVariants::GreaterThan("".to_string(), 0.0))
            }
            if ui.button(BAND).clicked() {
                self.filters
                    .push(HighlightFilterVariants::Band("".to_string(), 0.0, 0.0))
            }
        });
        ui.horizontal(|ui| {
            ui.label("Operator: ");
            match self.operator {
                Operator::And => {
                    if ui.button("AND").clicked() {
                        self.operator = Operator::Or;
                    }
                }
                Operator::Or => {
                    if ui.button("OR").clicked() {
                        self.operator = Operator::And;
                    }
                }
            }
        });
        ui.end_row();
    }
}
