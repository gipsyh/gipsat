use crate::{BlockResultNo, Frame, GipSAT, Solver};
use logic_form::Cube;
use satif::{SatResult, SatifSat, SatifUnsat};
use std::rc::Rc;
use transys::Transys;

pub struct Lift {
    solver: Solver,
}

impl Lift {
    pub fn new(ts: &Rc<Transys>, frame: &Frame) -> Self {
        let solver = Solver::new(None, ts, frame);
        Self { solver }
    }
}

impl GipSAT {
    pub fn minimal_predecessor(&mut self, unblock: BlockResultNo) -> Cube {
        let mut assumption = Cube::new();
        let cls = !&unblock.assumption;
        for input in self.ts.inputs.iter() {
            let lit = input.lit();
            match unblock.sat.lit_value(lit) {
                Some(true) => assumption.push(lit),
                Some(false) => assumption.push(!lit),
                None => (),
            }
        }
        let mut latchs = Cube::new();
        for latch in self.ts.latchs.iter() {
            let lit = latch.lit();
            match unblock.sat.lit_value(lit) {
                Some(true) => latchs.push(lit),
                Some(false) => latchs.push(!lit),
                None => (),
            }
        }
        let solver = unsafe { &*unblock.sat.solver };
        solver.vsids.activity.sort_by_activity(&mut latchs, false);
        assumption.extend_from_slice(&latchs);
        let res: Cube = match self
            .lift
            .solver
            .solve_with_constrain(&assumption, cls, false)
        {
            SatResult::Sat(_) => panic!(),
            SatResult::Unsat(conflict) => latchs.into_iter().filter(|l| conflict.has(*l)).collect(),
        };
        res
    }
}
