use macroquad::prelude::*;
use std::collections::HashMap;
use std::convert::TryInto;
use anyhow::Result;
use csv;
use std::f64::consts::PI;

use std::fs::File;
use std::io::{Write, BufWriter};

struct PointData {
    length: usize,
    headers: Vec<String>,
    data: HashMap<String, Vec<f64>>,
    aux: HashMap<String, Vec<String>>,
}

impl PointData {
    fn new() -> PointData {
        PointData {
            length: 0,
            headers: Vec::new(),
            data: HashMap::new(),
            aux: HashMap::new()
        }
    }
    fn len(&self) -> usize { self.length }
    fn with_data_column(&mut self, column: &str) -> &mut Self{
        self.headers.push(column.to_owned());
        self.data.insert(column.to_owned(), Vec::new());
        self
    }
    fn with_aux_column(&mut self, column: &str) -> &mut Self{
        self.headers.push(column.to_owned());
        self.aux.insert(column.to_owned(), Vec::new());
        self
    }
    fn allocate(&mut self, n:usize) -> &mut Self{
        self.length = n;
        for (key, value) in self.data.iter_mut(){
            value.resize(n,0.0);
        }
        for (key, value) in self.aux.iter_mut(){
            value.resize(n,"".to_string());
        }
        self
    }
    fn set_data(&mut self, column:&str, index:usize, value:f64)->&mut Self{
        if (index>=self.length){
            self.allocate(index);
        }
        if let Some(v) = self.data.get_mut(column){
            v[index]=value;
        }
        else{
            let mut v = Vec::with_capacity(self.length);
            v.resize(self.length, 0.0);
            v[index]=value;
           self.data.insert(column.to_owned(),v);
        }
        self
    }
    fn set_aux(&mut self, column:&str, index:usize, value:String)->&mut Self{
        if (index>=self.length){
            self.allocate(index);
        }
        if let Some(v) = self.aux.get_mut(column){
            v[index]=value;
        }
        else{
            let mut v:Vec<String> = Vec::with_capacity(self.length);
            v.resize(self.length, "".to_string());
            v[index]=value;
           self.aux.insert(column.to_owned(),v);
        }
        self
    }
    fn row(&self, index:usize)->Vec<String>{
        let mut v=Vec::with_capacity(self.headers.len());
        if (index<self.length){
            for column in self.headers.iter(){
                if let Some(column_data)=self.data.get(column){
                  v.push(format!("{}",column_data[index]));
                }
                else{
                    if let Some(aux_data)=self.aux.get(column){
                        v.push(format!("\"{}\"",aux_data[index]));
                    }
                }
            }
        }
        v
    }
    fn to_csv_simple(&self)->String{
        let sep=",";
        format!("{}\n{}",
        self.headers.join(sep),
        (0..self.length).map(|i| self.row(i).join(sep)).collect::<Vec<String>>().join("\n")
       )
    }
}
fn test_point_data() -> Result<PointData> {
    let mut point_data = PointData {
        length: 5,
        headers: vec!["x", "y", "label"]
            .into_iter()
            .map(|x| x.to_string())
            .collect(),
        data: HashMap::new(),
        aux: HashMap::new(),
    };
    point_data
        .data
        .insert("x".into(), vec![0.0, 0.0, 1.0, 1.0, 0.5]);
    point_data
        .data
        .insert("y".into(), vec![0.0, 1.0, 0.0, 1.0, 0.5]);
    point_data.aux.insert(
        "label".into(),
        vec!["A", "B", "C", "D", "E"]
            .into_iter()
            .map(|x| x.to_string())
            .collect(),
    );
    Ok(point_data)
}

fn test_point_data_circle(n:usize) -> Result<PointData> {
    let mut point_data = PointData::new();
    point_data
    .with_data_column("a")
    .with_data_column("x")
    .with_data_column("y")
    .with_aux_column("label")
    .allocate(n);

    for i in 0..n{
        let a = 2.0*PI*(i as f64)/(n as f64);
        let x = a.sin();
        let y = a.cos();
        point_data
        .set_data("a", i, a)
        .set_data("x", i, x)
        .set_data("y", i, y)
        .set_aux("label", i, format!("{}/{}",i+1,n));
    }
    Ok(point_data)
}

#[derive(Debug, Clone)]
struct Parameters {
    xcolumn: String,
    ycolumn: String,
    mesh_width: usize,
    mesh_height: usize,
    xmin: f64,
    xmax: f64,
    ymin: f64,
    ymax: f64,
}

impl Parameters {
    fn new() -> Parameters {
        Parameters {
            xcolumn: "".into(),
            ycolumn: "".into(),
            mesh_width: 1000,
            mesh_height: 1000,
            xmin: 0.0,
            xmax: 1.0,
            ymin: 0.0,
            ymax: 1.0,
        }
    }

    fn new_mesh(&self) -> Mesh {
        let mut mesh = Mesh::new();
        self.adapt_mesh(&mut mesh);
        mesh
    }

