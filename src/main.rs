mod problem;
use problem::*;

const POPULATION_SIZE: usize = 1000;

fn main() {
    let mut problem: Problem = load_problem("test_data/zad0.txt", None);
    // problem.init_population(POPULATION_SIZE);
    // let result = problem.random_search(1000000, Some(4));
    let result = problem.genetic_search(tournament_selection, None, None);
    println!("{}", result.0);
    println!("{}", result.1);

    // println!("{:?}", result.0);
    // println!("{}", result.0);
    // println!("{}", result.0.evaluate());
}

