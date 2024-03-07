use crate::Solver;

impl Solver {
    pub fn simplify(&mut self) {
        if self.statistic.num_solve % 1000 != 1 {
            return;
        }
        self.clausedb_simplify_satisfied();
        // self.lemma_subsumption_simplify();
    }
}
