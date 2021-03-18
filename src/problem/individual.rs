use Direction::*;

use colored::*;
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::collections::HashMap;
use std::fmt;

const WEIGHTS: (f32, f32, f32) = (100., 0.2, 0.1);

const COLLISION_FACTOR: f32 = 0.1;
const SIDE_FACTOR: f32 = 0.;
const STEP_BONUS: f32 = 0.5;
const BASE: f32 = 1.;

#[derive(Clone, Debug)]
pub struct Individual {
    connections: Vec<Connection>,
    dimensions: (u32, u32),
}

pub fn generate_individual(
    dimensions: (u32, u32),
    pin_locations: Vec<((u32, u32), (u32, u32))>,
    seed: Option<u64>,
) -> Individual {
    let mut individual = Individual {
        connections: Vec::new(),
        dimensions,
    };

    let mut point_map = vec![vec![false; dimensions.1 as usize]; dimensions.0 as usize];

    for pin_pair in &pin_locations {
        individual.mark_point(pin_pair.0, true, &mut point_map);
        individual.mark_point(pin_pair.1, true, &mut point_map);
    }
    // pin_locations.shuffle(&mut thread_rng());

    for pin_pair in &pin_locations {
        let connection = individual.random_walk(*pin_pair, seed, &mut point_map);
        individual.connections.push(connection);
    }

    individual
}

impl Individual {
    pub fn new() -> Individual {
        Individual {
            connections: vec![],
            dimensions: (0, 0),
        }
    }

    fn find_neighbors(&self, point: (u32, u32), point_map: &Vec<Vec<bool>>) -> [f32; 4] {
        // Up, DOWN, RIGHT, LEFT
        let mut neighbors: [f32; 4] = [1.0; 4];

        if point.0 <= 0 {
            neighbors[0] = SIDE_FACTOR;
        } else if point_map[point.0 as usize - 1][point.1 as usize] {
            neighbors[0] = COLLISION_FACTOR;
        }
        if point.0 >= point_map.len() as u32 - 1 {
            neighbors[1] = SIDE_FACTOR;
        } else if point_map[point.0 as usize + 1][point.1 as usize] {
            neighbors[1] = COLLISION_FACTOR;
        }
        if point.1 >= point_map[0].len() as u32 - 1 {
            neighbors[2] = SIDE_FACTOR;
        } else if point_map[point.0 as usize][point.1 as usize + 1] {
            neighbors[2] = COLLISION_FACTOR;
        }
        if point.1 <= 0 {
            neighbors[3] = SIDE_FACTOR;
        } else if point_map[point.0 as usize][point.1 as usize - 1] {
            neighbors[3] = COLLISION_FACTOR;
        }

        neighbors
    }

