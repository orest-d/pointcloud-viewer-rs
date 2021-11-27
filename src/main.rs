#![allow(dead_code)]
#[macro_use]
extern crate serde_derive;
use anyhow::Result;
use egui::containers::ScrollArea;
use macroquad::prelude::*;
use std::collections::HashMap;
mod column_filter;
mod highlight;
mod measures;
mod mesh;
mod pipeline;
mod pointdata;
mod transform;
use column_filter::*;
use highlight::*;
use mesh::HighlightType;
use pipeline::*;

fn window_conf() -> Conf {
    Conf {
        window_title: "Pointcloud Viewer".to_owned(),
        fullscreen: false,
        window_resizable: true,
        window_width: 1200,
        window_height: 820,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() -> Result<()> {
    let mut pipeline = Pipeline::new();
    let mut mouse_origin = None;
    pipeline.load("data.csv").await?;
    //    println!("{}", pipeline.point_data.to_csv_simple());
    let margin = 6.0f32;
    let size_x = pipeline.parameters.mesh_width as f32;
    let size_y = pipeline.parameters.mesh_height as f32;
    let mut statistics = None;
    let mut enable_data_display = true;
    let mut enable_statistics = false;
    let mut enable_column_selector = false;
    let mut column_selection = String::new();
    let mut enable_highlight = false;
    let mut highlight_filter = CombinedHighlightFilter::new();
    let mut stat_all = true;
    let mut stat_highlighted = true;
    let mut stat_non_highlighted = true;
    let measure_names = measures::NumericStatistics::new().all_measure_names();
    let mut enable_measure:HashMap<String, bool> = HashMap::new();
    let mut selected_x=-1.0f64;
    let mut selected_y=-1.0f64;

    for measure in measure_names.iter() {
        enable_measure.insert(measure.to_string(), measure=="Mean");
    }

    loop {
        //        clear_background(DARKBLUE);
        clear_background(Color::from_rgba(0x12, 0x12, 0x12, 0xff));
        egui_macroquad::ui(|egui_ctx| {
            egui::Window::new("Select columns")
                .open(&mut enable_column_selector)
                .default_pos((2.0 * margin + size_x, 320.0))
                .show(egui_ctx, |ui| {
                    ui.label("Columns");
                    ui.add(egui::TextEdit::singleline(&mut column_selection).desired_width(200.0));
                });
            if enable_column_selector {
                let filter = ColumnFilter::from_text(
                    &column_selection,
                    Interpretation::Contains,
                    Operator::Or,
                    false,
                );
                pipeline.filter_headers(&|x| filter.filter(x));
            } else {
                pipeline.point_data.reset_headers();
            }

            egui::Window::new("Control")
                .default_pos((2.0 * margin + size_x, margin))
                .show(egui_ctx, |ui| {
                    //ui.label("Test");

                    ui.horizontal(|ui| {
                        if ui
                            .add(egui::SelectableLabel::new(
                                enable_column_selector,
                                "Select columns",
                            ))
                            .clicked()
                        {
                            enable_column_selector = !enable_column_selector;
                        };
                        if ui
                            .add(egui::SelectableLabel::new(enable_highlight, "Highlight"))
                            .clicked()
                        {
                            enable_highlight = !enable_highlight;
                        };
                        if ui
                            .add(egui::SelectableLabel::new(
                                enable_data_display,
                                "Data display",
                            ))
                            .clicked()
                        {
                            enable_data_display = !enable_data_display;
                        };
                        if ui
                            .add(egui::SelectableLabel::new(enable_statistics, "Statistics"))
                            .clicked()
                        {
                            enable_statistics = !enable_statistics;
                            if enable_statistics && statistics.is_none() {
                                statistics = Some(pipeline.statistics(selected_x, selected_y));
                            }
                        };
                    });
                    ui.separator();
                    egui::Grid::new("Coordinates grid").show(ui, |ui| {
                        if ui.button("Zoom all").clicked() {
                            pipeline.zoom_all();
                        }
                        ui.end_row();
                        let mut zoom = pipeline.get_zoom();
                        ui.add(egui::Slider::new(&mut zoom, 0.5..=10.0));
                        pipeline.set_zoom(zoom);

                        let mut aspect_ratio = pipeline.get_aspect_ratio();
                        ui.add(egui::Slider::new(&mut aspect_ratio, 0.5..=2.0));
                        pipeline.set_aspect_ratio(aspect_ratio);
                        ui.end_row();
                        egui::ComboBox::from_label("X")
                            .selected_text(pipeline.xcolumn())
                            .show_ui(ui, |ui| {
                                let mut xcolumn = pipeline.xcolumn().to_owned();
                                for column in pipeline.point_data.headers.iter() {
                                    ui.selectable_value(&mut xcolumn, column.to_string(), column);
                                }
                                pipeline.set_xcolumn(xcolumn);
                            });
                        egui::ComboBox::from_label("X trans.")
                            .selected_text(pipeline.tx_type().text())
                            .show_ui(ui, |ui| {
                                let mut txtype = pipeline.tx_type();
                                ui.selectable_value(
                                    &mut txtype,
                                    TransformationType::Linear,
                                    "Linear",
                                );
                                ui.selectable_value(
                                    &mut txtype,
                                    TransformationType::Logarithmic,
                                    "Logarithmic",
                                );
                                ui.selectable_value(
                                    &mut txtype,
                                    TransformationType::Quantile,
                                    "Quantile",
                                );
                                pipeline.set_txtype(txtype);
                            });
                        ui.end_row();
                        egui::ComboBox::from_label("Y")
                            .selected_text(pipeline.ycolumn())
                            .show_ui(ui, |ui| {
                                let mut ycolumn = pipeline.ycolumn().to_owned();
                                for column in pipeline.point_data.headers.iter() {
                                    ui.selectable_value(&mut ycolumn, column.to_string(), column);
                                }
                                pipeline.set_ycolumn(ycolumn);
                            });
                        egui::ComboBox::from_label("Y trans.")
                            .selected_text(pipeline.ty_type().text())
                            .show_ui(ui, |ui| {
                                let mut tytype = pipeline.ty_type();
                                ui.selectable_value(
                                    &mut tytype,
                                    TransformationType::Linear,
                                    "Linear",
                                );
                                ui.selectable_value(
                                    &mut tytype,
                                    TransformationType::Logarithmic,
                                    "Logarithmic",
                                );
                                ui.selectable_value(
                                    &mut tytype,
                                    TransformationType::Quantile,
                                    "Quantile",
                                );
                                pipeline.set_tytype(tytype);
                            });
                        ui.end_row();
                        egui::ComboBox::from_label("Weight")
                            .selected_text(pipeline.weight_column())
                            .show_ui(ui, |ui| {
                                let mut weight_column = pipeline.weight_column().to_owned();
                                ui.selectable_value(&mut weight_column, "".to_string(), "");
                                for column in pipeline.point_data.headers.iter() {
                                    ui.selectable_value(
                                        &mut weight_column,
                                        column.to_string(),
                                        column,
                                    );
                                }
                                pipeline.set_weight_column(weight_column);
                            });
                        ui.end_row();
                        let mut gaussian_points = pipeline.gaussian_points();
                        ui.checkbox(&mut gaussian_points, "Gaussian points");
                        pipeline.set_gaussian_points(gaussian_points);

                        let mut point_sigma = pipeline.point_sigma();
                        ui.add(egui::Slider::new(&mut point_sigma, 0.0..=10.0));
                        pipeline.set_point_sigma(point_sigma);

                        ui.end_row();
                        ui.label("Brighthess:");
                        let mut density_multiplier = pipeline.density_multiplier();
                        ui.add(egui::Slider::new(&mut density_multiplier, -3.0..=3.0));
                        pipeline.set_density_multiplier(density_multiplier);
                        ui.end_row();

                        ui.label("Contrast:");
                        let mut contrast = pipeline.contrast();
                        ui.add(egui::Slider::new(&mut contrast, 0.1..=10.0));
                        pipeline.set_contrast(contrast);
                        ui.end_row();
                    });

                    //                    dbg!(&ui.input().pointer.hover_pos());
                });
            egui::Window::new("Data display")
                .open(&mut enable_data_display)
                .default_pos((2.0 * margin + size_x, 390.0))
                .show(egui_ctx, |ui| {
                    //                    ui.label(format!("{:?}", ui.input().pointer.hover_pos()));
                    if let Some(origin) = ui.input().pointer.press_origin() {
                        mouse_origin = Some(origin);
                    }

                    if let Some(pos) = ui.input().pointer.hover_pos() {
                        if let Some(index) = pipeline
                            .mesh
                            .get_index_wide((pos.x - margin) as usize, (pos.y - margin) as usize)
                        {
                            ui.label(format!("Index: {}", index));
                            egui::Grid::new("Data")
                                .striped(true)
                                .min_col_width(50.0)
                                .max_col_width(200.0)
                                .show(ui, |ui| {
                                    for column in pipeline.point_data.headers.iter() {
                                        ui.label(column);
                                        ui.label(pipeline.point_data.get(column, index));
                                        ui.end_row();
                                    }
                                });
                        }
                    }
                });

            egui::Window::new("Statistics")
                .open(&mut enable_statistics)
                .default_pos((2.0 * margin + size_x, 380.0))
                .show(egui_ctx, |ui| {
                    if ui.input().pointer.any_released() {
                        if let (Some(origin), Some(release)) =
                            (mouse_origin, ui.input().pointer.interact_pos())
                        {
                            //println!("Offset: {:?} {:?}", origin, release);
                            let x1 = (origin.x - margin) / size_x;
                            let y1 = (origin.y - margin) / size_y;
                            let x2 = (release.x - margin) / size_x;
                            let y2 = (release.y - margin) / size_y;
                            if x1 >= 0.0 && x1 <= 1.0 && y1 >= 0.0 && y1 <= 1.0 {
                                let dx = (x2 - x1) as f64;
                                let dy = (y2 - y1) as f64;
                                //println!("Shift {} {}",dx,dy);
                                pipeline.relative_offset(dx, dy);
                                selected_x = x2 as f64;
                                selected_y = y2 as f64;
                                statistics = Some(pipeline.statistics(selected_x, selected_y));
                            }
                        }
                    }

                    let mut stat = Vec::new();
                    if let Some(s) = &statistics {
                        stat = s.clone();
                    }
                    egui::Grid::new("Statistics config").show(ui, |ui|{
                        ui.checkbox(&mut stat_all, ALL);
                        ui.checkbox(&mut stat_highlighted, HIGHLIGHTED);
                        ui.checkbox(&mut stat_non_highlighted, NON_HIGHLIGHTED);
                        ui.end_row();
                        for (i,name) in measure_names.iter().enumerate() {
                            let flag = enable_measure.entry(name.to_string()).or_insert(true);
                            ui.checkbox(&mut *flag, name);
                            if i%4==3{
                                ui.end_row();
                            }
                            if !*flag{
                                stat = stat.iter().filter(|x| &x[1]!=name).map(|x| x.to_owned()).collect();
                            }
                        }
                    });
                    if !stat_all{
                        stat = stat.iter().filter(|x| x[0]!=ALL).map(|x| x.to_owned()).collect();
                    }
                    if !stat_highlighted{
                        stat = stat.iter().filter(|x| x[0]!=HIGHLIGHTED).map(|x| x.to_owned()).collect();
                    }
                    if !stat_non_highlighted{
                        stat = stat.iter().filter(|x| x[0]!=NON_HIGHLIGHTED).map(|x| x.to_owned()).collect();
                    }
                    

                    ScrollArea::both().show(ui, |ui| {
                        egui::Grid::new("Statistics")
                            .striped(true)
                            .min_col_width(50.0)
                            .max_col_width(200.0)
                            .show(ui, |ui| {
                                let tstat = stat.transpose();
                                for row in tstat.iter() {
                                    for item in row.iter() {
                                        ui.label(item);
                                    }
                                    ui.end_row();
                                }
                            });
                    });
                });
            egui::Window::new("Highlight Filter")
                .open(&mut enable_highlight)
                .default_pos((2.0 * margin + size_x, 320.0))
                .show(egui_ctx, |ui| {
                    egui::Grid::new("Highlight filter grid").show(ui, |ui| {
                        highlight_filter.interface(&pipeline.point_data, ui, 0);
                        pipeline.set_highlights(highlight_filter.filter(&pipeline.point_data));
                        ui.end_row();
                        ui.label("");
                        let mut highlight_type = pipeline.highlight_type();
                        ui.radio_value(&mut highlight_type, HighlightType::Highlight, "Highlight");
                        ui.radio_value(
                            &mut highlight_type,
                            HighlightType::NoHighlight,
                            "No highlight",
                        );
                        ui.end_row();
                        ui.label("");
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
                        statistics = Some(pipeline.statistics(selected_x, selected_y));  
                    });
                });
        });
        pipeline.run();
        if let Some(texture) = pipeline.texture {
            draw_texture(
                texture,
                margin,
                margin,
                Color::from_rgba(255, 255, 255, 255),
            );
        }
        egui_macroquad::draw();
        // Draw things after egui

        next_frame().await;
    }
}
