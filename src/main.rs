extern crate ndarray;
extern crate ndarray_npy;
use ordered_float::OrderedFloat;
use ndarray::Array2;
use ndarray::Array1;
use ndarray_npy::read_npy;
use std::time::Instant;

use std::collections::BTreeMap;
trait QualityFunction {
    fn evaluate(&self, subgroup: &Vec<bool>) -> (f32, f32, u32);
}

struct Task {
    search_space : Vec<Vec<bool>>,
    depth : usize,
    result_size : usize,
    min_quality : OrderedFloat<f32>,
}

fn main(){
    let arr: Array2<bool> = match read_npy("E:/tmp/arr.npy"){
        Err(e) => {println!("{:?}",e); return ()},
        Ok(f) => f,
        };

    let target_values_arr : Array1<f32> = match read_npy("E:/tmp/target.npy"){
            Err(e) => {println!("{:?}",e); return ()},
            Ok(f) => f,
            };
    let mut target_values : Vec<f32> = Vec::new();
    for x in target_values_arr.iter(){
        target_values.push(*x);
    }
    let mut search_space : Vec<Vec<bool>> = Vec::new();

    for axis in arr.axis_iter(ndarray::Axis(0)) {
        let mut new_vec : Vec<bool> = Vec::new();
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
    let min_quality : OrderedFloat<f32> = OrderedFloat(0.0);
    let task = Task{ search_space: search_space, depth: depth, result_size: 10, min_quality: min_quality };
    let mut result : BTreeMap<OrderedFloat<f32>, Vec<usize>> = BTreeMap::new();
    let base_sg = vec![true; dataset_size];
    let prefix : Vec<usize> = Vec::new();
    let now = Instant::now();
    let dataset_mean = StandardQFNumeric::mean(&target_values);
    let qf = StandardQFNumeric{target_values: target_values, dataset_mean: dataset_mean ,a:0.5};
    recurse(&prefix, &base_sg, &qf, &task, &mut result);
    println!("time = {}", now.elapsed().as_millis());
    println!("{:?}", result);
    println!("dataset mean: {:?}", dataset_mean);

    //let z = recurse();
    //println!("{:?}", z);
}

struct StandardQFNumeric {
    target_values : Vec<f32>,
    dataset_mean : f32,
    a : f32,
}

impl QualityFunction for StandardQFNumeric {
    fn evaluate(&self, subgroup: & Vec<bool>) -> (f32, f32, u32) {
        let mut cumsum = 0.0;
        let mut count = 0;
        let mut max : f32 = 10.0;
        max=max.powf( 10.0);
        let mut quality = 0.0;
        for i in 0..subgroup.len() {
        //for (is_in_subgroup, value)  in subgroup.iter().zip(self.target_values.iter()) {
            if subgroup[i] {
                cumsum += self.target_values[i];
                count += 1;
                quality = (count as f32).powf(self.a) * (cumsum/(count as f32) - self.dataset_mean);
                if quality > max {
                    max = quality;
                }
            }
        }
        return (quality, max, count);
    }
}

impl StandardQFNumeric {
    fn mean(target_values : & Vec<f32>) -> f32 {
        let mut cumsum : f32 = 0.0;
        for value  in target_values.iter() {
            cumsum += value;
        }
        return cumsum / (target_values.len() as f32)
    }
}

fn recurse(prefix : & Vec<usize> ,
            sg : & Vec<bool> ,
            qf : & impl QualityFunction,
            task :  & Task,
            result: &mut BTreeMap<OrderedFloat<f32>, Vec<usize>>) {
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
            for i in lastp1(prefix) .. task.search_space.len() {
                let mut new_prefix = prefix.clone();
                new_prefix.push(i);
                let new_sg = logical_and(sg, &task.search_space[i]);
                recurse(& new_prefix, & new_sg, qf, task, result);
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

fn logical_and(v1 : & Vec<bool>, v2: & Vec<bool>) -> Vec<bool> {
    assert_eq!(v1.len(),v2.len());
    let mut new_vec : Vec<bool> = vec![false; v1.len()];
    //let mut val = false;
    /*for (target, (b1, b2)) in new_vec.iter_mut().zip(v1.iter().zip(v2.iter())) {
        let val = *b1 && *b2;
        if val {
            *target = val;
        }
    }*/
    for i in 0..v1.len()
    {
        let val = v1[i] && v2[i];
        if val {
            new_vec[i] = val;
        }
    }

    //return  v1.iter().zip(v2.iter()).map(|(x, y)| *x && *y).to_vec()
    return new_vec
}

