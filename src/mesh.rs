#![allow(dead_code)]

#[derive(Debug, Clone, PartialOrd, PartialEq, Copy)]
pub enum HighlightType {
    Highlight,
    NoHighlight,
    HighlighedOnly,
    NonHighlightedOnly,
}

#[derive(Debug, Clone)]
pub struct Parameters {
    pub xcolumn: String,
    pub ycolumn: String,
    pub weight_column: String,
    pub highlight_column: String,
    pub highlight_value: String,
    pub highlight_type: HighlightType,
    pub mesh_width: usize,
    pub mesh_height: usize,
    pub xmin: f64,
    pub xmax: f64,
    pub ymin: f64,
    pub ymax: f64,
    pub gaussian_points: bool,
    pub point_sigma: f64,
    pub density_multiplier: f64,
}

impl Parameters {
    pub fn new() -> Parameters {
        Parameters {
            xcolumn: "".into(),
            ycolumn: "".into(),
            weight_column: "".into(),
            highlight_column: "".into(),
            highlight_value: "".into(),
            highlight_type: HighlightType::Highlight,
            mesh_width: 800,
            mesh_height: 800,
            xmin: 0.0,
            xmax: 1.0,
            ymin: 0.0,
            ymax: 1.0,
            gaussian_points: false,
            point_sigma: 1.0,
            density_multiplier: 1.0,
        }
    }

    pub fn new_mesh(&self) -> Mesh {
        let mut mesh = Mesh::new();
        self.adapt_mesh(&mut mesh);
        mesh
    }

    pub fn adapt_mesh(&self, mesh: &mut Mesh) {
        mesh.resize(self.mesh_width, self.mesh_height);
        mesh.xmin = self.xmin;
        mesh.xmax = self.xmax;
        mesh.ymin = self.ymin;
        mesh.ymax = self.ymax;
    }
    pub fn zoom_all_x(&mut self, v: &Vec<f64>) {
        if v.len() > 0 {
            self.xmin = v[0];
            self.xmax = v[0];
            for value in v.iter() {
                self.xmin = self.xmin.min(*value);
                self.xmax = self.xmax.max(*value);
            }
        }
        let delta = (self.xmax - self.xmin) * 3.0 / (self.mesh_width as f64);
        self.xmin -= delta;
        self.xmax += delta;
    }
    pub fn zoom_all_y(&mut self, v: &Vec<f64>) {
        if v.len() > 0 {
            self.ymin = v[0];
            self.ymax = v[0];
            for value in v.iter() {
                self.ymin = self.ymin.min(*value);
                self.ymax = self.ymax.max(*value);
            }
        }
        let delta = (self.ymax - self.ymin) * 3.0 / (self.mesh_height as f64);
        self.ymin -= delta;
        self.ymax += delta;
    }
    pub fn zoom_all(&mut self, vx: &Vec<f64>, vy: &Vec<f64>) {
        self.zoom_all_x(vx);
        self.zoom_all_y(vy);
    }
}

pub struct Mesh {
    pub width: usize,
    pub height: usize,
    pub xmin: f64,
    pub xmax: f64,
    pub ymin: f64,
    pub ymax: f64,
    pub mesh: Vec<f64>,
    pub highlight_mesh: Vec<f64>,
    pub index_mesh: Vec<usize>,
    pub processed_mesh: Vec<f64>,
    pub processed_highlight_mesh: Vec<f64>,
    pub rgba8: Vec<u8>,
}

impl Mesh {
    pub fn new() -> Mesh {
        Mesh {
            width: 0,
            height: 0,
            xmin: 0.0,
            xmax: 1.0,
            ymin: 0.0,
            ymax: 1.0,
            mesh: Vec::new(),
            highlight_mesh: Vec::new(),
            index_mesh: Vec::new(),
            processed_mesh: Vec::new(),
            processed_highlight_mesh: Vec::new(),
            rgba8: Vec::new(),
        }
    }

