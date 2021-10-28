#![allow(dead_code)]
#[macro_use]
extern crate serde_derive;
use anyhow::Result;
use macroquad::prelude::*;
use std::convert::TryInto;

//use std::io::{Write, BufWriter};

mod measures;
mod mesh;
mod pipeline;
mod pointdata;

use mesh::*;
use pipeline::*;
use pointdata::*;

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
    println!("{}", pipeline.point_data.to_csv_simple());
    loop {
        clear_background(DARKBLUE);
        egui_macroquad::ui(|egui_ctx| {
            egui::Window::new("Cloud viewer")
                .default_pos((900.0, 10.0))
                .show(egui_ctx, |ui| {
                    //ui.label("Test");
                    if ui.button("Zoom all").clicked() {
                        pipeline.zoom_all();
                    }
                    egui::ComboBox::from_label("X-column")
                        .selected_text(pipeline.xcolumn())
                        .show_ui(ui, |ui| {
                            let mut xcolumn = pipeline.xcolumn().to_owned();
                            for column in pipeline.data_columns.iter() {
                                ui.selectable_value(&mut xcolumn, column.to_string(), column);
                            }
                            pipeline.set_xcolumn(xcolumn);
                        });
                    egui::ComboBox::from_label("Y-column")
                        .selected_text(pipeline.ycolumn())
                        .show_ui(ui, |ui| {
                            let mut ycolumn = pipeline.ycolumn().to_owned();
                            for column in pipeline.data_columns.iter() {
                                ui.selectable_value(&mut ycolumn, column.to_string(), column);
                            }
                            pipeline.set_ycolumn(ycolumn);
                        });
                    let mut antialiased = pipeline.antialiased();
                    ui.checkbox(&mut antialiased, "Antialiased");
                    pipeline.set_antialiased(antialiased);
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

async fn main_old() -> Result<()> {
    //    let mut bytes: Vec<u8> = Vec::new();
    //    let point_data = test_point_data_circle(1000).unwrap();
    let csv_content = load_file("data.csv").await?;
    let point_data = PointData::from_csv(&mut csv_content.as_slice())?;
    println!("{}", point_data.to_csv_simple());

    /*
        {
            let file = std::fs::File::create("test_point_data.csv").unwrap();
            let mut writer = BufWriter::new(&file);
            write!(writer, "{}", point_data.to_csv_simple()).unwrap();
            writer.flush().unwrap();
        }
    */
    let mut parameters = Parameters::new();
    let mut mesh = parameters.new_mesh();
    let dummy_vxy = (&Vec::new(), &Vec::new());
    /*
    for j in 0..700 {
        for i in 0..1000 {
            bytes.push((i % 256) as u8);
            bytes.push((j % 256) as u8);
            bytes.push(0);
            bytes.push(255);
        }
    }
    */
    //    let mut texture = Texture2D::from_rgba8(1000, 700, &bytes);

    /*
        let png = load_file("cloud.png").await.unwrap();
        let image = Image::from_file_with_format(&png, Some(ImageFormat::Png));

        let texture = Texture2D::from_image(&image);
    */

    loop {
        parameters.adapt_mesh(&mut mesh);
        clear_background(WHITE);
        let vxy = if point_data.data.contains_key(&parameters.xcolumn)
            & point_data.data.contains_key(&parameters.ycolumn)
        {
            (
                &point_data.data[&parameters.xcolumn],
                &point_data.data[&parameters.ycolumn],
            )
        } else {
            dummy_vxy
        };

        if point_data.data.contains_key(&parameters.xcolumn)
            & point_data.data.contains_key(&parameters.ycolumn)
        {
            mesh.plain_points(vxy.0, vxy.1, false);
        }
        mesh.to_processed_mesh();
        mesh.to_rgba8_gray();
        //        mesh.test_pattern();
        let texture = Texture2D::from_rgba8(
            mesh.width.try_into().unwrap(),
            mesh.height.try_into().unwrap(),
            &mesh.rgba8,
        );
        // Process keys, mouse etc.

        egui_macroquad::ui(|egui_ctx| {
            egui::Window::new("Cloud viewer").show(egui_ctx, |ui| {
                ui.label("Test");
                if ui.button("Zoom all").clicked() {
                    parameters.zoom_all(vxy.0, vxy.1);
                }
                egui::ComboBox::from_label("X-column")
                    .selected_text(parameters.xcolumn.to_string())
                    .show_ui(ui, |ui| {
                        for column in point_data.headers.iter() {
                            if point_data.data.contains_key(column) {
                                ui.selectable_value(
                                    &mut parameters.xcolumn,
                                    column.to_string(),
                                    column,
                                );
                            }
                        }
                    });
                egui::ComboBox::from_label("Y-column")
                    .selected_text(parameters.ycolumn.to_string())
                    .show_ui(ui, |ui| {
                        for column in point_data.headers.iter() {
                            if point_data.data.contains_key(column) {
                                ui.selectable_value(
                                    &mut parameters.ycolumn,
                                    column.to_string(),
                                    column,
                                );
                            }
                        }
                    });
            });
        });
        //        println!("Parameters: {:?}",parameters);

        // Draw things before egui

        /*
        for i in 0..=255{
            draw_texture(texture,i as f32,i as f32,Color::from_rgba(i,i,i,i));
        }
        */
        draw_texture(texture, 0.0, 0.0, Color::from_rgba(255, 255, 255, 255));
        egui_macroquad::draw();
        // Draw things after egui

        next_frame().await;
    }
}
