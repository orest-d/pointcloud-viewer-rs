#![allow(dead_code)]
use crate::column_filter::Operator;
use crate::pointdata::PointData;
use bitvector::*;
use macroquad::prelude::*;
//use std::cmp::Ordering::*;

pub trait HighlightFilter {
    fn filter(&self, data: &PointData) -> BitVector;
    fn interface(&mut self, data: &PointData, ui: &mut egui::Ui);
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
        if ui.button("X").clicked() {
            *self = HighlightFilterVariants::Empty;
        }
    }

    fn threshold_ui(
        less: bool,
        column: &str,
        value: f64,
        data: &PointData,
        ui: &mut egui::Ui,
    ) -> (bool, String, f64) {
        let is_less = if less {
            !ui.button("<").clicked()
        } else {
            ui.button(">").clicked()
        };

        let mut highlight_column = column.to_string();
        let mut highlight_value = value;
        egui::ComboBox::from_id_source(format!("Highlight column Selection({},{})", column, value))
            .selected_text(column.to_string())
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut highlight_column, "".to_string(), "");
                for column in data.headers.iter() {
                    ui.selectable_value(&mut highlight_column, column.to_string(), column);
                }
            });
        ui.add(egui::Slider::new(&mut highlight_value, -1.0..=1.0));
        (is_less, highlight_column, highlight_value)
    }
}

impl HighlightFilter for HighlightFilterVariants {
    fn filter(&self, data: &PointData) -> BitVector {
        let mut bv = BitVector::new(data.len());
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
        bv
    }

    fn interface(&mut self, data: &PointData, ui: &mut egui::Ui) {
        match self {
            HighlightFilterVariants::Selection(column, value) => {
                ui.label("Value selection");
                ui.end_row();
                let mut highlight_column = column.to_string();
                let mut highlight_value = value.to_owned();
                egui::ComboBox::from_id_source(format!(
                    "Highlight column Selection({},{})",
                    column, value
                ))
                .selected_text(column.to_string())
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut highlight_column, "".to_string(), "");
                    for column in data.headers.iter() {
                        ui.selectable_value(&mut highlight_column, column.to_string(), column);
                    }
                });
                egui::ComboBox::from_id_source(format!(
                    "Highlight value Selection({},{})",
                    column, value
                ))
                .selected_text(value.to_string())
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut highlight_value, "".to_string(), "");
                    for value in data.unique_values(&column).iter() {
                        ui.selectable_value(&mut highlight_value, value.to_string(), value);
                    }
                });
                self.delete_button(ui);
            }
            HighlightFilterVariants::LessThan(column, value) => {
                let (is_less, new_column, new_value) = HighlightFilterVariants::threshold_ui(
                    true,
                    &column.to_string(),
                    *value,
                    data,
                    ui,
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
                );
                *self = if is_less {
                    HighlightFilterVariants::LessThan(new_column, new_value)
                } else {
                    HighlightFilterVariants::GreaterThan(new_column, new_value)
                };
                self.delete_button(ui);
            }
            HighlightFilterVariants::Band(column, value, width) => {}
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
                let mut bv = BitVector::ones(data.len());
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
    fn interface(&mut self, data: &PointData, ui: &mut egui::Ui) {
        for f in self.filters.iter_mut() {
            f.interface(data, ui);
            ui.end_row();
        }
        self.tidy();
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
        if ui.button("Band").clicked() {
            self.filters
                .push(HighlightFilterVariants::Band("".to_string(), 0.0, 0.0))
        }
        ui.end_row();
    }
}