    pub fn resize(&mut self, width: usize, height: usize) -> &mut Self {
        let size = width * height;
        self.mesh.resize(size, 0.0);
        self.highlight_mesh.resize(size, 0.0);
        self.index_mesh.resize(size, 0);
        self.processed_mesh.resize(size, 0.0);
        self.processed_highlight_mesh.resize(size, 0.0);
        self.rgba8.resize(4 * size, 0);
        self.width = width;
        self.height = height;
        self.clean()
    }

    pub fn clean(&mut self) -> &mut Self {
        for i in self.mesh.iter_mut() {
            *i = 0.0;
        }
        for i in self.highlight_mesh.iter_mut() {
            *i = 0.0;
        }
        for i in self.index_mesh.iter_mut() {
            *i = 0;
        }
        for i in self.processed_mesh.iter_mut() {
            *i = 0.0;
        }
        for i in self.processed_highlight_mesh.iter_mut() {
            *i = 0.0;
        }
        self
    }
    pub fn get_index(&self, x:usize, y:usize)->Option<usize>{
        if x<self.width && y<self.height{
            let index = self.index_mesh[x+self.width*y];
            if index==0{
                None
            }
            else{
                Some(index-1)
            }
        }
        else{
            None
        }
    }
    pub fn get_index_wide(&self, x:usize, y:usize)->Option<usize>{
        if let Some(index) = self.get_index(x, y){
            Some(index)
        }
        else if let Some(index) = self.get_index(x+1, y){
            Some(index)
        }
        else if let Some(index) = self.get_index(x, y+1){
            Some(index)
        }
        else if let Some(index) = self.get_index(x+1, y+1){
            Some(index)
        }
        else{
            None
        }
    }

    pub fn point(&mut self, x: f64, y: f64, weight: f64, index: usize, highlight: bool) {
        let fx = (x - self.xmin) / (self.xmax - self.xmin);
        let fy = (y - self.ymin) / (self.ymax - self.ymin);
        if fx >= 0.0 && fy >= 0.0 {
            let ix = (fx * (self.width as f64)) as usize;
            let iy = (fy * (self.height as f64)) as usize;
            if ix < self.width && iy < self.height {
                //                println!("  -> mesh {} {}",ix,iy);
                if highlight {
                    self.highlight_mesh[ix + iy * self.width] += weight;
                } else {
                    self.mesh[ix + iy * self.width] += weight;
                }
                self.index_mesh[ix + iy * self.width] = index;
            }
        }
    }

