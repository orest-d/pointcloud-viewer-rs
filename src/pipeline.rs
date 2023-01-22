#![allow(dead_code)]
use crate::measures::*;
use crate::mesh;
use crate::mesh::HighlightType;
use crate::pointdata::*;
use crate::transform::*;
use anyhow::*;
use bitvector::*;
use macroquad::prelude::*;
use std::collections::BTreeSet;
use std::convert::TryInto;
use std::str::FromStr;

pub const ALL:&str = "All";
pub const HIGHLIGHTED:&str = "Highlighted";
pub const NON_HIGHLIGHTED:&str = "Non-Highlighted";

pub trait SimpleTable {
    fn transpose(&self) -> Vec<Vec<String>>;
    fn print(&self);
    fn add_empty_row(&mut self);
    fn unique_values_in_row(&mut self, i: usize) -> BTreeSet<String>;
    fn remove_column(&mut self, i: usize);
}

impl SimpleTable for Vec<Vec<String>> {
    fn transpose(&self) -> Vec<Vec<String>> {
        let mut table = Vec::new();
        if self.len() > 0 {
            for i in 0..self[0].len() {
                let mut row = Vec::new();
                for j in 0..self.len() {
                    row.push(self[j][i].to_owned());
                }
                table.push(row);
            }
        }
        table
    }
    fn print(&self) {
        for row in self.iter() {
            for item in row.iter() {
                print!("| {item:>n$} ", item = item, n = 10);
            }
            println!("|");
        }
    }
    fn add_empty_row(&mut self) {
        if self.len() > 0 {
            let mut row = Vec::new();
            for _i in 0..self[0].len() {
                row.push("".to_owned());
            }
            self.push(row);
        }
    }
    fn unique_values_in_row(&mut self, i: usize) -> BTreeSet<String> {
        let mut set = BTreeSet::new();
        for key in self[i].iter() {
            set.insert(key.to_string());
            if set.len() >= 100 {
                break;
            }
        }
        set
    }
    fn remove_column(&mut self, i: usize){
        for row in self[i].iter_mut() {
            row.remove(i);
        }
    }
}

#[derive(Debug, Clone, PartialOrd, PartialEq, Copy)]
pub enum Stage {
    Stage0NewData,
    Stage1XYI,
    Stage2Mesh,
    Stage3ProcessedMesh,
    Stage4Texture,
}

