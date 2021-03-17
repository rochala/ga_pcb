extern crate num_cpus;

use self::Direction::*;
use colored::*;

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

use rand::{rngs::StdRng, Rng, SeedableRng};
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};

const COLLISION_FACTOR: f32 = 0.1;
const SIDE_FACTOR: f32 = 0.;
const STEP_BONUS: f32 = 0.5;
const BASE: f32 = 1.;

const WEIGHTS: (f32, f32, f32) = (10., 0.2, 0.01);

#[derive(Clone, Debug)]
pub struct Individual {
    connections: Vec<Connection>,
    pub point_map: Vec<Vec<bool>>,
    pub collisions: u64,
}

pub struct Problem {
    dimensions: (u32, u32),
    pin_locations: Vec<((u32, u32), (u32, u32))>,
    population: Vec<Individual>,
    random: Option<u64>,
}

impl Problem {
    pub fn init_population(&mut self, size: usize) {
        for i in 0..size {
            self.population.push(generate_individual(
                self.dimensions.clone(),
                self.pin_locations.clone(),
                Some(self.random.unwrap() + i as u64),
            ))
        }
    }

    pub fn genetic_search(&mut self, selector: &dyn Fn(Vec<Individual>) -> Vec<Individual>, cpus: Option<usize>) {
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

pub fn generate_individual(
    dimensions: (u32, u32),
    pin_locations: Vec<((u32, u32), (u32, u32))>,
    seed: Option<u64>,
) -> Individual {
    let mut individual = Individual {
        connections: Vec::new(),
        point_map: vec![vec![false; dimensions.1 as usize]; dimensions.0 as usize],
        collisions: 0,
    };

    for pin_pair in &pin_locations {
        individual.mark_point(pin_pair.0, true);
        individual.mark_point(pin_pair.1, true);
    }

    // pin_locations.shuffle(&mut thread_rng());

    for pin_pair in &pin_locations {
        let connection = individual.random_walk(*pin_pair, seed);
        individual.connections.push(connection);
    }

    individual
}

#[derive(Copy, Clone, Debug)]
enum Direction {
    North,
    South,
    East,
    West,
}

impl Direction {
    fn iterator() -> impl Iterator<Item = Direction> {
        [North, South, East, West].iter().copied()
    }
}

#[derive(Debug, Clone)]
struct Segment {
    length: u32,
    direction: Direction,
}

#[derive(Debug, Clone)]
struct Connection {
    start: (u32, u32),
    end: (u32, u32),
    segments: Vec<Segment>,
}

impl Individual {
    pub fn new() -> Individual {
        Individual {
            connections: vec![],
            point_map: vec![],
            collisions: 0,
        }
    }

    fn find_neighbors(&self, point: (u32, u32)) -> [f32; 4] {
        // Up, DOWN, RIGHT, LEFT
        let mut neighbors: [f32; 4] = [1.0; 4];

        if point.0 <= 0 {
            neighbors[0] = SIDE_FACTOR;
        } else if self.point_map[point.0 as usize - 1][point.1 as usize] {
            neighbors[0] = COLLISION_FACTOR;
        }
        if point.0 >= self.point_map.len() as u32 - 1 {
            neighbors[1] = SIDE_FACTOR;
        } else if self.point_map[point.0 as usize + 1][point.1 as usize] {
            neighbors[1] = COLLISION_FACTOR;
        }
        if point.1 >= self.point_map[0].len() as u32 - 1 {
            neighbors[2] = SIDE_FACTOR;
        } else if self.point_map[point.0 as usize][point.1 as usize + 1] {
            neighbors[2] = COLLISION_FACTOR;
        }
        if point.1 <= 0 {
            neighbors[3] = SIDE_FACTOR;
        } else if self.point_map[point.0 as usize][point.1 as usize - 1] {
            neighbors[3] = COLLISION_FACTOR;
        }

        neighbors
    }

    fn random_walk(&mut self, pins: ((u32, u32), (u32, u32)), seed: Option<u64>) -> Connection {
        let mut connection = Connection {
            start: pins.0,
            end: pins.1,
            segments: Vec::new(),
        };

        self.mark_point(pins.1, false);

        let mut actual_point = pins.0;

        let mut next_direction;

        let mut probabilities = self.find_neighbors(connection.start);

        let mut prob_sum = 0.;
        for i in 0..probabilities.len() {
            prob_sum += probabilities[i]
        }

        for i in 0..probabilities.len() {
            probabilities[i] /= prob_sum;
        }

        let mut random = match seed {
            Some(seed) => StdRng::seed_from_u64(seed),
            None => StdRng::from_entropy(),
        };

        let roll: f32 = random.gen();

        if roll <= probabilities[0] {
            next_direction = Some(North);
        } else if roll <= probabilities[0] + probabilities[1] {
            next_direction = Some(South);
        } else if roll <= probabilities[0] + probabilities[1] + probabilities[2] {
            next_direction = Some(East);
        } else {
            next_direction = Some(West);
        }

        while next_direction.is_some() {
            let (segment, dir_holder) = connection.create_segment(
                next_direction.unwrap(),
                self,
                &mut actual_point,
                &mut random,
            );
            next_direction = dir_holder;
            connection.segments.push(segment);
        }

        self.mark_point(pins.1, true);

        connection
    }

    fn mark_point(&mut self, point: (u32, u32), val: bool) {
        if self.point_map[point.0 as usize][point.1 as usize] && val == true {
            self.collisions += 1;
        };
        self.point_map[point.0 as usize][point.1 as usize] = val;
    }

    pub fn evaluate(&self) -> f32 {
        let mut connection_length: u32 = 0;
        let mut segment_number = 0;

        for connection in self.connections.as_slice() {
            for segment in connection.segments.as_slice() {
                connection_length += segment.length as u32;
            }
            segment_number += connection.segments.len();
        }

        self.collisions as f32 * WEIGHTS.0
            + connection_length as f32 * WEIGHTS.1
            + segment_number as f32 * WEIGHTS.2
    }
}

impl fmt::Display for Individual {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fn match_dir(dir: Direction, symbols: (char, char, char, char)) -> String {
            String::from(match dir {
                North => String::from(symbols.0),
                South => String::from(symbols.1),
                East => String::from(symbols.2),
                West => String::from(symbols.3),
            })
        }

        let mut character_map = vec![
            vec![
                String::from('\u{2219}').color("white");
                self.point_map.first().unwrap().len() as usize
            ];
            self.point_map.len() as usize
        ];

        let color = ["red", "green", "yellow", "blue", "magenta", "cyan", "white"];

        for c in 0..self.connections.len() {
            let mut actual_point = self.connections[c].start;
            character_map[actual_point.0 as usize][actual_point.1 as usize] = match_dir(
                self.connections[c].segments.first().unwrap().direction,
                ('\u{2568}', '\u{2565}', '\u{255E}', '\u{2561}'),
            )
            .color(color[c % color.len()]);

            for i in 0..self.connections[c].segments.len() {
                for j in 1..self.connections[c].segments[i].length {
                    match self.connections[c].segments[i].direction {
                        North => {
                            character_map[(actual_point.0 - j) as usize][actual_point.1 as usize] =
                                String::from('\u{2551}').color(color[c % color.len()]);
                        }
                        South => {
                            character_map[(actual_point.0 + j) as usize][actual_point.1 as usize] =
                                String::from('\u{2551}').color(color[c % color.len()]);
                        }
                        East => {
                            character_map[actual_point.0 as usize][(actual_point.1 + j) as usize] =
                                String::from('\u{2550}').color(color[c % color.len()]);
                        }
                        West => {
                            character_map[actual_point.0 as usize][(actual_point.1 - j) as usize] =
                                String::from('\u{2550}').color(color[c % color.len()]);
                        }
                    };
                }
                match self.connections[c].segments[i].direction {
                    North => actual_point.0 -= self.connections[c].segments[i].length,
                    South => actual_point.0 += self.connections[c].segments[i].length,
                    East => actual_point.1 += self.connections[c].segments[i].length,
                    West => actual_point.1 -= self.connections[c].segments[i].length,
                }

                if i < self.connections[c].segments.len() - 1 {
                    match self.connections[c].segments[i + 1].direction {
                        North => {
                            character_map[actual_point.0 as usize][(actual_point.1) as usize] =
                                match_dir(
                                    self.connections[c].segments[i].direction,
                                    ('\u{2551}', '\u{2551}', '\u{255D}', '\u{255A}'),
                                )
                                .color(color[c % color.len()]);
                        }
                        South => {
                            character_map[actual_point.0 as usize][(actual_point.1) as usize] =
                                match_dir(
                                    self.connections[c].segments[i].direction,
                                    ('\u{2551}', '\u{2551}', '\u{2557}', '\u{2554}'),
                                )
                                .color(color[c % color.len()]);
                        }
                        East => {
                            character_map[actual_point.0 as usize][(actual_point.1) as usize] =
                                match_dir(
                                    self.connections[c].segments[i].direction,
                                    ('\u{2554}', '\u{255A}', '\u{2550}', '\u{2550}'),
                                )
                                .color(color[c % color.len()]);
                        }
                        West => {
                            character_map[actual_point.0 as usize][(actual_point.1) as usize] =
                                match_dir(
                                    self.connections[c].segments[i].direction,
                                    ('\u{2557}', '\u{255D}', '\u{2550}', '\u{2550}'),
                                )
                                .color(color[c % color.len()]);
                        }
                    }
                }
            }

            character_map[actual_point.0 as usize][actual_point.1 as usize] = match_dir(
                self.connections[c].segments.last().unwrap().direction,
                ('\u{2565}', '\u{2568}', '\u{2561}', '\u{255E}'),
            )
            .color(color[c % color.len()]);
        }

        Ok(for i in character_map {
            for j in i {
                write!(f, "{}", &j).expect("Fail");
            }
            write!(f, "\n").expect("Fail");
        })
    }
}

impl Connection {
    fn create_segment(
        &mut self,
        direction: Direction,
        individual: &mut Individual,
        actual_point: &mut (u32, u32),
        random: &mut StdRng,
    ) -> (Segment, Option<Direction>) {
        let mut segment = Segment {
            length: 1,
            direction,
        };

        *actual_point = move_direction(*actual_point, direction);

        while *actual_point != self.end {
            let neighbors = individual.find_neighbors(*actual_point);
            let mut connection_length: u32 = 0;
            for segment in self.segments.as_slice() {
                connection_length += segment.length;
            }

            let probabilities = get_probability(
                (*actual_point, self.end),
                neighbors,
                direction,
                connection_length,
            );
            let roll: f32 = random.gen();

            if roll <= probabilities[0] {
                individual.mark_point(*actual_point, true);
                match segment.direction {
                    North => {
                        segment.length += 1;
                        *actual_point = move_direction(*actual_point, North);
                    }
                    _ => return (segment, Some(North)),
                }
            } else if roll <= probabilities[0] + probabilities[1] {
                individual.mark_point(*actual_point, true);
                match segment.direction {
                    South => {
                        segment.length += 1;
                        *actual_point = move_direction(*actual_point, South);
                    }
                    _ => return (segment, Some(South)),
                }
            } else if roll <= probabilities[0] + probabilities[1] + probabilities[2] {
                individual.mark_point(*actual_point, true);
                match segment.direction {
                    East => {
                        segment.length += 1;
                        *actual_point = move_direction(*actual_point, East);
                    }
                    _ => return (segment, Some(East)),
                }
            } else {
                individual.mark_point(*actual_point, true);
                match segment.direction {
                    West => {
                        segment.length += 1;
                        *actual_point = move_direction(*actual_point, West);
                    }
                    _ => return (segment, Some(West)),
                }
            };
        }

        (segment, None)
    }

