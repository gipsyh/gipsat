use crate::Solver;

impl Solver {
    pub fn verify(&mut self) -> bool {
        for i in 0..self.clauses.len() {
            if !self.clauses[i]
                .iter()
                .any(|l| matches!(self.value[*l], Some(true)))
            {
                return false;
            }
        }
        true
    }
}
