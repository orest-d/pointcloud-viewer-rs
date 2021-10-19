extern crate serde;
extern crate serde_json;
extern crate serde_yaml;

use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct NumericStatistics {
    sum_of_values: f64,
    sum_of_values2: f64,
    sum_of_values3: f64,
    sum_of_values4: f64,
    sum_of_weights: f64,
    minimum: Option<f64>,
    maximum: Option<f64>,
}

impl NumericStatistics {
    pub fn new() -> NumericStatistics {
        NumericStatistics {
            sum_of_values: 0.0,
            sum_of_values2: 0.0,
            sum_of_values3: 0.0,
            sum_of_values4: 0.0,
            sum_of_weights: 0.0,
            minimum: None,
            maximum: None,
        }
    }

    pub fn create_empty(&self) -> NumericStatistics {
        NumericStatistics::new()
    }

    pub fn add_weighted(&mut self, x: &[f64], weight: &[f64]) {
        for (xi, wi) in x.iter().zip(weight.iter()) {
            let wx = wi * xi;
            let wx2 = wx * xi;
            let wx3 = wx2 * xi;
            let wx4 = wx3 * xi;

            self.sum_of_values += wx;
            self.sum_of_values2 += wx2;
            self.sum_of_values3 += wx3;
            self.sum_of_values4 += wx4;
            self.sum_of_weights += wi;
            self.minimum = if let Some(mx) = self.minimum {
                Some(mx.min(*xi))
            } else {
                Some(*xi)
            };
            self.maximum = if let Some(mx) = self.maximum {
                Some(mx.max(*xi))
            } else {
                Some(*xi)
            };
        }
    }

    pub fn add(&mut self, x: &[f64]) {
        for xi in x.iter() {
            let wx = xi;
            let wx2 = wx * xi;
            let wx3 = wx2 * xi;
            let wx4 = wx3 * xi;

            self.sum_of_values += wx;
            self.sum_of_values2 += wx2;
            self.sum_of_values3 += wx3;
            self.sum_of_values4 += wx4;
            self.sum_of_weights += 1.;
            self.minimum = if let Some(mx) = self.minimum {
                Some(mx.min(*xi))
            } else {
                Some(*xi)
            };
            self.maximum = if let Some(mx) = self.maximum {
                Some(mx.max(*xi))
            } else {
                Some(*xi)
            };
        }
    }

    pub fn add_analyzer(&mut self, analyzer: &Self) {
        self.sum_of_values += analyzer.sum_of_values;
        self.sum_of_values2 += analyzer.sum_of_values2;
        self.sum_of_values3 += analyzer.sum_of_values3;
        self.sum_of_values4 += analyzer.sum_of_values4;
        self.sum_of_weights += analyzer.sum_of_weights;

        if let Some(x) = self.minimum {
            if let Some(y) = analyzer.minimum {
                self.minimum = Some(x.min(y))
            }
        } else {
            self.minimum = analyzer.minimum;
        }

        if let Some(x) = self.maximum {
            if let Some(y) = analyzer.maximum {
                self.maximum = Some(x.max(y))
            }
        } else {
            self.maximum = analyzer.maximum;
        }
    }
    pub fn mean(&self) -> Option<f64> {
        if self.sum_of_weights == 0.0 {
            None
        } else {
            Some(self.sum_of_values / self.sum_of_weights)
        }
    }

    pub fn variance(&self) -> Option<f64> {
        if self.sum_of_weights == 0.0 {
            None
        } else {
            let mean = self.sum_of_values / self.sum_of_weights;
            Some(self.sum_of_values2 / self.sum_of_weights - mean * mean)
        }
    }

    pub fn stddev(&self) -> Option<f64> {
        self.variance().map(|x| x.sqrt())
    }
    /// Skewness: https://en.wikipedia.org/wiki/Skewness
    pub fn skewness(&self) -> Option<f64> {
        if self.sum_of_weights == 0.0 {
            None
        } else {
            self.stddev().map(|stddev| {
                let mean = self.sum_of_values / self.sum_of_weights;
                let mean_cube = self.sum_of_values3 / self.sum_of_weights;
                (mean_cube - 3.0 * mean * stddev * stddev - mean * mean * mean)
                    / (stddev * stddev * stddev)
            })
        }
    }

    /// Calculates Fisher's kurtosis
    pub fn kurtosis(&self) -> Option<f64> {
        if self.sum_of_weights == 0.0 {
            None
        } else {
            self.variance().map(|variance| {
                let mean = self.sum_of_values / self.sum_of_weights;
                let mean2 = mean * mean;
                let x2 = self.sum_of_values2 / self.sum_of_weights;
                let x3 = self.sum_of_values3 / self.sum_of_weights;
                let x4 = self.sum_of_values4 / self.sum_of_weights;

                (x4 - 3. * mean2 * mean2 - 4. * mean * x3 + 6. * mean2 * x2) / (variance * variance)
                    - 3.
            })
        }
    }
}
