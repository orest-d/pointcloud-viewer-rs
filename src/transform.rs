trait Transform{
    fn new()->Self;
    fn calibrate(&mut self, values:&[f64])->&mut Self;
    fn transform(&self, value:f64)->Option<f64>;
    fn inverse(&self, value:f64)->Option<f64>;
}

struct Trivial;

impl Transform for Trivial{
    fn new()->Self{Trivial}
    fn calibrate(&mut self, _values:&[f64])->&mut Self{self}
    fn transform(&self, value:f64)->Option<f64>{
        Some(value)
    }
    fn inverse(&self, value:f64)->Option<f64>{
        Some(value)
    }
}

struct Normalize{
    minimum:f64,
    delta:f64,
}

impl Transform for Normalize{
    fn new()->Self{Normalize{minimum:0.0,delta:1.0}}
    fn calibrate(&mut self, values:&[f64])->&mut Self{
        if values.len()>0{
            self.minimum = values[0];
            let mut maximum = values[0];
            for &x in values{
                if x<self.minimum{
                    self.minimum =x;
                }
                if x>maximum{
                    maximum =x;
                }
            }
            self.delta = maximum-self.minimum;
            if self.delta== 0.0{
                self.delta=1.0;
            }
        }
        else{
            self.minimum=0.0;
            self.delta=1.0;
        }
        self
    }
    fn transform(&self, value:f64)->Option<f64>{
        Some((value-self.minimum)/self.delta)
    }
    fn inverse(&self, value:f64)->Option<f64>{
        Some(value*self.delta+self.minimum)
    }
}

struct Logarithmic{
    base:f64
}

impl Transform for Logarithmic{
    fn new()->Self{Logarithmic{base:std::f64::consts::E}}
    fn calibrate(&mut self, _values:&[f64])->&mut Self{
        self
    }
    fn transform(&self, value:f64)->Option<f64>{
        if value>0.0{
            Some(value.log(self.base))
        }
        else{
            None
        }
    }
    fn inverse(&self, value:f64)->Option<f64>{
        Some(self.base.powf(value))
    }
}

struct ComposedTransform<A:Transform,B:Transform>{
    first: Box<A>,
    second: Box<B>
}

impl<A:Transform,B:Transform> Transform for ComposedTransform<A,B>{
    fn new()->Self{
        ComposedTransform{
            first:Box::new(A::new()),
            second:Box::new(B::new())
        }
    }
    fn calibrate(&mut self,values:&[f64])->&mut Self{
        self.first.calibrate(values);
        let intermediate = values.iter().flat_map(|&x| self.first.transform(x)).collect::<Vec<_>>();
        self.second.calibrate(&intermediate);
        self
    }
    fn transform(&self, value:f64)->Option<f64>{
        if let Some(x) = self.first.transform(value){
            self.second.transform(x)
        }
        else{
            None
        }
    }
    fn inverse(&self, value:f64)->Option<f64>{
        if let Some(x)=self.second.inverse(value){
            self.first.inverse(x)
        }
        else{
            None
        }
    }
}

type NormalizedLogarithmic = ComposedTransform<Logarithmic, Normalize>;

struct Quantile{
    size:usize,
    values:Vec<f64>,
    quantiles: Vec<f64>
}

impl Quantile{
    fn with_size(n: usize)->Self{
        Quantile{
            size:n,
            values:Vec::with_capacity(n),
            quantiles:Vec::with_capacity(n)
        }
    }
    fn general_transform(value:f64, v:&[f64], w:&[f64])->f64{
        let mut low=0;
        let mut high=v.len()-1;
        loop{
            if value<=v[low]{
                return w[low];
            }
            if value>=v[high]{
                return w[high];
            }
            if low==high{
                return w[low];
            }
            if low+1==high{
                let delta = v[high]-v[low];
                if delta==0.0{
                    return w[low];
                }
                let factor = (value-v[low])/delta;
                
                return w[low]+factor*(w[high]-w[low]);
            }
            let next = (low + high)/2;
            
            if v[next]>=value{
                high=next;
            }
            else{
                low=next;
            }
        }
    }
}

impl Transform for Quantile{
    fn new() -> Self{Quantile::with_size(100)}
    fn calibrate(&mut self, values:&[f64])->&mut Self{
        let mut buffer = values.iter().filter(|x| x.is_finite()).map(|&x| x).collect::<Vec<f64>>();
        buffer.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let n = buffer.len();
        let step = 1.max(n/self.size);
        let mut index=0;
        self.values.clear();
        self.quantiles.clear();
        while index<buffer.len()-1{
            self.values.push(buffer[index]);
            self.quantiles.push((index as f64)/((n-1) as f64));
            index+=step;
        }
        self.values.push(buffer[n-1]);
        self.quantiles.push(1.0);
        self
    }
    fn transform(&self, value:f64)->Option<f64>{
        Some(Quantile::general_transform(value, &self.values, &self.quantiles))
        /*
        if self.values.len()>1 && self.values.len() == self.quantiles.len(){
            Some(Quantile::general_transform(value, &self.values, &self.quantiles))
        }
        else{
            None
        }
        */
    }
    fn inverse(&self, value:f64)->Option<f64>{
        Some(Quantile::general_transform(value, &self.quantiles, &self.values))
        /*
        if self.values.len()>1 && self.values.len() == self.quantiles.len(){
            Some(Quantile::general_transform(value, &self.quantiles, &self.values))
        }
        else{
            None
        }
        */
    }
}
#[cfg(test)]
mod test{
    use super::*;
    #[test]
    fn test_normalize(){
        let data = [1.0, 2.0, 3.0];
        let mut transform = Normalize::new();
        transform.calibrate(&data);
        assert_eq!(transform.transform(1.0), Some(0.0));
        assert_eq!(transform.transform(2.0), Some(0.5));
        assert_eq!(transform.inverse(0.0), Some(1.0));
        assert_eq!(transform.inverse(0.5), Some(2.0));
    }

    #[test]
    fn test_logarithmic(){
        let transform = Logarithmic::new();
        assert_eq!(transform.transform(0.0), None);
        assert!((transform.inverse(transform.transform(123.0).unwrap()).unwrap()-123.0).abs()<1e-5);
    }

    #[test]
    fn test_normalized_logarithmic(){
        let data = [0.0, 1.0, 2.0, 3.0];
        let mut transform = NormalizedLogarithmic::new();
        transform.calibrate(&data);

        assert_eq!(transform.transform(0.0), None);
        assert_eq!(transform.transform(1.0), Some(0.0));
        assert_eq!(transform.transform(3.0), Some(1.0));
        assert!((transform.inverse(transform.transform(123.0).unwrap()).unwrap()-123.0).abs()<1e-5);
    }
    #[test]
    fn test_quantile(){
        let data = [10.0, 1.0, 2.0];
        let mut transform = Quantile::new();
        transform.calibrate(&data);
        assert_eq!(transform.transform(0.0), Some(0.0));
        assert_eq!(transform.transform(1.0), Some(0.0));
        assert_eq!(transform.transform(1.5), Some(0.25));
        assert_eq!(transform.transform(10.0), Some(1.0));
        assert!((transform.inverse(transform.transform(3.0).unwrap()).unwrap()-3.0).abs()<1e-5);
    }
}    