    fn random_walk(
        &mut self,
        pins: ((u32, u32), (u32, u32)),
        seed: Option<u64>,
        point_map: &mut Vec<Vec<bool>>,
    ) -> Connection {
        let mut connection = Connection {
            start: pins.0,
            end: pins.1,
            segments: Vec::new(),
        };

        self.mark_point(pins.1, false, point_map);

        let mut actual_point = pins.0;

        let mut next_direction;

        let mut probabilities = self.find_neighbors(connection.start, point_map);

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
                point_map,
            );
            next_direction = dir_holder;
            connection.segments.push(segment);
        }

        self.mark_point(pins.1, true, point_map);

        connection
    }

    fn mark_point(&mut self, point: (u32, u32), val: bool, point_map: &mut Vec<Vec<bool>>) {
        point_map[point.0 as usize][point.1 as usize] = val;
    }

    fn collisions(&self) -> u32 {
        // println!("start");
        let mut points: HashMap<(u32, u32), bool> = HashMap::new();
        let mut collisions = 0;
        for point in self.collect_points() {
            // println!("Punkt hashmapa {:?}", point);
            if points.contains_key(&point) {
                collisions += 1;
            } else {
                points.insert(point, true);
            }
        }

        collisions
    }

    fn collect_points(&self) -> Vec<(u32, u32)> {
        let mut points = vec![];
        if self.connections.len() > 0 {
            for i in 0..self.connections.len() {
                points.append(&mut self.connections[i].following_points());
            }
        }
        // println!("---");

        points
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

        self.collisions() as f32 * WEIGHTS.0
            + connection_length as f32 * WEIGHTS.1
            + segment_number as f32 * WEIGHTS.2
    }

    pub fn crossover(&mut self, other: &Self, roll: f32) {
        let index = (roll * self.connections.len() as f32) as usize;
        self.connections[index] = other.connections[index].clone();
    }

    pub fn mutate(&mut self, random: &mut StdRng, mutation_chance: f32) {
        for connection in &mut self.connections {
            if random.gen::<f32>() < mutation_chance {
                connection.mutate_segment(
                    (random.gen::<f32>(), random.gen::<f32>()),
                    (self.dimensions.0 as u32, self.dimensions.1 as u32),
                )
            }
        }
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

        let mut character_map =
            vec![
                vec![String::from('\u{2219}').color("white"); self.dimensions.1 as usize];
                self.dimensions.0 as usize
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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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

#[derive(Debug, Clone, Copy)]
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

impl Connection {
    fn create_segment(
        &mut self,
        direction: Direction,
        individual: &mut Individual,
        actual_point: &mut (u32, u32),
        random: &mut StdRng,
        point_map: &mut Vec<Vec<bool>>,
    ) -> (Segment, Option<Direction>) {
        let mut segment = Segment {
            length: 1,
            direction,
        };

        *actual_point = move_direction(*actual_point, direction);

        while *actual_point != self.end {
            let neighbors = individual.find_neighbors(*actual_point, point_map);
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
                individual.mark_point(*actual_point, true, point_map);
                match segment.direction {
                    North => {
                        segment.length += 1;
                        *actual_point = move_direction(*actual_point, North);
                    }
                    _ => return (segment, Some(North)),
                }
            } else if roll <= probabilities[0] + probabilities[1] {
                individual.mark_point(*actual_point, true, point_map);
                match segment.direction {
                    South => {
                        segment.length += 1;
                        *actual_point = move_direction(*actual_point, South);
                    }
                    _ => return (segment, Some(South)),
                }
            } else if roll <= probabilities[0] + probabilities[1] + probabilities[2] {
                individual.mark_point(*actual_point, true, point_map);
                match segment.direction {
                    East => {
                        segment.length += 1;
                        *actual_point = move_direction(*actual_point, East);
                    }
                    _ => return (segment, Some(East)),
                }
            } else {
                individual.mark_point(*actual_point, true, point_map);
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

    fn following_points(&self) -> Vec<(u32, u32)> {
        fn segment_to_points(
            actual_point: (u32, u32),
            length: u32,
            direction: Direction,
        ) -> Vec<(u32, u32)> {
            let mut points = vec![];
            for i in 0..length {
                let actual_point = match direction {
                    North => (actual_point.0 - i, actual_point.1),
                    South => (actual_point.0 + i, actual_point.1),
                    East => (actual_point.0, actual_point.1 + i),
                    West => (actual_point.0, actual_point.1 - i),
                };
                points.push(actual_point);
            }
            // println!("SEGMETNY: {:?}", points);
            points
        }

        let mut points: Vec<(u32, u32)> = vec![];
        let mut current_point: (u32, u32) = self.start.clone();
        for segment in self.segments.as_slice() {
            let mut segment_points;
            match segment.direction {
                North => {
                    segment_points = segment_to_points(current_point, segment.length, North);
                    current_point = (current_point.0 - segment.length, current_point.1);
                }
                South => {
                    segment_points = segment_to_points(current_point, segment.length, South);
                    current_point = (current_point.0 + segment.length, current_point.1);
                }
                East => {
                    segment_points = segment_to_points(current_point, segment.length, East);
                    current_point = (current_point.0, current_point.1 + segment.length);
                }
                West => {
                    segment_points = segment_to_points(current_point, segment.length, West);
                    current_point = (current_point.0, current_point.1 - segment.length);
                }

            };
            points.append(&mut segment_points);
        }
        points.push(self.end);

        points
    }


    fn find_point(&self, index: usize) -> (u32, u32) {
        let mut point = self.start.clone();
        for i in 0..(index + 1) {
            match self.segments[i].direction {
                North => {
                    point.0 -= self.segments[i].length;
                }
                South => {
                    point.0 += self.segments[i].length;
                }
                East => {
                    point.1 += self.segments[i].length;
                }
                West => {
                    point.1 -= self.segments[i].length;
                }
            }
        }
        point
    }

    fn flatten(&mut self) {
        let mut new_segments: Vec<Segment> = vec![];
        new_segments.push(self.segments[0]);

        for i in 1..self.segments.len() {
            // println!("Stary {:?}", self.segments);
            // println!("Nowy {:?}", new_segments);
            if new_segments.len() == 0 {
                new_segments.push(self.segments[i]);
            } else if new_segments.len() == 1 && new_segments[0].length == 0 {
                new_segments.pop();
                new_segments.push(self.segments[i]);
            } else {
                if self.segments[i].direction == new_segments.last().unwrap().direction {
                    new_segments.last_mut().unwrap().length += self.segments[i].length;
                } else if self.segments[i].direction
                    == invert_direction(new_segments.last().unwrap().direction)
                {
                    if self.segments[i].length > new_segments.last().unwrap().length {
                        new_segments.last_mut().unwrap().length =
                            self.segments[i].length - new_segments.last().unwrap().length;
                        new_segments.last_mut().unwrap().direction = self.segments[i].direction;
                    } else if self.segments[i].length == new_segments.last().unwrap().length {
                        new_segments.pop();
                    } else {
                        new_segments.last_mut().unwrap().length -= self.segments[i].length;
                    }
                } else {
                    new_segments.push(self.segments[i]);
                }
            }
        }

        self.segments = new_segments;
    }

    fn split_segment() {
    }

    pub fn mutate_segment(&mut self, roll: (f32, f32), dimensions: (u32, u32)) {
        let index = (roll.0 * self.segments.len() as f32) as usize;
        let mutant: &Segment = &self.segments[index];
        let segment_point = self.find_point(index);

        let mut mutation_value = (roll.1 * dimensions.0 as f32) as u32 + 1;
        let direction: Option<Direction>;
        // println!("{}", index);



        match mutant.direction {
            North | South => {
                if roll.0 <= 0.5 {
                    // ->
                    if segment_point.1 + mutation_value >= dimensions.1 {
                        mutation_value = dimensions.1 - 1 - segment_point.1;
                    }
                    direction = Some(East);
                } else {
                    // <-
                    if segment_point.1 < mutation_value {
                        mutation_value = segment_point.1;
                    }
                    direction = Some(West);
                }
            }
            East | West => {
                if roll.0 <= 0.5 {
                    if segment_point.0 < mutation_value {
                        mutation_value = segment_point.0;
                    }
                    direction = Some(North);
                } else {
                    if segment_point.0 + mutation_value >= dimensions.0 {
                        mutation_value = dimensions.0 - 1 - segment_point.0;
                    }
                    direction = Some(South);
                }
            }
        }

        // println!("{}", mutation_value);

        if direction.is_none() {
            panic!("Direction is uninitialized");
        }

        self.segments.insert(
            index,
            Segment {
                length: mutation_value,
                direction: direction.unwrap(),
            },
        );
        self.segments.insert(
            index + 2,
            Segment {
                length: mutation_value,
                direction: invert_direction(direction.unwrap()),
            },
        );

        // println!("{:?}", self.segments);

        self.flatten();
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

fn invert_direction(direction: Direction) -> Direction {
    match direction {
        North => South,
        South => North,
        East => West,
        West => East,
    }
}

#[cfg(test)]
#[path = "test.rs"]
mod test;
