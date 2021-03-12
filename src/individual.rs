use self::Direction::*;
use rand::Rng;

const COLLISION_FACTOR: f32 = 0.1;
const SIDE_FACTOR: f32 = 0.;
const STEP_BONUS: f32 = 0.1;
const BASE: f32 = 1.;

const WEIGHTS: (f32, f32, f32) = (0.5, 0.3, 0.2);

pub struct Individual {
    pub connections: Vec<Connection>,
    pub point_map: Vec<Vec<bool>>,
    pub collisions: u8,
}

pub fn generate_individual(
    dimensions: (u8, u8),
    pin_locations: Vec<((u8, u8), (u8, u8))>,
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

    for pin_pair in &pin_locations {
        let connection = individual.random_walk(*pin_pair);
        individual.connections.push(connection);
    }

    // println!("{}", individual.collisions);

    individual
}

#[derive(Copy, Clone, Debug)]
pub enum Direction {
    North,
    South,
    East,
    West,
}

impl Direction {
    pub fn iterator() -> impl Iterator<Item = Direction> {
        [North, South, East, West].iter().copied()
    }
}

#[derive(Debug)]
pub struct Segment {
    length: u8,
    direction: Direction,
}

#[derive(Debug)]
pub struct Connection {
    start: (u8, u8),
    end: (u8, u8),
    actual_point: (u8, u8),
    pub segments: Vec<Segment>,
}

impl Individual {
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

    pub fn random_walk(&mut self, pins: ((u8, u8), (u8, u8))) -> Connection {
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

        // println!("Prawdy {:?}", probabilities);

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
            // println!("Moved: {:?} by {:?}", segment.direction, segment.length);
            connection.segments.push(segment);
        }
        // println!("finish");
        // println!("{:?}", self.point_map);

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
        for connection in self.connections.as_slice() {
            for segment in connection.segments.as_slice() {
                connection_length += segment.length;
            }
        }

        self.collisions as f32 * WEIGHTS.0
            + connection_length as f32 * WEIGHTS.1
            + self.connections.len() as f32 * WEIGHTS.2
    }
}

impl Connection {
    fn create_segment(
        &mut self,
        direction: Direction,
        individual: &mut Individual,
    ) -> (Segment, Direction) {
        //zwracamy segment + kierunek w ktorym ma byc kolejny

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

            //logika tworzenia segmentu
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

pub fn move_direction(point: (u8, u8), direction: Direction) -> (u8, u8) {
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

pub fn distance_factor(start: (u8, u8), end: (u8, u8)) -> f32 {
    let distance =
        (end.0 as i8 - start.0 as i8).abs() as f32 + (end.1 as i8 - start.1 as i8).abs() as f32;
    if distance == 0. {
        10.
    } else {
        1.0 / distance
    }
}

// North South East West
pub fn get_probability(
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
