extern crate num_cpus;
mod individual;

use individual::*;

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use rand::seq::SliceRandom;
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

use std::fs::File;
use std::io::{BufRead, BufReader};

const CROSSOVER: f32 = 0.8;
const MUTATION: f32 = 0.03;
const ITERATIONS: u32 = 100;
const POPULATION: usize = 1000;
const BATCH_SIZE: usize = 20;

pub struct Problem {
    dimensions: (u32, u32),
    pin_locations: Vec<((u32, u32), (u32, u32))>,
    population: Vec<(Individual, f32)>,
    random: Option<u64>,
}

type FnType = fn(problem: &mut Problem, batch_size: usize, random: &mut StdRng) -> Individual;

pub fn tournament_selection(
    problem: &mut Problem,
    batch_size: usize,
    random: &mut StdRng,
) -> Individual {
    let mut tournament_batch: Vec<(Individual, f32)> = problem
        .population
        .choose_multiple(random, batch_size)
        .cloned()
        .collect();
    // tournament_batch
    //     .choose_weighted_mut(random, |item| 1. / item.evaluate())
    //     .unwrap()
    //     .clone()
    //
    // tournament_batch.iter().min_by(|item1, item2| (item1.evaluate().partial_cmp(&item2.evaluate())).unwrap()).unwrap().clone()
    tournament_batch
        .iter()
        .min_by(|item1, item2| item1.1.partial_cmp(&item2.1).unwrap())
        .unwrap()
        .0
        .clone()
}

pub fn roulette_selection() {}

impl Problem {
    fn init_population(&mut self, size: usize) {
        let bar = ProgressBar::new(size as u64);
        let sty = ProgressStyle::default_bar()
            .template("{prefix:.cyan}   [{bar:40.white}] {pos:>7}/{len:7} [{elapsed_precise}]")
            .progress_chars("=> ");
        bar.set_style(sty);
        bar.set_prefix("Generating population #");

        for i in 0..size {
            let individual: Individual = generate_individual(
                self.dimensions.clone(),
                self.pin_locations.clone(),
                match self.random {
                    Some(seed) => Some(seed + i as u64),
                    None => None,
                },
            );
            let points = individual.evaluate();
            self.population.push((individual, points));
            bar.inc(1);
        }
        bar.finish();
    }

    pub fn genetic_search(&mut self, selector: FnType, cpus: Option<usize>, seed: Option<u64>) -> (Individual, f32) {
        let mut random = match seed {
            Some(seed) => StdRng::seed_from_u64(seed),
            None => StdRng::from_entropy(),
        };

        self.init_population(POPULATION);

        let bar = ProgressBar::new(ITERATIONS as u64);
        let sty = ProgressStyle::default_bar()
            .template("{prefix:.cyan}   [{bar:40.white}] {pos:>7}/{len:7} [{elapsed_precise}]")
            .progress_chars("=> ");
        bar.set_style(sty);
        bar.set_prefix("Iterating #");

        for _ in 0..ITERATIONS {
            let mut new_population: Vec<(Individual, f32)> = vec![];
            while new_population.len() < POPULATION {
                let mut i1 = selector(self, BATCH_SIZE, &mut random);
                if random.gen::<f32>() < CROSSOVER {
                    let mut i2 = selector(self, BATCH_SIZE, &mut random);
                    i1.crossover(&mut i2, random.gen());
                }
                i1.mutate(&mut random, MUTATION);
                let points = i1.evaluate();
                new_population.push((i1, points));
            }
            self.population = new_population;
            bar.inc(1);
        }


       self.population.iter().min_by(|item1, item2| (item1.1.partial_cmp(&item2.1)).unwrap()).unwrap().clone()
    }

    pub fn random_search(&mut self, iterations: u64, cpus: Option<usize>) -> (Individual, u128) {
        let now = Instant::now();

        let cpus = cpus.unwrap_or(num_cpus::get() / 2);
        // let mut stats = vec![vec![0; problem.dimensions.1 as usize]; problem.dimensions.0 as usize];
        let safe_best_individuals: Arc<Mutex<Vec<Individual>>> = Arc::new(Mutex::new(vec![]));

        let m = MultiProgress::new();
        let sty = ProgressStyle::default_bar()
            .template("{prefix:.cyan}   [{bar:40.white}] {pos:>7}/{len:7} [{elapsed_precise}]")
            .progress_chars("=> ");

        if self.random.is_some() {
            let pb = m.add(ProgressBar::new(iterations / cpus as u64));
            let mut best: Individual = Individual::new();
            let mut best_value = std::f32::INFINITY;
            for i in 0..iterations {
                let temp = generate_individual(
                    self.dimensions.clone(),
                    self.pin_locations.clone(),
                    Some(self.random.unwrap() + i as u64),
                );
                let temp_value = temp.evaluate();
                if temp_value < best_value {
                    best_value = temp_value;
                    best = temp;
                };
                pb.inc(1);
            }
            safe_best_individuals.lock().unwrap().push(best);
        } else {
            let handles = (0..cpus)
                .into_iter()
                .map(|x| {
                    let dimensions = self.dimensions;
                    let pin_locations = self.pin_locations.clone();
                    let best_individuals = Arc::clone(&safe_best_individuals);
                    let pb = m.add(ProgressBar::new(iterations / cpus as u64));
                    pb.set_prefix(&format!("Thread #{}", x));
                    pb.set_style(sty.clone());
                    thread::spawn(move || {
                        let mut best: Individual = Individual::new();
                        let mut best_value = std::f32::INFINITY;
                        for _ in 0..iterations / cpus as u64 {
                            let temp = generate_individual(
                                dimensions.clone(),
                                pin_locations.clone(),
                                None,
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

            for thread in handles {
                thread.join().unwrap();
            }
        }

        let safe_best_individuals = safe_best_individuals.lock().unwrap();
        let mut best = &safe_best_individuals[0];

        for i in 1..safe_best_individuals.len() {
            if safe_best_individuals[i].evaluate() > best.evaluate() {
                best = &safe_best_individuals[i];
            }
        }

        (best.clone(), now.elapsed().as_millis())
    }
}

pub fn load_problem(problem_name: &str, seed: Option<u64>) -> Problem {
    fn parse_pairs(mut pairs: Vec<&str>) -> Vec<(u32, u32)> {
        let mut parsed_pairs: Vec<(u32, u32)> = Vec::new();

        while pairs.len() > 1 {
            let strings: (&str, &str) = (pairs.pop().unwrap(), pairs.pop().unwrap());
            let pair: (u32, u32) = (
                strings.1.trim().parse().unwrap(),
                strings.0.trim().parse().unwrap(),
            );
            parsed_pairs.push(pair);
        }

        return parsed_pairs;
    }

    let file = File::open(problem_name).expect("Failed to open file");
    let reader = BufReader::new(file);
    let mut dimensions = (0, 0);
    let mut pin_locations: Vec<((u32, u32), (u32, u32))> = Vec::new();

    for (index, line) in reader.lines().enumerate() {
        let line = line.unwrap();
        let unparsed_dimensions: Vec<_> = line.trim().split(';').collect();

        if index == 0 && unparsed_dimensions.len() == 2 {
            dimensions = parse_pairs(unparsed_dimensions)[0];
        } else if unparsed_dimensions.len() == 4 {
            let parsed_pairs = parse_pairs(unparsed_dimensions);
            pin_locations.push((parsed_pairs[1], parsed_pairs[0]));
        } else {
            panic!("Wrong test data format");
        }
    }

    Problem {
        dimensions,
        pin_locations,
        population: vec![],
        random: seed,
    }
}
