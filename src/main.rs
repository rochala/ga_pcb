mod problem;
use problem::*;

const POPULATION_SIZE: usize = 1000;

fn main() {
    let mut problem: Problem = load_problem("test_data/zad3.txt", None);
    // problem.init_population(POPULATION_SIZE);
    let result = problem.random_search(100000, Some(4));

    println!("{:?}", result.0);
    println!("{}", result.0);
    println!("{}", result.0.evaluate());

}