    pub fn point_antialiased(
        &mut self,
        x: f64,
        y: f64,
        weight: f64,
        index: usize,
        highlight: bool,
    ) {
        let fx = (x - self.xmin) / (self.xmax - self.xmin);
        let fy = (y - self.ymin) / (self.ymax - self.ymin);
        if fx >= 0.0 && fy >= 0.0 {
            let ix = (fx * (self.width as f64)) as usize;
            let iy = (fy * (self.height as f64)) as usize;
            if ix > 1 && ix < self.width - 1 && iy > 1 && iy < self.height - 1 {
                let dx = (fx * (self.width as f64)) + 0.5 - (ix as f64);
                let dy = (fy * (self.height as f64)) + 0.5 - (iy as f64);
                let two_sigma = 1.0;
                let mut w00 =
                    (-((dx - 1.5) * (dx - 1.5) + (dy - 1.5) * (dy - 1.5)) / two_sigma).exp();
                let mut w10 =
                    (-((dx - 0.5) * (dx - 0.5) + (dy - 1.5) * (dy - 1.5)) / two_sigma).exp();
                let mut w20 =
                    (-((dx + 0.5) * (dx + 0.5) + (dy - 1.5) * (dy - 1.5)) / two_sigma).exp();

                let mut w01 =
                    (-((dx - 1.5) * (dx - 1.5) + (dy - 0.5) * (dy - 0.5)) / two_sigma).exp();
                let mut w11 =
                    (-((dx - 0.5) * (dx - 0.5) + (dy - 0.5) * (dy - 0.5)) / two_sigma).exp();
                let mut w21 =
                    (-((dx + 0.5) * (dx + 0.5) + (dy - 0.5) * (dy - 0.5)) / two_sigma).exp();

                let mut w02 =
                    (-((dx - 1.5) * (dx - 1.5) + (dy + 0.5) * (dy + 0.5)) / two_sigma).exp();
                let mut w12 =
                    (-((dx - 0.5) * (dx - 0.5) + (dy + 0.5) * (dy + 0.5)) / two_sigma).exp();
                let mut w22 =
                    (-((dx + 0.5) * (dx + 0.5) + (dy + 0.5) * (dy + 0.5)) / two_sigma).exp();

                let w = weight / (w00 + w01 + w02 + w10 + w11 + w12 + w20 + w21 + w22);

                w00 *= w;
                w01 *= w;
                w02 *= w;
                w10 *= w;
                w11 *= w;
                w12 *= w;
                w20 *= w;
                w21 *= w;
                w22 *= w;

                if highlight {
                    self.highlight_mesh[ix - 1 + (iy - 1) * self.width] += w00;
                    self.highlight_mesh[ix + (iy - 1) * self.width] += w10;
                    self.highlight_mesh[ix + 1 + (iy - 1) * self.width] += w20;

                    self.highlight_mesh[ix - 1 + iy * self.width] += w01;
                    self.highlight_mesh[ix + iy * self.width] += w11;
                    self.highlight_mesh[ix + 1 + iy * self.width] += w21;

                    self.highlight_mesh[ix - 1 + (iy + 1) * self.width] += w02;
                    self.highlight_mesh[ix + (iy + 1) * self.width] += w12;
                    self.highlight_mesh[ix + 1 + (iy + 1) * self.width] += w22;
                } else {
                    self.mesh[ix - 1 + (iy - 1) * self.width] += w00;
                    self.mesh[ix + (iy - 1) * self.width] += w10;
                    self.mesh[ix + 1 + (iy - 1) * self.width] += w20;

                    self.mesh[ix - 1 + iy * self.width] += w01;
                    self.mesh[ix + iy * self.width] += w11;
                    self.mesh[ix + 1 + iy * self.width] += w21;

                    self.mesh[ix - 1 + (iy + 1) * self.width] += w02;
                    self.mesh[ix + (iy + 1) * self.width] += w12;
                    self.mesh[ix + 1 + (iy + 1) * self.width] += w22;
                }
                self.index_mesh[ix + iy * self.width] = index;
            }
        }
    }

