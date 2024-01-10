use clap::Parser;
use gipsat::{Args, Solver};
use logic_form::Cnf;
use std::time::Instant;

fn main() {
    let args = Args::parse();
    let cnf = Cnf::from_dimacs_file(&args.dimacs);
    let mut solver = Solver::new(args);
    for cls in cnf.iter() {
        solver.add_clause(cls);
    }
    let start = Instant::now();
    match solver.solve(&[]) {
        gipsat::SatResult::Sat(_) => println!("SAT"),
        gipsat::SatResult::Unsat(_) => println!("UNSAT"),
    };
    dbg!(start.elapsed());
}
