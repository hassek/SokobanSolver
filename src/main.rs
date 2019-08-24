use log::info;
use pretty_env_logger;
use sokoban::solver::Solver;
use std::env;
use std::time::Instant;

fn main() {
    pretty_env_logger::init();
    let args: Vec<String> = env::args().collect();
    let sokoban_level = &args[1];

    let mut solver = Solver::new(sokoban_level.clone());
    info!("{}", solver.sokoban);
    let start = Instant::now();
    let was_solved = solver.solve_sokoban();
    info!("Was solved? {} - steps: {}", was_solved, solver.counter);
    info!("Time elapsed solving sokoban is: {:?}", start.elapsed());
    if was_solved {
        println!("{}::{:?}", sokoban_level, start.elapsed());
    } else {
        println!("{}::notsolved", sokoban_level);
    }
}
