extern crate ndarray;
extern crate ndarray_npy;
use ordered_float::OrderedFloat;
use ndarray::Array2;
use ndarray::Array1;
use ndarray_npy::read_npy;
use std::time::Instant;

use std::collections::BTreeMap;

use bit_vec::BitVec;
trait QualityFunction {
    fn evaluate(&self, subgroup: &Vec<usize>) -> (f64, f64, u32);
}

struct Task {
    search_space : Vec<BitVec>,
    depth : usize,
    result_size : usize,
    min_quality : OrderedFloat<f64>,
}
use std::env;
fn main(){
    let args: Vec<String> = env::args().collect();
    //let query = &args[1];
    //let filename = &args[2];

    let arr: Array2<bool> = match read_npy(&args[1]){
        Err(e) => {println!("{:?}", e); return ()},
        Ok(f) => f,
        };

    let target_values_arr : Array1<f64> = match read_npy(&args[2]){
            Err(e) => {println!("{:?}", e); return ()},
            Ok(f) => f,
            };
    let mut target_values : Vec<f64> = Vec::new();
    for x in target_values_arr.iter(){
        target_values.push(*x);
    }
    //println!("{:?}",target_values);
    let mut search_space : Vec<BitVec> = Vec::new();

    for axis in arr.axis_iter(ndarray::Axis(0)) {
        let mut new_vec : BitVec = BitVec::new();
        for x in axis.iter() {
            new_vec.push(*x);
        }
        search_space.push(new_vec);
    }
    let num_selectors = search_space.len();
    println!("num selectors: {:?}", num_selectors);

    let dataset_size = match search_space.iter().next() {
        None =>  0,
        Some(v) => v.len()
    };
    println!("dataset size: {:?}", dataset_size);
    let depth = 5;
    let min_quality : OrderedFloat<f64> = OrderedFloat(0.0);
    let task = Task{ search_space: search_space, depth: depth, result_size: 10, min_quality: min_quality };
    let mut result : BTreeMap<OrderedFloat<f64>, Vec<usize>> = BTreeMap::new();
    let mut base_sg  : Vec<usize> = Vec::new();//BitVec::from_elem(dataset_size, true);
    for i in 0..dataset_size {
        base_sg.push(i)
    }
    let prefix : Vec<usize> = Vec::new();
    let now = Instant::now();
    let dataset_mean = StandardQFNumeric::mean(&target_values);
    let qf = StandardQFNumeric{target_values: target_values, dataset_mean: dataset_mean ,a:1.0};
    recurse(&prefix, &base_sg, &qf, &task, &mut result);
    println!("time = {}", now.elapsed().as_millis());
    println!("{:?}", result);
    println!("dataset mean: {:?}", dataset_mean);

    //let z = recurse();
    //println!("{:?}", z);
}

struct StandardQFNumeric {
    target_values : Vec<f64>,
    dataset_mean : f64,
    a : f64,
}

impl QualityFunction for StandardQFNumeric {
    fn evaluate(&self, subgroup: & Vec<usize>) -> (f64, f64, u32) {
        let mut cumsum = 0.0;
        let mut count = 0;
        let mut max : f64 = - (10.0 as f64) .powf( 10.0);
        let mut quality : f64 = 0.0;
        //assert_eq!(subgroup.len(),self.target_values.len());
        for i in subgroup {
            cumsum += self.target_values[*i];
            count += 1;
            quality = (count as f64).powf(self.a) * (cumsum/(count as f64) - self.dataset_mean);
            if quality > max {
                max = quality;
            }
        }

        return (quality, max, count);
    }
}

impl StandardQFNumeric {
    fn mean(target_values : & Vec<f64>) -> f64 {
        let mut cumsum : f64 = 0.0;
        for value  in target_values {
            cumsum += value;
        }
        return cumsum / (target_values.len() as f64)
    }
}

fn recurse(prefix : & Vec<usize> ,
            sg : & Vec<usize> ,
            qf : & impl QualityFunction,
            task :  & Task,
            result: &mut BTreeMap<OrderedFloat<f64>, Vec<usize>>) {
    let (quality, optimistic_estimate, size) = qf.evaluate(sg);
    if size == 0 { 
        return}
    let ord_quality = OrderedFloat(quality);
    let ord_estimate = OrderedFloat(optimistic_estimate);
    let min_quality = match result.keys().next() {
        None =>  task.min_quality,
        Some(qual) => *qual
    };
    if ord_quality > min_quality {
        if result.len() >= task.result_size {
            result.remove(&min_quality);
        }
        result.insert(OrderedFloat(quality), prefix.to_vec());
    }
    if prefix.len() < task.depth {
        if ord_estimate > min_quality {
            let mut new_prefix = prefix.clone();
            let mut new_sg = sg.clone();
            for i in lastp1(prefix) .. task.search_space.len() {
                new_prefix.push(i);
                
                //new_sg.set_all();
                //new_sg.intersect(&sg);
                //new_sg.intersect(&task.search_space[i]);
                intersect(&mut new_sg, sg, &task.search_space[i]);
                recurse(& new_prefix, & new_sg, qf, task, result);
                new_prefix.pop();
            }
        }
    }
}

fn lastp1(v : & Vec<usize>) -> usize {
    match v.last() {
        None => 0,
        Some(value) => *value + 1
    }
}

fn intersect(target : &mut Vec<usize>, v : &Vec<usize>, u : &BitVec) {
    target.clear();
    for index in v.iter() {
        let t = u[*index];
        if t {
            target.push(*index);
        }
    }

}