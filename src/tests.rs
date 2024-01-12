use super::*;

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
            assert!(sat.lit_value(lit0));
            assert!(sat.lit_value(lit1));
            assert!(sat.lit_value(lit2));
        }
        SatResult::Unsat(_) => {
            todo!()
        }
    }
    solver.add_clause(&Clause::from([!lit0, !lit1]));
    match solver.solve(&[lit2]) {
        SatResult::Sat(_) => {
            todo!();
        }
        SatResult::Unsat(_) => {}
    };
}
