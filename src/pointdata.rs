use std::collections::HashMap;
use std::convert::TryInto;
use anyhow::Result;
use csv;
use std::f64::consts::PI;

pub struct PointData {
    pub length: usize,
    pub headers: Vec<String>,
    pub data: HashMap<String, Vec<f64>>,
    pub aux: HashMap<String, Vec<String>>,
}

impl PointData {
    pub fn new() -> PointData {
        PointData {
            length: 0,
            headers: Vec::new(),
            data: HashMap::new(),
            aux: HashMap::new(),
        }
    }
    pub fn len(&self) -> usize {
        self.length
    }
    pub fn with_data_column(&mut self, column: &str) -> &mut Self {
        self.headers.push(column.to_owned());
        self.data.insert(column.to_owned(), Vec::new());
        self
    }
    pub fn with_aux_column(&mut self, column: &str) -> &mut Self {
        self.headers.push(column.to_owned());
        self.aux.insert(column.to_owned(), Vec::new());
        self
    }
    pub fn allocate(&mut self, n: usize) -> &mut Self {
        self.length = n;
        for (key, value) in self.data.iter_mut() {
            value.resize(n, 0.0);
        }
        for (key, value) in self.aux.iter_mut() {
            value.resize(n, "".to_string());
        }
        self
    }
    pub fn set_data(&mut self, column: &str, index: usize, value: f64) -> &mut Self {
        if (index >= self.length) {
            self.allocate(index);
        }
        if let Some(v) = self.data.get_mut(column) {
            v[index] = value;
        } else {
            let mut v = Vec::with_capacity(self.length);
            v.resize(self.length, 0.0);
            v[index] = value;
            self.data.insert(column.to_owned(), v);
        }
        self
    }
    pub fn set_aux(&mut self, column: &str, index: usize, value: String) -> &mut Self {
        if (index >= self.length) {
            self.allocate(index);
        }
        if let Some(v) = self.aux.get_mut(column) {
            v[index] = value;
        } else {
            let mut v: Vec<String> = Vec::with_capacity(self.length);
            v.resize(self.length, "".to_string());
            v[index] = value;
            self.aux.insert(column.to_owned(), v);
        }
        self
    }
    pub fn row(&self, index: usize) -> Vec<String> {
        let mut v = Vec::with_capacity(self.headers.len());
        if (index < self.length) {
            for column in self.headers.iter() {
                if let Some(column_data) = self.data.get(column) {
                    v.push(format!("{}", column_data[index]));
                } else {
                    if let Some(aux_data) = self.aux.get(column) {
                        v.push(format!("\"{}\"", aux_data[index]));
                    }
                }
            }
        }
        v
    }
    pub fn to_csv_simple(&self) -> String {
        let sep = ",";
        format!(
            "{}\n{}",
            self.headers.join(sep),
            (0..self.length)
                .map(|i| self.row(i).join(sep))
                .collect::<Vec<String>>()
                .join("\n")
        )
    }
}

pub fn test_point_data() -> Result<PointData> {
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

pub fn test_point_data_circle(n: usize) -> Result<PointData> {
    let mut point_data = PointData::new();
    point_data
        .with_data_column("a")
        .with_data_column("x")
        .with_data_column("y")
        .with_aux_column("label")
        .allocate(n);

    for i in 0..n {
        let a = 2.0 * PI * (i as f64) / (n as f64);
        let x = a.sin();
        let y = a.cos();
        point_data
            .set_data("a", i, a)
            .set_data("x", i, x)
            .set_data("y", i, y)
            .set_aux("label", i, format!("{}/{}", i + 1, n));
    }
    Ok(point_data)
}
