use std::thread;
use std::sync::{Arc, Mutex};
use std::time::Instant;

mod individual;
mod problem;

const ITERATIONS: u32 = 1000000;

fn main() {
    let now = Instant::now();

    // let mut stats = vec![vec![0; problem.dimensions.1 as usize]; problem.dimensions.0 as usize];
    let safe_best_individuals: Arc<Mutex<Vec<individual::Individual>>> = Arc::new(Mutex::new(vec![]));

    let handles = (0..4)
        .into_iter()
        .map(|_| {
            println!("Starting Thread");
            let best_individuals = Arc::clone(&safe_best_individuals);
            thread::spawn(move || {
                let problem = problem::load_problem("test_data/zad3.txt");
                let mut best: individual::Individual = individual::generate_empty_individual();
                let mut best_value = std::f32::INFINITY;
                for _ in 0..ITERATIONS/4 {
                    let temp = individual::generate_individual(
                        problem.dimensions,
                        problem.pin_locations.clone(),
                    );
                    let temp_value = temp.evaluate();
                    if temp_value < best_value {
                        best_value = temp_value;
                        best = temp;
                    }
                }
                best_individuals.lock().unwrap().push(best);
            })
        })
        .collect::<Vec<thread::JoinHandle<_>>>();

        for thread in handles {
            thread.join().unwrap();
        }

        let safe_best_individuals = safe_best_individuals.lock().unwrap();
        let mut best = &safe_best_individuals[0];

        for i in 1..4 {
            if safe_best_individuals[i].evaluate() > best.evaluate() {
                best = &safe_best_individuals[i];
            }
        }

        println!("Best evaluate: {}", best.evaluate());
        println!("Conflict at: {} points", best.collisions);
        println!("Compute time: {}", now.elapsed().as_millis());
        println!("{}", best);
}
