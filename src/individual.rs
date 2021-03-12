use self::Direction::*;
use colored::*;
use rand::Rng;
use std::fmt;
use rand::seq::SliceRandom;
use rand::thread_rng;

const COLLISION_FACTOR: f32 = 0.1;
const SIDE_FACTOR: f32 = 0.;
const STEP_BONUS: f32 = 0.1;
const BASE: f32 = 1.;

const WEIGHTS: (f32, f32, f32) = (10., 0.02, 0.01);

#[derive(Clone)]
pub struct Individual {
    connections: Vec<Connection>,
    pub point_map: Vec<Vec<bool>>,
    pub collisions: u8,
}

pub fn generate_individual(
    dimensions: (u8, u8),
    mut pin_locations: Vec<((u8, u8), (u8, u8))>,
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

    pin_locations.shuffle(&mut thread_rng());

    for pin_pair in &pin_locations {
        let connection = individual.random_walk(*pin_pair);
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
    length: u8,
    direction: Direction,
}

#[derive(Debug, Clone)]
struct Connection {
    start: (u8, u8),
    end: (u8, u8),
    actual_point: (u8, u8),
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

    fn find_neighbors(&self, point: (u8, u8)) -> [f32; 4] {
        // Up, DOWN, RIGHT, LEFT
        let mut neighbors: [f32; 4] = [1.0; 4];

        if point.0 <= 0 {
            neighbors[0] = SIDE_FACTOR;
        } else if self.point_map[point.0 as usize - 1][point.1 as usize] {
            neighbors[0] = COLLISION_FACTOR;
        }
        if point.0 >= self.point_map.len() as u8 - 1 {
            neighbors[1] = SIDE_FACTOR;
        } else if self.point_map[point.0 as usize + 1][point.1 as usize] {
            neighbors[1] = COLLISION_FACTOR;
        }
        if point.1 >= self.point_map[0].len() as u8 - 1 {
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

    fn random_walk(&mut self, pins: ((u8, u8), (u8, u8))) -> Connection {
        let mut connection = Connection {
            start: pins.0,
            end: pins.1,
            actual_point: pins.0,
            segments: Vec::new(),
        };

        self.mark_point(pins.1, false);

        let mut next_direction;

        let mut probabilities = self.find_neighbors(connection.start);
        let mut prob_sum = 0.;
        for i in 0..probabilities.len() {
            prob_sum += probabilities[i]
        }

        for i in 0..probabilities.len() {
            probabilities[i] /= prob_sum;
        }

        let roll: f32 = rand::thread_rng().gen();

        if roll <= probabilities[0] {
            next_direction = North;
        } else if roll <= probabilities[0] + probabilities[1] {
            next_direction = South;
        } else if roll <= probabilities[0] + probabilities[1] + probabilities[2] {
            next_direction = East;
        } else {
            next_direction = West;
        }

        while connection.actual_point != connection.end {
            let (segment, dir_holder) = connection.create_segment(next_direction, self);
            next_direction = dir_holder;
            connection.segments.push(segment);
        }

        self.mark_point(pins.1, true);

        connection
    }

    fn mark_point(&mut self, point: (u8, u8), val: bool) {
        if self.point_map[point.0 as usize][point.1 as usize] && val == true {
            self.collisions += 1;
        };
        self.point_map[point.0 as usize][point.1 as usize] = val;
    }

    pub fn evaluate(&self) -> f32 {
        let mut connection_length = 0;
        let mut segment_number = 0;

        for connection in self.connections.as_slice() {
            for segment in connection.segments.as_slice() {
                connection_length += segment.length;
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
    ) -> (Segment, Direction) {
        let mut segment = Segment {
            length: 1,
            direction,
        };

        self.actual_point = move_direction(self.actual_point, direction);

        while self.actual_point != self.end {
            let neighbors = individual.find_neighbors(self.actual_point);
            let mut connection_length = 0;
            for segment in self.segments.as_slice() {
                connection_length += segment.length;
            }

            let probabilities = get_probability(
                (self.actual_point, self.end),
                neighbors,
                direction,
                connection_length,
            );
            let roll: f32 = rand::thread_rng().gen();

            if roll <= probabilities[0] {
                individual.mark_point(self.actual_point, true);
                match segment.direction {
                    North => {
                        segment.length += 1;
                        self.actual_point = move_direction(self.actual_point, North);
                    }
                    _ => return (segment, North),
                }
            } else if roll <= probabilities[0] + probabilities[1] {
                individual.mark_point(self.actual_point, true);
                match segment.direction {
                    South => {
                        segment.length += 1;
                        self.actual_point = move_direction(self.actual_point, South);
                    }
                    _ => return (segment, South),
                }
            } else if roll <= probabilities[0] + probabilities[1] + probabilities[2] {
                individual.mark_point(self.actual_point, true);
                match segment.direction {
                    East => {
                        segment.length += 1;
                        self.actual_point = move_direction(self.actual_point, East);
                    }
                    _ => return (segment, East),
                }
            } else {
                individual.mark_point(self.actual_point, true);
                match segment.direction {
                    West => {
                        segment.length += 1;
                        self.actual_point = move_direction(self.actual_point, West);
                    }
                    _ => return (segment, West),
                }
            };
        }
        (segment, North)
    }
}

fn move_direction(point: (u8, u8), direction: Direction) -> (u8, u8) {
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

fn distance_factor(start: (u8, u8), end: (u8, u8)) -> f32 {
    let distance =
        (end.0 as i8 - start.0 as i8).abs() as f32 + (end.1 as i8 - start.1 as i8).abs() as f32;
    if distance == 0. {
        10.
    } else {
        1.0 / distance
    }
}

fn get_probability(
    points: ((u8, u8), (u8, u8)),
    neighbors: [f32; 4],
    previous_direction: Direction,
    steps: u8,
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
    // println!("{:?}", probabilities);
    probabilities
}
