use super::*;
use logic_form::Cnf;

#[test]
fn test1() {
    let mut solver = Solver::default();
    let lit0: Lit = solver.new_var().into();
    let lit1: Lit = solver.new_var().into();
    let lit2: Lit = solver.new_var().into();
    solver.add_clause(&Clause::from([lit0, !lit2]));
    solver.add_clause(&Clause::from([lit1, !lit2]));
    solver.add_clause(&Clause::from([!lit0, !lit1, lit2]));
    match solver.solve(&[lit2]) {
        SatResult::Sat(sat) => {
            assert!(sat.lit_value(lit0).unwrap());
            assert!(sat.lit_value(lit1).unwrap());
            assert!(sat.lit_value(lit2).unwrap());
        }
        SatResult::Unsat(_) => {
            todo!()
        }
    }
}

#[test]
fn test2() {
    let cnf = "p cnf 3 2\n1 -2 3 0\n-1 2 0\n";
    let cnf = Cnf::from_dimacs_str(cnf);
    let mut solver = Solver::default();
    for cls in cnf.iter() {
        solver.add_clause(cls);
    }
    dbg!(solver.solve(&[]));
}