    pub fn point_gaussian5x5(
        &mut self,
        x: f64,
        y: f64,
        weight: f64,
        index: usize,
        highlight: bool,
        sigma: f64,
    ) {
        let fx = (x - self.xmin) / (self.xmax - self.xmin);
        let fy = (y - self.ymin) / (self.ymax - self.ymin);
        let two_sigma = 2.0 * sigma;
        if fx >= 0.0 && fy >= 0.0 {
            let ix = (fx * (self.width as f64)) as usize;
            let iy = (fy * (self.height as f64)) as usize;
            if ix > 2 && ix < self.width - 2 && iy > 2 && iy < self.height - 2 {
                let dx = (fx * (self.width as f64)) + 0.5 - (ix as f64);
                let dy = (fy * (self.height as f64)) + 0.5 - (iy as f64);

                let dx0 = (dx - 2.5) * (dx - 2.5);
                let dx1 = (dx - 1.5) * (dx - 1.5);
                let dx2 = (dx - 0.5) * (dx - 0.5);
                let dx3 = (dx + 0.5) * (dx + 0.5);
                let dx4 = (dx + 1.5) * (dx + 1.5);

                let dy0 = (dy - 2.5) * (dy - 2.5);
                let dy1 = (dy - 1.5) * (dy - 1.5);
                let dy2 = (dy - 0.5) * (dy - 0.5);
                let dy3 = (dy + 0.5) * (dy + 0.5);
                let dy4 = (dy + 1.5) * (dy + 1.5);

                let mut w00 = (-(dx0 + dy0) / two_sigma).exp();
                let mut w10 = (-(dx1 + dy0) / two_sigma).exp();
                let mut w20 = (-(dx2 + dy0) / two_sigma).exp();
                let mut w30 = (-(dx3 + dy0) / two_sigma).exp();
                let mut w40 = (-(dx4 + dy0) / two_sigma).exp();

                let mut w01 = (-(dx0 + dy1) / two_sigma).exp();
                let mut w11 = (-(dx1 + dy1) / two_sigma).exp();
                let mut w21 = (-(dx2 + dy1) / two_sigma).exp();
                let mut w31 = (-(dx3 + dy1) / two_sigma).exp();
                let mut w41 = (-(dx4 + dy1) / two_sigma).exp();

                let mut w02 = (-(dx0 + dy2) / two_sigma).exp();
                let mut w12 = (-(dx1 + dy2) / two_sigma).exp();
                let mut w22 = (-(dx2 + dy2) / two_sigma).exp();
                let mut w32 = (-(dx3 + dy2) / two_sigma).exp();
                let mut w42 = (-(dx4 + dy2) / two_sigma).exp();

                let mut w03 = (-(dx0 + dy3) / two_sigma).exp();
                let mut w13 = (-(dx1 + dy3) / two_sigma).exp();
                let mut w23 = (-(dx2 + dy3) / two_sigma).exp();
                let mut w33 = (-(dx3 + dy3) / two_sigma).exp();
                let mut w43 = (-(dx4 + dy3) / two_sigma).exp();

                let mut w04 = (-(dx0 + dy4) / two_sigma).exp();
                let mut w14 = (-(dx1 + dy4) / two_sigma).exp();
                let mut w24 = (-(dx2 + dy4) / two_sigma).exp();
                let mut w34 = (-(dx3 + dy4) / two_sigma).exp();
                let mut w44 = (-(dx4 + dy4) / two_sigma).exp();

                let w = weight
                    / (w00
                        + w01
                        + w02
                        + w03
                        + w04
                        + w10
                        + w11
                        + w12
                        + w13
                        + w14
                        + w20
                        + w21
                        + w22
                        + w23
                        + w24
                        + w30
                        + w31
                        + w32
                        + w33
                        + w34
                        + w40
                        + w41
                        + w42
                        + w43
                        + w44);

                w00 *= w;
                w01 *= w;
                w02 *= w;
                w03 *= w;
                w04 *= w;

                w10 *= w;
                w11 *= w;
                w12 *= w;
                w13 *= w;
                w14 *= w;

                w20 *= w;
                w21 *= w;
                w22 *= w;
                w23 *= w;
                w24 *= w;

                w30 *= w;
                w31 *= w;
                w32 *= w;
                w33 *= w;
                w34 *= w;

                w40 *= w;
                w41 *= w;
                w42 *= w;
                w43 *= w;
                w44 *= w;

                if highlight {
                    self.highlight_mesh[ix - 2 + (iy - 2) * self.width] += w00;
                    self.highlight_mesh[ix - 1 + (iy - 2) * self.width] += w10;
                    self.highlight_mesh[ix + (iy - 2) * self.width] += w20;
                    self.highlight_mesh[ix + 1 + (iy - 2) * self.width] += w30;
                    self.highlight_mesh[ix + 2 + (iy - 2) * self.width] += w40;

                    self.highlight_mesh[ix - 2 + (iy - 1) * self.width] += w01;
                    self.highlight_mesh[ix - 1 + (iy - 1) * self.width] += w11;
                    self.highlight_mesh[ix + (iy - 1) * self.width] += w21;
                    self.highlight_mesh[ix + 1 + (iy - 1) * self.width] += w31;
                    self.highlight_mesh[ix + 2 + (iy - 1) * self.width] += w41;

                    self.highlight_mesh[ix - 2 + iy * self.width] += w02;
                    self.highlight_mesh[ix - 1 + iy * self.width] += w12;
                    self.highlight_mesh[ix + iy * self.width] += w22;
                    self.highlight_mesh[ix + 1 + iy * self.width] += w32;
                    self.highlight_mesh[ix + 2 + iy * self.width] += w42;

                    self.highlight_mesh[ix - 2 + (iy + 1) * self.width] += w03;
                    self.highlight_mesh[ix - 1 + (iy + 1) * self.width] += w13;
                    self.highlight_mesh[ix + (iy + 1) * self.width] += w23;
                    self.highlight_mesh[ix + 1 + (iy + 1) * self.width] += w33;
                    self.highlight_mesh[ix + 2 + (iy + 1) * self.width] += w43;

                    self.highlight_mesh[ix - 2 + (iy + 2) * self.width] += w04;
                    self.highlight_mesh[ix - 1 + (iy + 2) * self.width] += w14;
                    self.highlight_mesh[ix + (iy + 2) * self.width] += w24;
                    self.highlight_mesh[ix + 1 + (iy + 2) * self.width] += w34;
                    self.highlight_mesh[ix + 2 + (iy + 2) * self.width] += w44;
                } else {
                    self.mesh[ix - 2 + (iy - 2) * self.width] += w00;
                    self.mesh[ix - 1 + (iy - 2) * self.width] += w10;
                    self.mesh[ix + (iy - 2) * self.width] += w20;
                    self.mesh[ix + 1 + (iy - 2) * self.width] += w30;
                    self.mesh[ix + 2 + (iy - 2) * self.width] += w40;

                    self.mesh[ix - 2 + (iy - 1) * self.width] += w01;
                    self.mesh[ix - 1 + (iy - 1) * self.width] += w11;
                    self.mesh[ix + (iy - 1) * self.width] += w21;
                    self.mesh[ix + 1 + (iy - 1) * self.width] += w31;
                    self.mesh[ix + 2 + (iy - 1) * self.width] += w41;

                    self.mesh[ix - 2 + iy * self.width] += w02;
                    self.mesh[ix - 1 + iy * self.width] += w12;
                    self.mesh[ix + iy * self.width] += w22;
                    self.mesh[ix + 1 + iy * self.width] += w32;
                    self.mesh[ix + 2 + iy * self.width] += w42;

                    self.mesh[ix - 2 + (iy + 1) * self.width] += w03;
                    self.mesh[ix - 1 + (iy + 1) * self.width] += w13;
                    self.mesh[ix + (iy + 1) * self.width] += w23;
                    self.mesh[ix + 1 + (iy + 1) * self.width] += w33;
                    self.mesh[ix + 2 + (iy + 1) * self.width] += w43;

                    self.mesh[ix - 2 + (iy + 2) * self.width] += w04;
                    self.mesh[ix - 1 + (iy + 2) * self.width] += w14;
                    self.mesh[ix + (iy + 2) * self.width] += w24;
                    self.mesh[ix + 1 + (iy + 2) * self.width] += w34;
                    self.mesh[ix + 2 + (iy + 2) * self.width] += w44;
                }
                self.index_mesh[ix + iy * self.width] = index;
            }
        }
    }

