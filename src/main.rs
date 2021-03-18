mod problem;
use problem::*;

fn main() {
    let mut problem: Problem = load_problem("test_data/zad3.txt", None);
    let result = problem.genetic_search(tournament_selection, None, None);
    println!("{}", result.0);
    println!("{}", result.1);

    // let result = problem.random_search(100000, Some(4));
    // println!("{}", result.0);
    // println!("{}", result.1);
}

