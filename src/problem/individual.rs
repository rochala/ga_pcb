use Direction::*;

use colored::*;
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::fmt;

const WEIGHTS: (f32, f32, f32) = (10., 0.2, 0.01);

const COLLISION_FACTOR: f32 = 0.1;
const SIDE_FACTOR: f32 = 0.;
const STEP_BONUS: f32 = 0.5;
const BASE: f32 = 1.;

#[derive(Clone, Debug)]
pub struct Individual {
    connections: Vec<Connection>,
    pub point_map: Vec<Vec<bool>>,
    pub collisions: u64,
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
            if new_segments.len() == 0 {
                new_segments.push(self.segments[i]);
            } else {
                if self.segments[i].direction == new_segments.last().unwrap().direction {
                    new_segments.last_mut().unwrap().length += self.segments[i].length;
                } else if self.segments[i].direction == invert_direction(new_segments.last().unwrap().direction) {
                    if self.segments[i].length > new_segments.last().unwrap().length {
                        new_segments.last_mut().unwrap().length = self.segments[i].length - new_segments.last().unwrap().length;
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

    pub fn mutate_segment(&mut self, roll: (f32, f32), dimensions: (u32, u32)) {
        let index = (roll.0 * 100.0) as usize % self.segments.len();
        let mutant: &Segment = &self.segments[index];
        let segment_point = self.find_point(index);

        let mut mutation_value = (roll.1 * 4.) as u32;
        let direction: Option<Direction>;

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