    pub fn point_gaussian(
        &mut self,
        x: f64,
        y: f64,
        weight: f64,
        index: usize,
        highlight: bool,
        sigma: f64,
    ) {
        let fx = (x - self.xmin) / (self.xmax - self.xmin);
        let fy = (y - self.ymin) / (self.ymax - self.ymin);
        let two_sigma = 2.0 * sigma;
        let n = (2.0 * sigma) as usize;
        if n<=2{
            self.point_gaussian5x5(x, y, weight, index, highlight, sigma);
            return;
        }
        let dim = 2 * n + 1;
        let mut w = Vec::with_capacity(dim * dim);
        w.resize(dim * dim, 0.0);
        let sub = n as f64;

        if fx >= 0.0 && fy >= 0.0 {
            let ix = (fx * (self.width as f64)) as usize;
            let iy = (fy * (self.height as f64)) as usize;
            let dx = (fx * (self.width as f64))  - (ix as f64);
            let dy = (fy * (self.height as f64)) - (iy as f64);
            let mut sum = 0.0;
            for i in 0..dim {
                let rx = dx + (i as f64) - sub;
                let rx2 = rx * rx;
                for j in 0..dim {
                    let ry = dy + (j as f64) - sub;
                    let ry2 = ry * ry;
                    let weight = (-(rx2 + ry2) / two_sigma).exp();
                    w[i + dim * j] = weight;
                    sum += weight;
                }
            }
            if sum == 0.0 {
                sum = 1.0;
            }
            for value in w.iter_mut() {
                *value /= sum;
                *value *= weight;
            }

            if highlight {
                for i in 0..dim {
                    let ii = ((ix+i) as isize) - (n as isize);
                    if ii>=0 && (ii as usize)<self.width{
                        for j in 0..dim {
                            let jj = ((iy+j) as isize) - (n as isize);
                            if jj>=0 && (jj as usize)<self.height{
//                                println!("ix:{} i:{} iy:{} j:{} n:{} ii:{} jj:{}",ix,i,iy,j,n,ii,jj);
                                self.highlight_mesh[(ii as usize) + (jj as usize) * self.width] +=
                                w[i + dim * j];
                            }
                        }
                    }
                }
            } else {
                for i in 0..dim {
                    let ii = ((ix+i) as isize) - (n as isize);
                    if ii>=0 && (ii as usize)<self.width{
                        for j in 0..dim {
                            let jj = ((iy+j) as isize) - (n as isize);
                            if jj>=0 && (jj as usize)<self.height{
//                                println!("ix:{} i:{} iy:{} j:{} n:{} ii:{} jj:{}",ix,i,iy,j,n,ii,jj);
                                self.mesh[(ii as usize) + (jj as usize) * self.width] +=
                                w[i + dim * j];
                            }
                        }
                    }
                }
            }
            self.index_mesh[ix + iy * self.width] = index;
        }
    }

