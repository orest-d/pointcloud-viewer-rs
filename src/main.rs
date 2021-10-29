#![allow(dead_code)]
#[macro_use]
extern crate serde_derive;
use anyhow::Result;
use macroquad::prelude::*;

mod measures;
mod mesh;
mod pipeline;
mod pointdata;

use pipeline::*;

fn window_conf() -> Conf {
    Conf {
        window_title: "pointcloud viewer".to_owned(),
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
    pipeline.load("data.csv").await?;
//    println!("{}", pipeline.point_data.to_csv_simple());
    loop {
        clear_background(DARKBLUE);
        egui_macroquad::ui(|egui_ctx| {
            egui::Window::new("View setup")
                .default_pos((900.0, 10.0))
                .show(egui_ctx, |ui| {
                    //ui.label("Test");
                    if ui.button("Zoom all").clicked() {
                        pipeline.zoom_all();
                    }
                    egui::ComboBox::from_label("X column")
                        .selected_text(pipeline.xcolumn())
                        .show_ui(ui, |ui| {
                            let mut xcolumn = pipeline.xcolumn().to_owned();
                            for column in pipeline.data_columns.iter() {
                                ui.selectable_value(&mut xcolumn, column.to_string(), column);
                            }
                            pipeline.set_xcolumn(xcolumn);
                        });
                    egui::ComboBox::from_label("Y column")
                        .selected_text(pipeline.ycolumn())
                        .show_ui(ui, |ui| {
                            let mut ycolumn = pipeline.ycolumn().to_owned();
                            for column in pipeline.data_columns.iter() {
                                ui.selectable_value(&mut ycolumn, column.to_string(), column);
                            }
                            pipeline.set_ycolumn(ycolumn);
                        });
                    egui::ComboBox::from_label("Weight column")
                        .selected_text(pipeline.weight_column())
                        .show_ui(ui, |ui| {
                            let mut weight_column = pipeline.weight_column().to_owned();
                            ui.selectable_value(&mut weight_column, "".to_string(), "");
                            for column in pipeline.data_columns.iter() {
                                ui.selectable_value(&mut weight_column, column.to_string(), column);
                            }
                            pipeline.set_weight_column(weight_column);
                        });
                    egui::ComboBox::from_label("Highlight column")
                        .selected_text(pipeline.highlight_column())
                        .show_ui(ui, |ui| {
                            let mut highlight_column = pipeline.highlight_column().to_owned();
                            ui.selectable_value(&mut highlight_column, "".to_string(), "");
                            for column in pipeline.aux_columns.iter() {
                                ui.selectable_value(&mut highlight_column, column.to_string(), column);
                            }
                            pipeline.set_highlight_column(highlight_column);
                        });
                    egui::ComboBox::from_label("Highlight value")
                        .selected_text(pipeline.highlight_value())
                        .show_ui(ui, |ui| {
                            let mut highlight_value = pipeline.highlight_value().to_owned();
                            ui.selectable_value(&mut highlight_value, "".to_string(), "");
                            for value in pipeline.highlightable_values.iter() {
                                ui.selectable_value(&mut highlight_value, value.to_string(), value);
                            }
                            pipeline.set_highlight_value(highlight_value);
                        });

                    let mut gaussian_points = pipeline.gaussian_points();
                    ui.checkbox(&mut gaussian_points, "Gaussian points");
                    pipeline.set_gaussian_points(gaussian_points);

                    let mut point_sigma = pipeline.point_sigma();
                    ui.add(egui::Slider::new(&mut point_sigma, 0.0..=5.0));
                    pipeline.set_point_sigma(point_sigma);

                    ui.label("Brighthess:");
                    let mut density_multiplier = pipeline.density_multiplier();
                    ui.add(egui::Slider::new(&mut density_multiplier, 0.0..=5.0));
                    pipeline.set_density_multiplier(density_multiplier);
                });
        });
        pipeline.run();
        if let Some(texture) = pipeline.texture {
            draw_texture(texture, 10.0, 10.0, Color::from_rgba(255, 255, 255, 255));
        }
        egui_macroquad::draw();
        // Draw things after egui

        next_frame().await;
    }
}