/*

if enable_highlight {
    egui::Window::new("Highlight Settings")
        .default_pos((2.0 * margin + size_x, 320.0))
        .show(egui_ctx, |ui| {
            egui::Grid::new("Coordinates grid").show(ui, |ui| {
                for (i, f) in highlight_filter.filters.iter_mut().enumerate() {
                    match f {
                        HighlightFilterVariants::Selection(column, value) => {
                            ui.label("Value selection");
                            ui.end_row();
                            let mut highlight_column = column.to_string();
                            let mut highlight_value = value.to_owned();
                            egui::ComboBox::from_id_source(format!("Highlight {}", i))
                                .selected_text(column.to_string())
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut highlight_column,
                                        "".to_string(),
                                        "",
                                    );
                                    for column in pipeline.point_data.headers.iter() {
                                        ui.selectable_value(
                                            &mut highlight_column,
                                            column.to_string(),
                                            column,
                                        );
                                    }
                                });
                            egui::ComboBox::from_id_source(format!(
                                "Highlight value {}",
                                i
                            ))
                            .selected_text(value.to_string())
                            .show_ui(
                                ui,
                                |ui| {
                                    ui.selectable_value(
                                        &mut highlight_value,
                                        "".to_string(),
                                        "",
                                    );
                                    for value in pipeline.highlightable_values.iter() {
                                        ui.selectable_value(
                                            &mut highlight_value,
                                            value.to_string(),
                                            value,
                                        );
                                    }
                                },
                            );
                            *f = HighlightFilterVariants::Selection(
                                highlight_column,
                                highlight_value,
                            );
                            if ui.button("X").clicked(){
                                *f = HighlightFilterVariants::Empty;
                            }
                            ui.end_row();
                            ui.separator();
                            ui.end_row();
                        }
                        _ => {}
                    }
                }
                if ui.button("Selected value").clicked() {
                    highlight_filter.filters.push(HighlightFilterVariants::Selection("".to_string(),"".to_string()))
                }
                if ui.button("Less than").clicked() {
                    highlight_filter.filters.push(HighlightFilterVariants::LessThan("".to_string(),0.0))
                }
                if ui.button("Greater than").clicked() {
                    highlight_filter.filters.push(HighlightFilterVariants::GreaterThan("".to_string(),0.0))
                }
                if ui.button("Band").clicked() {
                    highlight_filter.filters.push(HighlightFilterVariants::Band("".to_string(),0.0,0.0))
                }
                ui.end_row();
                egui::ComboBox::from_label("Highlight")
                    .selected_text(pipeline.highlight_column())
                    .show_ui(ui, |ui| {
                        let mut highlight_column =
                            pipeline.highlight_column().to_owned();
                        ui.selectable_value(&mut highlight_column, "".to_string(), "");
                        for column in pipeline.point_data.headers.iter() {
                            ui.selectable_value(
                                &mut highlight_column,
                                column.to_string(),
                                column,
                            );
                        }
                        pipeline.set_highlight_column(highlight_column);
                    });
                egui::ComboBox::from_label("Value")
                    .selected_text(pipeline.highlight_value())
                    .show_ui(ui, |ui| {
                        let mut highlight_value = pipeline.highlight_value().to_owned();
                        ui.selectable_value(&mut highlight_value, "".to_string(), "");
                        for value in pipeline.highlightable_values.iter() {
                            ui.selectable_value(
                                &mut highlight_value,
                                value.to_string(),
                                value,
                            );
                        }
                        pipeline.set_highlight_value(highlight_value);
                    });
                ui.end_row();
                let mut highlight_type = pipeline.highlight_type();
                ui.radio_value(
                    &mut highlight_type,
                    HighlightType::Highlight,
                    "Highlight",
                );
                ui.radio_value(
                    &mut highlight_type,
                    HighlightType::NoHighlight,
                    "No highlight",
                );
                ui.end_row();
                ui.radio_value(
                    &mut highlight_type,
                    HighlightType::HighlighedOnly,
                    "Highlighted only",
                );
                ui.radio_value(
                    &mut highlight_type,
                    HighlightType::NonHighlightedOnly,
                    "Non-highlighted only",
                );
                pipeline.set_highlight_type(highlight_type);
            });
        });
}
*/