    pub fn normalize_processed_mesh(&mut self) {
        let mut maximum = self.processed_mesh[0];
        for &value in self.processed_mesh.iter() {
            maximum = maximum.max(value);
        }
        if maximum == 0.0 {
            maximum = 1.0;
        }
        for i in 0..self.processed_mesh.len() {
            self.processed_mesh[i] /= maximum;
        }
    }

    pub fn normalize_processed_highlight_mesh(&mut self) {
        let mut maximum = self.processed_highlight_mesh[0];
        for &value in self.processed_highlight_mesh.iter() {
            maximum = maximum.max(value);
        }
        if maximum == 0.0 {
            maximum = 1.0;
        }
        for i in 0..self.processed_highlight_mesh.len() {
            self.processed_highlight_mesh[i] /= maximum;
        }
    }

    pub fn multiply_processed_mesh(&mut self, value: f64) {
        for i in 0..self.processed_mesh.len() {
            self.processed_mesh[i] *= value;
        }
    }
    pub fn multiply_processed_highlight_mesh(&mut self, value: f64) {
        for i in 0..self.processed_highlight_mesh.len() {
            self.processed_highlight_mesh[i] *= value;
        }
    }

    pub fn clean_processed_mesh(&mut self) -> &mut Self {
        for i in self.processed_mesh.iter_mut() {
            *i = 0.0;
        }
        self
    }
    pub fn clean_processed_highlight_mesh(&mut self) -> &mut Self {
        for i in self.processed_highlight_mesh.iter_mut() {
            *i = 0.0;
        }
        self
    }
    pub fn clean_rgba(&mut self) -> &mut Self {
        for (i, x) in self.rgba8.iter_mut().enumerate() {
            if i % 4 == 3 {
                *x = 255;
            } else {
                *x = 0;
            }
        }
        self
    }

