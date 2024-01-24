use crate::Solver;

impl Solver {
    pub fn simplify(&mut self) {
        self.clausedb_simplify_satisfied();
    }
}
