use std::time::{Instant};

mod individual;
mod problem;


fn main() {
    let problem = problem::load_problem("test_data/zad3.txt");
    let now = Instant::now();

    let mut best: individual::Individual =
        individual::generate_individual(problem.dimensions, problem.pin_locations.clone());
    for _ in 0..100000 {
        let temp =
            individual::generate_individual(problem.dimensions, problem.pin_locations.clone());
        if temp.evaluate() < best.evaluate() {
            best = temp;
        }
    }

    println!("{}", best.evaluate());
    println!("{:?}", best.point_map);
    println!("{:?}", best.connections);
    println!("{}", now.elapsed().as_millis());
}