    pub fn to_processed_mesh(&mut self) {
        for i in 0..self.mesh.len() {
            self.processed_mesh[i] = self.mesh[i];
        }
    }
    pub fn to_processed_highlight_mesh(&mut self) {
        for i in 0..self.mesh.len() {
            self.processed_highlight_mesh[i] = self.highlight_mesh[i];
        }
    }
    pub fn to_processed_mesh_sum_highlight(&mut self) {
        for i in 0..self.mesh.len() {
            self.processed_mesh[i] = self.mesh[i] + self.highlight_mesh[i];
            self.processed_highlight_mesh[i] = 0.0;
        }
    }

    pub fn to_processed_mesh_smear(&mut self) {
        for iy in 1..(self.height - 1) {
            let iy0 = (iy - 1) * self.width;
            let iy1 = iy * self.width;
            let iy2 = (iy + 1) * self.width;
            for ix in 1..(self.width - 1) {
                let ix0 = ix - 1;
                let ix1 = ix;
                let ix2 = ix + 1;

                self.processed_mesh[ix1 + iy1] = self.mesh[ix0 + iy0]
                    + 2.0 * self.mesh[ix1 + iy0]
                    + self.mesh[ix2 + iy0]
                    + 2.0 * self.mesh[ix0 + iy1]
                    + 4.0 * self.mesh[ix1 + iy1]
                    + 2.0 * self.mesh[ix2 + iy1]
                    + self.mesh[ix0 + iy2]
                    + 2.0 * self.mesh[ix1 + iy2]
                    + self.mesh[ix2 + iy2];
            }
        }
    }

    pub fn to_rgba8_gray(&mut self) {
        for (i, m) in self.processed_mesh.iter().enumerate() {
            let value: u8 = if *m < 0.0 {
                0
            } else {
                if *m >= 1.0 {
                    255
                } else {
                    (255.0 * m) as u8
                }
            };
            self.rgba8[4 * i] = value;
            self.rgba8[4 * i + 1] = value;
            self.rgba8[4 * i + 2] = value;
            self.rgba8[4 * i + 3] = 255;
        }
    }
    pub fn to_rgba8_blue_cyan(&mut self) {
        for (i, m) in self.processed_mesh.iter().enumerate() {
            let blue: u8 = if *m < 0.0 {
                0
            } else {
                if *m >= 0.5 {
                    255
                } else {
                    (255.0 * m * 2.0) as u8
                }
            };
            let green: u8 = if *m < 0.5 {
                0
            } else {
                if *m >= 1.0 {
                    255
                } else {
                    (255.0 * (m - 0.5) * 2.0) as u8
                }
            };
            self.rgba8[4 * i] = 0;
            self.rgba8[4 * i + 1] = green;
            self.rgba8[4 * i + 2] = blue;
            self.rgba8[4 * i + 3] = 255;
        }
    }
    pub fn add_rgba8_red_highlight(&mut self) {
        for (i, m) in self.processed_highlight_mesh.iter().enumerate() {
            let red: u8 = if *m < 0.0 {
                0
            } else {
                if *m >= 1.0 {
                    255
                } else {
                    (255.0 * m) as u8
                }
            };
            self.rgba8[4 * i] = red;
        }
    }

    pub fn add_points(&mut self, xyi: &[(f64, f64, f64, usize, bool)], antialiased: bool) {
        if antialiased {
            for (x, y, w, index, highlight) in xyi {
                self.point_antialiased(*x, *y, *w, *index, *highlight);
            }
        } else {
            for (x, y, w, index, highlight) in xyi {
                self.point(*x, *y, *w, *index, *highlight);
            }
        }
    }

    pub fn add_points_gaussian(&mut self, xyi: &[(f64, f64, f64, usize, bool)], sigma: f64) {
        for (x, y, w, index, highlight) in xyi {
            self.point_gaussian(*x, *y, *w, *index, *highlight, sigma);
        }
    }

    pub fn test_pattern(&mut self) {
        for y in 0..self.height {
            for x in 0..self.width {
                let i = x + self.width * y;
                self.rgba8[4 * i] = (x % 256) as u8;
                self.rgba8[4 * i + 1] = (y % 256) as u8;
                self.rgba8[4 * i + 2] = 0;
                self.rgba8[4 * i + 3] = 255;
            }
        }
    }
}
