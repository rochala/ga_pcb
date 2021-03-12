use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct Problem {
    pub dimensions: (u8, u8),
    pub pin_locations: Vec<((u8, u8), (u8, u8))>,
}

fn parse_pairs(mut pairs: Vec<&str>) -> Vec<(u8, u8)> {
    let mut parsed_pairs: Vec<(u8, u8)> = Vec::new();

    while pairs.len() > 1 {
        let strings: (&str, &str) = (pairs.pop().unwrap(), pairs.pop().unwrap());
        let pair: (u8, u8) = (
            strings.1.trim().parse().unwrap(),
            strings.0.trim().parse().unwrap(),
        );
        parsed_pairs.push(pair);
    }

    return parsed_pairs;
}

pub fn load_problem(problem_name: &str) -> Problem {
    let file = File::open(problem_name).expect("Failed to open file");
    let reader = BufReader::new(file);
    let mut dimensions = (0, 0);
    let mut pin_locations: Vec<((u8, u8), (u8, u8))> = Vec::new();

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
    }
}
