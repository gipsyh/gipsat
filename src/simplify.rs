use crate::{cdb::CREF_NONE, Solver};

#[derive(Default)]
pub struct Simplify {
    last_num_assign: u32,
}

impl Solver {
    pub fn simplify(&mut self) {
        if self.statistic.num_solve % 1000 == 1 {
            assert!(self.highest_level() == 0);
            assert!(self.propagate() == CREF_NONE);
            if self.simplify.last_num_assign < self.trail.len() {
                self.simplify_satisfied();
                self.simplify.last_num_assign = self.trail.len();
            }
        }
        // if self.statistic.num_solve % 5000 == 0 {
        //     self.lemma_subsumption_simplify();
        // }
    }
}
