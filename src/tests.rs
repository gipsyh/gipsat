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
    match solver.solve_with_constrain(&[lit2], &[!lit0, !lit1]) {
        SatResult::Sat(sat) => {
            dbg!(sat.lit_value(lit0));
            dbg!(sat.lit_value(lit1));
            dbg!(sat.lit_value(lit2));
        }
        SatResult::Unsat(_) => {}
    };
    match solver.solve(&[]) {
        SatResult::Sat(sat) => {
            dbg!(sat.lit_value(lit0));
            dbg!(sat.lit_value(lit1));
            dbg!(sat.lit_value(lit2));
        }
        SatResult::Unsat(_) => {}
    };
}