    fn adapt_mesh(&self, mesh: &mut Mesh) {
        mesh.resize(self.mesh_width, self.mesh_height);
        mesh.xmin = self.xmin;
        mesh.xmax = self.xmax;
        mesh.ymin = self.ymin;
        mesh.ymax = self.ymax;
    }
    fn zoom_all_x(&mut self, v:&Vec<f64>){
        if v.len()>0{
            self.xmin = v[0];
            self.xmax = v[0];
            for value in v.iter(){
                self.xmin = self.xmin.min(*value);
                self.xmax = self.xmax.max(*value);
            }
        }
    }
    fn zoom_all_y(&mut self, v:&Vec<f64>){
        if v.len()>0{
            self.ymin = v[0];
            self.ymax = v[0];
            for value in v.iter(){
                self.ymin = self.ymin.min(*value);
                self.ymax = self.ymax.max(*value);
            }
        }
    }
    fn zoom_all(&mut self, vx:&Vec<f64>, vy:&Vec<f64>){
        self.zoom_all_x(vx);
        self.zoom_all_y(vy);
    }
}

struct Mesh {
    width: usize,
    height: usize,
    xmin: f64,
    xmax: f64,
    ymin: f64,
    ymax: f64,
    mesh: Vec<f64>,
    rgba8: Vec<u8>,
}

impl Mesh {
    fn new() -> Mesh {
        Mesh {
            width: 0,
            height: 0,
            xmin: 0.0,
            xmax: 1.0,
            ymin: 0.0,
            ymax: 1.0,
            mesh: Vec::new(),
            rgba8: Vec::new(),
        }
    }


    fn resize(&mut self, width: usize, height: usize) -> &mut Self {
        let size = width * height;
        self.mesh.resize(size, 0.0);
        self.rgba8.resize(4*size, 0);
        self.width = width;
        self.height = height;
        self.clean()
    }

    fn clean(&mut self) -> &mut Self {
        for i in self.mesh.iter_mut() {
            *i = 0.0;
        }
        self
    }

    fn point(&mut self, x: f64, y: f64, weight:f64){
        let fx = (x-self.xmin)/(self.xmax-self.xmin);
        let fy = (y-self.ymin)/(self.ymax-self.ymin);
        if fx>=0.0 && fy>=0.0{
            let ix = (fx*(self.width as f64)) as usize;
            let iy = (fy*(self.height as f64)) as usize;
            if ix<self.width && iy<self.height{
//                println!("  -> mesh {} {}",ix,iy);
                self.mesh[ix+iy*self.width]+=weight;
            }
        }

    }

    fn to_rgba8_gray(&mut self){
        for (i,m) in self.mesh.iter().enumerate() {
            let value:u8 = if *m<0.0 {0} else {if *m>=1.0 {255} else {(255.0*m) as u8} };
            self.rgba8[4*i]= value;
            self.rgba8[4*i+1]= value;
            self.rgba8[4*i+2]= value;
            self.rgba8[4*i+3]= 255;
        }
    }

    fn plain_points(&mut self, vx:&Vec<f64>, vy:&Vec<f64>){
        for (i,(&x,&y)) in vx.iter().zip(vy.iter()).enumerate() {
//            println!("{}: {} {}",i,x,y);
            self.point(x, y, 1.0);
        }
    }
    fn test_pattern(&mut self){
        for y in 0..self.height {
            for x in 0..self.width {
                let i=x+self.width*y; 
                self.rgba8[4*i]= (x%256) as u8;
                self.rgba8[4*i+1]= (y%256) as u8;
                self.rgba8[4*i+2]= 0;
                self.rgba8[4*i+3]= 255;
            }
        }
    }
}

#[macroquad::main("cloudviewer")]
async fn main() {
//    let mut bytes: Vec<u8> = Vec::new();
    let point_data = test_point_data_circle(1000).unwrap();

    let mut file = std::fs::File::create("test_point_data.csv").unwrap();
    let mut writer = BufWriter::new(&file);

    write!(writer, "{}", point_data.to_csv_simple());

    let mut parameters = Parameters::new();
    let mut mesh = parameters.new_mesh();
    let dummy_vxy = (&Vec::new(),&Vec::new());
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
        parameters.adapt_mesh(& mut mesh);
        clear_background(WHITE);
        let vxy = 
        if point_data.data.contains_key(&parameters.xcolumn) & point_data.data.contains_key(&parameters.ycolumn){
            (&point_data.data[&parameters.xcolumn], &point_data.data[&parameters.ycolumn])
        }
        else{
            dummy_vxy
        };

        if point_data.data.contains_key(&parameters.xcolumn) & point_data.data.contains_key(&parameters.ycolumn){
            mesh.plain_points(vxy.0,vxy.1);
        }
        mesh.to_rgba8_gray();
//        mesh.test_pattern();
        let texture = Texture2D::from_rgba8(mesh.width.try_into().unwrap(), mesh.height.try_into().unwrap(), &mesh.rgba8);
        // Process keys, mouse etc.

        egui_macroquad::ui(|egui_ctx| {
            egui::Window::new("Cloud viewer").show(egui_ctx, |ui| {
                ui.label("Test");
                if ui.button("Zoom all").clicked(){
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
