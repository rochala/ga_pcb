extern crate num_cpus;

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

mod individual;
mod problem;

const ITERATIONS: u64 = 10000000;

fn main() {
    let now = Instant::now();

    let cpus = num_cpus::get();
    // let mut stats = vec![vec![0; problem.dimensions.1 as usize]; problem.dimensions.0 as usize];
    let safe_best_individuals: Arc<Mutex<Vec<individual::Individual>>> =
        Arc::new(Mutex::new(vec![]));

    let m = MultiProgress::new();
    let sty = ProgressStyle::default_bar()
        .template("{prefix:.cyan}   [{bar:40.white}] {pos:>7}/{len:7} [{elapsed_precise}]")
        .progress_chars("=> ");


    let handles = (0..cpus)
        .into_iter()
        .map(|x| {
            let best_individuals = Arc::clone(&safe_best_individuals);
            let pb = m.add(ProgressBar::new(ITERATIONS / cpus as u64));
            pb.set_prefix(&format!("Thread #{}", x));
            pb.set_style(sty.clone());
            thread::spawn(move || {
                let problem = problem::load_problem("test_data/zad1.txt");
                let mut best: individual::Individual = individual::Individual::new();
                let mut best_value = std::f32::INFINITY;
                for _ in 0..ITERATIONS / cpus as u64 {
                    let temp = individual::generate_individual(
                        problem.dimensions,
                        problem.pin_locations.clone(),
                    );
                    let temp_value = temp.evaluate();
                    if temp_value < best_value {
                        best_value = temp_value;
                        best = temp;
                    };
                    pb.inc(1);
                }
                best_individuals.lock().unwrap().push(best);
            })
        })
        .collect::<Vec<thread::JoinHandle<_>>>();

    m.join_and_clear().unwrap();
    println!("done");


    for thread in handles {
        thread.join().unwrap();
    }


    let safe_best_individuals = safe_best_individuals.lock().unwrap();
    let mut best = &safe_best_individuals[0];

    for i in 1..cpus {
        if safe_best_individuals[i].evaluate() > best.evaluate() {
            best = &safe_best_individuals[i];
        }
    }

    println!("Best evaluate: {}", best.evaluate());
    println!("Conflict at: {} points", best.collisions);
    println!("Compute time: {}", now.elapsed().as_millis());
    println!("{}", best);
}
