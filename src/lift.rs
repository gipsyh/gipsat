use crate::{BlockResultNo, GipSAT};
use logic_form::{Cube, Lit};
use satif::{SatResult, Satif, SatifSat, SatifUnsat};
use transys::Transys;

pub struct Lift {
    solver: minisat::Solver,
    num_act: usize,
}

impl Lift {
    pub fn new(model: &Transys) -> Self {
        let mut solver = minisat::Solver::new();
        let false_lit: Lit = solver.new_var().into();
        solver.add_clause(&[!false_lit]);
        model.load_trans(&mut solver);
        Self { solver, num_act: 0 }
    }
}

impl GipSAT {
    pub fn minimal_predecessor(&mut self, unblock: BlockResultNo, latchs: Cube) -> Cube {
        self.lift.num_act += 1;
        if self.lift.num_act > 1000 {
            self.lift = Lift::new(&self.ts)
        }
        let act: Lit = self.lift.solver.new_var().into();
        let mut assumption = Cube::from([act]);
        let mut cls = !&unblock.assumption;
        cls.push(!act);
        self.lift.solver.add_clause(&cls);
        for input in self.ts.inputs.iter() {
            let lit = input.lit();
            match unblock.sat.lit_value(lit) {
                Some(true) => assumption.push(lit),
                Some(false) => assumption.push(!lit),
                None => (),
            }
        }
        assumption.extend_from_slice(&latchs);
        let res: Cube = match self.lift.solver.solve(&assumption) {
            SatResult::Sat(_) => panic!(),
            SatResult::Unsat(conflict) => latchs.into_iter().filter(|l| conflict.has(*l)).collect(),
        };
        self.lift.solver.add_clause(&[!act]);
        res
    }
}