impl Stage {
    pub fn down(self, stage: Stage) -> Stage {
        match self {
            Stage::Stage0NewData => self,
            Stage::Stage1XYI => match stage {
                Stage::Stage0NewData => stage,
                _ => self,
            },
            Stage::Stage2Mesh => match stage {
                Stage::Stage0NewData => stage,
                Stage::Stage1XYI => stage,
                _ => self,
            },
            Stage::Stage3ProcessedMesh => match stage {
                Stage::Stage0NewData => stage,
                Stage::Stage1XYI => stage,
                Stage::Stage2Mesh => stage,
                _ => self,
            },
            Stage::Stage4Texture => stage,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum TransformationType {
    Linear,
    Logarithmic,
    Quantile,
    QuantileNormal,
}

impl TransformationType {
    pub fn to_transform(&self) -> Box<dyn Transform> {
        match self {
            TransformationType::Linear => Box::new(Normalize::new()),
            TransformationType::Logarithmic => Box::new(NormalizedLogarithmic::new()),
            TransformationType::Quantile => Box::new(Quantile::new()),
            TransformationType::QuantileNormal => Box::new(QuantileNormal::new()),
        }
    }
    pub fn text(&self) -> &str {
        match self {
            TransformationType::Linear => "Linear",
            TransformationType::Logarithmic => "Logarithmic",
            TransformationType::Quantile => "Quantile",
            TransformationType::QuantileNormal => "Quantile Normal",
        }
    }
}

impl ToString for TransformationType {
    fn to_string(&self) -> String {
        self.text().to_string()
    }
}

impl FromStr for TransformationType {
    type Err = Error;
    fn from_str(s: &str) -> Result<TransformationType> {
        match s {
            "Linear" => Ok(TransformationType::Linear),
            "Logarithmic" => Ok(TransformationType::Logarithmic),
            "Quantile" => Ok(TransformationType::Quantile),
            _ => Err(anyhow!("Failed to resolve transformation type '{}'", s)),
        }
    }
}

pub struct Pipeline {
    pub point_data: PointData,
    pub data_columns: Vec<String>,
    pub aux_columns: Vec<String>,
    //    pub highlightable_values: Vec<String>,
    pub parameters: mesh::Parameters,
    pub mesh: mesh::Mesh,
    pub unit_weights: Vec<f64>,
    pub highlights: BitVector,
    pub xyi: Vec<(f64, f64, f64, usize, bool)>,
    pub txtype: TransformationType,
    pub tytype: TransformationType,
    pub tx: Box<dyn Transform>,
    pub ty: Box<dyn Transform>,
    pub zoom: f64,
    pub aspect_ratio: f64,
    pub ox: f64,
    pub oy: f64,
    pub texture: Option<Texture2D>,
    pub stage: Stage,
}

impl Pipeline {
    pub fn new() -> Pipeline {
        Pipeline {
            point_data: PointData::new(),
            data_columns: Vec::<_>::new(),
            aux_columns: Vec::<_>::new(),
            //            highlightable_values: Vec::<_>::new(),
            parameters: mesh::Parameters::new(),
            mesh: mesh::Mesh::new(),
            unit_weights: Vec::<_>::new(),
            highlights: BitVector::new(0),
            xyi: Vec::<_>::new(),
            txtype: TransformationType::Linear,
            tytype: TransformationType::Linear,
            tx: Box::new(Quantile::new()),
            ty: Box::new(Quantile::new()),
            zoom: 1.0,
            aspect_ratio: 1.0,
            ox: 0.5,
            oy: 0.5,
            texture: None,
            stage: Stage::Stage0NewData,
        }
    }
    pub async fn load(&mut self, path: &str) -> Result<()> {
        let csv_content = load_file(path).await?;
        self.point_data = PointData::from_csv(&mut csv_content.as_slice())?;
        self.stage = Stage::Stage0NewData;
        self.data_columns.clear();
        self.aux_columns.clear();
        for column in self.point_data.headers.iter() {
            if self.point_data.data.contains_key(column) {
                self.data_columns.push(column.to_owned());
            }
        }
        for column in self.point_data.headers.iter() {
            if self.point_data.aux.contains_key(column) {
                self.aux_columns.push(column.to_owned());
            }
        }
        for column in self.point_data.headers.iter() {
            if self.point_data.data.contains_key(column) {
                self.aux_columns.push(column.to_owned());
            }
        }
        if self.data_columns.len() >= 2 {
            self.set_xcolumn(self.data_columns[0].to_owned());
            self.set_ycolumn(self.data_columns[1].to_owned());
        }
        self.unit_weights.clear();
        for _i in 0..self.point_data.length {
            self.unit_weights.push(1.0);
        }

        Ok(())
    }
    pub fn filter_headers(&mut self, filter: &dyn Fn(&str) -> bool) {
        self.point_data.filter_headers(filter);
    }

    pub fn view_box(&self) -> (f64, f64, f64, f64) {
        if self.aspect_ratio >= 1.0 {
            let dx = self.aspect_ratio / self.zoom;
            let dy = 1.0 / self.zoom;
            (
                self.ox - dx / 2.0,
                self.oy - dy / 2.0,
                self.ox + dx / 2.0,
                self.oy + dy / 2.0,
            )
        } else {
            let dx = 1.0 / self.zoom;
            let dy = 1.0 / self.zoom / self.aspect_ratio;
            (
                self.ox - dx / 2.0,
                self.oy - dy / 2.0,
                self.ox + dx / 2.0,
                self.oy + dy / 2.0,
            )
        }
    }
    pub fn update_view_box(&mut self) {
        let (x1, y1, x2, y2) = self.view_box();
        self.parameters.xmin = x1;
        self.parameters.ymin = y1;
        self.parameters.xmax = x2;
        self.parameters.ymax = y2;
    }

    pub fn get_zoom(&self) -> f64 {
        self.zoom
    }
    pub fn set_zoom(&mut self, zoom: f64) {
        if self.zoom != zoom {
            self.zoom = zoom;
            self.stage = self.stage.down(Stage::Stage1XYI);
        }
    }

    pub fn get_aspect_ratio(&self) -> f64 {
        self.aspect_ratio
    }
    pub fn set_aspect_ratio(&mut self, r: f64) {
        if self.aspect_ratio != r {
            self.aspect_ratio = r;
            self.stage = self.stage.down(Stage::Stage1XYI);
        }
    }

    pub fn offset_x(&self) -> f64 {
        self.ox
    }
    pub fn set_offset_x(&mut self, o: f64) {
        if self.ox != o {
            self.ox = o;
            self.stage = self.stage.down(Stage::Stage1XYI);
        }
    }
    pub fn offset_y(&self) -> f64 {
        self.oy
    }
    pub fn set_offset_y(&mut self, o: f64) {
        if self.oy != o {
            self.oy = o;
            self.stage = self.stage.down(Stage::Stage1XYI);
        }
    }
    pub fn relative_offset(&mut self, dx: f64, dy: f64) {
        if self.aspect_ratio >= 1.0 {
            self.ox -= dx * self.aspect_ratio / self.zoom;
            self.oy -= dy / self.zoom;
        } else {
            self.ox -= dx / self.zoom;
            self.oy -= dy / self.zoom / self.aspect_ratio;
        }
        self.stage = self.stage.down(Stage::Stage1XYI);
    }

    pub fn weight_column(&self) -> &str {
        &self.parameters.weight_column
    }
    pub fn set_weight_column(&mut self, column: String) {
        if self.parameters.weight_column != column {
            self.parameters.weight_column = column;
            self.stage = Stage::Stage0NewData;
        }
    }
    /*
    pub fn highlight_column(&self) -> &str{
        &self.parameters.highlight_column
    }
    */
    pub fn set_highlights(&mut self, new_highlights: BitVector) {
        if self.highlights != new_highlights {
            self.highlights = new_highlights;
            self.stage = Stage::Stage0NewData;
        }
    }
    pub fn highlight_type(&self) -> HighlightType {
        self.parameters.highlight_type
    }

    pub fn set_highlight_type(&mut self, value: HighlightType) {
        if self.parameters.highlight_type != value {
            self.parameters.highlight_type = value;
            self.stage = self.stage.down(Stage::Stage2Mesh);
        }
    }

    pub fn xcolumn(&self) -> &str {
        &self.parameters.xcolumn
    }
    pub fn set_xcolumn(&mut self, xcolumn: String) {
        if self.parameters.xcolumn != xcolumn {
            self.parameters.xcolumn = xcolumn;
            self.zoom_all();
            self.stage = Stage::Stage0NewData;
        }
    }
    pub fn ycolumn(&self) -> &str {
        &self.parameters.ycolumn
    }
    pub fn set_ycolumn(&mut self, ycolumn: String) {
        if self.parameters.ycolumn != ycolumn {
            self.parameters.ycolumn = ycolumn;
            self.zoom_all();
            self.stage = Stage::Stage0NewData;
        }
    }

    pub fn tx_type(&self) -> TransformationType {
        self.txtype
    }
    pub fn set_txtype(&mut self, txtype: TransformationType) {
        if self.txtype != txtype {
            self.txtype = txtype;
            self.stage = Stage::Stage0NewData;
        }
    }
    pub fn ty_type(&self) -> TransformationType {
        self.tytype
    }
    pub fn set_tytype(&mut self, tytype: TransformationType) {
        if self.tytype != tytype {
            self.tytype = tytype;
            self.stage = Stage::Stage0NewData;
        }
    }

    pub fn gaussian_points(&self) -> bool {
        self.parameters.gaussian_points
    }
    pub fn set_gaussian_points(&mut self, flag: bool) {
        if self.parameters.gaussian_points != flag {
            self.stage = self.stage.down(Stage::Stage1XYI);
            self.parameters.gaussian_points = flag;
        }
    }
    pub fn point_sigma(&self) -> f64 {
        self.parameters.point_sigma
    }
    pub fn set_point_sigma(&mut self, value: f64) {
        if self.parameters.point_sigma != value && self.gaussian_points() {
            self.stage = self.stage.down(Stage::Stage1XYI);
        }
        self.parameters.point_sigma = value;
    }
    pub fn density_multiplier(&self) -> f64 {
        self.parameters.density_multiplier
    }
    pub fn set_density_multiplier(&mut self, value: f64) {
        if self.parameters.density_multiplier != value {
            self.stage = self.stage.down(Stage::Stage2Mesh);
            self.parameters.density_multiplier = value;
        }
    }
    pub fn contrast(&self) -> f64 {
        self.parameters.contrast
    }
    pub fn set_contrast(&mut self, value: f64) {
        if self.parameters.contrast != value {
            self.stage = self.stage.down(Stage::Stage2Mesh);
            self.parameters.contrast = value;
        }
    }

    pub fn weights(&self) -> &Vec<f64> {
        if self.weight_column() == "" {
            &self.unit_weights
        } else {
            if self.point_data.data.contains_key(self.weight_column()) {
                &self.point_data.data[self.weight_column()]
            } else {
                &self.unit_weights
            }
        }
    }

    pub fn zoom_all(&mut self) {
        /*
        if self.point_data.data.contains_key(self.xcolumn())
            && self.point_data.data.contains_key(self.ycolumn())
        {
            let vx = &self.point_data.data[self.xcolumn()];
            let vy = &self.point_data.data[self.ycolumn()];
            self.parameters.zoom_all(vx, vy);
            self.stage = Stage::Stage0NewData;
        }
        */
        self.parameters.xmin = 0.0;
        self.parameters.xmax = 1.0;
        self.parameters.ymin = 0.0;
        self.parameters.ymax = 1.0;
        self.zoom = 1.0;
        self.aspect_ratio = 1.0;
        self.ox = 0.5;
        self.oy = 0.5;
        self.stage = self.stage.down(Stage::Stage1XYI);
    }
    pub fn extract_xyi(&mut self) {
        self.xyi.clear();
        /*
        self.highlights.clear();
        if self.highlight_column()==""{
            for _i in 0..self.point_data.length{
                self.highlights.push(false);
            }
        }
        else{
            if self.point_data.aux.contains_key(self.highlight_column()){
                let highlight_value = self.highlight_value().to_owned();
                for value in self.point_data.aux[self.highlight_column()].iter(){
                    self.highlights.push(value==&highlight_value);
                }
            }
            else{
                if self.point_data.data.contains_key(self.highlight_column()){
                    if let Ok(highlight_value) = f64::from_str(self.highlight_value()){
                        for value in self.point_data.data[self.highlight_column()].iter(){
                            self.highlights.push(*value==highlight_value);
                        }
                    }
                }
            }
        }
        */
        if self.highlights.len() != self.point_data.length {
            self.highlights = BitVector::new(self.point_data.len()) | self.highlights.clone();
        }

        if self.point_data.data.contains_key(self.xcolumn())
            && self.point_data.data.contains_key(self.ycolumn())
        {
            self.tx = self.txtype.to_transform();
            self.ty = self.tytype.to_transform();
            let xdata = &self.point_data.data[self.xcolumn()];
            let ydata = &self.point_data.data[self.ycolumn()];
            self.tx.calibrate(xdata);
            self.ty.calibrate(ydata);
            for (i, (&x, &y)) in xdata.iter().zip(ydata.iter()).enumerate() {
                if let (Some(xx), Some(yy)) = (self.tx.transform(x), self.ty.transform(y)) {
                    let w: f64 = self.weights()[i];
                    self.xyi
                        .push((xx, 1.0 - yy, w, i + 1, self.highlights.contains(i)));
                }
            }
        } else {
            let xdata = if self.point_data.data.contains_key(self.xcolumn()) {
                self.point_data.data[self.xcolumn()]
                    .iter()
                    .map(|x| Some(*x))
                    .collect::<Vec<_>>()
            } else {
                assert!(self.point_data.aux.contains_key(self.xcolumn()));
                self.point_data.aux[self.xcolumn()]
                    .iter()
                    .map(|x| x.parse::<f64>().ok())
                    .collect::<Vec<_>>()
            };
            let ydata = if self.point_data.data.contains_key(self.ycolumn()) {
                self.point_data.data[self.ycolumn()]
                    .iter()
                    .map(|x| Some(*x))
                    .collect::<Vec<_>>()
            } else {
                assert!(self.point_data.aux.contains_key(self.ycolumn()));
                self.point_data.aux[self.ycolumn()]
                    .iter()
                    .map(|x| x.parse::<f64>().ok())
                    .collect::<Vec<_>>()
            };

            self.tx.calibrate(
                xdata
                    .iter()
                    .flatten()
                    .map(|x| *x)
                    .collect::<Vec<_>>()
                    .as_slice(),
            );
            self.ty.calibrate(
                ydata
                    .iter()
                    .flatten()
                    .map(|x| *x)
                    .collect::<Vec<_>>()
                    .as_slice(),
            );
            for (i, pair) in xdata.iter().zip(ydata.iter()).enumerate() {
                if let (&Some(x), &Some(y)) = pair {
                    if let (Some(xx), Some(yy)) = (self.tx.transform(x), self.ty.transform(y)) {
                        let w: f64 = self.weights()[i];
                        self.xyi
                            .push((xx, 1.0 - yy, w, i + 1, self.highlights.contains(i)));
                    }
                }
            }

            //            self.tx = TransformationType::Linear.to_transform();
            //            self.ty = TransformationType::Linear.to_transform();
        }
        self.stage = Stage::Stage1XYI;
    }

    pub fn statistics(&self, x: f64, y: f64) -> Vec<Vec<String>> {
        let mut data = Vec::new();

        let mut row = Vec::new();
        row.push("".to_owned());
        row.push("".to_owned());
        for column in self.point_data.headers.iter() {
            row.push(column.to_owned());
        }

        data.push(row);

        if x >= 0.0 && y >= 0.0 {
            let xx = (x * (self.mesh.width as f64)) as usize;
            let yy = (y * (self.mesh.height as f64)) as usize;
            if let Some(index) = self.mesh.get_index_wide(xx, yy) {
                let mut row = Vec::new();
                row.push("Selected".to_owned());
                row.push("".to_owned());
                for column in self.point_data.headers.iter() {
                    row.push(self.point_data.get(column, index));
                }
                data.push(row);
            } else {
                let mut row = Vec::new();
                row.push("Selected".to_owned());
                row.push("".to_owned());
                for _column in self.point_data.headers.iter() {
                    row.push("".to_owned());
                }
                data.push(row);
            }
        }

        for row in self.statistics_table(ALL, None).transpose() {
            data.push(row);
        }
        if self.highlights.len() > 0 {
            for row in self
                .statistics_table(HIGHLIGHTED, Some(&self.highlights))
                .transpose()
            {
                data.push(row);
            }
            let mut nonhighlighted = BitVector::ones(self.point_data.len());
            for i in self.highlights.iter() {
                nonhighlighted.remove(i);
            }
            for row in self
                .statistics_table(NON_HIGHLIGHTED, Some(&nonhighlighted))
                .transpose()
            {
                data.push(row);
            }
        }

        data
    }

    pub fn statistics_table(
        &self,
        group_name: &str,
        selection: Option<&BitVector>,
    ) -> Vec<Vec<String>> {
        let mut table = Vec::new();
        let weights = self.weights();
        let measure_names = NumericStatistics::new().all_measure_names();
        let mut group_names = Vec::with_capacity(measure_names.len());
        group_names.resize(measure_names.len(), group_name.to_owned());
        table.push(group_names);
        table.push(measure_names);
        for column in self.point_data.headers.iter() {
            if self.point_data.data.contains_key(column) {
                let mut stat = NumericStatistics::new();
                let v = &self.point_data.data[column];
                if let Some(bv) = selection {
                    stat.add_weighted_selection(v, weights, bv.iter());
                } else {
                    stat.add_weighted(v, weights);
                }
                table.push(stat.all_measure_values());
            } else {
                table.add_empty_row();
            }
        }
        table
    }

    pub fn add_points(&mut self) {
        if self.parameters.gaussian_points {
            self.mesh
                .add_points_gaussian(&self.xyi, self.parameters.point_sigma);
        } else {
            self.mesh.add_points(&self.xyi, false);
        }
        self.stage = Stage::Stage2Mesh;
    }

    pub fn to_processed_mesh(&mut self) {
        match self.highlight_type() {
            HighlightType::Highlight => {
                self.mesh.to_processed_mesh();
                //self.mesh.normalize_processed_mesh();
                //self.mesh.multiply_processed_mesh(self.density_multiplier());
                self.mesh
                    .clamp_processed_mesh(self.density_multiplier(), self.contrast());
                self.mesh.to_processed_highlight_mesh();
                //self.mesh.normalize_processed_highlight_mesh();
                //self.mesh.multiply_processed_highlight_mesh(self.density_multiplier());
                self.mesh
                    .clamp_processed_highlight_mesh(self.density_multiplier(), self.contrast());
            }
            HighlightType::NoHighlight => {
                self.mesh.to_processed_mesh_sum_highlight();
                //self.mesh.normalize_processed_mesh();
                //self.mesh.multiply_processed_mesh(self.density_multiplier());
                self.mesh
                    .clamp_processed_mesh(self.density_multiplier(), self.contrast());
            }
            HighlightType::HighlighedOnly => {
                //self.mesh.clean_processed_mesh();
                self.mesh.to_processed_highlight_mesh();
                //self.mesh.normalize_processed_highlight_mesh();
                //self.mesh.multiply_processed_highlight_mesh(self.density_multiplier());
                self.mesh
                    .clamp_processed_highlight_mesh(self.density_multiplier(), self.contrast());
            }
            HighlightType::NonHighlightedOnly => {
                self.mesh.clean_processed_highlight_mesh();
                self.mesh.to_processed_mesh();
                //self.mesh.normalize_processed_mesh();
                //self.mesh.multiply_processed_mesh(self.density_multiplier());
                self.mesh
                    .clamp_processed_mesh(self.density_multiplier(), self.contrast());
            }
        }
        self.stage = Stage::Stage3ProcessedMesh;
    }

    pub fn to_rgba8(&mut self) {
        match self.highlight_type() {
            HighlightType::Highlight => {
                self.mesh.to_rgba8_blue_cyan();
                self.mesh.add_rgba8_red_highlight();
            }
            HighlightType::NoHighlight => {
                self.mesh.to_rgba8_blue_cyan();
            }
            HighlightType::HighlighedOnly => {
                self.mesh.clean_rgba();
                self.mesh.add_rgba8_red_highlight();
            }
            HighlightType::NonHighlightedOnly => {
                self.mesh.to_rgba8_blue_cyan();
            }
        }
    }

    pub fn to_texture(&mut self) {
        let texture = Texture2D::from_rgba8(
            self.mesh.width.try_into().unwrap(),
            self.mesh.height.try_into().unwrap(),
            &self.mesh.rgba8,
        );
        self.texture = Some(texture);
        self.stage = Stage::Stage4Texture;
    }

    pub fn pipeline_step(&mut self) -> bool {
        //        println!("Pipeline step {:?}", self.stage);
        match self.stage {
            Stage::Stage0NewData => {
                self.extract_xyi();
                false
            }
            Stage::Stage1XYI => {
                self.update_view_box();
                self.parameters.adapt_mesh(&mut self.mesh);
                self.add_points();
                false
            }
            Stage::Stage2Mesh => {
                self.to_processed_mesh();
                false
            }
            Stage::Stage3ProcessedMesh => {
                self.to_rgba8();
                self.to_texture();
                false
            }
            Stage::Stage4Texture => true,
        }
    }

    pub fn run(&mut self) {
        while !self.pipeline_step() {}
    }
}
