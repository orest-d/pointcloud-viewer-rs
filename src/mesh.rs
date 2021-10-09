#[derive(Debug, Clone)]
pub struct Parameters {
    pub xcolumn: String,
    pub ycolumn: String,
    pub mesh_width: usize,
    pub mesh_height: usize,
    pub xmin: f64,
    pub xmax: f64,
    pub ymin: f64,
    pub ymax: f64,
}

impl Parameters {
    pub fn new() -> Parameters {
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
    pub fn zoom_all_x(&mut self, v:&Vec<f64>){
        if v.len()>0{
            self.xmin = v[0];
            self.xmax = v[0];
            for value in v.iter(){
                self.xmin = self.xmin.min(*value);
                self.xmax = self.xmax.max(*value);
            }
        }
    }
    pub fn zoom_all_y(&mut self, v:&Vec<f64>){
        if v.len()>0{
            self.ymin = v[0];
            self.ymax = v[0];
            for value in v.iter(){
                self.ymin = self.ymin.min(*value);
                self.ymax = self.ymax.max(*value);
            }
        }
    }
    pub fn zoom_all(&mut self, vx:&Vec<f64>, vy:&Vec<f64>){
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
            rgba8: Vec::new(),
        }
    }


    pub fn resize(&mut self, width: usize, height: usize) -> &mut Self {
        let size = width * height;
        self.mesh.resize(size, 0.0);
        self.rgba8.resize(4*size, 0);
        self.width = width;
        self.height = height;
        self.clean()
    }

    pub fn clean(&mut self) -> &mut Self {
        for i in self.mesh.iter_mut() {
            *i = 0.0;
        }
        self
    }

    pub fn point(&mut self, x: f64, y: f64, weight:f64){
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

    pub fn to_rgba8_gray(&mut self){
        for (i,m) in self.mesh.iter().enumerate() {
            let value:u8 = if *m<0.0 {0} else {if *m>=1.0 {255} else {(255.0*m) as u8} };
            self.rgba8[4*i]= value;
            self.rgba8[4*i+1]= value;
            self.rgba8[4*i+2]= value;
            self.rgba8[4*i+3]= 255;
        }
    }

    pub fn plain_points(&mut self, vx:&Vec<f64>, vy:&Vec<f64>){
        for (i,(&x,&y)) in vx.iter().zip(vy.iter()).enumerate() {
//            println!("{}: {} {}",i,x,y);
            self.point(x, y, 1.0);
        }
    }
    pub fn test_pattern(&mut self){
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
