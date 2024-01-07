use clap::Parser;
use gipsat::{Args, Solver};
use logic_form::Cnf;

fn main() {
    let mut args = Args::parse();
    let cnf = Cnf::from_dimacs_file(args.dimacs);
    let mut solver = Solver::new();
    for cls in cnf.iter() {
        solver.add_clause(cls);
    }
    dbg!(solver.solve(&[]));
}