    fn mutate_segment(&self, roll: usize) {
        let mut mutant = self.segments.get(self.segments.len() % roll);
    }
}

fn move_direction(point: (u32, u32), direction: Direction) -> (u32, u32) {
    match direction {
        North => {
            if point.0 == 0 {
                (0, point.1)
            } else {
                (point.0 - 1, point.1)
            }
        }
        South => (point.0 + 1, point.1),
        East => (point.0, point.1 + 1),
        West => {
            if point.1 == 0 {
                (point.0, 0)
            } else {
                (point.0, point.1 - 1)
            }
        }
    }
}

fn distance_factor(start: (u32, u32), end: (u32, u32)) -> f32 {
    let distance =
        (end.0 as i32 - start.0 as i32).abs() as f32 + (end.1 as i32 - start.1 as i32).abs() as f32;
    if distance == 0. {
        10.
    } else {
        1.0 / distance
    }
}

fn get_probability(
    points: ((u32, u32), (u32, u32)),
    neighbors: [f32; 4],
    previous_direction: Direction,
    steps: u32,
) -> [f32; 4] {
    let mut probabilities: [f32; 4] = [BASE; 4];

    for i in 0..probabilities.len() {
        probabilities[i] *= neighbors[i];
    }

    for dir in Direction::iterator() {
        match dir {
            North => probabilities[0] *= distance_factor(move_direction(points.0, North), points.1),
            South => probabilities[1] *= distance_factor(move_direction(points.0, South), points.1),
            East => probabilities[2] *= distance_factor(move_direction(points.0, East), points.1),
            West => probabilities[3] *= distance_factor(move_direction(points.0, West), points.1),
        };
    }

    match previous_direction {
        North => probabilities[1] = 0.0,
        South => probabilities[0] = 0.0,
        East => probabilities[3] = 0.0,
        West => probabilities[2] = 0.0,
    };

    let mut prob_sum = 0.0;
    let mut max = 0.;

    for i in 0..probabilities.len() {
        if probabilities[i] > max {
            max = probabilities[i];
        }
    }

    for i in 0..probabilities.len() {
        if probabilities[i] == max {
            probabilities[i] += steps as f32 * STEP_BONUS;
        }
        prob_sum += probabilities[i];
    }

    for i in 0..probabilities.len() {
        probabilities[i] /= prob_sum;
    }
    probabilities
}
